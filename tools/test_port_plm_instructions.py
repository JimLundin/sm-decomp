"""Actual-C PLM dispatcher and retained evidence admission controls."""

import copy
import json
import tempfile
import unittest
from pathlib import Path

import port_family
import port_plm_instructions as adapter


class ProjectileHostEvidenceTests(unittest.TestCase):
    def test_host_only_mismatch_and_missing_sidecar_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            case = root / "case"
            case.mkdir()
            names = ("host-before.bin", "c-host-after.bin", "rust-host-after.bin")
            for name in names:
                (case / name).write_bytes(bytes(16))
            artifacts = {"case/" + name: "sealed" for name in names}
            adapter.verify_projectile_host(case, root, artifacts, [0] * 8)
            (case / names[2]).write_bytes(bytes([1]) + bytes(15))
            with self.assertRaisesRegex(ValueError, "host state differs"):
                adapter.verify_projectile_host(case, root, artifacts, [0] * 8)
            artifacts.pop("case/" + names[2])
            with self.assertRaisesRegex(ValueError, "host artifact missing"):
                adapter.verify_projectile_host(case, root, artifacts, [0] * 8)


class PlmInstructionEvidenceTests(unittest.TestCase):
    def test_actual_timer_and_retained_inventory_controls(self):
        full = json.loads(
            (port_family.ROOT / "tools/port/plm-instructions.json").read_text()
        )
        config = copy.deepcopy(full)
        config["routines"] = config["routines"][:1]
        config["routines"][0]["cases"] = [
            c for c in config["routines"][0]["cases"] if c["name"] == "op-874e-3"
        ]
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            manifest = directory / "manifest.json"
            manifest.write_text(json.dumps(config))
            predecessor = port_family.build
            try:
                port_family.build = adapter.build
                v = port_family.verify(
                    manifest,
                    Path("/home/jim/sm-decomp/deps/snesrev-sm"),
                    directory / "good",
                )
            finally:
                port_family.build = predecessor
            self.assertEqual(v["status"], "offline-verified", v.get("error"))
            path = Path(v["run"]) / "verdict.json"
            adapter.verify_plm_instruction_evidence(path)
            original = path.read_text()
            v["artifacts"].pop("op-874e-3/rust.stdout")
            path.write_text(json.dumps(v))
            with self.assertRaisesRegex(ValueError, "case artifacts"):
                adapter.verify_plm_instruction_evidence(path)
            path.write_text(original)
            candidate = directory / "wrong.rs"
            source = (
                port_family.ROOT / "crates/port/src/plm_instructions.rs"
            ).read_text()
            candidate.write_text(
                source.replace(
                    "r.set_byte(TIMER + k, byte(0)?);",
                    "r.set_word(TIMER + k, byte(0)? as u16);",
                )
            )
            self.assertNotEqual(candidate.read_text(), source)
            wrapper = directory / "wrong-wrapper.rs"
            wrapper.write_text(
                (path.parent / "wrapper.rs")
                .read_text()
                .replace(
                    str(port_family.ROOT / "crates/port/src/plm_instructions.rs"),
                    str(candidate),
                )
            )
            port_family.execute(
                [
                    "rustc",
                    "--edition=2024",
                    "-O",
                    str(wrapper),
                    "-o",
                    str(directory / "wrong"),
                ],
                directory,
                "wrong-build",
            )
            case = path.parent / "op-874e-3"
            wrong_output = directory / "wrong-after.wram"
            port_family.execute(
                [
                    str(directory / "wrong"),
                    "PlmInstruction",
                    "78",
                    str(config["routines"][0]["cases"][0]["b"]),
                    str(case / "before.wram"),
                    str(wrong_output),
                    str(path.parent / "rom.bin"),
                ],
                directory,
                "wrong",
            )
            self.assertNotEqual(
                wrong_output.read_bytes(), (case / "c-after.wram").read_bytes()
            )
            output = path.parent / "op-874e-3/rust-after.wram"
            data = bytearray(output.read_bytes())
            data[0x1D77 + 78 + 1] ^= 1
            output.write_bytes(data)
            with self.assertRaisesRegex(ValueError, "artifact changed"):
                adapter.verify_plm_instruction_evidence(path)
