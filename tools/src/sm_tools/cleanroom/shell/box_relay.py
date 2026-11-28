import http.client
import io
import json
import os
import socketserver
import subprocess
import sys
import tarfile
import threading
import time
from dataclasses import dataclass, replace
from http.server import BaseHTTPRequestHandler
from pathlib import Path
from typing import ClassVar, cast, override
lazy from collections.abc import Mapping, Sequence

from ordered import FrozenList

from sm_tools.cleanroom.core.box_relay import (
    API_KEY,
    HELPER_SECONDS,
    MAX_BODY,
    NAME,
    OK,
    SCAN_BYTES,
    SCAN_TARGET,
    USAGE,
    Auth,
    ScanReply,
    ScanSession,
    Upstream,
    admitted,
    allowed_target,
    auth_of,
    base_prefix,
    box_env,
    charge,
    credential_headers,
    custom_headers,
    explicit_length,
    folded,
    forwarded,
    hit_line_prefixes,
    kept_line,
    layer_id,
    refusal,
    relayed_response_headers,
    root_image_matches,
    scan_reply,
    scanner_answers,
    upstream,
    with_credential,
)

METHODS = FrozenList((
    "GET",
    "POST",
    "PUT",
    "DELETE",
    "PATCH",
    "HEAD",
    "OPTIONS",
    "TRACE",
    "CONNECT",
))
ROOT_FOLDERS = FrozenList(("usr", "etc", "tmp", "dev", "proc", "sys"))
USR_MERGED = FrozenList(("bin", "lib", "lib64", "sbin"))
SHARED_TMP = 0o1777
FOLDER_MODE = 0o755
OWNER_ONLY = 0o077
CHUNK = 65536
UPSTREAM_SECONDS = 600
SCANNER_SECONDS = 300
PARENT_POLL_SECONDS = 1


class Scanner:
    def __init__(self, exe: str, repo: str, model: str, lane: str) -> None:
        self.exe: str = exe
        self.repo: str = repo
        self.lane: str = lane
        self.log: Path = Path(lane, "scan.log")
        self.hit_lines_for: FrozenList[str] = hit_line_prefixes(model)
        self.lock: threading.Lock = threading.Lock()
        self.session: ScanSession = ScanSession()

    def answer(self, body: bytes) -> ScanReply:
        with self.lock:
            self.session = replace(self.session, calls=self.session.calls + 1)
            call = self.session.calls
        files = admitted(call, body)
        if isinstance(files, ScanReply):
            return self.noted(files)
        done = subprocess.run(
            (self.exe, "names", self.lane),
            cwd=self.repo,
            input=body,
            capture_output=True,
            timeout=SCANNER_SECONDS,
            check=False,
        )
        answers = scanner_answers(
            call, files, done.returncode, done.stdout, done.stderr
        )
        if isinstance(answers, ScanReply):
            return self.noted(answers)
        with self.lock:
            charged = charge(self.session, files, answers)
            self.session = charged.session
        return self.noted(
            scan_reply(
                call,
                FrozenList(zip(files, answers, strict=True)),
                len(body),
                charged,
                self.hit_lines_for,
            )
        )

    def noted(self, reply: ScanReply) -> ScanReply:
        stamp = time.strftime("%Y-%m-%dT%H:%M:%S")
        with self.lock, self.log.open("a", encoding="utf-8") as log:
            _ = log.write(f"{stamp} {reply.note}\n")
        return reply


class Credential:
    def __init__(self, auth: Auth) -> None:
        self.auth: Auth = auth
        self.custom: dict[str, str] = dict(custom_headers(auth.env))
        self.lock: threading.Lock = threading.Lock()
        self.value: str = ""
        self.at: float = 0.0

    def headers(self) -> dict[str, str]:
        helper = self.auth.helper
        if not helper:
            return dict(credential_headers(self.custom, self.auth.env, None))
        with self.lock:
            if time.monotonic() - self.at > HELPER_SECONDS or not self.value:
                self.value = subprocess.run(
                    helper,
                    shell=True,
                    check=True,
                    text=True,
                    capture_output=True,
                ).stdout.strip()
                self.at = time.monotonic()
            return dict(
                credential_headers(self.custom, self.auth.env, self.value)
            )


@dataclass(frozen=True, slots=True)
class Relay:
    base: Upstream
    credential: Credential
    scanner: Scanner | None

    @property
    def prefix(self) -> FrozenList[str]:
        return base_prefix(self.base)

    def connection(self) -> http.client.HTTPConnection:
        kind = (
            http.client.HTTPSConnection
            if self.base.scheme == "https"
            else http.client.HTTPConnection
        )
        return kind(self.base.host, self.base.port, timeout=UPSTREAM_SECONDS)


