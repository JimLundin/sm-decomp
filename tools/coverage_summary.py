#!/usr/bin/env python3
"""Summarise corpus source coverage of the instrumented verifier DLL.

Usage: coverage_summary.py LLVM_COV PROFDATA OBJECT OUTPUT.md [ROUTE_PROFDATA]

Reads `llvm-cov export` in memory and writes a markdown report covering the
engine sources only (crates/sm_game and sm/src/game): per-module line and
function coverage, files never executed, and the largest never-executed
functions. With ROUTE_PROFDATA (the full 100% route) it also lists what the
route executes that the corpus never does. Nothing ROM-derived is recorded;
only source locations and counts.
"""
import collections, functools, json, os, re, subprocess, sys

SCOPES = ("crates/sm_game/src/", "sm/src/game/")


def scope_of(path):
    path = os.path.normpath(path.replace("\\", "/")).replace("\\", "/")
    for scope in SCOPES:
        at = path.find(scope)
        if at >= 0:
            return path[at:]
    return None


def demangle(name):
    """Readable path for legacy Rust symbols (_ZN..E); other names unchanged."""
    match = re.match(r"_?_ZN(.*)E$", name)
    if not match:
        return name
    body, parts = match.group(1), []
    while body and body[0].isdigit():
        length = re.match(r"\d+", body).group(0)
        body = body[len(length):]
        part, body = body[:int(length)], body[int(length):]
        if not re.fullmatch(r"h[0-9a-f]{16}", part):
            parts.append(part.replace("$LT$", "<").replace("$GT$", ">").replace("$u20$", " ")
                         .replace("..", "::").replace("$C$", ","))
    return "::".join(parts) or name


@functools.cache
def source_lines(path):
    try:
        return open(path, encoding="utf-8").read().splitlines()
    except OSError:
        return []


def source_name(path, line, fallback):
    """Name generic instantiations by their source definition (v0 symbols are unreadable)."""
    for text in reversed(source_lines(path)[max(0, line - 4):line]):
        match = re.search(r"\bfn\s+(\w+)", text)
        if match:
            return match.group(1)
    return fallback


def load(llvm_cov, profdata, obj):
    """Per-file (lines hit, lines, functions hit, functions) and per-function (count, size)."""
    exported = subprocess.run(
        [llvm_cov, "export", f"-instr-profile={profdata}", obj, "-skip-expansions"],
        check=True, capture_output=True, text=True).stdout
    data = json.loads(exported)["data"][0]
    files = {}
    for entry in data["files"]:
        path = scope_of(entry["filename"])
        if path is not None:
            lines, functions = entry["summary"]["lines"], entry["summary"]["functions"]
            files[path] = (lines["covered"], lines["count"], functions["covered"], functions["count"])
    functions = {}
    for function in data["functions"]:
        path = next((p for p in map(scope_of, function["filenames"]) if p), None)
        if path is None:
            continue
        regions = function.get("regions") or [[0, 0, 0, 0]]
        start, end = regions[0][0], max(region[2] for region in regions)
        name = demangle(function["name"])
        key = (path, start, source_name(path, start, name) if name.startswith("_R") else name)
        count, size = functions.get(key, (0, 0))
        functions[key] = (count + function["count"], max(size, end - start + 1))
    return files, functions


def percent(hit, total):
    return f"{100 * hit / total:5.1f}%" if total else "  n/a"


def main():
    llvm_cov, profdata, obj, output = sys.argv[1:5]
    route = sys.argv[5] if len(sys.argv) > 5 else None
    files, functions = load(llvm_cov, profdata, obj)
    if not files:
        raise SystemExit("no engine sources in the coverage mapping (were .lcovfun records linked?)")
    modules = collections.defaultdict(lambda: [0, 0, 0, 0])
    for path, value in files.items():
        module = path.rsplit("/", 1)[0] if path.count("/") > 3 else path
        modules[module] = [a + b for a, b in zip(modules[module], value)]

    out = ["# Corpus source coverage (engine sources)", ""]
    hit = sum(v[0] for v in files.values()); total = sum(v[1] for v in files.values())
    fhit = sum(v[2] for v in files.values()); ftotal = sum(v[3] for v in files.values())
    out += [f"Lines {hit}/{total} ({percent(hit, total).strip()}); functions {fhit}/{ftotal} "
            f"({percent(fhit, ftotal).strip()}) across {len(files)} files.", ""]
    if route:
        route_files, route_functions = load(llvm_cov, route, obj)
        rhit = sum(v[0] for v in route_files.values())
        rfhit = sum(v[2] for v in route_files.values())
        out += [f"Full 100% route alone: lines {rhit}/{total} ({percent(rhit, total).strip()}); "
                f"functions {rfhit}/{ftotal} ({percent(rfhit, ftotal).strip()}).", ""]
    out += ["## By module", "", "| Module | Lines | Functions |", "| --- | --- | --- |"]
    for module, (lh, lt, fh, ft) in sorted(modules.items(), key=lambda item: item[1][0] / max(item[1][1], 1)):
        out.append(f"| `{module}` | {percent(lh, lt)} of {lt} | {percent(fh, ft)} of {ft} |")
    never = sorted((path for path, value in files.items() if value[0] == 0 and value[1]), key=lambda p: -files[p][1])
    out += ["", f"## Files never executed ({len(never)})", ""]
    out += [f"- `{path}` ({files[path][1]} lines)" for path in never]
    if route:
        gained = {key: value for key, value in route_functions.items()
                  if value[0] and not functions.get(key, (0, 0))[0]}
        by_file = collections.defaultdict(list)
        for (path, line, name), (count, size) in gained.items():
            by_file[path].append((size, line, name, count))
        out += ["", f"## Executed by the full route but never by the corpus ({len(gained)} functions)", "",
                "Grouped by file, largest uncovered total first; route call counts in brackets.", ""]
        for path, items in sorted(by_file.items(), key=lambda item: -sum(i[0] for i in item[1])):
            out.append(f"- `{path}` (~{sum(i[0] for i in items)} lines)")
            for size, line, name, count in sorted(items, reverse=True)[:8]:
                out.append(f"  - `:{line}` {name} (~{size} lines) [{count}]")
        missed = {key for key, value in route_functions.items() if not value[0]} & {
            key for key, value in functions.items() if not value[0]}
        out += ["", f"Neither the corpus nor the full route executes {len(missed)} functions."]
    cold = {key: value[1] for key, value in functions.items() if not value[0]}
    out += ["", "## Largest never-executed functions in the corpus (top 150)", ""]
    for (path, line, name), size in sorted(cold.items(), key=lambda item: -item[1])[:150]:
        reached = " (route executes)" if route and route_functions.get((path, line, name), (0, 0))[0] else ""
        out.append(f"- `{path}:{line}` {name} (~{size} lines){reached}")
    with open(output, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("\n".join(out) + "\n")
    print("\n".join(out[:5]))


if __name__ == "__main__":
    main()
