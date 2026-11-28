lazy from pathlib import Path

from ordered import FrozenList
lazy import pytest

from sm_tools.oracle.shell import movie_inputs


def loaded(*_: object) -> FrozenList[int]:
    return FrozenList((0, 0x100, 0xFFFF))


def test_import_keeps_crlf_hex_bytes(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output = tmp_path / "inputs"
    monkeypatch.setattr(movie_inputs, "load", loaded)
    assert movie_inputs.main(["movie", str(output), "--rom", "rom"]) == 0
    _ = capsys.readouterr()
    assert output.read_bytes() == b"0000\r\n0100\r\nFFFF\r\n"
