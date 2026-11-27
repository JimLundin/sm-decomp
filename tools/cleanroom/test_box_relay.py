"""box_relay.py against a fake API: what passes, what is refused, and what the API sees."""

import http.server
import importlib.util
import json
import os
import pathlib
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest

HERE = pathlib.Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("box_relay", HERE / "box_relay.py")
box_relay = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(box_relay)

PREFIX = ["lms", "proxy", "anthropic"]


class Target(unittest.TestCase):
    def test_the_two_calls_of_a_session_pass(self):
        for t in ["/lms/proxy/anthropic/v1/messages", "/lms/proxy/anthropic/v1/messages?beta=true",
                  "/lms/proxy/anthropic/v1/messages/count_tokens"]:
            with self.subTest(t=t):
                self.assertEqual(box_relay.allowed_target(t, PREFIX), t)

    def test_every_other_api_path_is_refused(self):
        for t in ["/lms/proxy/anthropic/v1/models", "/lms/proxy/anthropic/v1/files",
                  "/lms/proxy/anthropic/v1/messages/batches", "/lms/proxy/anthropic/v1/skills",
                  "/lms/proxy/anthropic/v1/complete", "/lms/proxy/anthropic/v1/messages?beta=false",
                  "/lms/proxy/anthropic/v1/messages?beta=true&x=1", "/v1/messages"]:
            with self.subTest(t=t):
                self.assertIsNone(box_relay.allowed_target(t, PREFIX))

    def test_everything_else_is_refused(self):
        for t in ["http://evil.example/lms/proxy/anthropic/v1/messages", "evil.example:443",
                  "//evil.example/lms/proxy/anthropic/v1/messages", "/other",
                  "/lms/proxy/anthropic/admin", "/lms/proxy/anthropic/v1/../../admin",
                  "/lms/proxy/anthropic/v1/..;/..;/admin", "/lms/proxy/anthropic/v1/..\\..\\admin",
                  "/lms/proxy/anthropic/v1/%2e%2e/admin", "/lms/proxy/anthropic/v1/./messages",
                  "/lms/proxy/anthropic/v1//messages", "/lms/proxy/anthropic/v1/messages;x=1",
                  "/lms/proxy/anthropic/v1/messages?x=%0d%0a", "/lms/proxy/anthropic/v1/.hidden",
                  "/lms/proxy/anthropic/../anthropic/v1/messages", "*"]:
            with self.subTest(t=t):
                self.assertIsNone(box_relay.allowed_target(t, PREFIX))


def user(*blocks):
    return {"model": "m", "messages": [{"role": "user", "content": list(blocks)}]}


BASE64 = {"type": "base64", "media_type": "image/png", "data": "AA=="}


