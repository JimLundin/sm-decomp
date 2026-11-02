#!/usr/bin/env python3
# Check the Cargo verifier source/library guards and required actor capabilities.
from pathlib import Path
import runpy
import subprocess

root = Path.cwd()
verifier = runpy.run_path(str(root / 'tools/verify_dll.py'))
binary = root / 'target/verify-host' / verifier['HOST_BINARY']
host = verifier['load_host'](binary)
verifier['check_source'](host, verifier['source_hash'](root))
verifier['check_library'](host, root)
capabilities = subprocess.check_output(
    [str(binary), '--host-capabilities'], env=verifier['host_environment'](),
    text=True, encoding='utf-8').splitlines()
for name in ('sm_verify_actor_movement_case', 'sm_verify_actor_movement_table_case',
             'sm_verify_actor_population_animation_case'):
    if name not in capabilities:
        raise SystemExit('Cargo verifier missing required capability: ' + name)
print('Cargo verifier source and all three actor capabilities match')
