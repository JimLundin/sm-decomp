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
        self.assertEqual(env, {"ANTHROPIC_AUTH_TOKEN": "cleanroom-relay",
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
        self.relay = subprocess.Popen([sys.executable, "-I", str(HERE / "box_relay.py"), "serve", self.sock, "-"],
                                      stdin=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.relay.stdin.write(json.dumps(auth))
        self.relay.stdin.close()
        for _ in range(100):
            if os.path.exists(self.sock) and pathlib.Path(self.sock).is_socket():
                break
            time.sleep(0.05)

    def tearDown(self):
        self.relay.kill()
        self.relay.wait()
        self.relay.stderr.close()
        self.api.shutdown()
        self.api.server_close()
        self.dir.cleanup()

    def send(self, raw: bytes) -> int:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.connect(self.sock)
        s.sendall(raw)
        data = b""
        while chunk := s.recv(65536):
            data += chunk
        s.close()
        return int(data.split(b" ", 2)[1])

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
                    b"POST /lms/proxy/anthropic/v1/messages HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n"]:
            with self.subTest(raw=raw[:40]):
                self.assertIn(self.send(raw), (400, 403, 405))
        self.assertEqual(self.seen, [])


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