def logged(text: str) -> None:
    _ = sys.stderr.write(f"{NAME}: {text}\n")


class RelayHandler(BaseHTTPRequestHandler):
    protocol_version: str = "HTTP/1.1"
    settings: ClassVar[Relay]

    @override
    def log_message(self, format: str, *args: object) -> None:
        first = str(args[0]).split("?", maxsplit=1)[0] if args else ""
        logged(f"{first} {args[1] if len(args) > 1 else ''}")

    @override
    def address_string(self) -> str:
        return "box"

    def relay(self) -> None:
        self.close_connection: bool = True
        relay = self.settings
        headers = FrozenList(self.headers.items())
        if folded(headers):
            self.send_error(400, "a folded header")
            return
        if relay.scanner and self.path == SCAN_TARGET:
            self.scan(relay.scanner, headers)
            return
        if self.command != "POST":
            self.send_error(405, "not a method of the API")
            return
        target = allowed_target(self.path, relay.prefix)
        if target is None:
            self.send_error(403, "outside the API")
            return
        length = explicit_length(headers, MAX_BODY)
        if length is None:
            self.send_error(400, "a body of an explicit length only")
            return
        body = self.rfile.read(length) if length else b""
        if why := refusal(body):
            logged(f"refused {why}")
            self.send_error(403, f"the relay refuses {why}")
            return
        self.forward(relay, target, headers, body)

    def forward(
        self,
        relay: Relay,
        target: str,
        headers: FrozenList[tuple[str, str]],
        body: bytes,
    ) -> None:
        passed = forwarded(headers)
        if passed.unlisted_betas:
            logged(f"dropped betas {', '.join(passed.unlisted_betas)}")
        if passed.dropped:
            logged(f"dropped headers {', '.join(passed.dropped)}")
        sent = with_credential(passed.headers, relay.credential.headers())
        upstream_connection = relay.connection()
        try:
            self.exchange(relay, upstream_connection, target, sent, body)
        except BrokenPipeError, ConnectionResetError:
            pass
        finally:
            upstream_connection.close()

    def exchange(
        self,
        relay: Relay,
        upstream_connection: http.client.HTTPConnection,
        target: str,
        sent: Mapping[str, str],
        body: bytes,
    ) -> None:
        upstream_connection.putrequest(
            self.command, target, skip_host=True, skip_accept_encoding=True
        )
        upstream_connection.putheader("Host", relay.base.netloc)
        for key, value in sent.items():
            upstream_connection.putheader(key, value)
        if self.command == "POST" or body:
            upstream_connection.putheader("Content-Length", str(len(body)))
        upstream_connection.endheaders(body or None)
        self.stream(upstream_connection.getresponse())

    def stream(self, response: http.client.HTTPResponse) -> None:
        self.send_response(response.status, response.reason)
        for key, value in relayed_response_headers(response.getheaders()):
            self.send_header(key, value)
        self.send_header("transfer-encoding", "chunked")
        self.send_header("connection", "close")
        self.end_headers()
        while chunk := response.read1(CHUNK):
            _ = self.wfile.write(b"%x\r\n%s\r\n" % (len(chunk), chunk))
            self.wfile.flush()
        _ = self.wfile.write(b"0\r\n\r\n")

    def scan(
        self, scanner: Scanner, headers: FrozenList[tuple[str, str]]
    ) -> None:
        length = explicit_length(headers, SCAN_BYTES)
        if self.command != "POST" or length is None:
            self.send_error(
                400, "a POST of a scan request of an explicit length only"
            )
            return
        reply = scanner.answer(self.rfile.read(length))
        if reply.status != OK:
            self.send_error(reply.status, reply.text)
            return
        data = reply.text.encode()
        self.send_response(OK)
        self.send_header("content-type", "text/plain; charset=utf-8")
        self.send_header("content-length", str(len(data)))
        self.send_header("connection", "close")
        self.end_headers()
        _ = self.wfile.write(data)


for method in METHODS:
    setattr(RelayHandler, f"do_{method}", RelayHandler.relay)


def handler_for(relay: Relay) -> type[RelayHandler]:
    class Handler(RelayHandler):
        settings: ClassVar[Relay] = relay

    return Handler


class Server(socketserver.ThreadingUnixStreamServer):
    daemon_threads: bool = True


def read_auth(path: str) -> object:
    text = sys.stdin.read() if path == "-" else Path(path).read_text("utf-8")
    return cast("object", json.loads(text))


