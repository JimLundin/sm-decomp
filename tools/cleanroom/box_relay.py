"""The API path out of `cleanroom run`'s box, and the box's first process.

The box (a container with no network) holds no credential. Its CLI talks plain HTTP to
127.0.0.1 inside the box; `box` forwards each connection there to a Unix socket mounted from the
export's lane folder, where `serve`, outside the box, adds the real credential and sends the request
on to the API over TLS. So the key never enters the box, and the API is the one place the box
can reach.

What passes is an allow-list, since the box chooses every byte it sends: a POST of
`v1/messages` or `v1/messages/count_tokens` under the base URL's path, with no query but
`beta=true`, whose body is a JSON object of the members a coding session sends. Its tools are
the session's own, run in the box; a server tool is refused, as is an MCP server or a
container. Its content blocks are text, inline images, tool calls, tool results and thinking;
a document, a file, a container upload or an image by URL or file id is refused. So the API
runs nothing and fetches nothing for the box. The request goes to the configured host with
its own Host header and only the headers a client of the API sends (content type, accept,
user agent, anthropic-*, x-stainless-*, x-claude-*, x-app), and anthropic-beta keeps only the
values in BETAS. A body whose object repeats a member and a folded header are refused, since
the API could read either otherwise than the relay does.

  serve SOCKET AUTH [PARENT] [--scan SCANNER REPO MODEL]
                              outside: AUTH (a file, or - for stdin, so it touches no disk) is an
                              object of Claude Code's top-level settings members (apiKeyHelper,
                              env); this process's ANTHROPIC_API_KEY counts too.
                              It exits when PARENT (a PID) does. With --scan it answers a POST of
                              /cleanroom/scan itself, never forwarding it: `SCANNER names LANE`
                              runs in REPO on the request, with LANE the folder of SOCKET, and the
                              box gets each file's hit count alone, never the lexicon's words, and
                              the first SCAN_LINES hit lines of a file under MODEL's src/ or
                              tests/. A request holds each file's path and head; the scanner reads
                              its start from the lane's baseline of the export's first commit and
                              reports the bytes of the head's lines the start doesn't hold. A call
                              may cost at most SCAN_CALL_BYTES of those and SCAN_FILES files, and a
                              session SCAN_SESSION_BYTES and SCAN_SESSION_FILES over at most
                              SCAN_CALLS calls; a file asked about again as it was costs nothing.
                              It logs each call and the session's total in scan.log in LANE.
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
import tomllib
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
ENDPOINTS = (["v1", "messages"], ["v1", "messages", "count_tokens"])
QUERIES = ("", "beta=true")
# The betas Claude Code 2.1.331 sent from the box, with and without --bare. Any other value is
# dropped: a beta is how a request turns on a server feature.
BETAS = frozenset({
    "claude-code-20250219", "interleaved-thinking-2025-05-14", "context-management-2025-06-27",
    "per-turn-control-2026-07-01", "mid-conversation-tool-changes-2026-07-01",
    "thinking-token-count-2026-05-13", "effort-2025-11-24", "thinking-display-updates-2026-08-18",
})
MEMBERS = frozenset({
    "model", "messages", "system", "tools", "tool_choice", "metadata", "max_tokens", "thinking",
    "context_management", "output_config", "stream", "temperature", "top_p", "top_k",
    "stop_sequences",
})
BLOCKS = frozenset({"text", "image", "tool_use", "tool_result", "thinking", "redacted_thinking"})
RESULT_BLOCKS = frozenset({"text", "image"})
HELPER_SECONDS = 300
MAX_BODY = 64 << 20
SCAN_TARGET = "/cleanroom/scan"
# Enough for an implementer's `cleanroom test` runs in one session, and few enough that the scan
# is no fast way to try words against the lexicon.
SCAN_CALLS = 100
SCAN_BYTES = 4 << 20
# A guess costs the bytes of the new lines that hold it: the head's lines the export's first
# commit doesn't hold, as the scanner reads it on the host. A call may spend this much, and a
# session this much in all, which is room for a model written in a session and many test runs,
# and little for a list of guesses. The files are capped too, so many small files buy nothing.
SCAN_CALL_BYTES = 256 << 10
SCAN_SESSION_BYTES = 2 << 20
SCAN_FILES = 64
SCAN_SESSION_FILES = 1024
# The hit lines a file of the model gets back; any other file gets its count alone.
SCAN_LINES = 200
SCAN_PATH = re.compile(r"crates/clean/[A-Za-z0-9_./-]+")
SCAN_ANSWER = re.compile(r"(\S+)\t([0-9]+)\t([0-9]+(?:,[0-9]+)*)?\t([0-9]+)")


def scan_files(body: bytes) -> list[dict] | None:
    """The files of a scan request, or None when it is not one."""
    try:
        request = tomllib.loads(body.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError):
        return None
    files = request.get("file")
    if set(request) != {"file"} or not isinstance(files, list):
        return None
    for f in files:
        if not isinstance(f, dict) or set(f) != {"path", "head"} \
                or not all(isinstance(v, str) for v in f.values()) \
                or not SCAN_PATH.fullmatch(f["path"]) or ".." in f["path"].split("/"):
            return None
    return files


def asked(f: dict) -> str:
    return hashlib.sha256(json.dumps([f["path"], f["head"]]).encode()).hexdigest()


class Scanner:
    """The name scan the relay serves to the box, counted, capped and logged."""

    def __init__(self, exe: str, repo: str, model: str, lane: str):
        self.exe, self.repo, self.lane = exe, repo, lane
        self.log = os.path.join(lane, "scan.log")
        self.lines_for = (f"{model.rstrip('/')}/src/", f"{model.rstrip('/')}/tests/")
        self.lock = threading.Lock()
        self.calls = 0
        self.spent = 0
        self.files = 0
        self.answered: set[str] = set()

    def answer(self, body: bytes) -> tuple[int, str]:
        with self.lock:
            self.calls += 1
            call = self.calls
        files = scan_files(body)
        if call > SCAN_CALLS:
            self.note(f"call {call}: refused, over {SCAN_CALLS} a session")
            return 429, f"the name scan answers {SCAN_CALLS} calls a session"
        if files is None:
            self.note(f"call {call}: refused, not a scan request")
            return 400, "not a scan request"
        if len(files) > SCAN_FILES:
            self.note(f"call {call}: refused, {len(files)} files, over {SCAN_FILES} a call")
            return 413, f"a scan call holds at most {SCAN_FILES} files"
        done = subprocess.run([self.exe, "names", self.lane], cwd=self.repo, input=body,
                              capture_output=True, timeout=300)
        lines = done.stdout.decode("utf-8", "replace").splitlines()
        paths = [f["path"] for f in files]
        answers = [SCAN_ANSWER.fullmatch(line) for line in lines]
        if done.returncode not in (0, 1) or not all(answers) or [a[1] for a in answers] != paths:
            self.note(f"call {call}: the scanner failed: {done.stderr.decode('utf-8', 'replace')[:200]}")
            return 500, "the name scan failed"
        with self.lock:
            fresh = [(f, a) for f, a in zip(files, answers) if asked(f) not in self.answered]
            cost = sum(int(a[4]) for _, a in fresh)
            spent, counted = self.spent + cost, self.files + len(fresh)
            within = cost <= SCAN_CALL_BYTES and spent <= SCAN_SESSION_BYTES and counted <= SCAN_SESSION_FILES
            if within:
                self.spent, self.files = spent, counted
                self.answered.update(asked(f) for f, _ in fresh)
        if cost > SCAN_CALL_BYTES:
            self.note(f"call {call}: refused, {cost} new bytes, over {SCAN_CALL_BYTES} a call")
            return 413, f"a scan call holds at most {SCAN_CALL_BYTES} bytes of new lines"
        if spent > SCAN_SESSION_BYTES:
            self.note(f"call {call}: refused, {cost} new bytes, over {SCAN_SESSION_BYTES} a session")
            return 429, f"the name scan reads {SCAN_SESSION_BYTES} bytes of new lines a session"
        if counted > SCAN_SESSION_FILES:
            self.note(f"call {call}: refused, {len(fresh)} new files, over {SCAN_SESSION_FILES} a session")
            return 429, f"the name scan reads {SCAN_SESSION_FILES} new files a session"
        note = [f"call {call}: {len(files)} files, {len(body)} bytes, {cost} new, {spent} new this session"]
        reply = []
        for f, a in zip(files, answers):
            numbers = a[3].split(",")[:SCAN_LINES] if a[3] and f["path"].startswith(self.lines_for) else []
            reply.append(f"{a[1]}\t{a[2]}\t{','.join(numbers)}\n")
            if a[2] != "0":
                text = f["head"].splitlines()
                hit = [f"    {n}: {text[int(n) - 1] if 0 < int(n) <= len(text) else ''}" for n in (a[3] or "").split(",") if n]
                note.append(f"  {f['path']}: {a[2]} hits\n" + "\n".join(hit))
        self.note("\n".join(note))
        return 200, "".join(reply)

    def note(self, text: str) -> None:
        with self.lock, open(self.log, "a", encoding="utf-8") as f:
            f.write(f"{time.strftime('%Y-%m-%dT%H:%M:%S')} {text}\n")


def blocks_refusal(blocks: object, allowed: frozenset[str]) -> str | None:
    if not isinstance(blocks, list):
        return "content that is not a list"
    for block in blocks:
        kind = block.get("type") if isinstance(block, dict) else None
        if kind not in allowed:
            return f"the content block {kind!r}"
        source = block.get("source")
        if kind == "image" and not (isinstance(source, dict) and source.get("type") == "base64"):
            return "an image that is not inline"
        content = block.get("content")
        if kind == "tool_result" and not isinstance(content, str) and content is not None:
            if why := blocks_refusal(content, RESULT_BLOCKS):
                return why
    return None


class Repeated(ValueError):
    """A JSON object names a member twice."""


def unique(pairs: list[tuple[str, object]]) -> dict:
    """A JSON object, refused when a member repeats: the relay checks the last value and the
    API might read the first."""
    keys = [k for k, _ in pairs]
    if len(set(keys)) != len(keys):
        raise Repeated(next(k for k in keys if keys.count(k) > 1))
    return dict(pairs)


def refusal(body: bytes) -> str | None:
    """Why the request body may not go to the API, or None."""
    try:
        request = json.loads(body, object_pairs_hook=unique)
    except Repeated as e:
        return f"the repeated member {e.args[0]!r}"
    except ValueError:
        return "a body that is not JSON"
    if not isinstance(request, dict):
        return "a body that is not a JSON object"
    if extra := sorted(set(request) - MEMBERS):
        return f"the request member {extra[0]!r}"
    tools = request.get("tools", [])
    if not isinstance(tools, list):
        return "tools that are not a list"
    for tool in tools:
        kind = tool.get("type", "custom") if isinstance(tool, dict) else None
        if kind != "custom":
            return f"the tool type {kind!r}"
    system = request.get("system")
    if not isinstance(system, str) and system is not None:
        if why := blocks_refusal(system, frozenset({"text"})):
            return why
    messages = request.get("messages", [])
    if not isinstance(messages, list):
        return "messages that are not a list"
    for message in messages:
        content = message.get("content") if isinstance(message, dict) else None
        if not isinstance(content, str):
            if why := blocks_refusal(content, BLOCKS):
                return why
    return None


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
    # The CLI runs with --bare, which takes ANTHROPIC_API_KEY alone, so the placeholder is a key
    # whatever the relay holds; the relay replaces it with the real credential.
    if not (helper or any(k in env for k in CREDENTIALS)):
        sys.exit("box_relay: no credential: no apiKeyHelper, ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN")
    out["ANTHROPIC_API_KEY"] = PLACEHOLDER
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
    """The origin-form target to send, or None: `/`, the base path's segments, one of ENDPOINTS
    and one of QUERIES, exactly. Anything else is refused, not normalised."""
    path, _, query = target.partition("?")
    if any(path == "/" + "/".join(prefix + endpoint) for endpoint in ENDPOINTS) and query in QUERIES:
        return target
    return None


def betas(values: list[str]) -> tuple[list[str], list[str]]:
    """The anthropic-beta values kept and those dropped."""
    seen = [v.strip() for value in values for v in value.split(",") if v.strip()]
    return [v for v in seen if v in BETAS], [v for v in seen if v not in BETAS]


def watch_parent(parent: int) -> None:
    while True:
        try:
            os.kill(parent, 0)
        except OSError:
            os._exit(0)
        if os.getppid() != parent:
            os._exit(0)
        time.sleep(1)


def serve(sock_path: str, auth: str, parent: int | None, scan: tuple[str, str, str] | None) -> None:
    env, helper = load_auth(auth)
    scanner = Scanner(*scan, os.path.dirname(os.path.abspath(sock_path))) if scan else None
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
            if any("\r" in v or "\n" in v for v in self.headers.values()):
                # A folded value reads as one header here and maybe as two upstream.
                self.send_error(400, "a folded header")
                return
            if scanner and self.path == SCAN_TARGET:
                self.scan()
                return
            if self.command != "POST":
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
            if why := refusal(body):
                sys.stderr.write(f"box_relay: refused {why}\n")
                self.send_error(403, f"the relay refuses {why}")
                return
            headers = {k.lower(): v for k, v in self.headers.items() if HEADERS.fullmatch(k.lower())}
            kept, unlisted = betas(self.headers.get_all("anthropic-beta") or [])
            headers.pop("anthropic-beta", None)
            if kept:
                headers["anthropic-beta"] = ",".join(kept)
            if unlisted:
                sys.stderr.write(f"box_relay: dropped betas {', '.join(unlisted)}\n")
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

        def scan(self) -> None:
            length = int(self.headers.get("content-length") or 0)
            if self.command != "POST" or length < 0 or length > SCAN_BYTES or "transfer-encoding" in self.headers:
                self.send_error(400, "a POST of a scan request of an explicit length only")
                return
            status, text = scanner.answer(self.rfile.read(length))
            if status != 200:
                self.send_error(status, text)
                return
            data = text.encode()
            self.send_response(200)
            self.send_header("content-type", "text/plain; charset=utf-8")
            self.send_header("content-length", str(len(data)))
            self.send_header("connection", "close")
            self.end_headers()
            self.wfile.write(data)

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
    scan = None
    if len(args) >= 4 and args[-4] == "--scan":
        scan, args = (args[-3], args[-2], args[-1]), args[:-4]
    if len(args) in (3, 4) and args[0] == "serve":
        serve(args[1], args[2], int(args[3]) if len(args) == 4 else None, scan)
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
