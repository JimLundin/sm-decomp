import asyncio
import contextlib
import io
import json
import os
import sys
from concurrent.futures import ThreadPoolExecutor
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import ClassVar
lazy from collections.abc import Generator, Iterator, Mapping
lazy from pathlib import Path

import pytest
from ordered import FrozenList, FrozenOrderedDict

from sm_tools.cleanroom.core.box_relay import (
    SCAN_CALLS,
    SCAN_FILES,
    SCAN_SESSION_FILES,
    Auth,
    allowed_target,
    auth_of,
    betas,
    refusal,
)
from sm_tools.cleanroom.shell.box_relay import (
    command,
    orphaned,
    relay_for,
    relay_server,
)

PREFIX = FrozenList(("lms", "proxy", "anthropic"))
BASE64 = FrozenOrderedDict({
    "type": "base64",
    "media_type": "image/png",
    "data": "AA==",
})
SOCKET = "api.sock"
HIGHEST_PID = 1 << 22

type Seen = list[tuple[str, dict[str, str]]]


def test_the_two_calls_of_a_session_pass() -> None:
    for target in (
        "/lms/proxy/anthropic/v1/messages",
        "/lms/proxy/anthropic/v1/messages?beta=true",
        "/lms/proxy/anthropic/v1/messages/count_tokens",
    ):
        assert allowed_target(target, PREFIX) == target


def test_every_other_api_path_is_refused() -> None:
    for target in (
        "/lms/proxy/anthropic/v1/models",
        "/lms/proxy/anthropic/v1/files",
        "/lms/proxy/anthropic/v1/messages/batches",
        "/lms/proxy/anthropic/v1/skills",
        "/lms/proxy/anthropic/v1/complete",
        "/lms/proxy/anthropic/v1/messages?beta=false",
        "/lms/proxy/anthropic/v1/messages?beta=true&x=1",
        "/v1/messages",
    ):
        assert allowed_target(target, PREFIX) is None


def test_everything_else_is_refused() -> None:
    for target in (
        "http://evil.example/lms/proxy/anthropic/v1/messages",
        "evil.example:443",
        "//evil.example/lms/proxy/anthropic/v1/messages",
        "/other",
        "/lms/proxy/anthropic/admin",
        "/lms/proxy/anthropic/v1/../../admin",
        "/lms/proxy/anthropic/v1/..;/..;/admin",
        "/lms/proxy/anthropic/v1/..\\..\\admin",
        "/lms/proxy/anthropic/v1/%2e%2e/admin",
        "/lms/proxy/anthropic/v1/./messages",
        "/lms/proxy/anthropic/v1//messages",
        "/lms/proxy/anthropic/v1/messages;x=1",
        "/lms/proxy/anthropic/v1/messages?x=%0d%0a",
        "/lms/proxy/anthropic/v1/.hidden",
        "/lms/proxy/anthropic/../anthropic/v1/messages",
        "*",
    ):
        assert allowed_target(target, PREFIX) is None


def user(*blocks: object) -> dict[str, object]:
    return {
        "model": "m",
        "messages": [{"role": "user", "content": list(blocks)}],
    }


def refused(body: object) -> str | None:
    return refusal(json.dumps(body).encode())


def test_a_body_that_is_not_a_json_object_is_refused() -> None:
    for body in (b"not json", b"", b"[]", b'"text"', b"\xff"):
        assert refusal(body) is not None


def test_a_repeated_member_is_refused() -> None:
    for body in (
        b'{"model": "m", "tools": [{"type": "web_search_20250305"}], "tools": []}',
        b'{"model": "m", "messages": [{"role": "user", "content": "x", "content": "y"}]}',
    ):
        assert "repeated member" in (refusal(body) or "")


def test_every_server_tool_is_refused() -> None:
    for kind in (
        "web_search_20250305",
        "web_fetch_20250910",
        "code_execution_20250825",
        "bash_20250124",
        "text_editor_20250728",
        "computer_20250124",
        "memory_20250818",
        "tool_search_tool_regex_20251119",
        "mcp_toolset",
        "advisor_20260301",
        None,
    ):
        assert refused({"model": "m", "tools": [{"type": kind, "name": "t"}]})


def test_a_member_that_names_an_outside_service_is_refused() -> None:
    for member in ("mcp_servers", "container", "betas", "service_tier"):
        assert refused({"model": "m", member: []}) is not None


