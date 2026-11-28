from pathlib import Path

TESTS = Path(__file__).resolve().parent


def test_every_test_file_is_in_a_package() -> None:
    folders = (
        folder
        for test in TESTS.rglob("test*.py")
        for folder in test.relative_to(TESTS).parents[:-1]
    )
    missing = [
        f"tests/{folder}"
        for folder in folders
        if not (TESTS / folder / "__init__.py").is_file()
    ]
    assert not missing, f"no __init__.py in {missing}"
