"""The API path out of `cleanroom run`'s box, and the box's first process.

The box (a container with no network) holds no credential. Its CLI talks plain HTTP to
127.0.0.1 inside the box; `box` forwards each connection there to a Unix socket mounted from the
export's lane folder, where `serve`, outside the box, adds the real credential and sends the request
on to the API over TLS. So the key never enters the box, and the API is the one place the box
can reach.

What passes is an allow-list, since the box chooses every byte it sends: GET or POST of an
origin-form target (`/...`) under the base URL's path and `v1/`, its segments plain names and
its query plain characters; the request goes to the configured host with its own Host header,
and only the headers a client of the API sends (content type, accept, user agent, anthropic-*,
x-stainless-*, x-claude-*, x-app) go with it. A JSON body that asks the API itself to reach
outside (a web search or fetch tool, an MCP server, a document or image by URL) is refused, so
the API is no way to the web.

  serve SOCKET AUTH [PARENT]  outside: AUTH (a file, or - for stdin, so it touches no disk) is an
                              object of Claude Code's top-level settings members (apiKeyHelper,
                              env); this process's ANTHROPIC_API_KEY counts too.
                              It exits when PARENT (a PID) does.
  env AUTH PORT               prints the environment the session gets instead, a KEY=VALUE a
                              line: the box's base URL, a placeholder credential and the model
                              and CLAUDE_CODE_DISABLE_* lines; the other ANTHROPIC_* lines
                              (custom headers among them) stay with the relay.
  box PORT SOCKET SECONDS -- ARGV
                              inside: listens on 127.0.0.1:PORT, runs ARGV, stops it after
                              SECONDS, exits with its status.
  image NAME                  makes the box's root image NAME unless docker's NAME is exactly
                              it: one layer holding the folders the mounts go in and the
                              /usr-merged symlinks and an empty configuration, made here, so no
                              registry is reached.
  image-id NAME               makes the same root image and prints its verified immutable ID.
"""

import hashlib
import http.client
import io
import json
import os
import re
import socket
import socketserver
import subprocess
import sys
import tarfile
import threading
import time
import urllib.parse
from http.server import BaseHTTPRequestHandler

DEFAULT_BASE = "https://api.anthropic.com"
PLACEHOLDER = "cleanroom-relay"
CREDENTIALS = ("ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN")
# The settings lines the box's CLI gets as they are.
BOX_SETTINGS = re.compile(r"(ANTHROPIC_(MODEL|SMALL_FAST_MODEL|DEFAULT_[A-Z]+_MODEL)|CLAUDE_CODE_DISABLE_[A-Z_]+)")
# The request headers passed on; every other is dropped (X-Forwarded-*, X-Original-URL, Forwarded,
# hop-by-hop headers, and the box's own credential headers, which the relay replaces).
HEADERS = re.compile(r"(content-type|accept|accept-encoding|accept-language|user-agent|x-app|"
                     r"anthropic-[a-z0-9-]+|x-stainless-[a-z0-9-]+|x-claude-[a-z0-9-]+)")
SEGMENT = re.compile(r"[A-Za-z0-9_-][A-Za-z0-9._-]*")
QUERY = re.compile(r"[A-Za-z0-9._~=&,-]*")
METHODS = ("GET", "POST")
HELPER_SECONDS = 300
MAX_BODY = 64 << 20
OUTSIDE = re.compile(r"(web_search|web_fetch|mcp)\w*|url")


def reaches_outside(item: object) -> bool:
    if isinstance(item, dict):
        kind = item.get("type")
        if "mcp_servers" in item or (isinstance(kind, str) and OUTSIDE.fullmatch(kind)):
            return True
        return any(reaches_outside(value) for value in item.values())
    if isinstance(item, list):
        return any(reaches_outside(value) for value in item)
    return False


def body_reaches_outside(body: bytes) -> bool:
    try:
        return reaches_outside(json.loads(body))
    except ValueError:
        return False


def load_auth(path: str) -> tuple[dict[str, str], str | None]:
    if path == "-":
        auth = json.load(sys.stdin)
    else:
        with open(path, encoding="utf-8") as f:
            auth = json.load(f)
    env = {k: str(v) for k, v in auth.get("env", {}).items()}
    if "ANTHROPIC_API_KEY" not in env and os.environ.get("ANTHROPIC_API_KEY"):
        env["ANTHROPIC_API_KEY"] = os.environ["ANTHROPIC_API_KEY"]
    return env, auth.get("apiKeyHelper")


def upstream(env: dict[str, str]) -> urllib.parse.SplitResult:
    url = urllib.parse.urlsplit(env.get("ANTHROPIC_BASE_URL", DEFAULT_BASE))
    if url.scheme not in ("https", "http") or not url.hostname or url.query or url.fragment \
            or url.username or url.password:
        sys.exit("box_relay: the base URL is not a plain http(s) URL")
    return url


