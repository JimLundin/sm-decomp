import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from port_evidence import packed_replay, authenticate_subject, materialize, digest
from port_live import generate_effect_writer


class PackedEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.environ.get("SM_EVIDENCE_TEST_BINARY"):
            cls.temp = None
            cls.binary = Path(os.environ["SM_EVIDENCE_TEST_BINARY"]).resolve()
            return
        cls.temp = tempfile.TemporaryDirectory()
        cls.build = Path(cls.temp.name)
        subprocess.run(['cargo', 'build', '--locked', '--release', '-p', 'evidence',
                        '--target-dir', str(cls.build / 'target')], cwd=ROOT, check=True,
                       stdout=subprocess.DEVNULL)
        cls.binary = cls.build / 'target/release/sm-evidence'

    @classmethod
    def tearDownClass(cls):
        if cls.temp is not None:
            cls.temp.cleanup()

    def setup_run(self, root):
        shutil.copy2(self.binary, root / 'sm-evidence')
        source = root / 'writer.c'
        source.write_text('#include <stdio.h>\n#include <stdlib.h>\n#include <unistd.h>\n' + generate_effect_writer(packed=True) + r'''
int main(int argc,char **argv){
 if(chdir(getenv("SM_EVIDENCE_STAGING")))return 2;
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 for(unsigned i=0;i<40000;i++)PortEffect(port_effects,"ppu 7 0 %u %u\n",i%2?24:25,i%256);
 PortEffect(port_effects,"apu 0 3 4\n");PortEffect(port_effects,"unknown 0 exact\n");
 if(argc>1)return 3;
 FILE *f=fopen("frame-000001.bin","wb");if(!f)return 2;
 for(unsigned i=0;i<1401335;i++)fputc(i%13,f);
 if(fclose(f))return 2;
 f=fopen("fixtures/capture.wram","wb");if(!f)return 2;fputs("memory bytes",f);if(fclose(f))return 2;
 return PortEffectsClose();
}
''')
        binary = root / 'producer'
        subprocess.run(['cc', str(source), '-o', str(binary)], check=True)
        return binary

    def test_streamed_capture_release_reopen_and_exact_extraction(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); producer = self.setup_run(root); store = root / 'store'
            pointers = []
            for subject in ('c-0', 'mixed-0'):
                directory = root / subject; directory.mkdir()
                p = packed_replay([str(producer)], directory, root / 'sm-evidence', store,
                                  {'subject': subject}, 60)
                pointers.append(p)
                self.assertFalse((directory / '.evidence-stage').exists())
                self.assertFalse((directory / 'hardware.log').exists())
                self.assertEqual(authenticate_subject(directory), p)
                dest = root / (subject + '-frame.log')
                materialize(directory, dest, 'hardware.log', 0)
                self.assertEqual(dest.read_bytes(), b'apu 0 3 4\nunknown 0 exact\n')
                extracted = root / (subject + '-extracted')
                materialize(directory, extracted)
                self.assertEqual((extracted / 'fixtures/capture.wram').read_bytes(), b'memory bytes')
            self.assertEqual(pointers[0]['observations'], pointers[1]['observations'])
            self.assertNotEqual(pointers[0]['run'], pointers[1]['run'])
            self.assertLess(sum(p.stat().st_size for p in store.glob('*.pack')), 100000)
            pointer = root / 'c-0/evidence.json'
            expected = digest(pointer)
            pointer.write_text('{}')
            with self.assertRaises(ValueError):
                authenticate_subject(root / 'c-0', pointer_sha256=expected)

    def test_missing_pack_fails_authentication_after_release(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); producer = self.setup_run(root); store = root / 'store'
            directory = root / 'c-0'; directory.mkdir()
            packed_replay([str(producer)], directory, root / 'sm-evidence', store, {}, 60)
            next(store.glob('*.pack')).unlink()
            with self.assertRaises(subprocess.CalledProcessError):
                authenticate_subject(directory)

    def test_timeout_reaps_processes_and_preserves_staging(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); self.setup_run(root)
            directory = root / 'c-0'; directory.mkdir()
            command = [sys.executable, '-c', 'import time; time.sleep(60)']
            with self.assertRaises(subprocess.TimeoutExpired):
                packed_replay(command, directory, root / 'sm-evidence', root / 'store', {}, 0.1)
            self.assertTrue((directory / '.evidence-stage').exists())
            self.assertFalse((directory / '.effects.pipe').exists())
            self.assertFalse((directory / 'evidence.json').exists())

    def test_failed_producer_never_releases_stage_or_publishes_pointer(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); producer = self.setup_run(root)
            directory = root / 'c-0'; directory.mkdir()
            with self.assertRaises(ValueError):
                packed_replay([str(producer), 'fail'], directory, root / 'sm-evidence', root / 'store', {}, 60)
            self.assertTrue((directory / '.evidence-stage').exists())
            self.assertFalse((directory / 'evidence.json').exists())
            self.assertTrue((directory / 'trace-receipt.json').exists())
            self.assertFalse((directory / '.effects.pipe').exists())

    def test_locked_store_does_not_start_producer(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); producer = self.setup_run(root)
            store = root / 'store'; store.mkdir(); (store / 'writer.lock').write_text('crashed writer')
            directory = root / 'c-0'; directory.mkdir()
            with self.assertRaises(ValueError):
                packed_replay([str(producer)], directory, root / 'sm-evidence', store, {}, 60)
            self.assertFalse((directory / 'replay.stdout').exists())
            self.assertEqual((store / 'writer.lock').read_text(), 'crashed writer')


if __name__ == '__main__':
    unittest.main()
