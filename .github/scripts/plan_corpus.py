#!/usr/bin/env python3
# Select and shard the corpus scenarios (every scenario but the route rooms)
# with their exact seed/movie identities.
import hashlib, json, math, os, pathlib, sys, tomllib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / 'tools'))
import scenario  # noqa: E402
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from lock_gate import oracle_sample, oracle_slot  # noqa: E402
all_rows = scenario.rows()
rows = [row for row in all_rows if 'chain' not in row]
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
        groups.setdefault((False, kind, identity), []).append(r)

if os.environ.get('ORACLE_SAMPLE') == '1':
    locked = scenario.check()
    slot = oracle_slot()
    sampled = set(oracle_sample(all_rows, locked, slot))
    for row in all_rows:
        if row['id'] not in sampled:
            continue
        # Route rooms are seeds too; the existing runtime-only restore supports
        # explicitly selected chain ids and checks their pinned bundle members.
        kind = 'checkpoint' if 'checkpoint' in row else ('seed' if 'seed' in row else ('route' if row['inputs'].startswith('corpus-') else 'captured'))
        identity = seed_bundle(row)[0] if kind in ('seed', 'checkpoint') else row['inputs']
        groups.setdefault((True, kind, identity), []).append(row)
def cost(kind, row):
    # Replay frames, plus, for a checkpoint row, the boot prefix it regenerates
    # when its checkpoint is rejected; every verified input frame is real work.
    return row.get('frames', 0) + (row.get('start_frame', 0) if kind == 'checkpoint' else 0)

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

FRAME_CAP = sum(r.get('frames', 0) for (record, _, _), members in groups.items()
                if not record for r in members) / 12
PREFIX_FRAMES, PREFIX_STAGES = 8192, 28
prefix_groups = {}
prefix_keys = {}
for (record, kind, identity), members in groups.items():
    if kind != 'checkpoint':
        continue
    oracle = 'faithful' if record else os.environ.get('ORACLE', 'patched')
    for row in members:
        key = (oracle, identity, row['inputs'], row['input_sha256'], row.get('seed_sha256', ''))
        prefix_groups.setdefault(key, {})[row['id']] = row
        prefix_keys[(record, row['id'])] = key
prefix_plans = {}
prefix_matrices = {f'stage_{stage:02d}': {'include': []} for stage in range(PREFIX_STAGES)}
for key, members in sorted(prefix_groups.items()):
    maximum = max(row['start_frame'] for row in members.values())
    stages = math.ceil(maximum / PREFIX_FRAMES)
    if not 0 < stages <= PREFIX_STAGES:
        raise SystemExit('configured prefix exceeds existing any/100/low scheduling bound')
    chain = hashlib.sha256('\0'.join(key).encode()).hexdigest()[:24]
    rows = sorted(members.values(), key=lambda row: row['id'])
    asset, digest = seed_bundle(rows[0])
    planned = dict(kind='checkpoint', prefix_chain=chain, prefix_stage=stages - 1,
                   oracle=key[0], seed_asset=asset, seed_sha256=digest,
                   segments=' '.join(row['id'] for row in rows), route='', inputs=key[2])
    prefix_plans[key] = planned
    for stage in range(stages):
        prefix_matrices[f'stage_{stage:02d}']['include'].append(planned)
with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
    output.write('prefix_matrices=' + json.dumps(prefix_matrices) + '\n')
    output.write('prefix_stages=' + str(max((plan['prefix_stage'] + 1 for plan in prefix_plans.values()), default=0)) + '\n')
shards = []
for (record, kind, identity), members in sorted(groups.items()):
    members.sort(key=lambda r: r.get('start_frame', 0))
    for i, chunk in enumerate(chunks(kind, members)):
        inputs = chunk[0]['inputs']
        asset, digest = seed_bundle(chunk[0]) if kind != 'route' else ('', '')
        name = identity.removeprefix('layout150-').removeprefix('seeds-v1-').split('-')[0] if kind in ('seed', 'checkpoint') else identity.removesuffix('.inputs')
        if record and 'chain' in chunk[0]:
            name = chunk[0]['chain']
        prefix = prefix_plans[prefix_keys[(record, chunk[0]['id'])]] if kind == 'checkpoint' else None
        prefix_fields = dict(prefix_chain=prefix['prefix_chain'], prefix_stage=prefix['prefix_stage'],
                             prefix_segments=prefix['segments']) if prefix else dict(prefix_chain='', prefix_segments='')
        shards.append({**prefix_fields, 'name': f"{'oracle-' if record else ''}{kind}-{name}-{i}",
                       'record_oracle': record, 'cost': sum(cost(kind, r) for r in chunk),
                       'kind': kind, 'inputs': inputs,
                       'seed_asset': asset, 'seed_sha256': digest,
                       'route': inputs.removeprefix('corpus-').removesuffix('.inputs') if kind == 'route' else '',
                       'segments': ' '.join(r['id'] for r in chunk)})
# Start the heaviest shards first; the workflow bounds simultaneous jobs.
shards.sort(key=lambda shard: -shard['cost'])
if not shards:
    raise SystemExit('no runnable rows selected')
with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as out:
    out.write(f"Selection: {source}\n\n")
    if os.environ.get('ORACLE_SAMPLE') == '1':
        out.write(f"Faithful oracle sample: {len(sampled)} locked scenarios, UTC slot {slot}\n\n")
    out.write(f"{sum(len(s['segments'].split()) for s in shards)} rows in {len(shards)} shards\n\n")
    out.write('Skipped:\n\n' + ''.join(f'- {i}: {why}\n' for i, why in skipped))
with open(os.environ['GITHUB_OUTPUT'], 'a') as out:
    out.write('matrix=' + json.dumps({'include': shards}) + '\n')
    ready = [shard for shard in shards if shard['kind'] != 'checkpoint']
    checkpoints = [shard for shard in shards if shard['kind'] == 'checkpoint']
    out.write('ready_matrix=' + json.dumps({'include': ready}) + '\n')
    out.write('checkpoint_matrix=' + json.dumps({'include': checkpoints}) + '\n')
    out.write('ready_count=' + str(len(ready)) + '\n')
    out.write('checkpoint_count=' + str(len(checkpoints)) + '\n')