def box_env(auth: str, port: int) -> dict[str, str]:
    env, helper = load_auth(auth)
    base = upstream(env)
    out = {k: v for k, v in env.items() if BOX_SETTINGS.fullmatch(k)}
    kept = sorted(k for k in env if k not in out and k not in CREDENTIALS and k != "ANTHROPIC_BASE_URL")
    if kept:
        print(f"box_relay: kept outside the box: {', '.join(kept)}", file=sys.stderr)
    out["ANTHROPIC_BASE_URL"] = f"http://127.0.0.1:{port}{base.path.rstrip('/')}"
    # The CLI sends what it holds the way the real one goes: a token as a bearer, a key or a
    # helper's output as x-api-key. The relay replaces either.
    kind = "ANTHROPIC_AUTH_TOKEN" if "ANTHROPIC_AUTH_TOKEN" in env and not helper else "ANTHROPIC_API_KEY"
    if not (helper or any(k in env for k in CREDENTIALS)):
        sys.exit("box_relay: no credential: no apiKeyHelper, ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN")
    out[kind] = PLACEHOLDER
    return out


class Credential:
    def __init__(self, env: dict[str, str], helper: str | None):
        self.env, self.helper = env, helper
        self.lock = threading.Lock()
        self.value, self.at = "", 0.0
        # ANTHROPIC_CUSTOM_HEADERS, one `Name: value` a line, which the relay adds itself.
        self.custom = {}
        for line in env.get("ANTHROPIC_CUSTOM_HEADERS", "").splitlines():
            name, sep, value = line.partition(":")
            if sep and name.strip():
                self.custom[name.strip()] = value.strip()

    def headers(self) -> dict[str, str]:
        out = dict(self.custom)
        if self.helper:
            with self.lock:
                if time.monotonic() - self.at > HELPER_SECONDS or not self.value:
                    self.value = subprocess.run(self.helper, shell=True, check=True, text=True,
                                                capture_output=True).stdout.strip()
                    self.at = time.monotonic()
                out.update({"x-api-key": self.value, "authorization": f"Bearer {self.value}"})
            return out
        if "ANTHROPIC_AUTH_TOKEN" in self.env:
            out["authorization"] = f"Bearer {self.env['ANTHROPIC_AUTH_TOKEN']}"
        if "ANTHROPIC_API_KEY" in self.env:
            out["x-api-key"] = self.env["ANTHROPIC_API_KEY"]
        return out


def allowed_target(target: str, prefix: list[str]) -> str | None:
    """The origin-form target to send, or None: `/` + the base path's segments + `v1` + plain
    segments, and an optional plain query. Anything else (absolute form, `..`, `;`, `\\`, `%`,
    empty segments) is refused, not normalised."""
    if not target.startswith("/") or target.startswith("//"):
        return None
    path, sep, query = target.partition("?")
    segments = path[1:].split("/")
    if segments[:len(prefix)] != prefix or segments[len(prefix):len(prefix) + 1] != ["v1"]:
        return None
    if not all(SEGMENT.fullmatch(s) for s in segments) or not QUERY.fullmatch(query):
        return None
    return target


def watch_parent(parent: int) -> None:
    while True:
        try:
            os.kill(parent, 0)
        except OSError:
            os._exit(0)
        if os.getppid() != parent:
            os._exit(0)
        time.sleep(1)


def serve(sock_path: str, auth: str, parent: int | None) -> None:
    env, helper = load_auth(auth)
    base = upstream(env)
    prefix = [s for s in base.path.split("/") if s]
    credential = Credential(env, helper)
    conn_type = http.client.HTTPSConnection if base.scheme == "https" else http.client.HTTPConnection

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, format: str, *args: object) -> None:
            first = str(args[0]).split("?")[0] if args else ""
            sys.stderr.write(f"box_relay: {first} {args[1] if len(args) > 1 else ''}\n")

        def address_string(self) -> str:
            return "box"

        def relay(self) -> None:
            self.close_connection = True
            if self.command not in METHODS:
                self.send_error(405, "not a method of the API")
                return
            target = allowed_target(self.path, prefix)
            if target is None:
                self.send_error(403, "outside the API")
                return
            length = int(self.headers.get("content-length") or 0)
            if length < 0 or length > MAX_BODY or "transfer-encoding" in self.headers:
                self.send_error(400, "a body of an explicit length only")
                return
            body = self.rfile.read(length) if length else b""
            if body_reaches_outside(body):
                sys.stderr.write("box_relay: refused a request that asks the API to reach outside\n")
                self.send_error(403, "the API may not reach outside for the box")
                return
            headers = {k.lower(): v for k, v in self.headers.items() if HEADERS.fullmatch(k.lower())}
            dropped = sorted({k.lower() for k in self.headers if not HEADERS.fullmatch(k.lower())}
                             - {"host", "content-length", "connection", "authorization", "x-api-key"})
            if dropped:
                sys.stderr.write(f"box_relay: dropped headers {', '.join(dropped)}\n")
            headers.update(credential.headers())
            conn = conn_type(base.hostname, base.port, timeout=600)
            try:
                # The request line holds the checked origin-form target alone, and Host is the
                # configured host's, whatever the box wrote.
                conn.putrequest(self.command, target, skip_host=True, skip_accept_encoding=True)
                conn.putheader("Host", base.netloc)
                for k, v in headers.items():
                    conn.putheader(k, v)
                if self.command == "POST" or body:
                    conn.putheader("Content-Length", str(len(body)))
                conn.endheaders(body or None)
                response = conn.getresponse()
                self.send_response(response.status, response.reason)
                for k, v in response.getheaders():
                    if k.lower() not in ("connection", "keep-alive", "transfer-encoding", "content-length"):
                        self.send_header(k, v)
                self.send_header("transfer-encoding", "chunked")
                self.send_header("connection", "close")
                self.end_headers()
                while chunk := response.read1(65536):
                    self.wfile.write(b"%x\r\n%s\r\n" % (len(chunk), chunk))
                    self.wfile.flush()
                self.wfile.write(b"0\r\n\r\n")
            except (BrokenPipeError, ConnectionResetError):
                # The CLI closed its side, as it does for a request it abandons.
                pass
            finally:
                conn.close()

        do_GET = do_POST = do_PUT = do_DELETE = do_PATCH = do_HEAD = do_OPTIONS = do_TRACE = relay
        do_CONNECT = relay

    class Server(socketserver.ThreadingMixIn, socketserver.UnixStreamServer):
        daemon_threads = True

    if parent is not None:
        threading.Thread(target=watch_parent, args=(parent,), daemon=True).start()
    if os.path.exists(sock_path):
        os.unlink(sock_path)
    old = os.umask(0o077)
    with Server(sock_path, Handler) as server:
        os.umask(old)
        print("box_relay: ready", file=sys.stderr, flush=True)
        server.serve_forever()


