lazy from collections.abc import Mapping
lazy from pathlib import Path

lazy import pytest

from sm_tools.ci.shell.run_tests import run

PASSING = """def test_one() -> None:
    pass


def test_two() -> None:
    pass
"""
FAILING = """def test_breaks() -> None:
    raise AssertionError("planted")
"""
COLLECTED_ONLY = """import sys

if "--collect-only" in sys.argv:

    def test_only_under_collection() -> None:
        pass
"""
DYING = """import os


def test_exits() -> None:
    os._exit(3)
"""
EXITING_AFTER_RESULTS = """import atexit
import os


def test_passes() -> None:
    atexit.register(os._exit, 4)
"""
SKIPPING = """import pytest


@pytest.mark.skip(reason="planted")
def test_skipped() -> None:
    pass
"""


def run_folder(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    modules: Mapping[str, str],
    folder: Path | None = None,
) -> tuple[int, str]:
    (tmp_path / "nested").mkdir()
    (tmp_path / "nested" / "__init__.py").touch()
    for name, source in modules.items():
        _ = (tmp_path / name).write_text(source)
    status = run(folder or tmp_path)
    return status, capsys.readouterr().out


def test_every_module_runs_and_the_counts_match(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    status, out = run_folder(
        tmp_path,
        capsys,
        {"test_first.py": PASSING, "nested/test_second.py": PASSING},
    )
    assert status == 0, out
    assert "ran 4 of 4 tests, 0 skipped, in 2 modules" in out


def test_a_failure_names_its_test(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    status, out = run_folder(
        tmp_path,
        capsys,
        {"test_first.py": PASSING, "nested/test_broken.py": FAILING},
    )
    assert status == 1
    assert "ran 3 of 3 tests" in out
    assert "FAILED nested/test_broken.py::test_breaks" in out.splitlines()


def test_tests_that_never_ran_fail_the_run(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    status, out = run_folder(
        tmp_path,
        capsys,
        {"test_first.py": PASSING, "test_collected.py": COLLECTED_ONLY},
    )
    assert status == 1
    assert "ran 2 of 3 tests" in out
    assert "FAILED test_collected.py (ran 0 of 1 tests)" in out.splitlines()


def test_a_module_whose_process_dies_is_named(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    status, out = run_folder(
        tmp_path, capsys, {"test_first.py": PASSING, "test_dying.py": DYING}
    )
    assert status == 1
    assert "ran 2 of 3 tests" in out
    assert "FAILED test_dying.py (its process exited 3)" in out.splitlines()


def test_a_module_whose_process_exits_non_zero_after_its_results_fails(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    status, out = run_folder(
        tmp_path,
        capsys,
        {"test_first.py": PASSING, "test_late.py": EXITING_AFTER_RESULTS},
    )
    assert status == 1
    assert "ran 3 of 3 tests" in out
    assert "FAILED test_late.py (its process exited 4)" in out.splitlines()


def test_skipped_tests_are_counted(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    status, out = run_folder(
        tmp_path, capsys, {"test_first.py": PASSING, "test_skips.py": SKIPPING}
    )
    assert status == 0, out
    assert "ran 3 of 3 tests, 1 skipped, in 2 modules" in out


def test_a_folder_inside_a_project_runs_its_own_modules(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    _ = (tmp_path / "pyproject.toml").write_text(
        '[tool.pytest]\ntestpaths = ["elsewhere"]\n'
    )
    status, out = run_folder(
        tmp_path, capsys, {"nested/test_first.py": PASSING}, tmp_path / "nested"
    )
    assert status == 0, out
    assert "ran 2 of 2 tests, 0 skipped, in 1 modules" in out
