#!/usr/bin/env python3
# Check the Cargo verifier source/library guards and required actor exports.
from pathlib import Path
import runpy
root = Path.cwd()
verifier = runpy.run_path(str(root / 'tools/verify_dll.py'))
host = verifier['load_host'](root / 'target/verify-dll/sm_verify.dll')
verifier['check_source'](host, verifier['source_hash'](root))
verifier['check_library'](host, root)
for name in ('sm_verify_actor_movement_case', 'sm_verify_actor_movement_table_case',
             'sm_verify_actor_population_animation_case'):
    if not hasattr(host, name):
        raise SystemExit('Cargo verifier missing required export: ' + name)
print('Cargo verifier source and all three actor exports match')
