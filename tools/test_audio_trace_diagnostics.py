"""Synthetic trace validation: no ROM, DLL, capture or replay is required."""
import contextlib
import io
import json
from pathlib import Path
import re
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_dll  # noqa: E402


class AudioTraceDiagnostics(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "synthetic.trace"
        self.header = {"audio_trace": 1, "first": 0, "ceiling_end": 1}
        self.footer = {"audio_trace_complete": True, "first": 0, "end": 1,
                       "frames": 1, "cues": 0, "ports": 0}
        self.outputs = ["AudioOutput InputBegin synthetic",
                        "AudioOutput ExecutionBegin synthetic",
                        "AudioOutput ExecutionBegin synthetic",
                        "AudioOutput InputComplete synthetic"]
        workflow = (Path(__file__).resolve().parents[1]
                    / ".github/workflows/segments.yml").read_text(encoding="utf-8")
        self.allowed = re.compile(re.search(r"\$diagnostic = '([^']+)'", workflow)[1])

    def write_trace(self, lines):
        self.path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")

    def lines(self):
        return [json.dumps(self.header), *self.outputs, json.dumps(self.footer)]

    def rejected(self, category, exception=RuntimeError):
        output = io.StringIO()
        with contextlib.redirect_stdout(output), self.assertRaises(exception):
            verify_dll.check_audio_trace(self.path, {})
        diagnostic = output.getvalue().strip()
        self.assertEqual(diagnostic, "DIAGNOSTIC stage=audio-trace-validation "
                         f"category={category} tick=unknown")
        self.assertIsNotNone(self.allowed.fullmatch(diagnostic))
        self.assertNotIn(str(self.path), diagnostic)

    def test_valid_trace_has_no_failure_metadata(self):
        self.write_trace(self.lines())
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            verify_dll.check_audio_trace(self.path, {})
        self.assertEqual(output.getvalue(), "")

    def test_missing_header(self):
        self.path.write_text("", encoding="utf-8")
        self.rejected("header-missing")

    def test_provenance(self):
        self.header["audio_trace"] = 2
        self.write_trace(self.lines())
        self.rejected("header-provenance")

    def test_duplicate_footer(self):
        self.write_trace([*self.lines(), json.dumps(self.footer)])
        self.rejected("footer-duplicate")

    def test_unrecognized_output(self):
        self.outputs[0] = "private raw error /private/capture.trace"
        self.write_trace(self.lines())
        self.rejected("output-stream")

    def test_unknown_output_kind(self):
        self.outputs[0] = "AudioOutput Unknown synthetic"
        self.write_trace(self.lines())
        self.rejected("output-kind")

    def test_incomplete_footer(self):
        self.write_trace(self.lines()[:-1])
        self.rejected("footer-incomplete")

    def test_counts(self):
        self.outputs.pop()
        self.write_trace(self.lines())
        self.rejected("output-counts")

    def test_malformed_metadata(self):
        self.write_trace(["{private raw error /private/capture.trace"])
        self.rejected("trace-format", ValueError)

    def test_missing_file(self):
        self.rejected("trace-read", OSError)

    def test_sanitizer_rejects_raw_suffix_and_unknown_category(self):
        for line in ("DIAGNOSTIC stage=post-replay category=required-facts tick=8 raw",
                     "DIAGNOSTIC stage=post-replay category=private-data tick=8",
                     "DIAGNOSTIC stage=post-replay category=required-facts tick=/private"):
            self.assertIsNone(self.allowed.fullmatch(line))

    def test_exception_identity_is_preserved(self):
        error = OSError("private path /private/capture.trace")
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            result = verify_dll.diagnostic_error("post-replay-provenance", "source-read", error)
        self.assertIs(result, error)
        self.assertIsNotNone(self.allowed.fullmatch(output.getvalue().strip()))


if __name__ == "__main__":
    unittest.main()
