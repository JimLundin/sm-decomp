import os
import tempfile
from pathlib import Path


def scratch() -> Path:
    return Path(os.environ.get("TEMP") or tempfile.gettempdir())