def pump(a: socket.socket, b: socket.socket) -> None:
    try:
        while data := a.recv(65536):
            b.sendall(data)
    except OSError:
        pass
    finally:
        for s in (a, b):
            try:
                s.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass


def box(port: int, sock_path: str, seconds: float, argv: list[str]) -> int:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", port))
    listener.listen(64)

    def accept() -> None:
        while True:
            client, _ = listener.accept()
            out = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            try:
                out.connect(sock_path)
            except OSError:
                client.close()
                continue
            for a, b in ((client, out), (out, client)):
                threading.Thread(target=pump, args=(a, b), daemon=True).start()

    threading.Thread(target=accept, daemon=True).start()
    child = subprocess.Popen(argv)
    try:
        return child.wait(timeout=seconds)
    except subprocess.TimeoutExpired:
        print(f"box_relay: the session passed its {seconds:g} s deadline; stopped", file=sys.stderr)
        child.kill()
        child.wait()
        return 124


def root_tar() -> bytes:
    data = io.BytesIO()
    with tarfile.open(fileobj=data, mode="w", format=tarfile.USTAR_FORMAT) as tar:
        for d in ("usr", "etc", "tmp", "dev", "proc", "sys"):
            info = tarfile.TarInfo(d)
            info.type, info.mode = tarfile.DIRTYPE, 0o1777 if d == "tmp" else 0o755
            tar.addfile(info)
        for link in ("bin", "lib", "lib64", "sbin"):
            info = tarfile.TarInfo(link)
            info.type, info.linkname = tarfile.SYMTYPE, f"usr/{link}"
            tar.addfile(info)
    return data.getvalue()


def image_ok(name: str, want: list[str]) -> bool:
    """Whether docker's NAME is exactly the root: its one layer this tar, and a configuration
    with nothing in it (no entrypoint, command, environment, user or working directory, which
    docker would apply to the box)."""
    seen = subprocess.run(["docker", "image", "inspect", "--format", "{{json .RootFS.Layers}}\n{{json .Config}}", name],
                          capture_output=True, text=True)
    if seen.returncode != 0:
        return False
    layers, _, config = seen.stdout.partition("\n")
    config = json.loads(config or "null") or {}
    return json.loads(layers or "null") == want and not any(config.values())


def image(name: str) -> int:
    # The tag is trusted only when it is exactly this root (docker's diff ID is the layer tar's
    # SHA-256); else it is made again, which moves the tag.
    tar = root_tar()
    want = [f"sha256:{hashlib.sha256(tar).hexdigest()}"]
    if image_ok(name, want):
        return 0
    made = subprocess.run(["docker", "import", "-", name], input=tar, stdout=subprocess.DEVNULL).returncode
    if made != 0:
        return made
    return 0 if image_ok(name, want) else 1


def main() -> int:
    args = sys.argv[1:]
    if len(args) in (3, 4) and args[0] == "serve":
        serve(args[1], args[2], int(args[3]) if len(args) == 4 else None)
        return 0
    if len(args) == 3 and args[0] == "env":
        for key, value in sorted(box_env(args[1], int(args[2])).items()):
            print(f"{key}={value}")
        return 0
    if len(args) == 2 and args[0] == "image":
        return image(args[1])
    if len(args) >= 6 and args[0] == "box" and args[4] == "--":
        return box(int(args[1]), args[2], float(args[3]), args[5:])
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
