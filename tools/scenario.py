#!/usr/bin/env python3
"""Scenarios as data (docs/SCENARIO_HARNESS.md): the scenario files, their lock, `lock` and `cut`.

Each scenarios/*.toml file but lock.toml holds one table per scenario, and
only what REVIEW lane S condition 5 allows: its id, its start by digest, the
facts it must show and the domains it compares. Inputs, seeds and checkpoints
stay in the encrypted ci-data bundles.

  start = "boot" | "seed:<payload SHA-256>"   the oracle-side start
  prefix = N        input ticks from the start before the scenario's own (default 0)
  ticks = N         the budget
  facts = [...]     probe:NAME, a probe the engine must hit;
                    observe:DOMAIN[.FIELD]@TICK, a field the comparison covers at that tick;
                    or an expectation the verifier checks (room:DF45, boss:kraid, state:8, ...)
  after_expectations, after_hits   ticks run on after the expectations or probes
  tags = [...]      required (the coverage scope), route:100, ...
  pending = "why"   not runnable yet
  unlocked = "why"  stays on patched lockstep, with its docs/LEGACY.md entry
  regression = "why"  a divergence it once exposed; it must keep passing
A file's top-level `tags` apply to each of its scenarios; its `domains` name
the observation domains they compare.

scenarios/lock.toml is generated; never edit it. `pins` give each scenario's
private files by name and SHA-256 and the bundle holding them, `bundles` each
ci-data asset's ciphertext SHA-256, and each `[[scenario]]` the faithful
oracle's running digests (REVIEW lane S condition 8).

  lock [--bundle FILE]... [--pin ID KEY=VALUE...]... [--record]
        rehash the local inputs, seeds, checkpoints and bundles, write the lock
        and, with --record, record the oracle for every stale lockable scenario
  lock --check
        check the scenarios and lock are complete and current, without data (CI)
"""
import argparse
import csv
import dataclasses
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
LOCK = "scenarios/lock.toml"
# The observation domains every scenario compares: the whole-machine streams
# (`ram`) and each typed domain (sm_tools::observation). Narrowing them would
# be a tolerance (REVIEW lane S, decision (a)), so every file names them all.
DOMAINS = ("ram", "input", "samus", "camera", "room", "progress", "actors", "effects")
FIELDS = {"id", "start", "prefix", "ticks", "facts", "after_expectations", "after_hits",
          "tags", "pending", "unlocked", "regression"}
PIN_FIELDS = ("id", "bundle", "seed", "inputs", "input_sha256", "checkpoint", "checkpoint_sha256",
              "faithful_checkpoint", "faithful_checkpoint_sha256")
SEED_REGIONS = (("wram", 131072), ("sram", 8192), ("vram", 65536),
                ("cgram", 512), ("oam", 544), ("apuram", 65536))
DIGEST = re.compile(r"[0-9a-f]{64}")
OBSERVE = re.compile(r"observe:([a-z]+)(?:\.[A-Za-z0-9_.\[\]]+)?@([0-9]+)")


@dataclasses.dataclass
class Lock:
    bundles: dict
    pins: dict
    entries: dict


def toml_value(value):
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool) or not isinstance(value, (int, list, dict)):
        raise TypeError(f"no TOML form for {value!r}")
    if isinstance(value, int):
        return str(value)
    if isinstance(value, list):
        return "[" + ", ".join(map(toml_value, value)) + "]"
    return "{ " + ", ".join(f"{toml_value(key)} = {toml_value(item)}" for key, item in value.items()) + " }"


def table(entry, keys=None):
    """One inline table, with bare keys in the given order."""
    return "{ " + ", ".join(f"{key} = {toml_value(entry[key])}" for key in (keys or entry) if key in entry) + " }"


