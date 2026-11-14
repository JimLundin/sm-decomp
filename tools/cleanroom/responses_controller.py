"""Closed Responses tool broker. All process commands come from trusted operator TOML."""

import argparse
import configparser
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tomllib
import urllib.request
import urllib.error
from pathlib import Path
from typing import Any

# Only this trusted directory supplies broker helpers, including under python -I.
sys.path.insert(0, str(Path(__file__).resolve().parent))

MODEL = "gpt-6.1-sol"
OBSERVED_MODEL = "global.openai.gpt-6.1-sol"
# This provider returns either the canonical identifier or its deployment label for
# the same fixed Sol request. No other label, request model or route is accepted.
OBSERVED_MODELS = frozenset({OBSERVED_MODEL, "gpt-6-1-sol-dz"})
ACTIONS = ("read", "list", "search", "write", "edit", "delete", "check", "clippy",
           "fmt", "fmtcheck", "test", "runner", "status", "diff", "log", "add", "commit")
FIELDS = {
    "read": {"path"}, "list": {"path"}, "search": {"path", "text"},
    "write": {"path", "text"}, "edit": {"path", "old", "text"}, "delete": {"path"},
    "commit": {"text"},
}
TOOL = {
    "type": "function", "name": "cleanroom", "description":
    "Export-only tools. read/list/search take a relative path (list/search may use '.'). "
    "write/edit/delete modify only Rust src/tests. edit replaces exactly one old string. "
    "Fixed commands accept no arguments; add stages all permitted changes. "
    "commit takes text paragraphs; Role: implementer is appended. runner runs vectors.",
    "strict": False,
    "parameters": {"type": "object", "properties": {
        "action": {"type": "string", "enum": list(ACTIONS)},
        "path": {"type": "string"}, "text": {"type": "string"},
        "old": {"type": "string"}}, "required": ["action"], "additionalProperties": False},
}


class Refused(ValueError):
    """A request exceeds the closed tool contract."""


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise Refused("duplicate argument key")
        result[key] = value
    return result


def safe_label(value: object) -> str:
    if isinstance(value, str) and re.fullmatch(r"[A-Za-z0-9_.-]{1,80}", value):
        return value
    return "<invalid>"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def plain(path: Path) -> None:
    """Reject links, Windows reparse points and multiply linked files."""
    info = path.lstat()
    if (stat.S_ISLNK(info.st_mode) or
            getattr(info, "st_file_attributes", 0) & 0x400 or
            (stat.S_ISREG(info.st_mode) and info.st_nlink != 1)):
        raise Refused("linked path")


def root_plain(path: Path) -> None:
    for part in [path.absolute(), *path.absolute().parents]:
        if part.exists() or part.is_symlink():
            plain(part)


def git_config_plain(root: Path) -> None:
    config = configparser.ConfigParser(strict=True)
    config.read(root / ".git" / "config", encoding="utf-8")
    if config.sections() != ["core"] or set(config["core"]) - {
            "repositoryformatversion", "filemode", "bare", "logallrefupdates", "symlinks", "ignorecase"}:
        raise Refused("unexpected repository configuration")
    if config["core"].get("bare") != "false" or config["core"].get("repositoryformatversion") != "0":
        raise Refused("unsupported repository configuration")


