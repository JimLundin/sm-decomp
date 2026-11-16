"""Invented decoder checks; no ROM, native fixture or model advance is required."""

import pathlib
import runpy
import struct
import unittest

DECODER = runpy.run_path(str(pathlib.Path(__file__).with_name("audio_expiry_seed.py")),
                        run_name="seed_decode_tests")


class AdjustmentTests(unittest.TestCase):
    def setUp(self):
        self.before = {name: bytearray(size) for name, size in DECODER["SIZES"].items()}
        self.after = {name: bytearray(value) for name, value in self.before.items()}
        for device, offset in (("APURAM", 0), ("SMP_STATE", 48)):
            self.after[device][offset + 128] = self.after[device][offset + 130] = 1

    def test_complete_paused_adjustment(self):
        DECODER["validate_adjustment"](self.before, self.after)

    def test_cpu_change_is_rejected(self):
        self.after["CPU_STATE"][100] = 1
        with self.assertRaises(ValueError):
            DECODER["validate_adjustment"](self.before, self.after)

    def test_extra_audio_change_is_rejected(self):
        self.after["APURAM"][129] = 1
        with self.assertRaises(ValueError):
            DECODER["validate_adjustment"](self.before, self.after)

    def test_missing_adjustment_is_rejected(self):
        with self.assertRaises(ValueError):
            DECODER["validate_adjustment"](self.before, self.before)

    def test_truncated_snapshot_is_rejected(self):
        self.after["DSP_STATE"].pop()
        with self.assertRaises(ValueError):
            DECODER["validate_adjustment"](self.before, self.after)


class ClockTests(unittest.TestCase):
    def setUp(self):
        self.smp = bytearray(131208)
        self.smp[131128:131131] = bytes((6, 1, 3))
        self.smp[131137:131139] = bytes((1, 16))
        struct.pack_into("<I", self.smp, 131132, 5)
        struct.pack_into("<I", self.smp, 131172, 3)

    def test_prescaler_and_unread_are_separate(self):
        self.assertEqual(DECODER["timer"](self.smp), (450, 5))

    def test_invalid_unread_is_rejected(self):
        struct.pack_into("<I", self.smp, 131132, 16)
        with self.assertRaises(ValueError):
            DECODER["timer"](self.smp)

    def test_disabled_clock_is_rejected(self):
        self.smp[131137] = 0
        with self.assertRaises(ValueError):
            DECODER["timer"](self.smp)

    def test_other_speed_is_rejected(self):
        self.smp[131177] = 1
        with self.assertRaises(ValueError):
            DECODER["timer"](self.smp)


class CursorTests(unittest.TestCase):
    def test_timing_bundle_cursor_mapping(self):
        events = [("Duration", (), 20, 21), ("Articulation", (), 21, 22),
                  ("Tone", (), 22, 23), ("PhraseEnd", (), 23, 24)]
        self.assertEqual(DECODER["semantic_cursor"](events, 23), 2)
        with self.assertRaises(ValueError):
            DECODER["semantic_cursor"](events, 21)

    def test_ambiguous_or_missing_cursor_is_rejected(self):
        events = [("Tone", (), 20, 21), ("Sustain", (), 20, 21)]
        for address in (20, 22):
            with self.assertRaises(ValueError):
                DECODER["semantic_cursor"](events, address)

    def test_effect_spans_validate_asset_identity(self):
        ram = bytes((3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 255))
        expected = [(3, 4, 5, 6, 7), (8, 9, 10, 11, 12)]
        self.assertEqual(DECODER["span_boundaries"](ram, 0, expected), {0: 0, 5: 1, 10: 2})
        with self.assertRaises(ValueError):
            DECODER["span_boundaries"](ram, 0, expected[:1])

    def test_truncated_effect_asset_is_rejected(self):
        with self.assertRaises(ValueError):
            DECODER["span_boundaries"](bytes((3, 4)), 0, [(3, 4, 5, 6, 7)])


if __name__ == "__main__":
    unittest.main()
