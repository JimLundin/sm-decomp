from concurrent.futures import ThreadPoolExecutor, wait
from dataclasses import dataclass
lazy from collections.abc import Mapping
lazy from pathlib import Path

import pytest
from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

from sm_tools.ci.shell.mtimes import built_from, record, restore, touch_all
from sm_tools.shell.process import run

BUILT = 1_000_000_000
SOURCES = FrozenList(("leaf.rs", "added.rs", "root.rs"))
EVERYTHING = FrozenOrderedDict({
    "leaf.rs": True,
    "added.rs": True,
    "root.rs": True,
})
CHANGED = FrozenOrderedDict({
    "leaf.rs": True,
    "added.rs": True,
    "root.rs": False,
})
BUILD_CONFIG = FrozenList((
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
))
IDENTITY = FrozenOrderedDict({
    "GIT_AUTHOR_NAME": "mtimes-check",
    "GIT_AUTHOR_EMAIL": "mtimes-check@localhost",
    "GIT_COMMITTER_NAME": "mtimes-check",
    "GIT_COMMITTER_EMAIL": "mtimes-check@localhost",
})
CARGO_SETTINGS = FrozenList((
    "CARGO",
    "CARGO_TARGET_DIR",
    "CARGO_BUILD_TARGET_DIR",
    "CARGO_INCREMENTAL",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTFLAGS",
))
HANG_LIMIT = 10


def ran(root: Path, *command: str) -> str:
    done = run(command, cwd=root)
    assert done.status == 0, done.output()
    return done.stdout.decode()


def git(root: Path, *arguments: str) -> str:
    return ran(root, "git", *arguments).strip()


def write(root: Path, files: Mapping[str, str]) -> None:
    for name, text in files.items():
        (root / name).parent.mkdir(parents=True, exist_ok=True)
        _ = (root / name).write_text(text + "\n")


@dataclass(frozen=True, slots=True)
class Origin:
    scratch: Path
    built: str
    monkeypatch: pytest.MonkeyPatch

    @property
    def path(self) -> Path:
        return self.scratch / "origin"

    def commit(self, files: Mapping[str, str], message: str) -> None:
        write(self.path, files)
        _ = git(self.path, "add", "-A")
        _ = git(self.path, "commit", "-q", "-m", message)

    def checkout(self, *clone: str, name: str = "checkout") -> Path:
        path = self.scratch / name
        _ = git(
            self.scratch, "clone", "-q", *clone, self.path.as_uri(), str(path)
        )
        return path

    def restored(
        self, root: Path, record_text: str | None
    ) -> FrozenOrderedDict[str, bool]:
        (root / "target").mkdir(exist_ok=True)
        if record_text is not None:
            _ = (root / "target/.built-from").write_text(record_text + "\n")
        self.monkeypatch.chdir(root)
        assert restore(root) == 0
        return FrozenOrderedDict(
            (name, (root / name).stat().st_mtime > BUILT) for name in SOURCES
        )


@pytest.fixture
def origin(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Origin:
    for name, value in IDENTITY.items():
        monkeypatch.setenv(name, value)
    path = tmp_path / "origin"
    path.mkdir()
    _ = git(path, "init", "-q", "-b", "main", ".")
    _ = git(path, "config", "commit.gpgsign", "false")
    write(
        path,
        {
            ".gitignore": "target/",
            "leaf.rs": "leaf",
            "root.rs": "root",
            "deleted.rs": "gone",
        },
    )
    _ = git(path, "add", "-A")
    _ = git(path, "commit", "-q", "-m", "built")
    made = Origin(tmp_path, git(path, "rev-parse", "HEAD"), monkeypatch)
    (path / "deleted.rs").unlink()
    made.commit({"leaf.rs": "planted", "added.rs": "added"}, "changed")
    return made


def test_a_build_recorded_at_its_commit_rebuilds_only_the_changed_sources(
    origin: Origin,
) -> None:
    checkout = origin.checkout()
    _ = git(checkout, "checkout", "-q", origin.built)
    assert record(checkout) == 0
    _ = git(checkout, "checkout", "-q", "main")
    assert origin.restored(checkout, None) == CHANGED


def test_a_shallow_checkout_fetches_the_recorded_commit(origin: Origin) -> None:
    checkout = origin.checkout("--depth=1")
    assert origin.restored(checkout, origin.built) == CHANGED


def test_a_build_without_a_record_rebuilds_everything(origin: Origin) -> None:
    assert origin.restored(origin.checkout(), None) == EVERYTHING


def test_a_build_from_an_unknown_commit_rebuilds_everything(
    origin: Origin,
) -> None:
    assert origin.restored(origin.checkout(), "0" * 40) == EVERYTHING


def test_a_changed_build_configuration_file_rebuilds_everything(
    origin: Origin,
) -> None:
    for index, config in enumerate(BUILD_CONFIG):
        origin.commit({config: f"changed {index}"}, f"configure {config}")
        checkout = origin.checkout(name=f"config-{index}")
        assert origin.restored(checkout, origin.built) == EVERYTHING, config


def test_a_fetch_that_hangs_gives_up_and_rebuilds_everything(
    origin: Origin,
) -> None:
    checkout = origin.checkout("--depth=1")
    _ = git(checkout, "config", "protocol.ext.allow", "always")
    _ = git(checkout, "remote", "set-url", "origin", "ext::sleep 30")
    (checkout / "target").mkdir()
    _ = (checkout / "target/.built-from").write_text(origin.built + "\n")
    with ThreadPoolExecutor(1) as pool:
        found = pool.submit(built_from, checkout, 0.5)
        done, _ = wait((found,), timeout=HANG_LIMIT)
        assert found in done
        assert found.result() is None


def compiled(root: Path) -> FrozenOrderedSet[str]:
    built = run(("cargo", "build", "--offline", "--workspace"), cwd=root)
    assert built.status == 0, built.output()
    return FrozenOrderedSet(
        line.split()[1]
        for line in built.stderr.decode().splitlines()
        if line.strip().startswith("Compiling")
    )


def test_cargo_rebuilds_only_the_crate_whose_source_changed(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    for name, value in IDENTITY.items():
        monkeypatch.setenv(name, value)
    for name in CARGO_SETTINGS:
        monkeypatch.delenv(name, raising=False)
    write(
        tmp_path,
        {
            ".gitignore": "target/",
            "Cargo.toml": (
                '[workspace]\nmembers = ["kept", "edited"]\nresolver = "2"'
            ),
            "kept/Cargo.toml": '[package]\nname = "kept"\nedition = "2021"',
            "kept/src/lib.rs": "pub fn kept() {}",
            "edited/Cargo.toml": '[package]\nname = "edited"\nedition = "2021"',
            "edited/src/lib.rs": "pub fn edited() {}",
        },
    )
    _ = git(tmp_path, "init", "-q", "-b", "main", ".")
    _ = git(tmp_path, "config", "commit.gpgsign", "false")
    _ = git(tmp_path, "add", "-A")
    _ = git(tmp_path, "commit", "-q", "-m", "built")
    try:
        first = compiled(tmp_path)
    except FileNotFoundError:
        pytest.skip("cargo is not installed")
    assert sorted(first) == ["edited", "kept"]
    assert record(tmp_path) == 0
    _ = (tmp_path / "edited/src/lib.rs").write_text(
        "pub fn edited() -> u8 { 1 }\n"
    )
    _ = git(tmp_path, "commit", "-q", "-am", "edited")
    touch_all(tmp_path)
    assert restore(tmp_path) == 0
    assert sorted(compiled(tmp_path)) == ["edited"]