def test_a_file_a_document_or_an_upload_is_refused() -> None:
    blocks: FrozenList[Mapping[str, object]] = FrozenList((
        {
            "type": "document",
            "source": {"type": "url", "url": "https://x/a.pdf"},
        },
        {"type": "document", "source": {"type": "base64", "data": "AA=="}},
        {"type": "document", "source": {"type": "file", "file_id": "f"}},
        {"type": "image", "source": {"type": "url", "url": "https://x/a.png"}},
        {"type": "image", "source": {"type": "file", "file_id": "f"}},
        {"type": "container_upload", "file_id": "f"},
        {
            "type": "search_result",
            "source": "https://x",
            "title": "t",
            "content": [],
        },
        {
            "type": "server_tool_use",
            "id": "s",
            "name": "web_search",
            "input": {},
        },
        {"type": "web_search_tool_result", "tool_use_id": "s", "content": []},
        {
            "type": "mcp_tool_use",
            "id": "m",
            "name": "x",
            "server_name": "s",
            "input": {},
        },
        {
            "type": "tool_result",
            "tool_use_id": "t",
            "content": [
                {
                    "type": "document",
                    "source": {"type": "url", "url": "https://x/a.pdf"},
                }
            ],
        },
        {
            "type": "tool_result",
            "tool_use_id": "t",
            "content": [
                {
                    "type": "image",
                    "source": {"type": "url", "url": "https://x/a.png"},
                }
            ],
        },
    ))
    for block in blocks:
        assert refused(user(block)) is not None
    system = [{"type": "document", "source": dict(BASE64)}]
    assert refused({"model": "m", "system": system}) is not None


def test_the_cli_s_own_requests_pass() -> None:
    body: dict[str, object] = {
        "model": "m",
        "max_tokens": 1,
        "stream": True,
        "metadata": {"user_id": "u"},
        "thinking": {"type": "adaptive"},
        "output_config": {"effort": "medium"},
        "context_management": {
            "edits": [{"type": "clear_thinking_20251015", "keep": "all"}]
        },
        "system": [
            {
                "type": "text",
                "text": "s",
                "cache_control": {"type": "ephemeral"},
            }
        ],
        "tools": [
            {
                "name": "Bash",
                "description": "web_search url",
                "input_schema": {"type": "object"},
            },
            {
                "type": "custom",
                "name": "Read",
                "input_schema": {"type": "object"},
            },
        ],
        "tool_choice": {"type": "auto"},
        "messages": [
            {"role": "user", "content": "plain"},
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": '"type": "url"'},
                    {"type": "image", "source": dict(BASE64)},
                ],
            },
            {
                "role": "assistant",
                "content": [
                    {"type": "thinking", "thinking": "t", "signature": "s"},
                    {"type": "redacted_thinking", "data": "d"},
                    {
                        "type": "tool_use",
                        "id": "t",
                        "name": "Bash",
                        "input": {"url": "https://x"},
                    },
                ],
            },
            {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "t",
                        "content": "out",
                    },
                    {
                        "type": "tool_result",
                        "tool_use_id": "t",
                        "is_error": True,
                        "content": [
                            {"type": "text", "text": "x"},
                            {"type": "image", "source": dict(BASE64)},
                        ],
                    },
                ],
            },
        ],
    }
    assert refused(body) is None


def test_only_the_cli_s_betas_are_kept() -> None:
    kept, dropped = betas((
        "claude-code-20250219, files-api-2025-04-14",
        "mcp-client-2025-04-04,effort-2025-11-24",
    ))
    assert kept == ("claude-code-20250219", "effort-2025-11-24")
    assert dropped == ("files-api-2025-04-14", "mcp-client-2025-04-04")


