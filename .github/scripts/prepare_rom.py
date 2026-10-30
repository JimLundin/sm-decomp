#!/usr/bin/env python3
# Restore only the original ROM from the repository secrets (no corpus inputs).
import base64, gzip, hashlib, os, pathlib
encoded = "".join(os.environ[f"ROM_{i:02d}"] for i in range(45))
if not encoded:
    raise SystemExit("ROM secrets are missing")
rom = gzip.decompress(base64.b64decode(encoded))
if hashlib.sha1(rom).hexdigest() != 'da957f0d63d14cb441d215462904c4fa8519c613':
    raise SystemExit('wrong ROM')
pathlib.Path('rom').mkdir(exist_ok=True)
pathlib.Path('rom/sm.smc').write_bytes(rom)
