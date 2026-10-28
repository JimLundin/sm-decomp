#!/usr/bin/env python3
# Prepare the original ROM and selected seed or movie-route corpus inputs.
import base64, gzip, hashlib, io, os, pathlib, subprocess, sys, time, tomllib, urllib.error, urllib.request, zipfile
encoded = "".join(os.environ[f"ROM_{i:02d}"] for i in range(45))
if not encoded:
    raise SystemExit("ROM secrets are missing")
rom = gzip.decompress(base64.b64decode(encoded))
if hashlib.sha1(rom).hexdigest() != 'da957f0d63d14cb441d215462904c4fa8519c613':
    raise SystemExit('wrong ROM')
pathlib.Path('rom').mkdir(exist_ok=True)
pathlib.Path('rom/sm.smc').write_bytes(rom)
if os.environ['KIND'] != 'route':
    bundle = pathlib.Path(os.environ['RUNNER_TEMP']) / 'seeds-v1.tar.gz.enc'
    asset, digest = os.environ['SEED_ASSET'], os.environ['SEED_SHA256']
    url = 'https://github.com/JimLundin/sm-decomp/releases/download/ci-data/' + asset
    urllib.request.urlretrieve(url, bundle)
    command = [sys.executable, 'tools/ci_seed_data.py', 'restore', str(bundle),
               '--sha256', digest, '--runtime-only']
    for row in os.environ['SEGMENTS'].split():
        command += ['--segment', row]
    subprocess.run(command, check=True)
    sys.exit(0)
movies = tomllib.loads(pathlib.Path('movies/manifest.toml').read_text())['movies']
movie = next(m for m in movies if m['category'] == 'any%')
path = pathlib.Path('movies', movie['file'])
if path.exists():
    data = path.read_bytes()
else:
    request = urllib.request.Request(movie['download'], headers={'User-Agent': 'sm-decomp-ci'})
    # Four total attempts; checksum validation below is never retried.
    for attempt in range(4):
        try:
            with urllib.request.urlopen(request, timeout=120) as response:
                data = response.read()
            break
        except urllib.error.HTTPError as error:
            if error.code not in (429, 500, 502, 503, 504) or attempt == 3:
                raise
            print(f'Movie download HTTP {error.code}; attempt {attempt + 1}/4 failed, retrying', file=sys.stderr)
            time.sleep(2 ** attempt)
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        if 'input' not in archive.namelist():
            data = archive.read(next(n for n in archive.namelist() if n.lower().endswith('.lsmv')))
# Cache hits and fresh downloads must pass the same manifest checksum.
if hashlib.sha256(data).hexdigest() != movie['sha256']:
    raise SystemExit('wrong movie')
path.write_bytes(data)
subprocess.run([sys.executable, 'tools/movie_inputs.py', str(path), 'movies/' + os.environ['INPUTS'],
                '--rom', 'rom/sm.smc', '--route', os.environ['ROUTE']], check=True)
