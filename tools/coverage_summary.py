#!/usr/bin/env python3
"""Summarise corpus source coverage of the instrumented verifier DLL.

Usage: coverage_summary.py LLVM_COV PROFDATA OBJECT OUTPUT.md

Reads `llvm-cov export` in memory and writes a markdown report covering the
engine sources only (crates/sm_game and sm/src/game): per-module line and
function coverage, files never executed, and the largest never-executed
functions. Nothing ROM-derived is recorded; only source locations and counts.
"""
import collections, json, re, subprocess, sys

SCOPES = ("crates/sm_game/src/", "sm/src/game/")


def scope_of(path):
    path = path.replace("\\", "/")
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


def main():
    llvm_cov, profdata, obj, output = sys.argv[1:5]
    exported = subprocess.run(
        [llvm_cov, "export", f"-instr-profile={profdata}", obj, "-skip-expansions"],
        check=True, capture_output=True, text=True).stdout
    data = json.loads(exported)["data"][0]
    modules = collections.defaultdict(lambda: [0, 0, 0, 0])  # lines hit/total, functions hit/total
    files = {}
    for entry in data["files"]:
        path = scope_of(entry["filename"])
        if path is None:
            continue
        lines = entry["summary"]["lines"]
        functions = entry["summary"]["functions"]
        files[path] = (lines["covered"], lines["count"], functions["covered"], functions["count"])
        module = path.rsplit("/", 1)[0] if path.count("/") > 3 else path
        totals = modules[module]
        totals[0] += lines["covered"]; totals[1] += lines["count"]
        totals[2] += functions["covered"]; totals[3] += functions["count"]
    cold = collections.defaultdict(int)
    for function in data["functions"]:
        paths = [scope_of(name) for name in function["filenames"]]
        path = next((p for p in paths if p), None)
        if path is None or function["count"] != 0:
            continue
        regions = function.get("regions") or [[0, 0, 0, 0]]
        start, end = regions[0][0], max(region[2] for region in regions)
        cold[(path, start, demangle(function["name"]))] = max(cold[(path, start, demangle(function["name"]))], end - start + 1)

    def percent(hit, total):
        return f"{100 * hit / total:5.1f}%" if total else "  n/a"

    out = ["# Corpus source coverage (engine sources)", ""]
    hit = sum(v[0] for v in files.values()); total = sum(v[1] for v in files.values())
    fhit = sum(v[2] for v in files.values()); ftotal = sum(v[3] for v in files.values())
    out += [f"Lines {hit}/{total} ({percent(hit, total).strip()}); functions {fhit}/{ftotal} "
            f"({percent(fhit, ftotal).strip()}) across {len(files)} files.", "",
            "## By module", "", "| Module | Lines | Functions |", "| --- | --- | --- |"]
    for module, (lh, lt, fh, ft) in sorted(modules.items(), key=lambda item: item[1][0] / max(item[1][1], 1)):
        out.append(f"| `{module}` | {percent(lh, lt)} of {lt} | {percent(fh, ft)} of {ft} |")
    never = sorted((path for path, value in files.items() if value[0] == 0 and value[1]), key=lambda p: -files[p][1])
    out += ["", f"## Files never executed ({len(never)})", ""]
    out += [f"- `{path}` ({files[path][1]} lines)" for path in never]
    out += ["", "## Largest never-executed functions (top 150)", ""]
    for (path, line, name), size in sorted(cold.items(), key=lambda item: -item[1])[:150]:
        out.append(f"- `{path}:{line}` {name} (~{size} lines)")
    with open(output, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("\n".join(out) + "\n")
    print("\n".join(out[:3]))


if __name__ == "__main__":
    main()
