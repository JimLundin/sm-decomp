import ast
import sys
from pathlib import Path

import pytest
from ordered import FrozenList

from sm_tools.shell.process import run

BOX = (
    Path(__file__).resolve().parents[2] / "src/sm_tools/cleanroom/shell/box.py"
)
BOX_PYTHON = (3, 12)
IMPORT_ALONE = """import importlib.util, sys
spec = importlib.util.spec_from_file_location("box", sys.argv[1])
spec.loader.exec_module(importlib.util.module_from_spec(spec))
"""


def imported(tree: ast.Module) -> FrozenList[str]:
    return FrozenList(
        name
        for node in ast.walk(tree)
        for name in (
            (alias.name for alias in node.names)
            if isinstance(node, ast.Import)
            else (
                ("." * node.level + (node.module or ""),)
                if isinstance(node, ast.ImportFrom)
                else ()
            )
        )
    )


def test_the_box_runs_on_python_3_12_with_the_standard_library_alone() -> None:
    tree = ast.parse(
        BOX.read_text(encoding="utf-8"), feature_version=BOX_PYTHON
    )
    modules = imported(tree)
    assert modules
    outside = FrozenList(
        name
        for name in modules
        if name.partition(".")[0] not in sys.stdlib_module_names
    )
    assert not outside, f"box.py imports {outside}, which the box lacks"
    lazy = FrozenList(
        node.lineno
        for node in ast.walk(tree)
        if isinstance(node, ast.Import | ast.ImportFrom) and node.is_lazy
    )
    assert not lazy, f"box.py imports lazily on lines {lazy}, which 3.12 lacks"


def test_python_3_12_imports_the_box_without_site_packages() -> None:
    found = run(("uv", "python", "find", "--no-project", "3.12"))
    if found.status:
        pytest.skip(f"uv finds no Python 3.12 here: {found.output().strip()}")
    python = found.stdout.decode().strip()
    imported_alone = run((python, "-I", "-S", "-c", IMPORT_ALONE, str(BOX)))
    assert not imported_alone.status, imported_alone.output()
