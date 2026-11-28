import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import relock

ROOM = 0xDF45


def record(room: int = ROOM, counter: int = 7839, memories: bool = True) -> bytearray:
    data = bytearray(relock.HEADER + relock.WRAM)
    data[relock.HEADER - 1] = int(memories)
    data[relock.HEADER + relock.ROOM_WORD:relock.HEADER + relock.ROOM_WORD + 2] = room.to_bytes(2, "little")
    data[relock.HEADER + relock.COUNTER_WORD:relock.HEADER + relock.COUNTER_WORD + 2] = counter.to_bytes(2, "little")
    return data


class Plan(unittest.TestCase):
    def test_a_window_holds_its_captures_together_in_frame_order(self) -> None:
        lines = ["skip 100-x 3 no-lsnes-movie", "cut any 9000 any-boundary 10 DF8D 12793",
                 "cut any 900 any-boundary 2 DF8D 8950", "cut any 9000 any-boundary 4 DF45 9500",
                 "derive scenario-a 5 any 5 --word 09C2=0001"]
        self.assertEqual(sorted(lines, key=relock.plan_order), [
            "cut any 900 any-boundary 2 DF8D 8950", "cut any 9000 any-boundary 4 DF45 9500",
            "cut any 9000 any-boundary 10 DF8D 12793", "derive scenario-a 5 any 5 --word 09C2=0001",
            "skip 100-x 3 no-lsnes-movie"])

    def test_a_capture_starts_from_the_latest_rung_before_its_step(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            ladders = Path(directory)
            (ladders / "any").mkdir()
            (ladders / "any/ladder.txt").write_text("# rung\n3000\n9000\n")
            self.assertEqual(relock.rung_for(ladders, "any", 8999), 3000)
            self.assertEqual(relock.rung_for(ladders, "any", 9000), 9000)
            self.assertEqual(relock.rung_for(ladders, "any", 100), 0)
            self.assertEqual(relock.rung_for(ladders, "100", 9000), 0)


class Check(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.relock = relock.Relock(self.directory / "plan", self.directory / "out", self.directory / "logs",
                                    frozenset())
        self.cut = relock.Cut("any", 0, "any-boundary", 1, "DF45", 8538)
        (self.directory / "out/any-boundary").mkdir(parents=True)
        (self.directory / "out/any-boundary/0001.lss").write_bytes(b"state")
        self.enterContext(mock.patch.dict(os.environ, {"CAPTURES": str(self.directory / "captures")}))
        self.table = self.enterContext(mock.patch.object(relock, "table_seeds", return_value={1: {"nmi": 7839}}))

    def check(self, data: bytearray) -> tuple[bool, bool, str]:
        (self.directory / "out/any-boundary/0001.record").write_bytes(bytes(data))
        return self.relock.check_cut(self.cut)

    def capture(self, data: bytes) -> None:
        folder = self.directory / "captures/target/seeds/any-boundary"
        folder.mkdir(parents=True, exist_ok=True)
        (folder / "0001-DF45-wram.bin").write_bytes(data)

    def test_a_seed_needs_its_memories_room_and_frame_counter(self) -> None:
        self.assertEqual(self.check(record())[:2], (True, False))
        self.assertIn("without memories", self.check(record(memories=False))[2])
        self.assertIn("room DF8D, table DF45", self.check(record(room=0xDF8D))[2])
        self.assertIn("frame counter 7838, table 7839", self.check(record(counter=7838))[2])

    def test_work_ram_differs_from_its_capture_only_outside_the_stack_page(self) -> None:
        wram = bytearray(record()[relock.HEADER:])
        wram[0x1F10] ^= 1
        self.capture(bytes(wram))
        self.assertEqual(self.check(record())[:2], (True, True))
        wram[0x0010] ^= 1
        self.capture(bytes(wram))
        passed, compared, line = self.check(record())
        self.assertEqual((passed, compared), (False, True))
        self.assertIn(": 1 WRAM bytes differ", line)

    def test_a_table_without_a_frame_counter_takes_the_captures(self) -> None:
        self.table.return_value = {1: {}}
        self.assertIn("neither the table nor a capture", self.check(record())[2])
        self.capture(bytes(record(counter=7838)[relock.HEADER:]))
        self.assertIn("frame counter 7839, table 7838", self.check(record())[2])
        self.assertEqual(self.check(record(counter=7838))[:2], (True, True))


if __name__ == "__main__":
    unittest.main()
