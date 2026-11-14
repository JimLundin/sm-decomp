"""Synthetic closed-tool acceptance checks; no API, credentials or game material."""

import json
import io
from contextlib import redirect_stderr
import os
import shutil
import sys
import tempfile
import subprocess
import tomllib
import time
import unittest
from pathlib import Path
from typing import Any
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))

from launch_stream import capture
from responses_controller import Broker, Refused, OBSERVED_MODEL, prepare, response_loop, unique_object


class Contract(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.base = Path(self.temp.name)
        self.root = self.base / "export"
        self.lane = self.base / "private"
        self.lane.mkdir()
        (self.root / "crates/clean/types/src").mkdir(parents=True)
        (self.root / "crates/clean/demo/tests").mkdir(parents=True)
        (self.root / "crates/clean/types/src/lib.rs").write_text("//! Types.\n", encoding="utf-8")
        (self.root / "Cargo.toml").write_text("# Immutable.\n", encoding="utf-8")
        (self.root / ".git").mkdir()
        (self.root / ".git/config").write_text("private metadata", encoding="utf-8")
        self.config: dict[str, Any] = {
            "writable": ["crates/clean/types/", "crates/clean/demo/"],
            "trusted_files": [], "environment": {}, "commands": {},
        }
        self.broker = Broker(self.root, self.lane, self.config)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def call(self, action: str, **fields: str) -> str:
        return self.broker.dispatch("cleanroom", {"action": action, **fields})

    def test_all_probe_denials(self) -> None:
        # Each original launch_probe CHECK has the same denied capability here.
        checks: list[tuple[str, dict[str, str]]] = [
            ("cleanroom", {"action": "read", "path": "C:/Users/JIM~1.LUN/record.txt"}),
            ("cleanroom", {"action": "read", "path": "C:/Users/Jim.Lundin/record.html"}),
            ("cleanroom", {"action": "search", "path": "C:/Users/Jim.Lundin/logs", "text": "speed"}),
            ("cleanroom", {"action": "list", "path": "../**"}),
            ("Bash", {"action": "git -C external show HEAD"}),
            ("Bash", {"action": "git show HEAD:CLEANROOM.md"}),
            ("Bash", {"action": "cat ../anything"}),
            ("Bash", {"action": "cd .. and ls"}),
            ("Bash", {"action": "curl https://example.invalid"}),
            ("WebFetch", {"action": "fetch"}),
            ("WebSearch", {"action": "search"}),
            ("cleanroom", {"action": "read", "path": "C:/Users/user/.claude/memory/MEMORY.md"}),
            ("Agent", {"action": "spawn"}),
            ("cleanroom", {"action": "write", "path": str(self.lane / "config"), "text": "{}"}),
            ("PowerShell", {"action": "read tokens"}),
            ("cleanroom", {"action": "check", "args": "--manifest-path ../Cargo.toml"}),
            ("cleanroom", {"action": "write", "path": "crates/clean/types/build.rs", "text": "fn main() {}"}),
            ("cleanroom", {"action": "write", "path": ".git/hooks/pre-commit", "text": "exit 0"}),
            ("Bash", {"action": "python -c print(1)"}),
            ("cleanroom", {"action": "runner", "args": "--help"}),
            ("Bash", {"action": "python -I other.py"}),
            ("Bash", {"action": "py -c print(1)"}),
            ("Bash", {"action": "python3 -c print(1)"}),
            ("cleanroom", {"action": "write", "path": "crates/clean/types/vector_runner.py", "text": "print(1)"}),
            ("cleanroom", {"action": "commit", "text": "Clean types; a probe marker"}),
        ]
        # Commit validation must precede code_check, as malformed requests start no process.
        self.config["commands"]["code_check"] = [sys.executable, "-c", "raise SystemExit(99)"]
        for number, (name, args) in enumerate(checks, 1):
            with self.subTest(check=number):
                with self.assertRaises(Refused):
                    self.broker.dispatch(name, args)

    def test_path_mutants(self) -> None:
        for path in ["../file", "x/../../file", "C:/file", "C:\\file", "//host/file", "a:stream",
                     "a/./b", "a//b", "a/..", "NUL.rs", "a/COM1", "file.", ".GIT/config",
                     ".GiT/objects/a", "TARGET/a", "a\\b", "%USERPROFILE%/a"]:
            with self.subTest(path=path), self.assertRaises(Refused):
                self.broker.path(path)

    def test_protected_mutants(self) -> None:
        for path in ["Cargo.toml", ".cargo/config.toml", "rust-toolchain.toml", "vendor/a.rs",
                     "docs/behaviours/demo.md", "crates/clean/demo/Cargo.toml", "crates/clean/demo/src/a.py",
                     "crates/clean/other/src/lib.rs", "crates/clean/demo/src/other/build.rs"]:
            # build.rs below src is ordinary module source, not a build script.
            if path.endswith("src/other/build.rs"):
                continue
            with self.subTest(path=path), self.assertRaises(Refused):
                self.call("write", path=path, text="")

    def test_legitimate_files(self) -> None:
        path = "crates/clean/types/src/probe.rs"
        self.assertEqual(self.call("write", path=path, text="//! Marker.\npub const PROBE: u8 = 1;\n"), "ok")
        self.call("edit", path=path, old="PROBE", text="MARKER")
        self.assertIn("MARKER", self.call("read", path=path))
        self.assertIn(path, self.call("list", path="."))
        self.assertIn(path + ":2:", self.call("search", path=".", text="MARKER"))
        self.assertNotIn(".git", self.call("list", path="."))
        self.call("delete", path=path)
        self.assertFalse((self.root / path).exists())

    def test_large_utf8_read_pages_are_exact_and_read_only(self) -> None:
        rel = "crates/clean/demo/tests/large.rs"
        path = self.root / rel
        content = "é漢😀\r\n" * 35000 + "Final page.\n"
        path.write_text(content, encoding="utf-8", newline="")
        self.assertGreater(path.stat().st_size, 100000)
        self.config["mode"] = "blind-reviewer"
        self.broker = Broker(self.root, self.lane, self.config)
        before = path.read_bytes()
        original_open = Path.open
        def read_only_open(item: Path, mode: str = "r", *args: Any, **kwargs: Any) -> Any:
            self.assertFalse(any(flag in mode for flag in "wax+"))
            return original_open(item, mode, *args, **kwargs)
        pages = []
        offset = 0
        with patch.object(Path, "open", read_only_open):
            # The default first call works for large files, without a prior size probe.
            page = json.loads(self.broker.dispatch("cleanroom", {"action": "read", "path": rel}))
            while True:
                self.assertEqual(page["offset"], offset)
                self.assertLessEqual(len(page["text"].encode("utf-8")), 100000)
                self.assertEqual(page["next_offset"], offset + len(page["text"]))
                pages.append(page["text"])
                if page["eof"]:
                    break
                self.assertGreater(page["next_offset"], offset)
                offset = page["next_offset"]
                page = json.loads(self.broker.dispatch("cleanroom", {
                    "action": "read", "path": rel, "offset": offset, "limit": 12345}))
        self.assertEqual("".join(pages), content)
        self.assertEqual(path.read_bytes(), before)
        eof = json.loads(self.broker.dispatch("cleanroom", {
            "action": "read", "path": rel, "offset": len(content), "limit": 1}))
        self.assertTrue(eof["eof"])
        self.assertEqual(eof["text"], "")

    def test_page_bounds_and_paths_are_refused(self) -> None:
        rel = "crates/clean/types/src/lib.rs"
        bounds: list[dict[str, object]] = [
            {"offset": -1}, {"limit": 0}, {"limit": 25001}, {"offset": True},
            {"offset": "0"}, {"limit": 1.5}, {"offset": 1000000}, {"limit": None}]
        for fields in bounds:
            with self.subTest(fields=fields), self.assertRaises(Refused):
                request: dict[str, object] = {"action": "read", "path": rel, **fields}
                self.broker.dispatch("cleanroom", request)
        for path in ("../private", ".GIT/config", "target/output", "file:stream", "C:/outside"):
            with self.subTest(path=path), self.assertRaises(Refused):
                self.broker.dispatch("cleanroom", {"action": "read", "path": path, "offset": 0, "limit": 1})
        with self.assertRaises(Refused):
            self.broker.dispatch("cleanroom", {"action": "status", "offset": 0})

    def test_large_file_search_finds_tail_and_reports_limits(self) -> None:
        rel = "crates/clean/demo/tests/search.rs"
        path = self.root / rel
        path.write_text("ordinary line\n" * 15000 + "tail sentinel\n", encoding="utf-8")
        self.assertGreater(path.stat().st_size, 100000)
        result = json.loads(self.call("search", path=rel, text="tail sentinel"))
        self.assertFalse(result["truncated"])
        self.assertEqual(result["matches"], [rel + ":15001:tail sentinel"])
        limited = json.loads(self.call("search", path=rel, text="ordinary"))
        self.assertTrue(limited["truncated"])
        self.assertLessEqual(len(limited["matches"]), 1000)

    def test_duplicate_and_tool_mutants(self) -> None:
        with self.assertRaises(Refused):
            json.loads('{"action":"read","action":"write"}', object_pairs_hook=unique_object)
        for name, args in [("mcp__tool", {}), ("cleanroom", {"action": "read", "path": 1}),
                           ("cleanroom", {"action": "runner", "env": {"TOKEN": "bad"}}),
                           ("cleanroom", {"action": "check", "args": []})]:
            with self.subTest(name=name, args=args), self.assertRaises(Refused):
                self.broker.dispatch(name, args)

    def test_environment_mutants(self) -> None:
        for key in ["FOUNDRY_LMS_TOKEN", "HTTP_PROXY", "CODEX_SESSION_ID", "GIT_CONFIG_COUNT",
                    "RUSTFLAGS", "PYTHONPATH", "CARGO_BUILD_RUSTC_WRAPPER"]:
            self.config["environment"] = {key: "host-secret"}
            with self.subTest(key=key), self.assertRaises(Refused):
                Broker(self.root, self.lane, self.config)

    def test_frozen_and_target(self) -> None:
        (self.root / "target").mkdir()
        (self.root / "target/output").write_text("build", encoding="utf-8")
        self.broker.audit()
        with self.assertRaises(Refused):
            self.broker.path("target/output")
        (self.root / "Cargo.toml").write_text("mutant", encoding="utf-8")
        with self.assertRaises(Refused):
            self.broker.audit()

    def test_hardlink_and_symlink(self) -> None:
        outside = self.base / "outside"
        outside.write_text("private", encoding="utf-8")
        inside = self.root / "crates/clean/types/src/link.rs"
        os.link(outside, inside)
        with self.assertRaises(Refused):
            self.broker.path("crates/clean/types/src/link.rs")
        inside.unlink()
        try:
            inside.symlink_to(outside)
        except OSError:
            return  # Windows developer-mode is not guaranteed; hardlink case always runs.
        with self.assertRaises(Refused):
            self.broker.path("crates/clean/types/src/link.rs")

    def test_fixed_commands_and_clean_environment(self) -> None:
        fixture = self.lane / "command.py"
        fixture.write_text("import os,json;print(json.dumps(dict(os.environ)))", encoding="utf-8")
        for action in ("check", "clippy", "fmt", "test", "runner", "status", "diff", "log", "add", "commit", "code_check"):
            self.config["commands"][action] = [sys.executable, "-I", str(fixture)]
        for action in ("check", "clippy", "fmt", "test", "runner", "status", "diff", "log", "add", "commit"):
            with self.subTest(action=action):
                result = json.loads(self.call(action, **({"text": "Clean types: marker"} if action == "commit" else {})))
                self.assertEqual(result["exit_code"], 0)
                env = json.loads(result["output"])
                self.assertNotIn("FOUNDRY_LMS_TOKEN", env)
                self.assertNotIn("HTTP_PROXY", env)

    def test_response_identity_completion_reasoning(self) -> None:
        final = {"model": OBSERVED_MODEL, "status": "completed", "output": [
            {"type": "message", "role": "assistant", "status": "completed", "content": [{"type": "output_text", "text": "Done"}]}]}
        reasoning = {"type": "reasoning", "id": "r1", "encrypted_content": "invented", "summary": []}
        first = {"model": OBSERVED_MODEL, "status": "completed", "output": [reasoning,
                 {"type": "function_call", "status": "completed", "name": "cleanroom", "call_id": "c1",
                  "arguments": '{"action":"read","path":"missing"}'}]}
        requests = []
        def send(request: dict[str, Any]) -> dict[str, Any]:
            requests.append(json.loads(json.dumps(request)))
            return first if len(requests) == 1 else final
        response_loop(self.broker, "Invented brief", send, self.lane / "transcript")
        self.assertIn(reasoning, requests[1]["input"])
        self.assertEqual(requests[0]["include"], ["reasoning.encrypted_content"])
        self.assertEqual(len(requests[0]["tools"]), 1)
        self.assertFalse(requests[0]["store"])
        for label in (OBSERVED_MODEL, "gpt-6-1-sol-dz", "gpt-6-1-sol-gs"):
            response_loop(self.broker, "Invented", lambda _: dict(final, model=label), self.lane / "transcript")
        for mutant in [dict(final, model="other"), dict(final, model="gpt-6-1-sol-dz-other"),
                       dict(final, model="gpt-6-1-sol-gs-other"),
                       dict(final, model="global.openai.gpt-6-sol"), dict(final, model="gpt-6.1-sol"),
                       dict(final, status="incomplete"),
                       dict(final, error={"code": "failed"}), dict(final, output=[]),
                       dict(final, output=[{"type": "web_search_call"}]),
                       dict(final, output=[{"type": "message", "status": "completed", "role": "assistant",
                                            "content": [{"type": "refusal"}]}])]:
            with self.subTest(mutant=mutant), self.assertRaises(Refused):
                response_loop(self.broker, "Invented", lambda _: mutant, self.lane / "transcript")
        diagnostics = io.StringIO()
        with redirect_stderr(diagnostics), self.assertRaises(Refused):
            response_loop(self.broker, "Invented", lambda _: dict(final, model="invalid\nbody"), self.lane / "transcript")
        self.assertNotIn("body", diagnostics.getvalue())


class Streams(unittest.TestCase):
    def test_stderr_descendant_completion(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            code = "import subprocess,sys; subprocess.Popen([sys.executable,'-I','-c',\"import time,sys;time.sleep(.3);sys.stderr.write('late')\"],stdout=subprocess.DEVNULL);print('done')"
            start = time.monotonic()
            status = capture([sys.executable, "-I", "-c", code], raw, str(root / "out"), str(root / "err"),
                             2, validate_result=False, environment={})
            self.assertEqual(status, 0)
            self.assertGreaterEqual(time.monotonic() - start, .3)
            self.assertEqual((root / "err").read_text(), "late")

    def test_runtime_timeout_uncertified(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            status = capture([sys.executable, "-I", "-c", "import time;time.sleep(5)"], raw,
                             str(root / "out"), str(root / "err"), .2,
                             validate_result=False, environment={}, runtime_timeout=.1)
            self.assertEqual(status, 124)


class Preparation(unittest.TestCase):
    @unittest.skipUnless(os.name == "nt", "Windows launcher")
    def test_wrapper_prepares_only_and_forwards_role(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            base = Path(raw)
            here = Path(__file__).resolve().parent
            shutil.copyfile(here / "launch_responses.sh", base / "launch_responses.sh")
            (base / "responses_controller.py").write_text(
                "import sys\n"
                "assert '--prepare' in sys.argv\n"
                "assert sys.argv[sys.argv.index('--mode')+1]=='blind-reviewer'\n"
                "print('invented-controller.toml')\n", encoding="utf-8")
            env = {"SYSTEMROOT": os.environ["SYSTEMROOT"], "PATH": os.environ.get("PATH", ""),
                   "CLEANROOM_PY": sys.executable, "CLEANROOM_BASH": str(Path(os.environ["LOCALAPPDATA"]) / "Programs/Git/bin/bash.exe"),
                   "CLEANROOM_GIT": shutil.which("git") or "git", "CLEANROOM_CARGO": "C:/dummy/cargo.exe"}
            result = subprocess.run([env["CLEANROOM_BASH"], str(base / "launch_responses.sh"),
                                     "--prepare-only", "--mode", "blind-reviewer", "export", "brief", "transcript", "lane"],
                                    cwd=base, env=env, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(), "invented-controller.toml")

    def test_roles_staging_and_ignored_files(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            base = Path(raw)
            export = base / "export"
            home = base / "home"
            (home / ".codex").mkdir(parents=True)
            (home / ".codex/config.toml").write_text("# Invented provider configuration.\n", encoding="utf-8")
            for folder in ("crates/clean/types/src", "crates/clean/demo/src", "docs/behaviours", "scenarios/vectors"):
                (export / folder).mkdir(parents=True)
            for crate in ("types", "demo"):
                (export / f"crates/clean/{crate}/Cargo.toml").write_text(
                    f'[package]\nname = "{crate}"\nversion = "0.1.0"\nedition = "2024"\n', encoding="utf-8")
                (export / f"crates/clean/{crate}/src/lib.rs").write_text("//! Invented model.\n", encoding="utf-8")
            (export / ".gitignore").write_text("target/\nsecret.note\n", encoding="utf-8")
            (export / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/clean/types", "crates/clean/demo"]\n', encoding="utf-8")
            (export / "docs/behaviours/demo.md").write_text("Invented behaviour.\n", encoding="utf-8")
            brief = base / "brief.txt"
            brief.write_text("Build the invented model.\n", encoding="utf-8")
            git = shutil.which("git")
            assert git is not None
            env = {"PATH": os.environ.get("PATH", ""), "SYSTEMROOT": os.environ.get("SYSTEMROOT", ""),
                   "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull}
            def run(*args: str) -> str:
                return subprocess.run([git, *args], cwd=export, env=env, capture_output=True,
                                      text=True, timeout=30, check=True).stdout.strip()
            run("init", "-q", "--template=", "-b", "main")
            run("add", "--all")
            run("-c", "user.name=fixture", "-c", "user.email=fixture@invalid", "commit", "-qm", "Invented export")
            overrides = {"CLEANROOM_GIT": git}
            overrides["RUSTUP_HOME"] = os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup"))
            if os.name == "nt":
                overrides["CLEANROOM_BASH"] = str(Path(os.environ["LOCALAPPDATA"]) / "Programs/Git/bin/bash.exe")
                overrides["CLEANROOM_CARGO"] = str(Path(os.environ["USERPROFILE"]) / ".rustup/toolchains/1.99.1-x86_64-pc-windows-gnu/bin/cargo.exe")
                overrides["CARGO_TARGET_DIR"] = str(Path(os.environ["TEMP"]) / "collab/checks-can-fail/target")
            with patch.object(Path, "home", return_value=home), patch.dict(os.environ, overrides):
                def prepared(mode: str, label: str) -> Broker:
                    path = prepare(export, base / label, brief, base / (label + ".transcript"), mode=mode)
                    config = tomllib.loads(path.read_text(encoding="utf-8"))
                    return Broker(export, base / label, config)
                broker = prepared("implementer", "initial")
                broker.dispatch("cleanroom", {"action": "write", "path": "crates/clean/demo/tests/new.rs", "text": "//! A new test.\n"})
                self.assertEqual(json.loads(broker.dispatch("cleanroom", {"action": "add"}))["exit_code"], 0)
                self.assertIn("crates/clean/demo/tests/new.rs", run("diff", "--cached", "--name-only"))
                run("-c", "user.name=fixture", "-c", "user.email=fixture@invalid", "commit", "-qm", "New test", "-m", "Role: implementer")
                prepared("followup", "followup")
                (export / "secret.note").write_text("Invented forbidden extra file", encoding="utf-8")
                with self.assertRaises(Refused):
                    prepared("followup", "ignored")
                (export / "secret.note").unlink()
                with self.assertRaises(Refused):
                    prepared("blind-reviewer", "wrong-blind")
                # A code-only one-commit copy, like blind_copy.sh, excludes all observer prose.
                blind_export = base / "blind-export"
                shutil.copytree(export, blind_export, ignore=shutil.ignore_patterns(".git", "docs"))
                export = blind_export
                (export / ".agents/skills/blind-reviewer").mkdir(parents=True)
                (export / ".agents/skills/blind-reviewer/SKILL.md").write_text("Review the invented code.\n", encoding="utf-8")
                run("init", "-q", "--template=", "-b", "main")
                run("add", "--all")
                run("-c", "user.name=fixture", "-c", "user.email=fixture@invalid", "commit", "-qm", "Invented blind copy")
                blind = prepared("blind-reviewer", "blind")
                with self.assertRaises(Refused):
                    blind.dispatch("cleanroom", {"action": "write", "path": "crates/clean/types/src/lib.rs", "text": ""})
                invented_blob = base / "invented-blob"
                invented_blob.write_text("Invented unreachable object.\n", encoding="utf-8")
                run("hash-object", "-w", str(invented_blob))
                with self.assertRaises(Refused):
                    prepared("blind-reviewer", "unreachable")


if __name__ == "__main__":
    unittest.main()
