#!/usr/bin/env python3
# Select and shard the corpus scenarios (every scenario but the route rooms)
# with their exact seed/movie identities.
import json, os, pathlib, sys, tomllib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / 'tools'))
import scenario  # noqa: E402
rows = [row for row in scenario.rows() if 'chain' not in row]
bundles = scenario.read_lock().bundles
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
    # The ci-data bundle with the row's private data, as scenarios/lock.toml pins it.
    return row['bundle'], bundles[row['bundle']]

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
