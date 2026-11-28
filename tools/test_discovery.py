import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent


class Discovery(unittest.TestCase):
    def test_every_test_file_is_in_a_package_discovery_enters(self) -> None:
        for test in TOOLS.rglob("test*.py"):
            for folder in test.relative_to(TOOLS).parents[:-1]:
                with self.subTest(test=str(test.relative_to(TOOLS))):
                    self.assertTrue((TOOLS / folder / "__init__.py").is_file(), f"tools/{folder} has no __init__.py")


if __name__ == "__main__":
    unittest.main()