class Body(unittest.TestCase):
    def refusal(self, body):
        return box_relay.refusal(json.dumps(body).encode())

    def test_a_body_that_is_not_a_json_object_is_refused(self):
        for body in [b"not json", b"", b"[]", b'"text"', b"\xff"]:
            with self.subTest(body=body):
                self.assertIsNotNone(box_relay.refusal(body))

    def test_a_repeated_member_is_refused(self):
        for body in [b'{"model": "m", "tools": [{"type": "web_search_20250305"}], "tools": []}',
                     b'{"model": "m", "messages": [{"role": "user", "content": "x", "content": "y"}]}']:
            with self.subTest(body=body):
                self.assertIn("repeated member", box_relay.refusal(body) or "")

    def test_every_server_tool_is_refused(self):
        for kind in ["web_search_20250305", "web_fetch_20250910", "code_execution_20250825",
                     "bash_20250124", "text_editor_20250728", "computer_20250124", "memory_20250818",
                     "tool_search_tool_regex_20251119", "mcp_toolset", "advisor_20260301", None]:
            with self.subTest(kind=kind):
                self.assertIsNotNone(self.refusal({"model": "m", "tools": [{"type": kind, "name": "t"}]}))

    def test_a_member_that_names_an_outside_service_is_refused(self):
        for member in ["mcp_servers", "container", "betas", "service_tier"]:
            with self.subTest(member=member):
                self.assertIsNotNone(self.refusal({"model": "m", member: []}))

    def test_a_file_a_document_or_an_upload_is_refused(self):
        for block in [{"type": "document", "source": {"type": "url", "url": "https://x/a.pdf"}},
                      {"type": "document", "source": {"type": "base64", "data": "AA=="}},
                      {"type": "document", "source": {"type": "file", "file_id": "f"}},
                      {"type": "image", "source": {"type": "url", "url": "https://x/a.png"}},
                      {"type": "image", "source": {"type": "file", "file_id": "f"}},
                      {"type": "container_upload", "file_id": "f"},
                      {"type": "search_result", "source": "https://x", "title": "t", "content": []},
                      {"type": "server_tool_use", "id": "s", "name": "web_search", "input": {}},
                      {"type": "web_search_tool_result", "tool_use_id": "s", "content": []},
                      {"type": "mcp_tool_use", "id": "m", "name": "x", "server_name": "s", "input": {}},
                      {"type": "tool_result", "tool_use_id": "t", "content": [
                          {"type": "document", "source": {"type": "url", "url": "https://x/a.pdf"}}]},
                      {"type": "tool_result", "tool_use_id": "t", "content": [
                          {"type": "image", "source": {"type": "url", "url": "https://x/a.png"}}]}]:
            with self.subTest(block=block):
                self.assertIsNotNone(self.refusal(user(block)))
        self.assertIsNotNone(self.refusal({"model": "m", "system": [{"type": "document", "source": BASE64}]}))

    def test_the_cli_s_own_requests_pass(self):
        body = {"model": "m", "max_tokens": 1, "stream": True, "metadata": {"user_id": "u"},
                "thinking": {"type": "adaptive"}, "output_config": {"effort": "medium"},
                "context_management": {"edits": [{"type": "clear_thinking_20251015", "keep": "all"}]},
                "system": [{"type": "text", "text": "s", "cache_control": {"type": "ephemeral"}}],
                "tools": [{"name": "Bash", "description": "web_search url", "input_schema": {"type": "object"}},
                          {"type": "custom", "name": "Read", "input_schema": {"type": "object"}}],
                "tool_choice": {"type": "auto"},
                "messages": [
                    {"role": "user", "content": "plain"},
                    {"role": "user", "content": [{"type": "text", "text": '"type": "url"'},
                                                 {"type": "image", "source": BASE64}]},
                    {"role": "assistant", "content": [
                        {"type": "thinking", "thinking": "t", "signature": "s"},
                        {"type": "redacted_thinking", "data": "d"},
                        {"type": "tool_use", "id": "t", "name": "Bash", "input": {"url": "https://x"}}]},
                    {"role": "user", "content": [
                        {"type": "tool_result", "tool_use_id": "t", "content": "out"},
                        {"type": "tool_result", "tool_use_id": "t", "is_error": True,
                         "content": [{"type": "text", "text": "x"}, {"type": "image", "source": BASE64}]}]}]}
        self.assertIsNone(self.refusal(body))


class Betas(unittest.TestCase):
    def test_only_the_cli_s_betas_are_kept(self):
        kept, dropped = box_relay.betas(["claude-code-20250219, files-api-2025-04-14",
                                         "mcp-client-2025-04-04,effort-2025-11-24"])
        self.assertEqual(kept, ["claude-code-20250219", "effort-2025-11-24"])
        self.assertEqual(dropped, ["files-api-2025-04-14", "mcp-client-2025-04-04"])


