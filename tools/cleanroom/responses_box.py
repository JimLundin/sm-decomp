"""Linux execution of the closed broker's commands in the shared clean-room image."""

import os
import subprocess
import sys
import uuid
from pathlib import Path
from typing import Any

from launch_stream import capture

IMAGE = "sm-cleanroom-root:1"
HERE = Path(__file__).resolve().parent
SCAN_FILES = ("code_check.sh", "variant_import_warn.txt", "allowed_deps.txt", "check_manifest.py",
              "strip_rust.awk", "lint_scan.awk", "source_scan.awk", "shape_scan.awk")
RUNNER_FILES = ("vector_runner.py", "strip_rust.awk", "shape_scan.awk", "awk.path")
PROXIES = ("cargo", "rustc", "rustup", "rustdoc", "cargo-clippy", "clippy-driver", "cargo-fmt", "rustfmt")


class CommandBox:
    def __init__(self, root: Path, lane: Path, config: dict[str, Any], environment: dict[str, str]) -> None:
        self.root = root
        self.lane = lane
        self.config = config
        self.environment = environment
        self.docker_env = {"PATH": "/usr/bin:/bin", "HOME": str(lane / "home")}
        self.docker = ["/usr/bin/docker", "--host", "unix:///var/run/docker.sock"]
        self.image_id = ""

    def argv(self, name: str, container: str) -> list[str]:
        # The shared root image supplies the /usr-merged symlinks and empty system configuration.
        argv = self.docker + ["run", "--rm", "--pull=never", "--name", container,
                             "--network", "none", "--read-only", "--cap-drop", "ALL",
                             "--security-opt", "no-new-privileges", "--user", f"{os.getuid()}:{os.getgid()}",
                             "--workdir", str(self.root), "--pids-limit", "4096", "--memory", "12g",
                             "--tmpfs", "/tmp:rw,nosuid,nodev,mode=1777,size=2g"]

        def mount(path: Path, writable: bool = False) -> None:
            if any(char in str(path) for char in (",", "\n", "\r")) or not path.exists():
                raise ValueError("invalid container mount")
            mode = "" if writable else ",readonly"
            argv.extend(("--mount", f"type=bind,src={path},dst={path}{mode}"))

        for path in (Path("/usr"), Path("/etc/alternatives"), Path("/etc/ld.so.cache"), self.root):
            mount(path)
        mount(self.root / "target", writable=True)
        if name in ("status", "add", "commit"):
            mount(self.root / ".git", writable=True)
            for part in ("config", "info", "hooks"):
                path = self.root / ".git" / part
                if path.exists():
                    mount(path)
        if name == "fmt":
            for base in self.config["writable"]:
                for part in ("src", "tests"):
                    path = self.root / base / part
                    if path.is_dir():
                        mount(path, writable=True)
        for part in ("home", "cargo", "hooks", *RUNNER_FILES):
            mount(self.lane / part)
        for part in SCAN_FILES:
            mount(HERE / part)
        mount(HERE.parent.parent / "Cargo.toml")
        cargo_dir = Path(self.config["commands"]["check"][0]).parent
        if not cargo_dir.is_relative_to("/usr"):
            for part in PROXIES:
                path = cargo_dir / part
                if path.is_file():
                    mount(path)
        rustup = Path(self.environment["RUSTUP_HOME"]).resolve()
        if rustup.is_dir():
            mount(rustup)
        prefix = Path(sys.base_prefix)
        if not prefix.is_relative_to("/usr"):
            mount(prefix)
        for key, value in self.environment.items():
            argv.extend(("--env", f"{key}={value}"))
        # This deadline also works if the controller is killed before it can remove the box.
        seconds = self.config.get("command_seconds", 300)
        return argv + [self.image_id, "/usr/bin/timeout", "--kill-after=5", str(seconds)]

    def initialize(self) -> None:
        if not self.image_id:
            self.image_id = subprocess.check_output(
                [sys.executable, "-I", str(HERE / "box_relay.py"), "image-id", IMAGE],
                cwd=self.lane, env={**self.docker_env, "DOCKER_HOST": "unix:///var/run/docker.sock"},
                timeout=180, text=True).strip()

    def run(self, name: str, command: list[str], stdout: Path, stderr: Path) -> int:
        self.initialize()
        (self.root / "target").mkdir(exist_ok=True)
        container = "cleanroom-responses-" + uuid.uuid4().hex
        argv = self.argv(name, container)
        try:
            status = capture(argv + command, str(self.lane), str(stdout), str(stderr),
                           float(self.config.get("completion_seconds", 60)), validate_result=False,
                             environment=self.docker_env,
                             runtime_timeout=float(self.config.get("command_seconds", 300)) + 10)
            # Docker startup errors and a forced SIGKILL (including timeout escalation) leave
            # command completion uncertified.
            return 124 if status in (125, 126, 127, 137) else status
        finally:
            # Killing the Docker client alone can leave its workload on the daemon.
            removed = subprocess.run(self.docker + ["rm", "-f", container], env=self.docker_env,
                                     capture_output=True, timeout=30)
            if removed.returncode:
                # --rm may already have won the race. Verify absence through the same daemon.
                present = subprocess.run(self.docker + ["ps", "-aq", "--filter", f"name=^/{container}$"],
                                         env=self.docker_env, capture_output=True, timeout=30, check=True)
                if present.stdout.strip():
                    raise RuntimeError("container cleanup could not be certified")