class Broker:
    def __init__(self, root: Path, lane: Path, config: dict[str, Any]) -> None:
        root_plain(root)
        root_plain(lane)
        self.root = root.resolve(strict=True)
        self.lane = lane.resolve(strict=True)
        if self.lane == self.root or self.root in self.lane.parents:
            raise Refused("private lane is inside export")
        self.config = config
        self.writable = tuple(config["writable"])
        self.mode = config.get("mode", "implementer")
        if self.mode not in ("implementer", "followup", "blind-reviewer"):
            raise Refused("unknown operator role")
        if not self.writable or any(not re.fullmatch(r"crates/clean/[a-z0-9_]+/", p)
                                    for p in self.writable):
            raise Refused("invalid import paths")
        self.env = dict(config["environment"])
        allowed = {"PATH", "HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "TEMP", "TMP",
                   "SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT", "CARGO_HOME", "RUSTUP_HOME",
                   "CARGO_TARGET_DIR", "RUSTC", "RUSTDOC", "MSYS", "SM_PY", "CLEANROOM_PY", "SM_SCAN_DLL_DIR", "SM_RUST_BIN"}
        if set(self.env) - allowed or any(not isinstance(v, str) for v in self.env.values()):
            raise Refused("environment not allowlisted")
        self.env.update({"GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
                         "GIT_AUTHOR_NAME": "implementer", "GIT_AUTHOR_EMAIL": "implementer@cleanroom.invalid",
                         "GIT_COMMITTER_NAME": "implementer", "GIT_COMMITTER_EMAIL": "implementer@cleanroom.invalid"})
        self.frozen: dict[Path, str] = {}
        self.inventory: set[str] = set()
        self.command_count = 0
        for path in self.files():
            plain(path)
            rel = path.relative_to(self.root).as_posix()
            if path.is_file() and not rel.startswith((".git/", "target/")) and not self.mutable(rel):
                self.frozen[path] = digest(path)
                self.inventory.add(rel)
        for raw in config["trusted_files"]:
            path = Path(raw).resolve(strict=True)
            if self.root == path or self.root in path.parents:
                raise Refused("trusted file inside export")
            plain(path)
            self.frozen[path] = digest(path)
        git_config = self.root / ".git" / "config"
        if git_config.is_file():
            self.frozen[git_config] = digest(git_config)
        for path in (self.root / ".git").rglob("*"):
            plain(path)
        self.audit()
        records = ["export\t" + str(self.root), "lane\t" + str(self.lane)]
        records += [value + "\t" + str(path) for path, value in sorted(self.frozen.items())]
        (self.lane / "frozen.tsv").write_text("\n".join(records) + "\n", encoding="utf-8", newline="\n")

    def files(self) -> list[Path]:
        found = []
        for directory, dirs, names in os.walk(self.root, followlinks=False):
            parent = Path(directory)
            for name in dirs + names:
                plain(parent / name)
            dirs[:] = [d for d in dirs if d.casefold() not in (".git", "target")]
            found += [parent / name for name in names]
        return found

    def mutable(self, rel: str) -> bool:
        if self.mode == "blind-reviewer":
            return False
        return any(rel.startswith(base + part) for base in self.writable for part in ("src/", "tests/")) and rel.endswith(".rs")

    def path(self, raw: str, write: bool = False) -> Path:
        if not isinstance(raw, str) or not raw or len(raw) > 240:
            raise Refused("invalid path")
        if raw == "." and not write:
            return self.root
        parts = raw.split("/")
        if any(not re.fullmatch(r"[A-Za-z0-9_.-]+", p) or p in (".", "..") or
               p.endswith((".", " ")) or re.fullmatch(r"(?i)(con|prn|aux|nul|com[1-9]|lpt[1-9])(\..*)?", p)
               for p in parts) or any(p.casefold() in (".git", "target") for p in parts):
            raise Refused("invalid path")
        if write and not self.mutable(raw):
            raise Refused("protected path")
        current = self.root
        plain(current)
        for part in parts:
            current /= part
            if current.exists() or current.is_symlink():
                plain(current)
        if not current.resolve().is_relative_to(self.root):
            raise Refused("outside export")
        return current

    def audit(self) -> None:
        root_plain(self.root)
        root_plain(self.lane)
        inventory = set()
        for path in self.files():
            plain(path)
            rel = path.relative_to(self.root).as_posix()
            if path.is_file() and not rel.startswith((".git/", "target/")) and not self.mutable(rel):
                inventory.add(rel)
        if inventory != self.inventory:
            raise Refused("protected inventory changed")
        for path, value in self.frozen.items():
            root_plain(path)
            if not path.is_file() or digest(path) != value:
                raise Refused("immutable file changed")

    def run(self, name: str, suffix: list[str] | None = None) -> str:
        self.audit()
        command = self.config["commands"][name]
        if not isinstance(command, list) or not command or not all(isinstance(x, str) for x in command):
            raise Refused("invalid trusted command")
        if not Path(command[0]).is_absolute():
            raise Refused("executable is not absolute")
        self.command_count += 1
        out = self.lane / f"command-{self.command_count:05d}.stdout"
        errors = self.lane / f"command-{self.command_count:05d}.stderr"
        # capture waits for inherited stdout EOF after the direct child exits. Any timeout
        # makes this session uncertified; it is never reported as a successful tool result.
        from launch_stream import capture
        status = capture(command + (suffix or []), str(self.root), str(out), str(errors),
                         float(self.config.get("completion_seconds", 60)), validate_result=False,
                         environment=self.env, runtime_timeout=float(self.config.get("command_seconds", 300)))
        self.audit()
        output = out.read_text(encoding="utf-8", errors="replace") + errors.read_text(encoding="utf-8", errors="replace")
        if status == 124:
            raise Refused("command completion uncertified")
        return json.dumps({"exit_code": status, "output": output[:100000]})

    def dispatch(self, name: str, args: object) -> str:
        self.audit()
        if name != "cleanroom" or not isinstance(args, dict):
            raise Refused("unknown tool")
        action = args.get("action")
        if not isinstance(action, str) or action not in ACTIONS:
            raise Refused("unknown action")
        if set(args) != {"action"} | FIELDS.get(action, set()) or any(not isinstance(v, str) for v in args.values()):
            raise Refused("malformed arguments")
        if self.mode == "blind-reviewer" and action in ("write", "edit", "delete", "fmt", "add", "commit"):
            raise Refused("reviewer is read-only")
        if action in ("read", "list", "search", "write", "edit", "delete"):
            path = self.path(args["path"], action in ("write", "edit", "delete"))
            if action == "read":
                if path.stat().st_size > 100000:
                    raise Refused("file too large")
                return path.read_text(encoding="utf-8")
            if action in ("list", "search"):
                paths = [path] if path.is_file() else []
                if path.is_dir():
                    for directory, dirs, names in os.walk(path, followlinks=False):
                        dirs[:] = sorted(d for d in dirs if d.casefold() not in (".git", "target"))
                        paths += [Path(directory) / name for name in dirs + sorted(names)]
                result = []
                for item in paths:
                    rel = item.relative_to(self.root).as_posix()
                    try:
                        self.path(rel)
                    except Refused:
                        continue
                    if action == "list":
                        result.append(rel)
                    elif item.is_file() and item.stat().st_size <= 100000:
                        try:
                            for number, line in enumerate(item.read_text(encoding="utf-8").splitlines(), 1):
                                if args["text"] in line:
                                    result.append(f"{rel}:{number}:{line}")
                        except UnicodeError:
                            continue
                    if len(result) >= 1000:
                        break
                return "\n".join(result)[:100000]
            if action == "delete":
                path.unlink()
            else:
                text = args["text"]
                if len(text) > 100000 or "\r" in text or "\x00" in text:
                    raise Refused("invalid source text")
                if action == "edit":
                    old = path.read_text(encoding="utf-8")
                    if not args["old"] or old.count(args["old"]) != 1:
                        raise Refused("edit requires one matching occurrence")
                    text = old.replace(args["old"], text, 1)
                path.parent.mkdir(parents=True, exist_ok=True)
                with path.open("w", encoding="utf-8", newline="\n") as stream:
                    stream.write(text)
            self.audit()
            return "ok"
        suffix = None
        if action == "commit":
            paragraphs = args["text"].split("\n\n")
            if not paragraphs or any(not p.strip() or re.search(r"[;&|<>$`~\r\x00]", p) or
                                     ".." in p or "Role:" in p or len(p) > 4000 for p in paragraphs):
                raise Refused("invalid commit text")
            suffix = [item for p in paragraphs + ["Role: implementer"] for item in ("-m", p)]
        if action in ("check", "clippy", "fmt", "fmtcheck", "test", "runner", "add", "commit"):
            checked = json.loads(self.run("code_check"))
            if checked["exit_code"] != 0:
                return json.dumps(checked)
        if action == "commit":
            return self.run(action, suffix)
        return self.run(action)


