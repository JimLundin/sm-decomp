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
    if row['id'] in ('samus-gunship-save', 'samus-space-jump-chain', 'samus-screw-attack'):
        return ('seeds-v1-tricks2-891ceaf012b8dfb4.tar.gz.enc', '891ceaf012b8dfb4728c0707f845b38ec3499448ee7c35e4f9b778cd5fb6f19b')
    if row['id'].startswith(('samus-shinespark-', 'samus-walljump-', 'xray-scan-')) or row['id'] == 'samus-crystal-flash':
        return ('seeds-v1-tricks-95f7c4352cb18fbe.tar.gz.enc', '95f7c4352cb18fbe3a4c31b4da1de269b25b48db0232653e6fb020c832c5bdb3')
    if row['id'] == 'samus-mockball':
        return ('seeds-v1-mockball-4447309c78721b0b.tar.gz.enc', '4447309c78721b0b2a04d7ca70570f9d8b894aac59e5b8aee674275fb373bf7a')
    if row['id'].startswith('special-beam-'):
        return ('seeds-v1-special-beam-42ef643c82c0b3e4.tar.gz.enc', '42ef643c82c0b3e4b3fd8329e3df55416c639b703f30dfdc4b41049deb8816bb')
    if row['id'] in ('terrain-quicksand', 'samus-grapple-release', 'beam-reflect'):
        return ('seeds-v1-terrain-grapple-c77a6dd6e1e0de4e.tar.gz.enc', 'c77a6dd6e1e0de4e07ffe47a2ae55fc7ba3aaf9a7276a04841cd8746e6e72d24')
    if row['id'] == 'boss-draygon-grapple-repel':
        return ('seeds-v1-scenario-repel-ccb2329f0fbbd52c.tar.gz.enc', 'ccb2329f0fbbd52cca0d799cceb106f4ab32b716ac49ed75d17ab5fe074cb924')
    if row['id'].startswith('boss-kraid-pause-'):
        return ('seeds-v1-kraid-pause-52ba63dc0b528e28.tar.gz.enc', '52ba63dc0b528e2849dc880c7407afdb0365a28c0264c0e2e896585a883bddee')
    if row['id'] == 'boss-draygon-final-whips':
        return ('seeds-v1-final-whips-6295d2432aa88773.tar.gz.enc', '6295d2432aa88773bbedbcaaf4ddc4276878951281787e171620a9d40d13a9ce')
    if row['id'] in ('volcano-room-lavaquake', 'boss-draygon-tail-whip', 'boss-draygon-later-swoop'):
        return ('seeds-v1-offroute-4b5962a5bc7f6aaa.tar.gz.enc', '4b5962a5bc7f6aaa5050bd5f7d6abfb046e1b078c26b4242d82489300c7f044a')
    if row['id'] in ('pause-menu-tour', 'options-menu-tour', 'boss-draygon-grab', 'credits-ending'):
        return ('seeds-v1-coverage-54f838bfd8146bf7.tar.gz.enc', '54f838bfd8146bf76d631d567299b0721adbc1c4065584e9de5f3e19632ccb21')
    if row['id'] in ('boss-bomb-torizo', 'samus-bomb-jump-vertical', 'wall-crawler-slope-step'):
        return ('seeds-v1-seeded-checkpoints-f4e03e50190ab6dd.tar.gz.enc', 'f4e03e50190ab6dd9acfd595ddb544b93b36eef2e7231e1599dff3199d8e0631')
    if row['id'] in ('save-death-reload', 'save-memory', 'death'):
        return ('seeds-v1-realigned-ef199d44b0070294.tar.gz.enc', 'ef199d44b0070294bcbac363aaa3577304a3cbbc7a1122a6f488329d41441e52')
    if row['id'] in ('pickup-reserve', 'music-fir-selector10'):
        return ('layout150-checkpoints-147-93986acd14dc9d66.tar.gz.enc', '8c8f84e903917a06d996624358a1783e80920e9fa2084c438eaa950843d68517')
    if 'checkpoint' in row:
        return ('layout150-f158-faithful2-d4050aa32a965168.tar.gz.enc', 'd4050aa32a965168653e68a2389145af6aa6f2cd5666bec5eec7c737ffef874b')
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
def cost(kind, row):
    # Replay frames, plus, for a checkpoint row, the boot prefix it regenerates
    # when its checkpoint is rejected; that prefix replays about 5 times faster.
    return row.get('frames', 0) + (row.get('start_frame', 0) // 5 if kind == 'checkpoint' else 0)

def chunks(kind, members):
    # A rejected configured checkpoint regenerates its own complete boot prefix,
    # so bound that cold work per row. Seed groups share only a seconds-long
    # bundle restore, so split one heavier than a twelfth of the corpus.
    if kind == 'checkpoint':
        return [[r] for r in members]
    if kind != 'seed':
        return [members[i:i + 9] for i in range(0, len(members), 9)]
    out, current = [], []
    for r in members:
        if current and sum(m.get('frames', 0) for m in current) + r.get('frames', 0) > FRAME_CAP:
            out.append(current)
            current = []
        current.append(r)
    return out + [current]

FRAME_CAP = sum(r.get('frames', 0) for g in groups.values() for r in g) / 12
shards = []
for (kind, identity), members in sorted(groups.items()):
    members.sort(key=lambda r: r.get('start_frame', 0))
    for i, chunk in enumerate(chunks(kind, members)):
        inputs = chunk[0]['inputs']
        asset, digest = seed_bundle(chunk[0]) if kind != 'route' else ('', '')
        name = identity.removeprefix('layout150-').removeprefix('seeds-v1-').split('-')[0] if kind in ('seed', 'checkpoint') else identity.removesuffix('.inputs')
        shards.append({'name': f"{kind}-{name}-{i}", 'cost': sum(cost(kind, r) for r in chunk),
                       'kind': kind, 'inputs': inputs,
                       'seed_asset': asset, 'seed_sha256': digest,
                       'route': inputs.removeprefix('corpus-').removesuffix('.inputs') if kind == 'route' else '',
                       'segments': ' '.join(r['id'] for r in chunk)})
# Start the heaviest shards first: the matrix runs at most eight at a time.
shards.sort(key=lambda shard: -shard['cost'])
if not shards:
    raise SystemExit('no runnable rows selected')
with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as out:
    out.write(f"Selection: {source}\n\n")
    out.write(f"{sum(len(s['segments'].split()) for s in shards)} rows in {len(shards)} shards\n\n")
    out.write('Skipped:\n\n' + ''.join(f'- {i}: {why}\n' for i, why in skipped))
with open(os.environ['GITHUB_OUTPUT'], 'a') as out:
    out.write('matrix=' + json.dumps({'include': shards}) + '\n')
