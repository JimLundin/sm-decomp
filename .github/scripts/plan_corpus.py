#!/usr/bin/env python3
# Select and shard existing corpus rows with their exact seed/movie identities.
import json, os, tomllib
rows = tomllib.load(open('movies/segments.toml', 'rb'))['segment']
movie = next(m for m in tomllib.load(open('movies/manifest.toml', 'rb'))['movies'] if m['category'] == 'any%')
with open(os.environ['GITHUB_OUTPUT'], 'a') as out:
    out.write(f"movie_file={movie['file']}\nmovie_sha256={movie['sha256']}\n")
known = {r['id'] for r in rows}
wanted = os.environ.get('SELECTED', '').split()
source = 'dispatch input' if wanted else 'every row'
if wanted:
    unknown = set(wanted) - known
    if unknown:
        raise SystemExit(f'unknown rows: {sorted(unknown)}')
def seed_bundle(row):
    if row['id'] == 'samus-grapple-connect-swing':
        return ('CONNECT-e5c5a415-9.tar.gz.enc', 'fefac98ed43a3f3529349fa7d40dd469c353bd68dc1e697bba7f2f967ca06501')
    if row['id'] == 'xray-activate-release':
        return ('seeds-v1-xray-activation-9c5ad92da3d5d2dc.tar.gz.enc', '9c5ad92da3d5d2dc13cfae2e2a0906ca915b2329767f121183190d2ab96c25bc')
    if row['id'] in ('mother-brain-glass', 'mother-brain-fake-death', 'mother-brain-phase2',
                     'mother-brain-baby-metroid-drain', 'mother-brain-baby-metroid-sacrifice',
                     'mother-brain-escape-door'):
        return ('seeds-v1-mb-phases-13dd32bb039004ab.tar.gz.enc', '13dd32bb039004abe6a8a3225cc1dfe14421dab64bea27e8931b987603042caf')
    if row['id'].startswith('boss-kraid-pause-'):
        return ('seeds-v1-kraid-pause-52ba63dc0b528e28.tar.gz.enc', '52ba63dc0b528e2849dc880c7407afdb0365a28c0264c0e2e896585a883bddee')
    if row['id'] == 'boss-draygon-final-whips':
        return ('seeds-v1-final-whips-6295d2432aa88773.tar.gz.enc', '6295d2432aa88773bbedbcaaf4ddc4276878951281787e171620a9d40d13a9ce')
    if row['id'] in ('volcano-room-lavaquake', 'boss-draygon-tail-whip', 'boss-draygon-later-swoop'):
        return ('seeds-v1-offroute-4b5962a5bc7f6aaa.tar.gz.enc', '4b5962a5bc7f6aaa5050bd5f7d6abfb046e1b078c26b4242d82489300c7f044a')
    if row['id'] in ('pause-menu-tour', 'options-menu-tour', 'boss-draygon-grab', 'credits-ending'):
        return ('seeds-v1-coverage-54f838bfd8146bf7.tar.gz.enc', '54f838bfd8146bf76d631d567299b0721adbc1c4065584e9de5f3e19632ccb21')
    if row['id'] in ('pickup-reserve', 'save-death-reload', 'save-memory', 'music-fir-selector10'):
        return ('layout150-checkpoints-147-93986acd14dc9d66.tar.gz.enc', '8c8f84e903917a06d996624358a1783e80920e9fa2084c438eaa950843d68517')
    if 'checkpoint' in row:
        return ('layout150-f158-full22.tar.gz.enc', '723ccef6ac6df45f31d5179ba5d9569184de0fb5c678266365d3eaf62d3c10b9')
    # Match exact seed identities, not capture prefixes: pickups and the
    # general bundle contain different subsets of 100-input-phase.
    seed = row.get('seed')
    if seed in ('100-input-phase:1', '100-input-phase:5', '100-input-phase:6', '100-input-phase:12', '100-input-phase:16',
                '100-input-phase:17', '100-input-phase:19', '100-input-phase:27', '100-input-phase:37',
                '100-input-phase:55', '100-input-phase:73', '100-input-phase:74', '100-input-phase:80',
                '100-input-phase:85', '100-input-phase:92', '100-input-phase:98', '100-input-phase:104',
                '100-input-phase:142', '100-input-phase:163', '100-input-phase:170', '100-input-phase:173',
                '100-input-phase:179', '100-input-phase:294'):
        return ('seeds-v1-six-3a87864fbffb215a.tar.gz.enc', '3a87864fbffb215aa87ba3901c57b542abae1e907d105be4ee44bbb9921db793')
    if seed == 'golden-input-phase:1':
        return ('seeds-v1-golden-3b895ada4be8fcb4.tar.gz.enc', '3b895ada4be8fcb4e1b77941f0f4c4530150c626588b53d2e8cf5c2dc2cbeb0e')
    if seed == '100-credits:336':
        return ('seeds-v1-credits-42b93bf0bf83aba1.tar.gz.enc', '42b93bf0bf83aba1dd10fc24ca7d5f5b639c65e501e06ce2239c7c638c2a0ce8')
    return ('seeds-v1-spore-6b6b2c8fd70bc713.tar.gz.enc', '6b6b2c8fd70bc7139410cd576f015dfcb361360ae74cf2196a62d7edfbda7835')

groups, skipped = {}, []
for r in rows:
    if wanted and r['id'] not in wanted:
        continue
    if 'pending' in r:
        skipped.append((r['id'], 'pending'))
    else:
        kind = 'checkpoint' if 'checkpoint' in r else ('seed' if 'seed' in r else ('route' if r['inputs'].startswith('corpus-') else 'captured'))
        identity = seed_bundle(r)[0] if kind in ('seed', 'checkpoint') else r['inputs']
        groups.setdefault((kind, identity), []).append(r)
shards = []
for (kind, identity), members in sorted(groups.items()):
    members.sort(key=lambda r: r.get('start_frame', 0))
    # A rejected configured checkpoint regenerates its own complete boot prefix.
    # Bound that cold work per row; seed and route groups retain shared setup.
    width = 1 if kind == 'checkpoint' else (len(members) if kind == 'seed' else 9)
    for i in range(0, len(members), width):
        chunk = members[i:i + width]
        inputs = chunk[0]['inputs']
        asset, digest = seed_bundle(chunk[0]) if kind != 'route' else ('', '')
        name = identity.removeprefix('layout150-').removeprefix('seeds-v1-').split('-')[0] if kind in ('seed', 'checkpoint') else identity.removesuffix('.inputs')
        shards.append({'name': f"{kind}-{name}-{i // width}",
                       'kind': kind, 'inputs': inputs,
                       'seed_asset': asset, 'seed_sha256': digest,
                       'route': inputs.removeprefix('corpus-').removesuffix('.inputs') if kind == 'route' else '',
                       'segments': ' '.join(r['id'] for r in chunk)})
if not shards:
    raise SystemExit('no runnable rows selected')
with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as out:
    out.write(f"Selection: {source}\n\n")
    out.write(f"{sum(len(s['segments'].split()) for s in shards)} rows in {len(shards)} shards\n\n")
    out.write('Skipped:\n\n' + ''.join(f'- {i}: {why}\n' for i, why in skipped))
with open(os.environ['GITHUB_OUTPUT'], 'a') as out:
    out.write('matrix=' + json.dumps({'include': shards}) + '\n')
