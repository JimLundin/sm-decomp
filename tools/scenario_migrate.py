#!/usr/bin/env python3
"""Write the scenario files and their lock from movies/segments.toml and the route CSVs.

A one-time migration (docs/SCENARIO_HARNESS.md, Migration step 3): every corpus
row becomes a scenario in scenarios/corpus.toml, keeping its comments, and every
room of both routes one in scenarios/route-100.toml or route-any.toml. Their
input, seed and checkpoint names and SHA-256s, plan_corpus.py's bundle of each
row and the routes' bundles become pins in scenarios/lock.toml, beside the
oracle digests it already holds. `check` proves the scenarios give back every
row unchanged. Run it from the repository root.
"""
import ast, csv, os, pathlib, sys, tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path[:0] = [str(ROOT / "tools"), str(ROOT / ".github/scripts")]
os.environ.setdefault("FULL_ROUTE", "100")
import full_route, lock_gate, scenario  # noqa: E402

SEGMENTS = ROOT / "movies/segments.toml"
CHAINS = {"100": "movies/full-route-chain.csv", "any": "movies/full-route-chain-any.csv"}
BUNDLES = {"100": ("full-route-chain-v1-72e0f64e0d47b53d.tar.gz.enc",
                   "72e0f64e0d47b53d5110cb8c31e13571c14bc95684696aa8ce7f62c05a6f6dc1"),
           "any": ("full-route-chain-any-v1-5d139284193bc614.tar.gz.enc",
                   "5d139284193bc61443855b0fa82128abf44e66f2ad03a7dd5360bbbbef582f03")}


def seed_bundle():
    """plan_corpus.py's row-to-bundle mapping, without running the planner."""
    tree = ast.parse((ROOT / ".github/scripts/plan_corpus.py").read_text())
    function = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "seed_bundle")
    space = {}
    exec(compile(ast.Module([function], []), "plan_corpus.py", "exec"), space)
    return space["seed_bundle"]


def comments():
    """The comment lines before each row, by row id."""
    found, pending, current = {}, [], None
    for line in SEGMENTS.read_text().splitlines():
        if line.startswith("#"):
            pending.append(line)
        elif not line.strip():
            if pending:
                pending.append("")
        elif line == "[[segment]]":
            current = list(pending)
            pending = []
        elif line.startswith("id = ") and current is not None:
            while current and not current[-1]:
                current.pop()
            found[tomllib.loads(line)["id"]] = current
            current = None
        else:
            pending = []
    return found


def legacy():
    """Every row as the corpus and route jobs read them before the migration."""
    with SEGMENTS.open("rb") as stream:
        recipe = tomllib.load(stream)
    rows = {row["id"]: dict(row) for row in recipe["segment"]}
    for route, chain in CHAINS.items():
        settings = full_route.ROUTES[route]
        with (ROOT / settings["rooms"]).open(newline="") as stream:
            seeds = {row["room_index"]: row["observation_sha256"] for row in csv.DictReader(stream)}
        with (ROOT / chain).open(newline="") as stream:
            for entry in csv.DictReader(stream):
                index = int(entry["room_index"])
                name = full_route.room_id(route, index, entry["room"])
                rows[name] = dict(
                    id=name, seed=f"{settings['capture']}:{index}", seed_sha256=seeds[entry["room_index"]],
                    inputs=f"chain-{route}-{index:04d}.inputs", input_sha256=entry["input_sha256"],
                    frames=int(entry["steps"]), chain=route,
                    expect=[f"room:{entry['next_room']}" if entry["ends"] == "next-room" else "state:8"])
    return recipe["required"], rows


def migrate():
    required, rows = legacy()
    notes, bundle = comments(), seed_bundle()
    corpus, routes, pins, bundles = [], {"100": [], "any": []}, [], {}
    for name, row in rows.items():
        entry = {"id": name, "start": f"seed:{row['seed_sha256']}" if "seed" in row else "boot"}
        if row.get("start_frame"):
            entry["prefix"] = row["start_frame"]
        entry["ticks"] = row["frames"]
        entry["facts"] = row.get("expect", []) + [f"probe:{probe}" for probe in row.get("require_hit", [])]
        for key in ("after_expectations", "after_hits", "pending"):
            if key in row:
                entry[key] = row[key]
        if name in required:
            entry["tags"] = ["required"]
        if name in lock_gate.UNLOCKED:
            entry["unlocked"] = lock_gate.UNLOCKED[name]
        pin = {"id": name}
        if "chain" in row:
            pin["bundle"], bundles[pin["bundle"]] = BUNDLES[row["chain"]]
            index = int(row["seed"].split(":")[1])
            if row["chain"] == "100" and index in full_route.REGRESSIONS:
                entry["regression"] = full_route.REGRESSIONS[index]
            routes[row["chain"]].append(entry)
        else:
            if not (row["inputs"].startswith("corpus-") and "seed" not in row and "checkpoint" not in row):
                pin["bundle"], bundles[pin["bundle"]] = bundle(row)
            corpus.append((notes.get(name, []), entry))
        for key in scenario.PIN_FIELDS[1:]:
            if key in row and key != "bundle":
                pin[key] = row[key]
        pins.append(pin)
    lines = ["# Corpus scenarios (docs/SCENARIO_HARNESS.md, Formats). Candidate checks,",
             "# not pass records: coverage stays incomplete while any `required` one is pending.",
             "# Inputs, seeds and checkpoints are private; scenarios/lock.toml pins them.",
             "version = 1", f"domains = {scenario.toml_value(list(scenario.DOMAINS))}", ""]
    for note, entry in corpus:
        lines += note + ["[[scenario]]"] + [f"{key} = {scenario.toml_value(value)}" for key, value in entry.items()] + [""]
    (ROOT / "scenarios/corpus.toml").write_text("\n".join(lines), newline="\n")
    for route, entries in routes.items():
        scenario.write_route(ROOT, route, entries)
    lock = scenario.read_lock(ROOT)
    lock.bundles, lock.pins = bundles, {pin["id"]: pin for pin in pins}
    scenario.write_lock(ROOT / "scenarios/lock.toml", lock)


def check():
    """Every row comes back from the scenarios exactly; and every bundle mapping."""
    required, rows = legacy()
    loaded = {row["id"]: row for row in scenario.rows(ROOT)}
    bundle, lock = seed_bundle(), scenario.read_lock(ROOT)
    problems = [] if list(loaded) == list(rows) else ["scenario order or ids differ"]
    for name, row in rows.items():
        mine = {key: value for key, value in loaded[name].items()
                if key not in ("bundle", "tags", "unlocked", "regression", "observe") and value not in ([], 0)}
        theirs = {key: value for key, value in row.items() if value not in ([], 0)}
        if mine != theirs:
            problems.append(f"{name}: {sorted(key for key in mine.keys() | theirs.keys() if mine.get(key) != theirs.get(key))}")
        if "chain" not in row and "bundle" in loaded[name]:
            if bundle(row) != (loaded[name]["bundle"], lock.bundles[loaded[name]["bundle"]]):
                problems.append(f"{name}: bundle differs")
    if [name for name in loaded if "required" in loaded[name]["tags"]] != [n for n in rows if n in required]:
        problems.append("required ids differ")
    if problems:
        raise SystemExit("\n".join(str(problem) for problem in problems))
    print(f"{len(rows)} rows round-trip: {len(rows) - sum('chain' in r for r in rows.values())} corpus rows, "
          f"{sum('chain' in r for r in rows.values())} route rooms, {len(required)} required")


if __name__ == "__main__":
    {"migrate": migrate, "check": check}[sys.argv[1]]()
