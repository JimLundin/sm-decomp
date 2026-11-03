#!/usr/bin/env python3
# Prepare selected seed or movie-route corpus inputs; the Decrypt the ROM step already restored it.
import hashlib, io, os, pathlib, subprocess, sys, time, tomllib, urllib.error, urllib.request, zipfile
if os.environ['KIND'] != 'route':
    bundle = pathlib.Path(os.environ['RUNNER_TEMP']) / 'seeds-v1.tar.gz.enc'
    asset, digest = os.environ['SEED_ASSET'], os.environ['SEED_SHA256']
    url = 'https://github.com/JimLundin/sm-decomp/releases/download/ci-data/' + asset
    # A release download can fail transiently (HTTP 503 in run 41638757665); the pinned
    # SHA-256 check in ci_seed_data.py still rejects anything wrong.
    subprocess.run(['curl', '--fail', '--silent', '--show-error', '--location', '--retry', '4',
                    '--retry-all-errors', '--output', str(bundle), url], check=True)
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
