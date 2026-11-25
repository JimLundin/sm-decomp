"""The API path out of launch.sh's box on Linux, and the box's first process.

The box (a container with no network) holds no credential. Its CLI talks plain HTTP to
127.0.0.1 inside the box; `box` forwards each connection there to a Unix socket mounted from the
lane directory, where `serve`, outside the box, adds the real credential and sends the request
on to the API over TLS. So the key never enters the box, and the API is the one place the box
can reach.

  serve SOCKET AUTH          outside: AUTH (a file, or - for stdin, so it touches no disk) is an
                             object of the top-level settings members launch.sh collects
                             (apiKeyHelper, env); this process's ANTHROPIC_API_KEY counts too.
  settings AUTH PORT         prints the settings members the session gets instead: env with
                             the box's base URL, a placeholder credential and the other
                             non-credential lines.
  box PORT SOCKET -- ARGV    inside: listens on 127.0.0.1:PORT, runs ARGV, exits with its status.
  image NAME                 makes the box's root image NAME unless docker has it: the folders
                             the mounts go in and the /usr-merged symlinks, nothing else, made
                             here, so no registry is reached.
"""

import http.client
import io
import json
import os
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
# Headers that belong to one connection, not to the request.
HOP = {"connection", "keep-alive", "proxy-connection", "transfer-encoding", "te", "trailer",
       "upgrade", "host", "authorization", "x-api-key", "content-length"}
HELPER_SECONDS = 300


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
    if url.scheme not in ("https", "http") or not url.hostname:
        sys.exit(f"box_relay: the base URL is not http(s): {url.scheme}")
    return url


def settings(auth: str, port: int) -> None:
    env, helper = load_auth(auth)
    base = upstream(env)
    out = {k: v for k, v in env.items() if k not in CREDENTIALS and k != "ANTHROPIC_BASE_URL"}
    out["ANTHROPIC_BASE_URL"] = f"http://127.0.0.1:{port}{base.path.rstrip('/')}"
    # The CLI sends what it holds the way the real one goes: a token as a bearer, a key or a
    # helper's output as x-api-key. The relay replaces either.
    kind = "ANTHROPIC_AUTH_TOKEN" if "ANTHROPIC_AUTH_TOKEN" in env and not helper else "ANTHROPIC_API_KEY"
    if not (helper or any(k in env for k in CREDENTIALS)):
        sys.exit("box_relay: no credential: no apiKeyHelper, ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN")
    out[kind] = PLACEHOLDER
    print(f'  "env": {json.dumps(out, sort_keys=True)},')


class Credential:
    def __init__(self, env: dict[str, str], helper: str | None):
        self.env, self.helper = env, helper
        self.lock = threading.Lock()
        self.value, self.at = "", 0.0

    def headers(self) -> dict[str, str]:
        if self.helper:
            with self.lock:
                if time.monotonic() - self.at > HELPER_SECONDS or not self.value:
                    self.value = subprocess.run(self.helper, shell=True, check=True, text=True,
                                                capture_output=True).stdout.strip()
                    self.at = time.monotonic()
                return {"x-api-key": self.value, "authorization": f"Bearer {self.value}"}
        out = {}
        if "ANTHROPIC_AUTH_TOKEN" in self.env:
            out["authorization"] = f"Bearer {self.env['ANTHROPIC_AUTH_TOKEN']}"
        if "ANTHROPIC_API_KEY" in self.env:
            out["x-api-key"] = self.env["ANTHROPIC_API_KEY"]
        return out


def serve(sock_path: str, auth: str) -> None:
    env, helper = load_auth(auth)
    base = upstream(env)
    prefix = base.path.rstrip("/") + "/"
    credential = Credential(env, helper)
    conn_type = http.client.HTTPSConnection if base.scheme == "https" else http.client.HTTPConnection

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, format: str, *args: object) -> None:
            sys.stderr.write(f"box_relay: {self.command} {self.path.split('?')[0]} {args[1] if len(args) > 1 else ''}\n")

        def address_string(self) -> str:
            return "box"

        def relay(self) -> None:
            path = urllib.parse.urlsplit(self.path).path
            # Only the API's own paths: the base path and below, no way back up it.
            if not path.startswith(prefix) or "/../" in path + "/" or "%" in path:
                self.send_error(403, "outside the API")
                return
            length = int(self.headers.get("content-length") or 0)
            body = self.rfile.read(length) if length else None
            headers = {k: v for k, v in self.headers.items() if k.lower() not in HOP}
            headers.update(credential.headers())
            upstream_conn = conn_type(base.hostname, base.port, timeout=600)
            try:
                upstream_conn.request(self.command, self.path, body=body, headers=headers)
                response = upstream_conn.getresponse()
                self.send_response(response.status, response.reason)
                for k, v in response.getheaders():
                    if k.lower() not in HOP:
                        self.send_header(k, v)
                self.send_header("transfer-encoding", "chunked")
                self.send_header("connection", "close")
                self.end_headers()
                while chunk := response.read1(65536):
                    self.wfile.write(b"%x\r\n%s\r\n" % (len(chunk), chunk))
                    self.wfile.flush()
                self.wfile.write(b"0\r\n\r\n")
            finally:
                upstream_conn.close()
            self.close_connection = True

        do_GET = do_POST = do_PUT = do_DELETE = do_PATCH = do_HEAD = relay

    class Server(socketserver.ThreadingMixIn, socketserver.UnixStreamServer):
        daemon_threads = True

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


def box(port: int, sock_path: str, argv: list[str]) -> int:
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
    return subprocess.run(argv).returncode


def image(name: str) -> int:
    if subprocess.run(["docker", "image", "inspect", name], capture_output=True).returncode == 0:
        return 0
    data = io.BytesIO()
    with tarfile.open(fileobj=data, mode="w") as tar:
        for d in ("usr", "etc", "tmp", "dev", "proc", "sys"):
            info = tarfile.TarInfo(d)
            info.type, info.mode = tarfile.DIRTYPE, 0o1777 if d == "tmp" else 0o755
            tar.addfile(info)
        for link in ("bin", "lib", "lib64", "sbin"):
            info = tarfile.TarInfo(link)
            info.type, info.linkname = tarfile.SYMTYPE, f"usr/{link}"
            tar.addfile(info)
    return subprocess.run(["docker", "import", "-", name], input=data.getvalue(),
                          stdout=subprocess.DEVNULL).returncode


def main() -> int:
    args = sys.argv[1:]
    if len(args) == 3 and args[0] == "serve":
        serve(args[1], args[2])
        return 0
    if len(args) == 3 and args[0] == "settings":
        settings(args[1], int(args[2]))
        return 0
    if len(args) == 2 and args[0] == "image":
        return image(args[1])
    if len(args) >= 5 and args[0] == "box" and args[3] == "--":
        return box(int(args[1]), args[2], args[4:])
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