class Env(unittest.TestCase):
    def run_settings(self, env):
        out = subprocess.run([sys.executable, "-I", str(HERE / "box_relay.py"), "env", "-", "8765"],
                             input=json.dumps({"env": env}), capture_output=True, text=True, check=True)
        return dict(line.split("=", 1) for line in out.stdout.splitlines()), out.stderr

    def test_only_the_cli_lines_enter_the_box(self):
        env, err = self.run_settings({
            "ANTHROPIC_BASE_URL": "https://gw.example/lms/proxy/anthropic", "ANTHROPIC_AUTH_TOKEN": "tok-secret",
            "ANTHROPIC_CUSTOM_HEADERS": "X-Gateway-Key: gw-secret", "ANTHROPIC_FOUNDRY_API_KEY": "fk-secret",
            "ANTHROPIC_MODEL": "opus", "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1"})
        self.assertEqual(env, {"ANTHROPIC_API_KEY": "cleanroom-relay",
                               "ANTHROPIC_BASE_URL": "http://127.0.0.1:8765/lms/proxy/anthropic",
                               "ANTHROPIC_MODEL": "opus", "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1"})
        self.assertIn("ANTHROPIC_CUSTOM_HEADERS", err)
        self.assertNotIn("secret", err)


class Relay(unittest.TestCase):
    """A relay on a Unix socket in front of a fake API on 127.0.0.1."""

    def setUp(self):
        self.seen = []
        seen = self.seen

        class Api(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                self.rfile.read(int(self.headers.get("content-length") or 0))
                seen.append((self.requestline, dict((k.lower(), v) for k, v in self.headers.items())))
                body = b"ok"
                self.send_response(200)
                self.send_header("content-length", "2")
                self.end_headers()
                self.wfile.write(body)
            do_GET = do_POST

            def log_message(self, *args):
                pass

        self.api = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Api)
        threading.Thread(target=self.api.serve_forever, daemon=True).start()
        self.dir = tempfile.TemporaryDirectory()
        self.sock = os.path.join(self.dir.name, "api.sock")
        auth = {"env": {"ANTHROPIC_BASE_URL": f"http://127.0.0.1:{self.api.server_port}/lms/proxy/anthropic",
                        "ANTHROPIC_AUTH_TOKEN": "SECRET-TOKEN", "ANTHROPIC_CUSTOM_HEADERS": "X-Gateway-Key: gw"}}
        self.relay = subprocess.Popen([sys.executable, "-I", str(HERE / "box_relay.py"), "serve", self.sock, "-",
                                       *self.extra()],
                                      stdin=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.relay.stdin.write(json.dumps(auth))
        self.relay.stdin.close()
        for _ in range(100):
            if os.path.exists(self.sock) and pathlib.Path(self.sock).is_socket():
                break
            time.sleep(0.05)

    def extra(self) -> list[str]:
        return []

    def tearDown(self):
        self.relay.kill()
        self.relay.wait()
        self.relay.stderr.close()
        self.api.shutdown()
        self.api.server_close()
        self.dir.cleanup()

    def exchange(self, raw: bytes) -> bytes:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.connect(self.sock)
        s.sendall(raw)
        data = b""
        while chunk := s.recv(65536):
            data += chunk
        s.close()
        return data

    def send(self, raw: bytes) -> int:
        return int(self.exchange(raw).split(b" ", 2)[1])

    def test_a_request_goes_with_the_key_its_host_and_allowed_headers(self):
        status = self.send(b"POST /lms/proxy/anthropic/v1/messages?beta=true HTTP/1.1\r\nHost: evil.example\r\n"
                           b"Content-Length: 2\r\nX-Api-Key: cleanroom-relay\r\nAnthropic-Version: 1\r\n"
                           b"Anthropic-Beta: claude-code-20250219,files-api-2025-04-14\r\n"
                           b"anthropic-beta: mcp-client-2025-04-04\r\n"
                           b"X-Original-URL: /admin\r\nX-Forwarded-Host: evil.example\r\nForwarded: host=evil\r\n"
                           b"Connection: close\r\n\r\n{}")
        self.assertEqual(status, 200)
        line, headers = self.seen[0]
        self.assertEqual(line, "POST /lms/proxy/anthropic/v1/messages?beta=true HTTP/1.1")
        self.assertEqual(headers["host"], f"127.0.0.1:{self.api.server_port}")
        self.assertEqual(headers["authorization"], "Bearer SECRET-TOKEN")
        self.assertEqual(headers["x-gateway-key"], "gw")
        self.assertEqual(headers["anthropic-version"], "1")
        self.assertEqual(headers["anthropic-beta"], "claude-code-20250219")
        for dropped in ["x-original-url", "x-forwarded-host", "forwarded", "x-api-key"]:
            self.assertNotIn(dropped, headers)

    def test_absolute_connect_and_traversal_targets_never_reach_the_api(self):
        for raw in [b"GET http://evil.example/lms/proxy/anthropic/v1/messages HTTP/1.1\r\nHost: evil.example\r\n\r\n",
                    b"CONNECT evil.example:443 HTTP/1.1\r\nHost: evil.example:443\r\n\r\n",
                    b"POST /lms/proxy/anthropic/v1/..;/..;/admin HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
                    b"POST /lms/proxy/anthropic/v1/..\\..\\admin HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
                    b"POST /lms/proxy/anthropic/v1/%2e%2e/admin HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
                    b"DELETE /lms/proxy/anthropic/v1/messages HTTP/1.1\r\n\r\n",
                    b"GET /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}",
                    b"POST /lms/proxy/anthropic/v1/files HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}",
                    b"POST /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nContent-Length: 8\r\n\r\nnot json",
                    b"POST /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nContent-Length: 45\r\n\r\n"
                    b'{"tools": [{"type": "web_fetch_20250910"}]}  ',
                    b"POST /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n",
                    b"POST /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nContent-Length: 2\r\n"
                    b"Anthropic-Beta: claude-code-20250219\r\n files-api-2025-04-14\r\n\r\n{}"]:
            with self.subTest(raw=raw[:40]):
                self.assertIn(self.send(raw), (400, 403, 405))
        self.assertEqual(self.seen, [])


# A scanner that answers as `cleanroom names` does, one line a file, and adds a word to its line
# for a file whose path holds "leak".
FAKE_SCANNER = """import sys, tomllib
for f in tomllib.loads(sys.stdin.read())["file"]:
    hits = f["head"].count("quux")
    lines = ",".join(str(i + 1) for i, l in enumerate(f["head"].splitlines()) if "quux" in l)
    print(f["path"] + "\\t" + str(hits) + "\\t" + lines + ("\\tquux" if "leak" in f["path"] else ""))
sys.exit(1 if "quux" in sys.stdin.read() else 0)
"""


def scan_request(*files: tuple[str, str]) -> bytes:
    body = "".join(f'[[file]]\npath = "{p}"\nhead = """{h}"""\n' for p, h in files).encode()
    return (b"POST /cleanroom/scan HTTP/1.1\r\nContent-Length: %d\r\n\r\n" % len(body)) + body


class Scan(Relay):
    """The relay's own name scan endpoint, in front of a fake scanner."""

    def extra(self) -> list[str]:
        scanner = pathlib.Path(self.dir.name, "scanner")
        scanner.write_text(f"#!{sys.executable} -I\n{FAKE_SCANNER}")
        scanner.chmod(0o700)
        return ["--scan", str(scanner), self.dir.name, "crates/clean/a"]

    def test_the_box_gets_counts_and_lines_and_the_scan_is_logged_not_forwarded(self):
        reply = self.exchange(scan_request(("crates/clean/a/src/lib.rs", "fn f() {}\nlet quux = 1;\n"),
                                           ("crates/clean/a/src/b.rs", "fine\n")))
        self.assertTrue(reply.startswith(b"HTTP/1.1 200"), reply)
        self.assertTrue(reply.endswith(b"\r\n\r\ncrates/clean/a/src/lib.rs\t1\t2\ncrates/clean/a/src/b.rs\t0\t\n"), reply)
        self.assertEqual(self.seen, [])
        log = pathlib.Path(self.dir.name, "scan.log").read_text()
        self.assertIn("call 1: 2 files", log)
        self.assertIn("crates/clean/a/src/lib.rs: 1 hits\n    2: let quux = 1;", log)

    def test_only_the_model_s_files_get_hit_lines(self):
        reply = self.exchange(scan_request(("crates/clean/types/src/lib.rs", "quux\nquux\n"),
                                           ("crates/clean/a/tests/v.rs", "quux\n"),
                                           ("crates/clean/a/Cargo.toml", "quux\n")))
        self.assertTrue(reply.endswith(b"\r\n\r\ncrates/clean/types/src/lib.rs\t2\t\n"
                                       b"crates/clean/a/tests/v.rs\t1\t1\ncrates/clean/a/Cargo.toml\t1\t\n"), reply)

    def test_a_call_and_a_session_get_capped_bytes_of_new_lines(self):
        line = "x" * 1023 + "\n"
        big = scan_request(("crates/clean/a/src/lib.rs", line * 257))
        self.assertEqual(self.send(big), 413)
        part = line * 200
        for i in range(10):
            status = self.send(scan_request((f"crates/clean/a/src/f{i}.rs", part)))
            self.assertEqual(status, 200)
        self.assertEqual(self.send(scan_request(("crates/clean/a/src/f0.rs", part))), 200)
        self.assertEqual(self.send(scan_request(("crates/clean/a/src/f10.rs", part))), 429)
        body = f'[[file]]\npath = "crates/clean/a/src/old.rs"\nstart = """{part}"""\nhead = """{part}y\n"""\n'.encode()
        self.assertEqual(self.send(b"POST /cleanroom/scan HTTP/1.1\r\nContent-Length: %d\r\n\r\n" % len(body) + body), 200)
        log = pathlib.Path(self.dir.name, "scan.log").read_text()
        self.assertIn("over 262144 a call", log)
        self.assertIn("2048000 new this session", log)

    def test_an_answer_that_holds_more_than_counts_and_lines_never_reaches_the_box(self):
        reply = self.exchange(scan_request(("crates/clean/a/src/leak.rs", "quux\n")))
        self.assertTrue(reply.startswith(b"HTTP/1.1 500"), reply)
        self.assertNotIn(b"quux", reply)

    def test_a_request_that_is_not_a_scan_is_refused(self):
        for raw in [scan_request(("docs/behaviours/x.md", "x")), scan_request(("crates/clean/../../x", "x")),
                    b"POST /cleanroom/scan HTTP/1.1\r\nContent-Length: 3\r\n\r\n{x}",
                    b"GET /cleanroom/scan HTTP/1.1\r\n\r\n"]:
            with self.subTest(raw=raw[-30:]):
                self.assertIn(self.send(raw), (400,))
        self.assertEqual(self.seen, [])

    def test_a_session_gets_a_capped_number_of_scans(self):
        request = scan_request(("crates/clean/a/src/lib.rs", "fine\n"))
        statuses = [self.send(request) for _ in range(box_relay.SCAN_CALLS + 1)]
        self.assertEqual(statuses[:-1], [200] * box_relay.SCAN_CALLS)
        self.assertEqual(statuses[-1], 429)
        self.assertIn(f"call {box_relay.SCAN_CALLS + 1}: refused", pathlib.Path(self.dir.name, "scan.log").read_text())


class Parent(unittest.TestCase):
    def test_the_relay_exits_with_its_parent(self):
        with tempfile.TemporaryDirectory() as d:
            # A shell that starts the relay in the background and is then killed outright.
            script = (f"{sys.executable} -I {HERE / 'box_relay.py'} serve {d}/api.sock - $$ "
                      f"<<<'{{\"env\": {{\"ANTHROPIC_API_KEY\": \"k\"}}}}' 2>/dev/null & echo $! >{d}/pid; sleep 30")
            shell = subprocess.Popen(["bash", "-c", script])
            for _ in range(100):
                if os.path.exists(f"{d}/api.sock"):
                    break
                time.sleep(0.05)
            relay = int(pathlib.Path(d, "pid").read_text())
            shell.kill()
            shell.wait()
            for _ in range(50):
                try:
                    os.kill(relay, 0)
                except OSError:
                    break
                time.sleep(0.1)
            else:
                os.kill(relay, 9)
                self.fail("the relay outlived its parent")


if __name__ == "__main__":
    unittest.main()