def response_loop(broker: Broker, brief: str, send: Any, transcript: Path) -> None:
    history: list[dict[str, Any]] = [{"role": "user", "content": brief +
        "\nUse only the cleanroom function. Run runner for vector tests. Finish with a report."}]
    seen: set[str] = set()
    with transcript.open("w", encoding="utf-8", newline="\n") as stream:
        for _ in range(int(broker.config.get("max_turns", 200))):
            broker.audit()
            result = send({"model": MODEL, "input": history, "tools": [TOOL], "store": False,
                           "include": ["reasoning.encrypted_content"],
                           "parallel_tool_calls": False, "max_output_tokens": 16000})
            if (not isinstance(result, dict) or not isinstance(result.get("model"), str) or
                    result.get("model") not in OBSERVED_MODELS or
                    result.get("status") != "completed" or result.get("error")):
                metadata = result if isinstance(result, dict) else {}
                print("response_model: " + safe_label(metadata.get("model")), file=sys.stderr)
                print("response_status: " + safe_label(metadata.get("status")), file=sys.stderr)
                raise Refused("response identity or completion failed")
            output = result.get("output")
            if not isinstance(output, list) or not output:
                raise Refused("missing output")
            stream.write(json.dumps(result) + "\n")
            stream.flush()
            calls = []
            message = False
            for item in output:
                if not isinstance(item, dict) or item.get("type") not in ("reasoning", "message", "function_call"):
                    raise Refused("unknown response output")
                if item["type"] == "message":
                    if item.get("status") != "completed" or item.get("role") != "assistant":
                        raise Refused("partial message")
                    content = item.get("content", [])
                    if not content or any(not isinstance(c, dict) or c.get("type") != "output_text" or
                                          not isinstance(c.get("text"), str) for c in content):
                        raise Refused("refusal or malformed message")
                    message = True
                if item["type"] == "function_call":
                    if item.get("status") != "completed" or not isinstance(item.get("call_id"), str) or item["call_id"] in seen:
                        raise Refused("partial or repeated tool call")
                    seen.add(item["call_id"])
                    calls.append(item)
            history.extend(output)
            if not calls:
                if not message:
                    raise Refused("no final message")
                broker.audit()
                return
            for call in calls:
                try:
                    args = json.loads(call["arguments"], object_pairs_hook=unique_object)
                    value = broker.dispatch(call["name"], args)
                except (KeyError, TypeError, json.JSONDecodeError, Refused) as error:
                    # Only fixed broker refusal messages are exposed, never host exceptions.
                    value = "refused: " + (str(error) if isinstance(error, Refused) else "malformed call")
                    if "uncertified" in value or "immutable" in value or "inventory" in value:
                        raise Refused(value) from None
                except (OSError, UnicodeError):
                    value = "refused: unavailable file or invalid text"
                event = {"type": "function_call_output", "call_id": call["call_id"], "output": value}
                stream.write(json.dumps(event) + "\n")
                stream.flush()
                history.append(event)
        raise Refused("turn limit reached")