def sha256(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def load(root=ROOT):
    """Every scenario by id, in file order, with its file's tags and domains applied."""
    scenarios = {}
    for path in sorted((root / "scenarios").glob("*.toml")):
        if path.name == Path(LOCK).name:
            continue
        with path.open("rb") as stream:
            data = tomllib.load(stream)
        if set(data) - {"version", "domains", "tags", "scenario"} or data.get("version") != 1:
            raise ValueError(f"{path.name}: needs version = 1 and only domains, tags and scenario")
        if tuple(data.get("domains", ())) != DOMAINS:
            raise ValueError(f"{path.name}: domains must name every compared domain {list(DOMAINS)}")
        for entry in data.get("scenario", []):
            name = entry.get("id")
            if set(entry) - FIELDS or not isinstance(name, str) or not name or name in scenarios:
                raise ValueError(f"{path.name}: scenario {name!r} has unknown fields or a duplicate id")
            start = entry.get("start", "")
            if start != "boot" and not (start.startswith("seed:") and DIGEST.fullmatch(start[5:])):
                raise ValueError(f"{name}: start must be boot or seed:<SHA-256>")
            scenarios[name] = dict(entry, tags=data.get("tags", []) + entry.get("tags", []),
                                   domains=list(DOMAINS), file=path.name)
    return scenarios


def read_lock(root=ROOT, path=None):
    path = path or root / LOCK
    if not path.exists():
        return Lock({}, {}, {})
    with path.open("rb") as stream:
        data = tomllib.load(stream)
    return Lock({bundle["name"]: bundle["sha256"] for bundle in data.get("bundles", [])},
                {pin["id"]: pin for pin in data.get("pins", [])},
                {entry["id"]: entry for entry in data.get("scenario", [])})


def oracle_text(entry):
    """One `[[scenario]]` table of oracle digests."""
    lines = ["[[scenario]]"] + [f'{key} = "{entry[key]}"' for key in ("id", "oracle", "start", "inputs")]
    lines += [f"{key} = {entry[key]}" for key in ("frames", "format", "ticks")]
    lines.append(f'combined = "{entry["combined"]}"')
    for key in ("digests", "marks"):
        lines.append(f"{key} = {{ " + ", ".join(f'"{name}" = "{value}"' for name, value in entry[key].items()) + " }")
    return "\n".join(lines) + "\n"


def write_lock(path, lock):
    text = ["# Generated by tools/scenario.py lock and tools/verify_dll.py --mode lock; never edit.",
            "# bundles: each ci-data asset's ciphertext SHA-256. pins: each scenario's private",
            "# inputs, seed and checkpoints by name and SHA-256, and the bundle that holds them.",
            "# [[scenario]]: the faithful oracle's running digests (REVIEW lane S condition 8).", "",
            "bundles = ["]
    text += [f"    {table(dict(name=name, sha256=digest))}," for name, digest in sorted(lock.bundles.items())]
    text += ["]", "pins = ["]
    text += [f"    {table(lock.pins[name], PIN_FIELDS)}," for name in sorted(lock.pins)]
    text += ["]", ""]
    text += [oracle_text(lock.entries[name]) for name in sorted(lock.entries)]
    Path(path).write_text("\n".join(text), encoding="utf-8", newline="\n")


def write_route(root, route, entries):
    """A route's room scenarios, one line each (full_route.py inputs writes them)."""
    lines = [f"# Route {route}: each room from its original-emulator seed into the next room",
             "# (.github/scripts/full_route.py). Generated by full_route.py inputs, which keeps",
             "# each room's unlocked and regression notes.",
             "version = 1", f"domains = {toml_value(list(DOMAINS))}", f'tags = ["route:{route}"]', "scenario = ["]
    lines += [f"    {table(entry, ('id', 'start', 'ticks', 'facts', 'pending', 'unlocked', 'regression'))},"
              for entry in entries]
    (root / f"scenarios/route-{route}.toml").write_text("\n".join(lines + ["]", ""]), newline="\n")


def rows(root=ROOT):
    """Each scenario as the runner's row: its pinned files, budget and facts."""
    pins = read_lock(root).pins
    found = []
    for name, entry in load(root).items():
        pin = pins.get(name, {})
        row = {"id": name}
        if "pending" in entry:
            row["pending"] = entry["pending"]
        if entry["start"] != "boot":
            row["seed"], row["seed_sha256"] = pin.get("seed", ""), entry["start"].removeprefix("seed:")
        row.update((key, pin[key]) for key in PIN_FIELDS[2:] if key in pin)
        row["start_frame"], row["frames"] = entry.get("prefix", 0), entry["ticks"]
        facts = entry.get("facts", [])
        row["expect"] = [fact for fact in facts if not fact.startswith(("probe:", "observe:"))]
        row["require_hit"] = [fact.removeprefix("probe:") for fact in facts if fact.startswith("probe:")]
        row["observe"] = [fact for fact in facts if fact.startswith("observe:")]
        row.update((key, entry[key]) for key in ("after_expectations", "after_hits", "unlocked", "regression")
                   if key in entry)
        if "bundle" in pin:
            row["bundle"] = pin["bundle"]
        row["tags"] = entry["tags"]
        route = next((tag.removeprefix("route:") for tag in entry["tags"] if tag.startswith("route:")), None)
        if route:
            row["chain"] = route
        found.append(row)
    return found


def recipe(root=ROOT):
    """The corpus the runner selects from: the required ids and every scenario's row."""
    found = rows(root)
    return [row["id"] for row in found if "required" in row["tags"]], found


def seed_file(root, seed):
    """The WRAM file of seed capture:index, which names its companions."""
    capture, index = seed.split(":")
    directory = root / "target/seeds" / capture
    with (directory / "rooms.csv").open(newline="") as file:
        entries = [row for row in csv.DictReader(file)
                   if row["room_index"] == str(int(index)) and row["region"] == "wram"]
    if len(entries) != 1:
        raise ValueError("seed index requires exactly one captured WRAM entry")
    filename = entries[0]["file"]
    if Path(filename).name != filename or not filename.endswith("-wram.bin"):
        raise ValueError("invalid indexed seed filename")
    return directory / filename


def seed_digest(seed):
    """A seed's payload SHA-256: its regions and registers, as the start digest names it."""
    digest = hashlib.sha256()
    for region, size in SEED_REGIONS:
        data = seed.with_name(seed.name.replace("-wram.bin", f"-{region}.bin")).read_bytes()
        if len(data) != size:
            raise ValueError(f"seed {region} requires {size} bytes")
        digest.update(data)
    digest.update(seed.with_name(seed.name.replace("-wram.bin", "-registers.csv")).read_bytes())
    return digest.hexdigest()


def lock_key(row):
    """What an oracle lock entry was recorded from: the start, the inputs and the budget."""
    return dict(start=f"seed:{row['seed_sha256']}" if "seed" in row else "boot",
                inputs=row["input_sha256"], frames=row["frames"])


def lockable(row):
    # A prefix start needs an oracle-side prefix first; a pending one has no data.
    return "pending" not in row and not row["start_frame"] and "checkpoint" not in row


def captured_seeds(root, capture):
    """Each room's observation SHA-256 in a committed capture index, if there is one."""
    path = root / f"movies/room-seeds-{capture}.csv"
    if not path.exists():
        return None
    with path.open(newline="") as stream:
        return {row["room_index"]: row.get("observation_sha256") for row in csv.DictReader(stream)}


def problems(root=ROOT):
    """Everything that keeps the scenarios and their lock from being complete and current."""
    found, lock = [], read_lock(root)
    loaded = rows(root)
    names = {row["id"] for row in loaded}
    used = set()
    for row in loaded:
        name, pin = row["id"], lock.pins.get(row["id"], {})
        if "pending" not in row:
            if not pin.get("inputs") or not DIGEST.fullmatch(pin.get("input_sha256", "")):
                found.append(f"{name}: no pinned inputs")
            if "seed" in row and not re.fullmatch(r"[A-Za-z0-9-]+:[1-9][0-9]*", row["seed"]):
                found.append(f"{name}: a seeded start needs its pinned seed capture:index")
            if not row["require_hit"] and not row["expect"] and not row["observe"]:
                found.append(f"{name}: no fact demonstrates coverage")
            if "bundle" not in pin and not (pin.get("inputs", "").startswith("corpus-") and "seed" not in row
                                            and "checkpoint" not in pin):
                found.append(f"{name}: its private data is in no bundle")
        for key in ("checkpoint", "faithful_checkpoint"):
            if key in pin and not DIGEST.fullmatch(pin.get(key + "_sha256", "")):
                found.append(f"{name}: {key} needs its SHA-256")
        if "bundle" in pin:
            used.add(pin["bundle"])
            if pin["bundle"] not in lock.bundles:
                found.append(f"{name}: bundle {pin['bundle']} has no pinned SHA-256")
        if "seed" in row and row["seed"]:
            capture, index = row["seed"].split(":")
            seeds = captured_seeds(root, capture)
            if seeds and None not in seeds.values() and seeds.get(index) != row["seed_sha256"]:
                found.append(f"{name}: start differs from seed {row['seed']} in movies/room-seeds-{capture}.csv")
        for fact in row["observe"]:
            match = OBSERVE.fullmatch(fact)
            if (not match or match[1] not in DOMAINS
                    or not row["start_frame"] <= int(match[2]) < row["start_frame"] + row["frames"]):
                found.append(f"{name}: {fact} names no compared domain at a tick the scenario runs")
    found += [f"{name}: pinned, but no such scenario" for name in sorted(set(lock.pins) - names)]
    found += [f"{name}: pinned, but no scenario uses it" for name in sorted(set(lock.bundles) - used)]
    # The oracle lock: each lockable scenario is locked or listed as unlocked.
    keys = {row["id"]: row for row in loaded if lockable(row)}
    for name, entry in sorted(lock.entries.items()):
        if name not in keys:
            found.append(f"{name}: locked, but no such lockable scenario")
        elif "unlocked" in keys[name]:
            found.append(f"{name}: both locked and unlocked")
        elif entry["oracle"] != "faithful" or any(entry[key] != value for key, value in lock_key(keys[name]).items()):
            found.append(f"{name}: stale lock entry (start, inputs or budget changed); re-record it")
    for row in loaded:
        if "unlocked" in row and row["id"] not in keys:
            found.append(f"{row['id']}: unlocked, but not lockable")
        elif row["id"] in keys and row["id"] not in lock.entries and "unlocked" not in row:
            found.append(f"{row['id']}: neither locked nor unlocked")
    return found


def check(root=ROOT):
    """The locked ids; exits naming every problem."""
    found = problems(root)
    if found:
        raise SystemExit("Scenarios or their lock are not current:\n" + "\n".join(found))
    lock, loaded = read_lock(root), rows(root)
    print(f"Scenarios current: {len(loaded)} scenarios, {len(lock.pins)} pins, {len(lock.bundles)} bundles; "
          f"{len(lock.entries)} locked, {sum('unlocked' in row for row in loaded)} unlocked")
    return set(lock.entries)


def replace_start(root, row, digest):
    """Rewrite a scenario's start digest in its file, after its seed was recaptured."""
    scenarios = load(root)
    path = root / "scenarios" / scenarios[row["id"]]["file"]
    text = path.read_text(encoding="utf-8")
    at = text.index(f"id = {toml_value(row['id'])}")
    old = f'"seed:{row["seed_sha256"]}"'
    end = text.index(old, at)
    path.write_text(text[:end] + f'"seed:{digest}"' + text[end + len(old):], encoding="utf-8", newline="\n")


def lock(args):
    root, data = args.root, read_lock(args.root)
    for name, *values in args.pin:
        pin = data.pins.setdefault(name, {"id": name})
        for value in values:
            key, _, item = value.partition("=")
            if key not in PIN_FIELDS[1:]:
                raise SystemExit(f"--pin {name}: unknown key {key}")
            if item:
                pin[key] = item
            else:
                pin.pop(key, None)
    for bundle in args.bundle:
        data.bundles[bundle.name] = sha256(bundle)
    loaded = {row["id"]: row for row in rows(root)}
    changed = 0
    for name, pin in data.pins.items():
        files = [("input_sha256", root / "movies" / pin.get("inputs", ""))]
        files += [(key + "_sha256", root / "target/checkpoints" / pin[key])
                  for key in ("checkpoint", "faithful_checkpoint") if key in pin]
        for key, path in files:
            if path.is_file() and pin.get(key) != (digest := sha256(path)):
                pin[key], changed = digest, changed + 1
        row = loaded.get(name)
        if row and row.get("seed") and (root / "target/seeds" / row["seed"].split(":")[0] / "rooms.csv").exists():
            digest = seed_digest(seed_file(root, row["seed"]))
            if digest != row["seed_sha256"]:
                replace_start(root, row, digest)
                changed += 1
    data.bundles = {name: digest for name, digest in data.bundles.items()
                    if any(pin.get("bundle") == name for pin in data.pins.values())}
    write_lock(root / LOCK, data)
    print(f"Lock: {changed} digests updated; {len(data.pins)} pins, {len(data.bundles)} bundles")
    if args.record:
        stale = [row["id"] for row in rows(root) if lockable(row) and "unlocked" not in row
                 and (row["id"] not in data.entries
                      or any(data.entries[row["id"]][key] != value for key, value in lock_key(row).items()))]
        print(f"Lock: recording {len(stale)} stale scenarios on the faithful oracle", flush=True)
        if stale:
            subprocess.run([sys.executable, str(root / "tools/verify_dll.py"), str(args.rom), "--corpus",
                            str(root / "scenarios"), "--mode", "lock", "--oracle", "faithful", "--lock",
                            str(root / LOCK), *[part for name in stale for part in ("--segment", name)]],
                           cwd=root, check=True)
    check(root)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--rom", type=Path, default=ROOT / "rom/sm.smc")
    commands = parser.add_subparsers(dest="command", required=True)
    locking = commands.add_parser("lock")
    locking.add_argument("--check", action="store_true")
    locking.add_argument("--bundle", type=Path, action="append", default=[])
    locking.add_argument("--pin", nargs="+", action="append", default=[], metavar="ID KEY=VALUE")
    locking.add_argument("--record", action="store_true")
    args = parser.parse_args()
    args.root = args.root.resolve()
    check(args.root) if args.check else lock(args)


if __name__ == "__main__":
    main()
