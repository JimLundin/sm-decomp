"""The synchronization gate must reject drops, shifts and incorrect gameplay."""
import unittest
import hashlib
import json
import tempfile
from pathlib import Path

from compare_sync import FIELDS, compare, coverage


def row(frame, event):
    return dict(frame=frame, event=event, **{key: 0 for key in FIELDS})


class SyncTests(unittest.TestCase):
    def test_completed_capture_covers_frames_after_last_gameplay_event(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "capture.csv"
            path.write_text("exact checkpoint bytes")
            self.assertEqual(coverage(path, [row(11, 1)]), 11)
            path.with_suffix(".capture.json").write_text(json.dumps({
                "completed_frames": 20,
                "checkpoint_sha256": hashlib.sha256(path.read_bytes()).hexdigest()}))
            self.assertEqual(coverage(path, [row(11, 1)]), 20)
            path.write_text("truncated checkpoint")
            with self.assertRaises(ValueError):
                coverage(path, [row(11, 1)])

    def test_constant_frame_number_offset_is_explicit(self):
        expected = [row(10, 1), row(11, 2)]
        actual = [row(11, 1), row(12, 2)]
        self.assertEqual(compare(expected, actual, 10, 11, 1)["status"], "PASS")
        self.assertEqual(compare(expected, actual, 10, 11, 0)["status"], "FAIL")

    def test_value_mismatch_catches_real_desync(self):
        expected = [row(10, 1), row(11, 2)]
        actual = [row(10, 1), row(11, 2)]
        actual[1]["x"] = 42
        result = compare(expected, actual, 10, 11, 0)
        self.assertEqual(result["status"], "FAIL")
        self.assertEqual(result["differences"], {"x": [0, 42]})

    def test_dropped_and_extra_events_fail(self):
        expected = [row(10, 1), row(10, 2), row(11, 3)]
        self.assertEqual(compare(expected, expected[1:], 10, 11, 0)["status"], "FAIL")
        self.assertEqual(compare(expected, expected + [row(11, 4)], 10, 11, 0)["status"], "FAIL")


if __name__ == "__main__":
    unittest.main()