def prepare(export: Path, lane: Path, brief: Path, transcript: Path, probe: bool = False,
            mode: str = "implementer") -> Path:
    """Operator-side preparation; never invoked by a model tool."""
    root_plain(export)
    export = export.resolve(strict=True)
    git_config_plain(export)
    lane.mkdir(parents=True, exist_ok=False)
    lane = lane.resolve(strict=True)
    transcript = transcript.resolve()
    if transcript.is_relative_to(export) or lane.is_relative_to(export):
        raise Refused("private files inside export")
    if transcript.exists():
        raise Refused("transcript already exists")
    root_plain(transcript.parent)
    transcript.parent.mkdir(parents=True, exist_ok=True)
    for part in ("home", "tmp", "cargo", "hooks"):
        (lane / part).mkdir()
    here = Path(__file__).resolve().parent
    paths = {}
    for name in ("git", "cargo", "bash"):
        found = os.environ.get("CLEANROOM_" + name.upper()) or shutil.which(name)
        if found is None:
            raise Refused("missing operator executable")
        candidate = Path(found).absolute()
        if not candidate.is_file():
            raise Refused("missing operator executable")
        # Preserve argv[0]: rustup's cargo/rustc symlinks select their command by name.
        paths[name] = str(candidate)
    python = str(Path(sys.executable).resolve(strict=True))
    home = Path.home()
    git_install = Path(paths["bash"]).parent.parent
    environment = {
        "PATH": os.pathsep.join(dict.fromkeys([str(Path(p).parent) for p in paths.values()] +
                                              [str(git_install / "usr/bin"), str(git_install / "mingw64/bin")] +
                                              [str(Path(os.environ.get("SYSTEMROOT", "C:/Windows")) / "System32")])),
        "HOME": str(lane / "home"), "USERPROFILE": str(lane / "home"),
        "APPDATA": str(lane / "home" / "AppData" / "Roaming"),
        "LOCALAPPDATA": str(lane / "home" / "AppData" / "Local"),
        "TEMP": str(lane / "tmp"), "TMP": str(lane / "tmp"),
        "SYSTEMROOT": os.environ.get("SYSTEMROOT", "C:/Windows"),
        "WINDIR": os.environ.get("WINDIR", "C:/Windows"),
        "CARGO_HOME": str(lane / "cargo"),
        "RUSTUP_HOME": os.environ.get("RUSTUP_HOME", str(home / ".rustup")),
        "MSYS": "winsymlinks:native",
        "SM_PY": python, "CLEANROOM_PY": python,
        "SM_SCAN_DLL_DIR": str(lane / "scan-dlls"),
        "SM_RUST_BIN": str(Path(paths["cargo"]).parent),
        "CARGO_TARGET_DIR": str(export / "target"),
    }
    git = [paths["git"], "-c", "core.hooksPath=" + str(lane / "hooks"), "-c", "core.fsmonitor=false",
           "-c", "core.pager=cat", "-c", "diff.external=", "-c", "core.attributesFile=" + os.devnull]
    safe_env = dict(environment, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
    scan_env = dict(safe_env)
    scan_env["CARGO_TARGET_DIR"] = os.environ.get("CLEANROOM_SCAN_TARGET", os.environ.get(
        "CARGO_TARGET_DIR", str(Path(os.environ.get("TEMP", str(lane))) / "collab/checks-can-fail/target")))
    def operator(command: list[str], cwd: Path = export) -> str:
        result = subprocess.run(command, cwd=cwd, env=scan_env if cwd != export else safe_env, capture_output=True,
                                text=True, timeout=180, check=True)
        return result.stdout.strip()
    models = operator(git + ["ls-tree", "-r", "--name-only", "HEAD", "crates/clean/"])
    folders = [p.removesuffix("/Cargo.toml") for p in models.splitlines()
               if p.endswith("/Cargo.toml") and p != "crates/clean/types/Cargo.toml"]
    if len(folders) != 1:
        raise Refused("export must have one model")
    domain = folders[0].split("/")[-1].replace("_", "-")
    if probe and (domain != "launch-probe" or brief.resolve() != (here / "responses_probe.txt").resolve()):
        raise Refused("probe only uses committed dummy brief and launch-probe export")
    # The private record must lie outside every repository, not only this export.
    found_repo = subprocess.run(git + ["rev-parse", "--git-dir"], cwd=lane, env=safe_env,
                                capture_output=True, timeout=30)
    if found_repo.returncode == 0:
        raise Refused("private lane is inside a repository")
    # The same export-isolation and brief gates used by the CLI launcher.
    script = 'source "$1/lib.sh"; assert_isolated "$2" "$3" "$4"; allowed_entries "$3"; allowlist_section generated "$3"'
    isolation = operator([paths["bash"], "-c", script, "prepare", str(here), str(export), domain,
                          "0" if mode == "implementer" else "1"])
    allowed = [p for p in isolation.splitlines() if not p.startswith("  ok:")]
    if mode == "blind-reviewer":
        allowed = [p for p in allowed if not p.startswith(("docs/", ".agents/")) and p != "CLEANROOM.md"]
        allowed.append(".agents/skills/blind-reviewer/SKILL.md")
    tracked = operator(git + ["ls-tree", "-r", "--name-only", "HEAD"]).splitlines()
    root_commit = operator(git + ["rev-list", "--max-parents=0", "HEAD"])
    root_tree = operator(git + ["ls-tree", "-r", "--name-only", root_commit]).splitlines()
    files = []
    for directory, dirs, names in os.walk(export, followlinks=False):
        parent = Path(directory)
        for name in dirs + names:
            plain(parent / name)
        dirs[:] = [d for d in dirs if d.casefold() not in (".git", "target")]
        files += [(parent / name).relative_to(export).as_posix() for name in names]
    for path in tracked + root_tree + files:
        if not any(path.startswith(p) if p.endswith("/") else path == p for p in allowed):
            raise Refused("role export contains an unapproved file")
    if mode == "followup":
        commits = operator(git + ["rev-list", root_commit + "..HEAD"]).splitlines()
        for commit in commits:
            role = operator(git + ["log", "-1", "--format=%(trailers:key=Role,valueonly)", commit])
            if role.strip() != "implementer":
                raise Refused("followup contains a non-implementer commit")
            changes = operator(git + ["diff-tree", "-r", "--no-commit-id", "--no-renames", "--name-only", commit]).splitlines()
            if any(not (p.endswith(".rs") and any(p.startswith(base + part) for base in
                       ("crates/clean/types/", folders[0] + "/") for part in ("src/", "tests/"))) for p in changes):
                raise Refused("followup history changed a protected path")
    if mode == "blind-reviewer" and operator(git + ["rev-list", "--count", "--all"]) != "1":
        raise Refused("blind copy must be one fresh commit")
    if mode == "blind-reviewer":
        stored = set(operator(git + ["cat-file", "--batch-all-objects", "--batch-check=%(objectname)"]).splitlines())
        reachable = {line.split(" ", 1)[0] for line in operator(git + ["rev-list", "--objects", "--all"]).splitlines()}
        if stored != reachable:
            raise Refused("blind copy contains unreachable objects")
    if not probe:
        operator([paths["bash"], str(here / "scan_brief.sh"), str(brief.resolve(strict=True))], here.parent.parent)
    shutil.copyfile(here / "vector_runner.py", lane / "vector_runner.py")
    shutil.copyfile(brief, lane / "brief.txt")
    commands = {
        "code_check": [paths["bash"], str(here / "code_check.sh"), "--import",
                       str(export / "crates/clean/types"), str(export / folders[0])],
        "check": [paths["cargo"], "check", "--locked", "--offline", "--all-targets"],
        "clippy": [paths["cargo"], "clippy", "--locked", "--offline", "--all-targets", "--no-deps", "--", "-D", "warnings"],
        "fmt": [paths["cargo"], "fmt", "--all"],
        "fmtcheck": [paths["cargo"], "fmt", "--all", "--", "--check"],
        "test": [python, "-I", str(lane / "vector_runner.py")],
        "runner": [python, "-I", str(lane / "vector_runner.py")],
        "status": git + ["status", "--short"],
        "diff": git + ["diff", "--no-ext-diff", "--no-textconv", "HEAD", "--", "crates/clean/"],
        "log": git + ["log", "-5", "--format=%B"],
        "add": git + ["add", "--all", "--", "crates/clean/types", folders[0]],
        "commit": git + ["commit", "--no-gpg-sign"],
    }
    # Frozen manifests cannot change; staging both crate roots also covers new tests directories.
    config: dict[str, Any] = {
        "export": str(export), "lane": str(lane), "brief": str(lane / "brief.txt"),
        "transcript": str(transcript), "provider_config": str(home / ".codex" / "config.toml"),
        "writable": ["crates/clean/types/", folders[0] + "/"],
        "mode": mode,
        "trusted_files": [str(lane / "vector_runner.py"), str(lane / "brief.txt"),
                          str(here / "responses_controller.py"), str(here / "launch_stream.py")],
        "environment": environment, "commands": commands,
    }
    config["trusted_files"] += [str(p) for p in here.iterdir() if p.is_file() and p.suffix in (".sh", ".awk", ".txt")]
    config["trusted_files"].append(config["provider_config"])
    config_path = lane / "controller.toml"
    config["trusted_files"].append(str(config_path))
    def encode(value: Any) -> str:
        return json.dumps(value, ensure_ascii=True)
    lines = [key + " = " + encode(value) for key, value in config.items() if not isinstance(value, dict)]
    for section in ("environment", "commands"):
        lines += ["", "[" + section + "]"] + [key + " = " + encode(value) for key, value in config[section].items()]
    config_path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    Broker(export, lane, config)
    return config_path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("config", type=Path, nargs="?")
    parser.add_argument("--prepare", nargs=4, metavar=("EXPORT", "LANE", "BRIEF", "TRANSCRIPT"))
    parser.add_argument("--probe", action="store_true")
    parser.add_argument("--mode", choices=("implementer", "followup", "blind-reviewer"), default="implementer")
    args = parser.parse_args()
    if args.prepare:
        export, lane, brief, transcript = (Path(p) for p in args.prepare)
        print(prepare(export, lane, brief, transcript, probe=args.probe, mode=args.mode))
        return 0
    if args.config is None:
        parser.error("config or --prepare required")
    config = tomllib.loads(args.config.read_text(encoding="utf-8"))
    root, lane = Path(config["export"]), Path(config["lane"])
    broker = Broker(root, lane, config)
    provider_config = tomllib.loads(Path(config["provider_config"]).read_text(encoding="utf-8"))
    provider = provider_config["model_providers"][provider_config["model_provider"]]
    if provider.get("wire_api") != "responses" or not provider["base_url"].startswith("https://"):
        raise Refused("provider must be HTTPS Responses")
    token = os.environ[provider["env_key"]]
    # No proxy discovery and no redirects (credentials never follow a redirected URL).
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, req: Any, fp: Any, code: int, msg: str, headers: Any, newurl: str) -> None:
            return None
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
    def send(body: dict[str, Any]) -> Any:
        request = urllib.request.Request(provider["base_url"].rstrip("/") + "/responses",
                                         data=json.dumps(body).encode(),
                                         headers={"Authorization": "Bearer " + token, "Content-Type": "application/json"})
        with opener.open(request, timeout=180) as response:
            return json.load(response)
    brief_text = Path(config["brief"]).read_text(encoding="utf-8")
    response_loop(broker, brief_text, send, Path(config["transcript"]))
    print("RESPONSES COMPLETE (operator must run import and canary checks)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except urllib.error.HTTPError as failure:
        print(f"RESPONSES FAILED: HTTP {failure.code}", file=sys.stderr)
        try:
            detail = json.loads(failure.read(16384)).get("error", {})
            for field in ("type", "code"):
                value = detail.get(field)
                if isinstance(value, str) and re.fullmatch(r"[A-Za-z0-9_.-]{1,80}", value):
                    print(f"error_{field}: {value}", file=sys.stderr)
        except (ValueError, AttributeError, OSError):
            pass
        sys.exit(1)
    except Refused as failure:
        print("RESPONSES FAILED: " + str(failure), file=sys.stderr)
        sys.exit(1)
    except Exception as failure:
        print("RESPONSES FAILED: " + type(failure).__name__, file=sys.stderr)
        sys.exit(1)
