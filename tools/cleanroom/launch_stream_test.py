"""Invented CLI fixtures; no model sessions or credentials."""

import pathlib
import os
import shutil
import subprocess
import sys
import tempfile
import time
import unittest


HERE = pathlib.Path(__file__).resolve().parent


class Completion(unittest.TestCase):
    def run_cli(self, code, timeout=0.5):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            cli = root / "cli.py"
            cli.write_text(code, encoding="utf-8", newline="\n")
            start = time.monotonic()
            result = subprocess.run(
                [sys.executable, "-I", str(HERE / "launch_stream.py"),
                 "--cwd", directory, "--transcript", str(root / "stream"),
                 "--stderr", str(root / "stderr"), "--timeout", str(timeout),
                 "--", sys.executable, "-I", str(cli)],
                capture_output=True, timeout=5,
            )
            elapsed = time.monotonic() - start
            stream = (root / "stream").read_bytes()
            errors = (root / "stderr").read_bytes()
            return result.returncode, stream, errors, elapsed

    def test_no_output_failure(self):
        status, stream, errors, _ = self.run_cli("import sys; sys.stderr.write('invented failure'); sys.exit(7)")
        self.assertEqual(status, 7)
        self.assertEqual(stream, b"")
        self.assertEqual(errors, b"invented failure")

    def test_partial_and_missing_terminal(self):
        for code in ["print('{')", "print('{\"type\":\"assistant\"}')", "pass"]:
            with self.subTest(code=code):
                self.assertNotEqual(self.run_cli(code)[0], 0)

    def test_structural_success(self):
        for event in [
            '{"type":"result","duration_ms":1}',
            '{"duration_ms":1,"subtype":"success","type":"result"}',
            '{ "type": "result", "subtype": "success" }',
        ]:
            with self.subTest(event=event):
                self.assertEqual(self.run_cli(f"print({event!r})")[0], 0)

    def test_error_result(self):
        self.assertNotEqual(self.run_cli("print('{\"type\":\"result\",\"is_error\":true}')")[0], 0)
        self.assertNotEqual(self.run_cli("print('{\"type\":\"result\",\"subtype\":\"error_during_execution\"}')")[0], 0)

    def descendant(self, trailing, delay):
        child = f"import time; time.sleep({delay}); print({trailing!r}, flush=True)"
        return (
            "import subprocess, sys\n"
            "print('{\"type\":\"result\"}', flush=True)\n"
            f"subprocess.Popen([sys.executable, '-I', '-c', {child!r}], stderr=subprocess.DEVNULL, cwd={str(HERE)!r})\n"
        )

    def test_trailing_output_waits_for_inherited_writer(self):
        trailing = '{"type":"assistant"}\n{"subtype":"success","type":"result"}'
        status, stream, _, elapsed = self.run_cli(self.descendant(trailing, 0.3), timeout=1)
        self.assertEqual(status, 0)
        self.assertIn(trailing.encode(), stream.replace(b"\r\n", b"\n"))
        self.assertGreaterEqual(elapsed, 0.3)

    def test_trailing_nonterminal_fails(self):
        self.assertNotEqual(self.run_cli(self.descendant('{"type":"assistant"}', 0.2))[0], 0)

    def test_open_pipe_timeout(self):
        status, _, _, elapsed = self.run_cli(self.descendant('{"type":"result"}', 3), timeout=0.1)
        self.assertNotEqual(status, 0)
        self.assertLess(elapsed, 2.5)

    def test_launch_failure_runs_canary_and_keeps_diagnostics(self):
        # Only preflight dependencies are faked; the launch and completion scripts are real.
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            tools = root / "tools"
            tools.mkdir()
            export = root / "export"
            export.mkdir()
            for name in ["launch.sh", "launch_stream.py"]:
                shutil.copyfile(HERE / name, tools / name)
            (tools / "vector_runner.py").write_text("# Invented runner\n", encoding="utf-8")
            (tools / "lib.sh").write_text(
                'die() { echo "$*" >&2; exit 1; }\n'
                'git_isolated() { case "$2" in rev-list) echo invented-root ;; '
                'ls-tree) echo crates/clean/demo/src/lib.rs ;; esac; }\n'
                'model_dirs() { echo crates/clean/demo; }\n'
                'domain_of() { echo demo; }\n'
                'check_domain() { :; }\n'
                'allowlist_section() { echo crates/clean/demo/; }\n',
                encoding="utf-8", newline="\n",
            )
            (tools / "scan_brief.sh").write_text("exit 0\n", encoding="utf-8")
            (tools / "canary.sh").write_text('echo checked >"$CLEANROOM_PRIVATE/canary-checked"\nexit 0\n', encoding="utf-8")
            (root / "brief").write_text("Invented brief.\n", encoding="utf-8")
            (root / "auth").write_text("", encoding="utf-8")
            cli = root / ("fake.cmd" if os.name == "nt" else "fake.sh")
            cli.write_text("@echo invented failure 1>&2\n@exit /b 7\n" if os.name == "nt" else "#!/bin/sh\necho invented failure >&2\nexit 7\n", encoding="utf-8", newline="\n")
            cli.chmod(0o755)
            env = os.environ.copy()
            env.pop("FOUNDRY_LMS_TOKEN", None)
            env.update(CLAUDE_BIN=pathlib.Path(sys.executable).as_posix() if os.name == "nt" else cli.as_posix(), CLEANROOM_PY=pathlib.Path(sys.executable).as_posix(),
                       CLEANROOM_PRIVATE=(root / "private").as_posix(), CLEANROOM_AUTH=(root / "auth").as_posix(),
                       CLEANROOM_COMPLETION_SECONDS="0.2")
            env["USERPROFILE"] = root.as_posix()
            if os.name != "nt":
                # launch.sh's Windows path conversion is immaterial to the invented export.
                binary = root / "bin"
                binary.mkdir()
                converter = binary / "cygpath"
                converter.write_text('#!/bin/sh\nshift\nprintf "%s\\n" "$1"\n', encoding="utf-8")
                converter.chmod(0o755)
                env["PATH"] = str(binary) + os.pathsep + env["PATH"]
            bash = shutil.which("bash")
            if os.name == "nt":
                bash = str(pathlib.Path(os.environ["LOCALAPPDATA"]) / "Programs/Git/bin/bash.exe")
            result = subprocess.run([bash, str(tools / "launch.sh"), str(export),
                                     str(root / "brief"), str(root / "transcript")],
                                    env=env, capture_output=True, timeout=40)
            self.assertNotEqual(result.returncode, 0)
            self.assertTrue((root / "private/canary-checked").exists(), result.stderr)
            lanes = list((root / "private/lanes").iterdir())
            self.assertEqual(len(lanes), 1)
            self.assertTrue((lanes[0] / "stderr.txt").read_bytes())
            self.assertEqual((lanes[0] / "transcript.jsonl").read_bytes(), (root / "transcript").read_bytes())


if __name__ == "__main__":
    unittest.main()
