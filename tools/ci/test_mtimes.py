import os
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from collections.abc import Mapping
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import mtimes

MTIMES = Path(__file__).resolve().with_name("mtimes.py")
BUILT = 1_000_000_000
EVERYTHING = {"leaf.rs": True, "added.rs": True, "root.rs": True}
BUILD_CONFIG = (
    "Cargo.toml",
    "crates/leaf/Cargo.toml",
    "Cargo.lock",
    ".cargo/config.toml",
    ".cargo/config",
    "crates/leaf/.cargo/config.toml",
    "rust-toolchain.toml",
    "rust-toolchain",
    "crates/leaf/build.rs",
    "clippy.toml",
    "crates/leaf/.clippy.toml",
)


class RestoredBuild(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(
            self.enterContext(
                tempfile.TemporaryDirectory(dir=os.environ.get("TEMP"))
            )
        )
        self.origin = self.tmp / "origin"
        self.origin.mkdir()
        self.env = os.environ | {
            "GIT_AUTHOR_NAME": "mtimes-check",
            "GIT_AUTHOR_EMAIL": "mtimes-check@localhost",
            "GIT_COMMITTER_NAME": "mtimes-check",
            "GIT_COMMITTER_EMAIL": "mtimes-check@localhost",
        }
        self.git(self.origin, "init", "-q", "-b", "main", ".")
        self.git(self.origin, "config", "commit.gpgsign", "false")
        self.commit(
            {
                ".gitignore": "target/",
                "leaf.rs": "leaf",
                "root.rs": "root",
                "deleted.rs": "gone",
            },
            "built",
        )
        self.built = self.git(self.origin, "rev-parse", "HEAD")
        (self.origin / "deleted.rs").unlink()
        self.commit({"leaf.rs": "planted", "added.rs": "added"}, "changed")

    def git(self, root: Path, *args: str) -> str:
        return subprocess.run(
            ["git", *args],
            cwd=root,
            env=self.env,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    def commit(self, files: Mapping[str, str], message: str) -> None:
        for name, text in files.items():
            (self.origin / name).parent.mkdir(parents=True, exist_ok=True)
            (self.origin / name).write_text(text + "\n")
        self.git(self.origin, "add", "-A")
        self.git(self.origin, "commit", "-q", "-m", message)

    def checkout(self, *clone: str, name: str = "checkout") -> Path:
        path = self.tmp / name
        self.git(
            self.tmp, "clone", "-q", *clone, self.origin.as_uri(), str(path)
        )
        return path

    def mtimes(self, root: Path, command: str) -> str:
        return subprocess.run(
            [sys.executable, str(MTIMES), command],
            cwd=root,
            env=self.env,
            check=True,
            capture_output=True,
            text=True,
        ).stdout

    def restored(self, root: Path, record: str | None) -> dict[str, bool]:
        (root / "target").mkdir(exist_ok=True)
        if record is not None:
            (root / "target/.built-from").write_text(record + "\n")
        self.mtimes(root, "restore")
        return {
            name: (root / name).stat().st_mtime > BUILT
            for name in ("leaf.rs", "added.rs", "root.rs")
        }

    def test_a_build_recorded_at_its_commit_rebuilds_only_the_changed_sources(
        self,
    ) -> None:
        checkout = self.checkout()
        self.git(checkout, "checkout", "-q", self.built)
        self.mtimes(checkout, "record")
        self.git(checkout, "checkout", "-q", "main")
        self.assertEqual(
            self.restored(checkout, None),
            {"leaf.rs": True, "added.rs": True, "root.rs": False},
        )

    def test_a_shallow_checkout_fetches_the_recorded_commit(self) -> None:
        checkout = self.checkout("--depth=1")
        self.assertEqual(
            self.restored(checkout, self.built),
            {"leaf.rs": True, "added.rs": True, "root.rs": False},
        )

    def test_a_build_without_a_record_rebuilds_everything(self) -> None:
        self.assertEqual(
            self.restored(self.checkout(), None),
            {"leaf.rs": True, "added.rs": True, "root.rs": True},
        )

    def test_a_build_from_an_unknown_commit_rebuilds_everything(self) -> None:
        self.assertEqual(
            self.restored(self.checkout(), "0" * 40),
            EVERYTHING,
        )

    def test_a_changed_build_configuration_file_rebuilds_everything(
        self,
    ) -> None:
        for index, config in enumerate(BUILD_CONFIG):
            with self.subTest(config=config):
                self.commit({config: f"changed {index}"}, f"configure {config}")
                checkout = self.checkout(name=f"config-{index}")
                self.assertEqual(self.restored(checkout, self.built), EVERYTHING)

    def test_a_fetch_that_hangs_gives_up_and_rebuilds_everything(self) -> None:
        checkout = self.checkout("--depth=1")
        self.git(checkout, "config", "protocol.ext.allow", "always")
        self.git(checkout, "remote", "set-url", "origin", "ext::sleep 30")
        (checkout / "target").mkdir()
        (checkout / "target/.built-from").write_text(self.built + "\n")
        started = time.monotonic()
        self.assertIsNone(mtimes.built_from(checkout, timeout=0.5))
        self.assertLess(time.monotonic() - started, 10)


class CargoBuild(unittest.TestCase):
    def setUp(self) -> None:
        cargo = shutil.which("cargo")
        if cargo is None:
            self.skipTest("cargo is not installed")
        self.cargo = cargo
        self.root = Path(
            self.enterContext(
                tempfile.TemporaryDirectory(dir=os.environ.get("TEMP"))
            )
        )
        self.env = {
            name: value
            for name, value in os.environ.items()
            if not name.startswith("CARGO_") and name != "RUSTFLAGS"
        } | {
            "GIT_AUTHOR_NAME": "mtimes-check",
            "GIT_AUTHOR_EMAIL": "mtimes-check@localhost",
            "GIT_COMMITTER_NAME": "mtimes-check",
            "GIT_COMMITTER_EMAIL": "mtimes-check@localhost",
        }
        files = {
            ".gitignore": "target/",
            "Cargo.toml": '[workspace]\nmembers = ["kept", "edited"]\nresolver = "2"',
            "kept/Cargo.toml": '[package]\nname = "kept"\nedition = "2021"',
            "kept/src/lib.rs": "pub fn kept() {}",
            "edited/Cargo.toml": '[package]\nname = "edited"\nedition = "2021"',
            "edited/src/lib.rs": "pub fn edited() {}",
        }
        for name, text in files.items():
            (self.root / name).parent.mkdir(parents=True, exist_ok=True)
            (self.root / name).write_text(text + "\n")
        self.run_in("git", "init", "-q", "-b", "main", ".")
        self.run_in("git", "config", "commit.gpgsign", "false")
        self.run_in("git", "add", "-A")
        self.run_in("git", "commit", "-q", "-m", "built")

    def run_in(self, *command: str) -> str:
        return subprocess.run(
            command,
            cwd=self.root,
            env=self.env,
            check=True,
            capture_output=True,
            text=True,
        ).stderr

    def compiled(self) -> set[str]:
        built = self.run_in(self.cargo, "build", "--offline", "--workspace")
        return {
            line.split()[1]
            for line in built.splitlines()
            if line.strip().startswith("Compiling")
        }

    def test_cargo_rebuilds_only_the_crate_whose_source_changed(self) -> None:
        self.assertEqual(self.compiled(), {"kept", "edited"})
        self.run_in(sys.executable, str(MTIMES), "record")
        (self.root / "edited/src/lib.rs").write_text("pub fn edited() -> u8 { 1 }\n")
        self.run_in("git", "commit", "-q", "-am", "edited")
        mtimes.touch_all(self.root)
        self.run_in(sys.executable, str(MTIMES), "restore")
        self.assertEqual(self.compiled(), {"edited"})


if __name__ == "__main__":
    unittest.main()