def load_auth(path: str) -> Auth | str:
    return auth_of(read_auth(path), os.environ.get(API_KEY))


def relay_for(
    auth: Auth, sock_path: str, scan: tuple[str, str, str] | None
) -> Relay | str:
    lane = str(Path(sock_path).absolute().parent)
    scanner = Scanner(*scan, lane) if scan else None
    base = upstream(auth.env)
    if isinstance(base, str):
        return base
    return Relay(base, Credential(auth), scanner)


def relay_server(sock_path: str, relay: Relay) -> Server:
    path = Path(sock_path)
    if path.exists():
        path.unlink()
    old = os.umask(OWNER_ONLY)
    try:
        return Server(sock_path, handler_for(relay))
    finally:
        _ = os.umask(old)


def orphaned(parent: int) -> bool:
    try:
        os.kill(parent, 0)
    except OSError:
        return True
    return os.getppid() != parent


def watch_parent(parent: int) -> None:
    while not orphaned(parent):
        time.sleep(PARENT_POLL_SECONDS)
    os._exit(0)


def serve(
    sock_path: str,
    auth_path: str,
    parent: int | None,
    scan: tuple[str, str, str] | None,
) -> int:
    auth = load_auth(auth_path)
    relay = auth if isinstance(auth, str) else relay_for(auth, sock_path, scan)
    if isinstance(relay, str):
        print(relay, file=sys.stderr)
        return 1
    if parent is not None:
        threading.Thread(
            target=watch_parent, args=(parent,), daemon=True
        ).start()
    with relay_server(sock_path, relay) as server:
        print(f"{NAME}: ready", file=sys.stderr, flush=True)
        server.serve_forever()
    return 0


def env(auth_path: str, port: int) -> int:
    auth = load_auth(auth_path)
    base = auth if isinstance(auth, str) else upstream(auth.env)
    if isinstance(base, str) or isinstance(auth, str):
        print(base, file=sys.stderr)
        return 1
    boxed = box_env(auth, base, port)
    if boxed.kept:
        print(kept_line(boxed.kept), file=sys.stderr)
    if boxed.refusal is not None:
        print(boxed.refusal, file=sys.stderr)
        return 1
    for key, value in boxed.lines.items():
        print(f"{key}={value}")
    return 0


def root_tar() -> bytes:
    data = io.BytesIO()
    with tarfile.open(
        fileobj=data, mode="w", format=tarfile.USTAR_FORMAT
    ) as tar:
        for folder in ROOT_FOLDERS:
            info = tarfile.TarInfo(folder)
            info.type = tarfile.DIRTYPE
            mode = SHARED_TMP if folder == "tmp" else FOLDER_MODE
            tar.addfile(info.replace(mode=mode))
        for link in USR_MERGED:
            info = tarfile.TarInfo(link)
            info.type = tarfile.SYMTYPE
            tar.addfile(info.replace(linkname=f"usr/{link}"))
    return data.getvalue()


def image_ok(name: str, layer: str) -> bool:
    seen = subprocess.run(
        (
            "docker",
            "image",
            "inspect",
            "--format",
            "{{json .RootFS.Layers}}\n{{json .Config}}",
            name,
        ),
        capture_output=True,
        text=True,
        check=False,
    )
    return seen.returncode == 0 and root_image_matches(seen.stdout, layer)


def image(name: str) -> int:
    tar = root_tar()
    layer = layer_id(tar)
    if image_ok(name, layer):
        return 0
    made = subprocess.run(
        ("docker", "import", "-", name),
        input=tar,
        stdout=subprocess.DEVNULL,
        check=False,
    ).returncode
    if made != 0:
        return made
    return 0 if image_ok(name, layer) else 1


def command(arguments: Sequence[str]) -> int:
    match FrozenList(arguments):
        case [*args, "--scan", scanner, repo, model]:
            return chosen(FrozenList(args), (scanner, repo, model))
        case args:
            return chosen(args, None)


def chosen(args: Sequence[str], scan: tuple[str, str, str] | None) -> int:
    match FrozenList(args):
        case ["serve", sock_path, auth_path]:
            return serve(sock_path, auth_path, None, scan)
        case ["serve", sock_path, auth_path, parent]:
            return serve(sock_path, auth_path, int(parent), scan)
        case ["env", auth_path, port]:
            return env(auth_path, int(port))
        case ["image", name]:
            return image(name)
        case _:
            print(USAGE, file=sys.stderr)
            return 2


def main() -> int:
    return command(FrozenList(sys.argv[1:]))


if __name__ == "__main__":
    sys.exit(main())
