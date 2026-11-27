"""box_relay.py against a fake API: what passes, what is refused, and what the API sees."""

import http.server
import io
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
sys.path.insert(0, str(HERE))
import box_relay  # noqa: E402

PREFIX = ["lms", "proxy", "anthropic"]


class Target(unittest.TestCase):
    def test_the_api_paths_pass(self):
        for t in ["/lms/proxy/anthropic/v1/messages", "/lms/proxy/anthropic/v1/messages?beta=true",
                  "/lms/proxy/anthropic/v1/messages/count_tokens", "/lms/proxy/anthropic/v1/models"]:
            with self.subTest(t=t):
                self.assertEqual(box_relay.allowed_target(t, PREFIX), t)

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


class Body(unittest.TestCase):
    def test_a_request_that_reaches_outside_is_refused(self):
        for body in [{"tools": [{"type": "web_search_20250305", "name": "web_search"}]},
                     {"tools": [{"type": "web_fetch_20250910", "name": "web_fetch"}]},
                     {"mcp_servers": [{"type": "url", "url": "https://x"}]},
                     {"messages": [{"role": "user", "content": [
                         {"type": "document", "source": {"type": "url", "url": "https://x/a.pdf"}}]}]}]:
            with self.subTest(body=body):
                self.assertTrue(box_relay.body_reaches_outside(json.dumps(body).encode()))

    def test_the_cli_s_own_requests_pass(self):
        body = {"model": "m", "tools": [{"name": "Bash", "description": "web_search url",
                                         "input_schema": {"type": "object"}}],
                "messages": [{"role": "user", "content": [{"type": "text", "text": '"type": "url"'}]}]}
        self.assertFalse(box_relay.body_reaches_outside(json.dumps(body).encode()))
        self.assertFalse(box_relay.body_reaches_outside(b"not json"))


class Settings(unittest.TestCase):
    def run_settings(self, env):
        out = subprocess.run([sys.executable, "-I", str(HERE / "box_relay.py"), "settings", "-", "8765"],
                             input=json.dumps({"env": env}), capture_output=True, text=True, check=True)
        return json.loads("{" + out.stdout.strip().rstrip(",") + "}")["env"], out.stderr

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

    def test_the_box_environment_is_the_same_lines(self):
        env = {"ANTHROPIC_BASE_URL": "https://gw.example/p", "ANTHROPIC_API_KEY": "k-secret"}
        out = subprocess.run([sys.executable, "-I", str(HERE / "box_relay.py"), "env", "-", "8765"],
                             input=json.dumps({"env": env}), capture_output=True, text=True, check=True)
        self.assertEqual(out.stdout.splitlines(), ["ANTHROPIC_API_KEY=cleanroom-relay",
                                                   "ANTHROPIC_BASE_URL=http://127.0.0.1:8765/p"])


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
                           b"X-Original-URL: /admin\r\nX-Forwarded-Host: evil.example\r\nForwarded: host=evil\r\n"
                           b"Connection: close\r\n\r\n{}")
        self.assertEqual(status, 200)
        line, headers = self.seen[0]
        self.assertEqual(line, "POST /lms/proxy/anthropic/v1/messages?beta=true HTTP/1.1")
        self.assertEqual(headers["host"], f"127.0.0.1:{self.api.server_port}")
        self.assertEqual(headers["authorization"], "Bearer SECRET-TOKEN")
        self.assertEqual(headers["x-gateway-key"], "gw")
        self.assertEqual(headers["anthropic-version"], "1")
        for dropped in ["x-original-url", "x-forwarded-host", "forwarded", "x-api-key"]:
            self.assertNotIn(dropped, headers)

    def test_absolute_connect_and_traversal_targets_never_reach_the_api(self):
        for raw in [b"GET http://evil.example/lms/proxy/anthropic/v1/messages HTTP/1.1\r\nHost: evil.example\r\n\r\n",
                    b"CONNECT evil.example:443 HTTP/1.1\r\nHost: evil.example:443\r\n\r\n",
                    b"POST /lms/proxy/anthropic/v1/..;/..;/admin HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
                    b"POST /lms/proxy/anthropic/v1/..\\..\\admin HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
                    b"POST /lms/proxy/anthropic/v1/%2e%2e/admin HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
                    b"DELETE /lms/proxy/anthropic/v1/messages HTTP/1.1\r\n\r\n",
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