def test_only_the_cli_lines_enter_the_box(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    settings = {
        "env": {
            "ANTHROPIC_BASE_URL": "https://gw.example/lms/proxy/anthropic",
            "ANTHROPIC_AUTH_TOKEN": "tok-secret",
            "ANTHROPIC_CUSTOM_HEADERS": "X-Gateway-Key: gw-secret",
            "ANTHROPIC_FOUNDRY_API_KEY": "fk-secret",
            "ANTHROPIC_MODEL": "opus",
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
        }
    }
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    monkeypatch.setattr("sys.stdin", io.StringIO(json.dumps(settings)))
    assert command(("env", "-", "8765")) == 0
    printed = capsys.readouterr()
    lines = dict(line.split("=", 1) for line in printed.out.splitlines())
    assert lines == {
        "ANTHROPIC_API_KEY": "cleanroom-relay",
        "ANTHROPIC_BASE_URL": "http://127.0.0.1:8765/lms/proxy/anthropic",
        "ANTHROPIC_MODEL": "opus",
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
    }
    assert "ANTHROPIC_CUSTOM_HEADERS" in printed.err
    assert "secret" not in printed.err


class Api(BaseHTTPRequestHandler):
    seen: ClassVar[Seen]

    def answer(self) -> None:
        _ = self.rfile.read(int(self.headers.get("content-length") or 0))
        headers = {key.lower(): value for key, value in self.headers.items()}
        self.seen.append((self.requestline, headers))
        self.send_response(200)
        self.send_header("content-length", "2")
        self.end_headers()
        _ = self.wfile.write(b"ok")


for method in ("GET", "POST"):
    setattr(Api, f"do_{method}", Api.answer)


FAKE_SCANNER = """import pathlib, sys, tomllib
body = sys.stdin.read()
for f in tomllib.loads(body)["file"]:
    start = pathlib.Path(sys.argv[2], "baseline", f["path"])
    held = set(start.read_text().splitlines()) if start.exists() else set()
    new = sum(len(l.encode()) + 1 for l in f["head"].splitlines() if l not in held)
    hits = f["head"].count("quux")
    lines = ",".join(str(i + 1) for i, l in enumerate(f["head"].splitlines()) if "quux" in l)
    print(f["path"] + "\\t" + str(hits) + "\\t" + lines + "\\t" + str(new) + ("\\tquux" if "leak" in f["path"] else ""))
sys.exit(1 if "quux" in body else 0)
"""


async def exchanged(sock_path: str, raw: bytes) -> bytes:
    reader, writer = await asyncio.open_unix_connection(sock_path)
    writer.write(raw)
    await writer.drain()
    data = await reader.read()
    writer.close()
    await writer.wait_closed()
    return data


class Rig:
    def __init__(self, api_port: int, lane: Path, seen: Seen) -> None:
        self.api_port: int = api_port
        self.lane: Path = lane
        self.seen: Seen = seen
        self.sock_path: str = SOCKET

    def exchange(self, raw: bytes) -> bytes:
        return asyncio.run(exchanged(self.sock_path, raw))

    def send(self, raw: bytes) -> int:
        return int(self.exchange(raw).split(b" ", 2)[1])

    def log(self) -> str:
        return (self.lane / "scan.log").read_text(encoding="utf-8")


@contextlib.contextmanager
def rig(
    lane: Path, monkeypatch: pytest.MonkeyPatch, *, scan: bool
) -> Generator[Rig]:
    requests: Seen = []

    class SeenApi(Api):
        seen: ClassVar[Seen] = requests

    api = ThreadingHTTPServer(("127.0.0.1", 0), SeenApi)
    api_port = api.server_port
    monkeypatch.chdir(lane)
    settings = {
        "env": {
            "ANTHROPIC_BASE_URL": f"http://127.0.0.1:{api_port}/lms/proxy/anthropic",
            "ANTHROPIC_AUTH_TOKEN": "SECRET-TOKEN",
            "ANTHROPIC_CUSTOM_HEADERS": "X-Gateway-Key: gw",
        }
    }
    auth = auth_of(settings, None)
    assert isinstance(auth, Auth)
    scanner = lane / "scanner"
    _ = scanner.write_text(f"#!{sys.executable} -I\n{FAKE_SCANNER}")
    scanner.chmod(0o700)
    scanning = (str(scanner), str(lane), "crates/clean/a") if scan else None
    relay = relay_for(auth, SOCKET, scanning)
    assert not isinstance(relay, str)
    server = relay_server(SOCKET, relay)
    with ThreadPoolExecutor() as pool:
        _ = pool.submit(api.serve_forever)
        _ = pool.submit(server.serve_forever)
        try:
            yield Rig(api_port, lane, requests)
        finally:
            server.shutdown()
            server.server_close()
            api.shutdown()
            api.server_close()


@pytest.fixture
def relay(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Iterator[Rig]:
    with rig(tmp_path, monkeypatch, scan=False) as started:
        yield started


@pytest.fixture
def scanning(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Iterator[Rig]:
    with rig(tmp_path, monkeypatch, scan=True) as started:
        yield started


def goes_with_the_key_its_host_and_allowed_headers(rig: Rig) -> None:
    status = rig.send(
        b"POST /lms/proxy/anthropic/v1/messages?beta=true HTTP/1.1\r\nHost: evil.example\r\nContent-Length: 2\r\nX-Api-Key: cleanroom-relay\r\nAnthropic-Version: 1\r\nAnthropic-Beta: claude-code-20250219,files-api-2025-04-14\r\nanthropic-beta: mcp-client-2025-04-04\r\nX-Original-URL: /admin\r\nX-Forwarded-Host: evil.example\r\nForwarded: host=evil\r\nConnection: close\r\n\r\n{}"
    )
    assert status == 200
    line, headers = rig.seen[0]
    assert line == "POST /lms/proxy/anthropic/v1/messages?beta=true HTTP/1.1"
    assert headers["host"] == f"127.0.0.1:{rig.api_port}"
    assert headers["authorization"] == "Bearer SECRET-TOKEN"
    assert headers["x-gateway-key"] == "gw"
    assert headers["anthropic-version"] == "1"
    assert headers["anthropic-beta"] == "claude-code-20250219"
    for dropped in (
        "x-original-url",
        "x-forwarded-host",
        "forwarded",
        "x-api-key",
    ):
        assert dropped not in headers


def never_reach_the_api(rig: Rig) -> None:
    for raw in (
        b"GET http://evil.example/lms/proxy/anthropic/v1/messages HTTP/1.1\r\nHost: evil.example\r\n\r\n",
        b"CONNECT evil.example:443 HTTP/1.1\r\nHost: evil.example:443\r\n\r\n",
        b"POST /lms/proxy/anthropic/v1/..;/..;/admin HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
        b"POST /lms/proxy/anthropic/v1/..\\..\\admin HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
        b"POST /lms/proxy/anthropic/v1/%2e%2e/admin HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
        b"DELETE /lms/proxy/anthropic/v1/messages HTTP/1.1\r\n\r\n",
        b"GET /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}",
        b"POST /lms/proxy/anthropic/v1/files HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}",
        b"POST /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nContent-Length: 8\r\n\r\nnot json",
        b'POST /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nContent-Length: 45\r\n\r\n{"tools": [{"type": "web_fetch_20250910"}]}  ',
        b"POST /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n",
        b"POST /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nContent-Length: 2\r\nAnthropic-Beta: claude-code-20250219\r\n files-api-2025-04-14\r\n\r\n{}",
    ):
        assert rig.send(raw) in {400, 403, 405}
    assert rig.seen == []


def test_a_request_goes_with_the_key_its_host_and_allowed_headers(
    relay: Rig,
) -> None:
    goes_with_the_key_its_host_and_allowed_headers(relay)


def test_absolute_connect_and_traversal_targets_never_reach_the_api(
    relay: Rig,
) -> None:
    never_reach_the_api(relay)


def test_beside_the_scan_a_request_goes_with_the_key_its_host_and_allowed_headers(
    scanning: Rig,
) -> None:
    goes_with_the_key_its_host_and_allowed_headers(scanning)


def test_beside_the_scan_absolute_connect_and_traversal_targets_never_reach_the_api(
    scanning: Rig,
) -> None:
    never_reach_the_api(scanning)


def scan_request(*files: tuple[str, str]) -> bytes:
    body = "".join(
        f'[[file]]\npath = "{path}"\nhead = """{head}"""\n'
        for path, head in files
    ).encode()
    return (
        b"POST /cleanroom/scan HTTP/1.1\r\nContent-Length: %d\r\n\r\n"
        % len(body)
    ) + body


def test_the_box_gets_counts_and_lines_and_the_scan_is_logged_not_forwarded(
    scanning: Rig,
) -> None:
    reply = scanning.exchange(
        scan_request(
            ("crates/clean/a/src/lib.rs", "fn f() {}\nlet quux = 1;\n"),
            ("crates/clean/a/src/b.rs", "fine\n"),
        )
    )
    assert reply.startswith(b"HTTP/1.1 200"), reply
    assert reply.endswith(
        b"\r\n\r\ncrates/clean/a/src/lib.rs\t1\t2\ncrates/clean/a/src/b.rs\t0\t\n"
    ), reply
    assert scanning.seen == []
    log = scanning.log()
    assert "call 1: 2 files" in log
    assert "crates/clean/a/src/lib.rs: 1 hits\n    2: let quux = 1;" in log


def test_only_the_model_s_files_get_hit_lines(scanning: Rig) -> None:
    reply = scanning.exchange(
        scan_request(
            ("crates/clean/types/src/lib.rs", "quux\nquux\n"),
            ("crates/clean/a/tests/v.rs", "quux\n"),
            ("crates/clean/a/Cargo.toml", "quux\n"),
        )
    )
    assert reply.endswith(
        b"\r\n\r\ncrates/clean/types/src/lib.rs\t2\t\ncrates/clean/a/tests/v.rs\t1\t1\ncrates/clean/a/Cargo.toml\t1\t\n"
    ), reply


def test_a_call_and_a_session_get_capped_bytes_of_new_lines(
    scanning: Rig,
) -> None:
    line = "x" * 1023 + "\n"
    big = scan_request(("crates/clean/a/src/lib.rs", line * 257))
    assert scanning.send(big) == 413
    part = line * 200
    for i in range(10):
        request = scan_request((f"crates/clean/a/src/f{i}.rs", part))
        assert scanning.send(request) == 200
    again = scan_request(("crates/clean/a/src/f0.rs", part))
    assert scanning.send(again) == 200
    over = scan_request(("crates/clean/a/src/f10.rs", part))
    assert scanning.send(over) == 429
    old = scanning.lane / "baseline/crates/clean/a/src/old.rs"
    old.parent.mkdir(parents=True)
    _ = old.write_text(part)
    changed = scan_request(("crates/clean/a/src/old.rs", f"{part}y\n"))
    assert scanning.send(changed) == 200
    log = scanning.log()
    assert "over 262144 a call" in log
    assert "2048000 new this session" in log
    assert "2 new, 2048002 new this session" in log


def test_a_request_names_no_start_and_pays_for_each_file_it_sends(
    scanning: Rig,
) -> None:
    w = "let guess = 1;\n" * 10_000
    body = f'[[file]]\npath = "crates/clean/a/src/w.rs"\nstart = """{w}"""\nhead = """{w}"""\n'.encode()
    started = (
        b"POST /cleanroom/scan HTTP/1.1\r\nContent-Length: %d\r\n\r\n"
        % len(body)
        + body
    )
    assert scanning.send(started) == 400
    twice = scan_request(
        ("crates/clean/a/src/w.rs", w), ("crates/clean/a/src/w.rs", w)
    )
    assert scanning.send(twice) == 413
    assert scanning.send(scan_request(("crates/clean/a/src/w.rs", w))) == 200
    assert f"{2 * len(w)} new bytes, over" in scanning.log()


def files(call: int, count: int) -> bytes:
    return scan_request(
        *((f"crates/clean/a/src/f{call}_{i}.rs", "x\n") for i in range(count))
    )


def test_a_call_and_a_session_get_capped_files(scanning: Rig) -> None:
    assert scanning.send(files(0, SCAN_FILES + 1)) == 413
    for call in range(SCAN_SESSION_FILES // SCAN_FILES):
        assert scanning.send(files(call, SCAN_FILES)) == 200
    assert scanning.send(files(0, 1)) == 200
    assert scanning.send(files(99, 1)) == 429


def test_an_answer_that_holds_more_than_counts_and_lines_never_reaches_the_box(
    scanning: Rig,
) -> None:
    reply = scanning.exchange(
        scan_request(("crates/clean/a/src/leak.rs", "quux\n"))
    )
    assert reply.startswith(b"HTTP/1.1 500"), reply
    assert b"quux" not in reply


def test_a_request_that_is_not_a_scan_is_refused(scanning: Rig) -> None:
    for raw in (
        scan_request(("docs/behaviours/x.md", "x")),
        scan_request(("crates/clean/../../x", "x")),
        b"POST /cleanroom/scan HTTP/1.1\r\nContent-Length: 3\r\n\r\n{x}",
        b"GET /cleanroom/scan HTTP/1.1\r\n\r\n",
    ):
        assert scanning.send(raw) == 400
    assert scanning.seen == []


def test_a_session_gets_a_capped_number_of_scans(scanning: Rig) -> None:
    request = scan_request(("crates/clean/a/src/lib.rs", "fine\n"))
    statuses = [scanning.send(request) for _ in range(SCAN_CALLS + 1)]
    assert statuses[:-1] == [200] * SCAN_CALLS
    assert statuses[-1] == 429
    assert f"call {SCAN_CALLS + 1}: refused" in scanning.log()


def test_the_relay_exits_with_its_parent() -> None:
    assert not orphaned(os.getppid())
    assert orphaned(os.getpid())
    assert orphaned(HIGHEST_PID + 1)
