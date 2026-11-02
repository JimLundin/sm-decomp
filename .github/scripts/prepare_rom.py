#!/usr/bin/env python3
# Decrypt the original ROM from the private ci-data checkout, then remove the checkout.
import hashlib, pathlib, shutil, subprocess
source = pathlib.Path('.ci-data')
rom = subprocess.run(['openssl', 'enc', '-d', '-aes-256-cbc', '-pbkdf2', '-iter', '600000', '-pass', 'env:SM_ROM_KEY',
                      '-in', str(source / 'payload.enc')], check=True, stdout=subprocess.PIPE).stdout
if hashlib.sha1(rom).hexdigest() != 'da957f0d63d14cb441d215462904c4fa8519c613':
    raise SystemExit('wrong ROM')
pathlib.Path('rom').mkdir(exist_ok=True)
pathlib.Path('rom/sm.smc').write_bytes(rom)
shutil.rmtree(source)
