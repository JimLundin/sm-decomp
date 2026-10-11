"""Actual-C PLM dispatcher and retained evidence admission controls."""

import copy
import json
import tempfile
import unittest
import subprocess
from unittest.mock import patch
from pathlib import Path

import port_family
import port_plm_fixtures
import port_plm_instructions as adapter


class PointerBankTests(unittest.TestCase):
    def test_mixed_pointer_banks_and_invalid_configuration(self):
        config = json.loads((port_family.ROOT / "tools/port/pirate-host-callers.json").read_text())
        config["routines"].append(dict(symbol="PlmInstruction", arg_types=["uint16", "rom_ptr"],
                                      pointer_bank=0x84, **{"return": "rom_ptr"}, cases=[]))
        with tempfile.TemporaryDirectory() as directory:
            run = Path(directory)
            for name in ("c", "rust"):
                (run / name).write_bytes(b"mock")
            with patch.object(adapter, "execute"):
                adapter.build(config, port_family.ROOT / "deps/snesrev-sm", run)
            oracle = (run / "oracle.c").read_text()
            for bank in (0x84, 0x86, 0xb2):
                self.assertIn(f"PortPointerResult(pointer,{bank})", oracle)
            function = oracle[oracle.index("static uint32_t PortPointerResult"):oracle.index("\nextern const uint16_t")]
            test = run / "pointer-test.c"
            test.write_text("#include <stdint.h>\n#include <stdlib.h>\n#include <assert.h>\ntypedef uint16_t uint16; static uint8_t rom[0x400000]; static uint8_t *g_rom=rom;\n" + function + "\nint main(int argc,char **argv){if(argc>1){PortPointerResult((void*)(rom+0x190000),0x86);return 1;}for(int bank=0x84;bank<=0xb2;bank++){ if(bank!=0x84&&bank!=0x86&&bank!=0xb2)continue; assert(PortPointerResult((void*)(rom+((bank&0x7f)*0x8000)+0x1234),bank)==((bank<<16)|0x9234)); assert(PortPointerResult((void*)0x9234,bank)==((bank<<16)|0x9234)); assert(PortPointerResult(0,bank)==0);} return 0;}\n")
            subprocess.run(["cc", str(test), "-o", str(run / "pointer-test")], check=True)
            subprocess.run([str(run / "pointer-test")], check=True)
            rejected = subprocess.run([str(run / "pointer-test"), "wrong-bank"])
            self.assertEqual(rejected.returncode, 3)
            config["routines"][0]["pointer_bank"] = 0x100
            with patch.object(adapter, "execute"), self.assertRaisesRegex(ValueError, "invalid pointer bank"):
                adapter.build(config, port_family.ROOT / "deps/snesrev-sm", run)


class ProjectileHostEvidenceTests(unittest.TestCase):
    def test_continuation_carries_previous_wram_and_all_host_words(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            previous = root / 'previous'
            previous.mkdir()
            ram = bytearray(port_family.WRAM_SIZE)
            ram[0x10000] = 0x5A
            host = bytes(range(16))
            for subject in ('c', 'rust'):
                (previous / (subject + '-after.wram')).write_bytes(ram)
                (previous / (subject + '-host-after.bin')).write_bytes(host)
            config = dict(name='next', continue_from='previous', writes=[[7, [99]]])
            port_family.prepare_case(config, root / 'next', [])
            before = (root / 'next/before.wram').read_bytes()
            self.assertEqual(before[0x10000], 0x5A)
            self.assertEqual(before[7], 99)
            port_family.verify_continuation(config, root, before)
            self.assertEqual(port_family.projectile_host_words(config, root),
                             [int.from_bytes(host[i:i + 2], 'little') for i in range(0, 16, 2)])
            with self.assertRaisesRegex(ValueError, 'inherit previous WRAM'):
                port_family.verify_continuation(config, root, bytes(port_family.WRAM_SIZE))
            (previous / 'rust-host-after.bin').write_bytes(bytes(16))
            with self.assertRaisesRegex(ValueError, 'previous host state'):
                port_family.projectile_host_words(config, root)
            (previous / 'rust-after.wram').write_bytes(bytes(port_family.WRAM_SIZE))
            with self.assertRaisesRegex(ValueError, 'matching full previous WRAM'):
                port_family.prepare_case(dict(config, name='bad'), root / 'bad', [])

    def test_case_order_rejects_missing_duplicate_or_forward_continuation(self):
        manifest = dict(routines=[dict(symbol='A', cases=[dict(name='first')]),
                                  dict(symbol='B', cases=[dict(name='second', continue_from='first')])],
                        case_order=['first', 'second'])
        self.assertEqual([r['symbol'] for r in port_family.execution_routines(manifest)], ['A', 'B'])
        for order in (['second', 'first'], ['first', 'first'], ['first']):
            with self.assertRaises(ValueError):
                port_family.execution_routines(dict(manifest, case_order=order))

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
            generated = json.loads(port_plm_fixtures.generate(directory / "fixtures").read_text())
            config["rom"] = generated["rom"]
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
