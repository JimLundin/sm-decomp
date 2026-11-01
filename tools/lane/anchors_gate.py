# Fast gate: every anchor the dispatch-only room mutation step reaches resolves exactly once.
import importlib, pathlib, runpy, sys
root = pathlib.Path(sys.argv[1]).resolve(); V = root / "sm/src/verification"
sys.path.insert(0, str(V))
def resolve(rel):
    for c in (root / rel, root / "sm/src/game" / rel, root / "sm/src" / rel):
        if c.is_file(): return c
    hits = list((root / "sm/src").rglob(pathlib.Path(rel).name))
    return hits[0] if len(hits) == 1 else None
bad = []
def check(origin, rel, before):
    p = resolve(rel)
    if p is None: bad.append(f"{origin}: missing {rel}"); return
    c = p.read_bytes().decode().replace("\r\n", "\n").count(before.replace("\r\n", "\n"))
    if c != 1: bad.append(f"{origin}: {rel} x{c}: {before[:60]!r}")
def walk(o):
    if isinstance(o, dict):
        if "before" in o and "path" in o: yield o
        for v in o.values(): yield from walk(v)
    elif isinstance(o, list):
        for v in o: yield from walk(v)
for m in ("room_mutations", "room_entry_mutations"):
    if not (V / f"{m}.py").is_file(): continue
    for item in importlib.import_module(m).MUTATIONS:
        for rel, before, _ in item[3]: check(f"{m}:{item[0]}", rel, before)
for f in sorted(V.glob("*recipes*.py")):
    if f.name.endswith("mutations.py"): continue
    data = {k: v for k, v in runpy.run_path(str(f)).items() if k.isupper()}
    for e in walk(data): check(f.name, e["path"], e["before"])
print("\n".join(bad) or "ANCHORS OK"); sys.exit(1 if bad else 0)
