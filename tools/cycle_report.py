#!/usr/bin/env python3
"""Measure how much production code is free of the backend dependency cycle.

The milestone plan in docs/REVIEW.md counts progress as behaviour that leaves
the cycle formed by game/ and the RAM-compatible runtime (game_state, ram_map,
rtl, cpu_infra, snes/ ...). This report is static: it resolves `crate::`,
`super::`/`self::` and `use` paths to files, marks files that contain real
unsafe code or touch the backend, propagates that to everything depending on
them, and prints the remaining coupling counts.

Run from the repository root: python3 tools/cycle_report.py [--json]
"""
import json
import os
import re
import sys
from collections import Counter, defaultdict

SRC = "sm/src"
REAL_UNSAFE = re.compile(r"\bunsafe\s*(fn|\{|impl|extern)")
BACKEND_REF = re.compile(
    r"\b(ram_map|game_state::|crate::snes|snes::|cpu_infra|rtl::|g_ram|extern\s+\"C\"|no_mangle)\b|\*mut |\*const ")
BACKEND_FILE = re.compile(
    r"^(snes/|platform/)|^(lib|main|game_state|ram_map|cpu_infra|rtl|spc_player|native_\w+|apu_\w+|"
    r"audio_\w+|c_library|config|execution|recording|snapshot_io|save_ram|word_transfers)\.rs$")
COUPLINGS = {
    "Game handle (files)": (re.compile(r"\bgame_state::Game\b|\bGame\b.*game_state|&mut Game\b"), "files"),
    "ram_map addresses (files)": (re.compile(r"\bram_map\b"), "files"),
    "rom_table_word reads (sites)": (re.compile(r"\brom_table_word\b"), "sites"),
    "PPU register writes (sites)": (re.compile(r"\b(write_reg|write_reg_word|write_register_and_mirror)\b"), "sites"),
    "audio backend calls (sites)": (re.compile(r"native_audio::(write_sound_port|upload_bank)\b"), "sites"),
}

# Rewrite progress: compat RAM and register access still inside sm_game. Moved
# code that reads the emulator's RAM image is relocated, not yet owned state.
RAM_COUPLINGS = {
    "named RAM words (word!/checked_word!/ram_word!)": re.compile(r"\b(?:ram_)?(?:checked_)?word!\s*[({]"),
    "raw enemy slots (slot!)": re.compile(r"\bslot!\s*[({]"),
    "word transfers (word_transfers!)": re.compile(r"\bword_transfers!"),
    "register writes (register_writes!/write_register)": re.compile(r"\bregister_writes!|\bwrite_register(?:_word)?\("),
    "raw RAM hosts (RawRam/compatibility_byte)": re.compile(r"\bRawRam\b|\bcompatibility_(?:write_)?byte\b"),
    # The cinematic state view names its words by Session field, but each access
    # still reads and stores the host's projected RAM bytes; count its call sites.
    "cinematic state view (cinematic!/cinematic_state::)": re.compile(
        r"\bcinematic!\s*[({]\s*game\b"
        r"|\bcinematic_state::(?:get|set|get_at|set_at|get_offset|set_offset|add_pair|clear)\(\s*game\b"),
    # The other views that name RAM words and still resolve to the host's
    # projected bytes (CompatibilityRam, or SlotAccess for the boss view): count
    # their call sites, not their definitions (whose receivers are `$game`).
    "RAM word views (boss!/scroll!/inventory!/melt!/menu!)": re.compile(
        r"\b(?:boss|scroll|inventory|melt|menu)!\s*[({](?!\s*\$)"),
    "RAM word view calls (flow/inventory/scroll/message_box/room_view/...)": re.compile(
        r"\b(?:flow::(?:set|current)|inventory::(?:get|set|current)|scroll::(?:get|set)"
        r"|message_box::(?:get|set)|room_view::(?:set|set_effect|current|effects|enter_header)"
        r"|camera::current|menu_state::(?:get|set|current|get_byte|set_byte)"
        r"|screen_fade::(?:current|set_both|set_counter|set_low)|melt::(?:get|set)"
        r"|boss_ram::(?:get|set|get_at|set_at|fill)|LiquidWords::(?:read|write))\((?!\s*&?\*?\$)"),
}


def sm_game_ram_couplings():
    counts, files = {}, {}
    sources = {}
    for root, _, names in os.walk(os.path.join("crates", "sm_game", "src")):
        for name in names:
            if name.endswith(".rs"):
                with open(os.path.join(root, name), encoding="utf-8", errors="replace") as source:
                    sources[os.path.join(root, name)] = source.read()
    for label, pattern in RAM_COUPLINGS.items():
        hits = {path: len(pattern.findall(text)) for path, text in sources.items()}
        counts[label] = sum(hits.values())
        files[label] = sum(1 for n in hits.values() if n)
    coupled = sum(1 for text in sources.values() if any(p.search(text) for p in RAM_COUPLINGS.values()))
    return {"files": len(sources), "coupled_files": coupled, "sites": counts, "site_files": files}


def verification(rel):
    return (rel.startswith(("verification/", "verify/", "bin/"))
            or re.search(r"(^|/)verify[^/]*\.rs$", rel) is not None)


def load():
    files = {}
    for root, _, names in os.walk(SRC):
        for name in names:
            if name.endswith(".rs"):
                rel = os.path.relpath(os.path.join(root, name), SRC).replace(os.sep, "/")
                if not verification(rel):
                    with open(os.path.join(root, name), encoding="utf-8", errors="replace") as f:
                        files[rel] = re.sub(r"//[^\n]*", "", f.read())
    return files


def modpath(rel):
    parts = rel[:-3].split("/")
    return tuple(parts[:-1] if parts[-1] == "mod" else parts)


def use_paths(tree):
    """Expand a `use` tree such as a::{b, c::{d, e}} into paths."""
    tree = re.sub(r"\s+", "", tree)
    out = []

    def walk(prefix, text):
        depth, cur, parts = 0, "", []
        for ch in text:
            depth += (ch == "{") - (ch == "}")
            if ch == "," and depth == 0:
                parts.append(cur)
                cur = ""
            else:
                cur += ch
        parts.append(cur)
        for part in filter(None, parts):
            nested = re.match(r"^([\w:]*?)::\{(.*)\}$", part)
            if nested:
                walk(prefix + [p for p in nested.group(1).split("::") if p], nested.group(2))
            elif part.startswith("{") and part.endswith("}"):
                walk(prefix, part[1:-1])
            else:
                out.append(prefix + [p for p in re.sub(r"as\w+$", "", part).split("::") if p])

    walk([], tree)
    return out


def graph(files):
    by_mod = {modpath(r): r for r in files}

    def resolve(path):
        for i in range(len(path), 0, -1):
            hit = by_mod.get(tuple(path[:i]))
            if hit:
                return hit
        return None

    deps = defaultdict(set)
    for rel, src in files.items():
        me = list(modpath(rel))
        paths = [p for m in re.finditer(r"\buse\s+([^;]+);", src) for p in use_paths(m.group(1))]
        # Only maximal paths: a prefix of a::b::{..} or a::* belongs to a use tree expanded above.
        paths += [[m.group(1)] + m.group(2).split("::")[1:]
                  for m in re.finditer(r"\b(crate|super|self)((?:::\w+)+)(?!\w|::)", src)]
        for p in paths:
            if not p or p[0] not in ("crate", "super", "self"):
                continue
            if p[0] == "crate":
                p = p[1:]
            else:
                base = me[:]
                while p and p[0] in ("super", "self"):
                    base = base[:-1] if p[0] == "super" else base
                    p = p[1:]
                p = base + p
            target = resolve(p)
            if target and target != rel:
                deps[rel].add(target)
        for m in re.finditer(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;", src, re.M):
            child = by_mod.get(tuple(me + [m.group(1)]))
            if child:
                deps[rel].add(child)
    return deps


def main():
    files = load()
    lines = {r: len(s.splitlines()) for r, s in files.items()}
    deps = graph(files)
    tainted = {r for r, s in files.items()
               if REAL_UNSAFE.search(s) or BACKEND_FILE.search(r) or BACKEND_REF.search(s)}
    changed = True
    while changed:
        changed = False
        for rel in files:
            if rel not in tainted and deps[rel] & tainted:
                tainted.add(rel)
                changed = True
    game = [r for r in files if r.startswith("game/")]
    free = sorted(r for r in game if r not in tainted)
    legacy = [r for r in game if re.fullmatch(r"game/sm_[0-9a-f]{2}\.rs", r)]
    coupling = {}
    for name, (pattern, unit) in COUPLINGS.items():
        hits = Counter({r: len(pattern.findall(files[r])) for r in game})
        coupling[name] = sum(1 for n in hits.values() if n) if unit == "files" else sum(hits.values())
    package_lines = {}
    for crate in sorted(os.listdir("crates")) if os.path.isdir("crates") else []:
        total = 0
        for root, _, names in os.walk(os.path.join("crates", crate, "src")):
            for name in names:
                if name.endswith(".rs"):
                    with open(os.path.join(root, name), encoding="utf-8", errors="replace") as f:
                        total += len(f.read().splitlines())
        package_lines[crate] = total
    report = {
        "target_package_lines": package_lines,
        "game_files": len(game),
        "game_lines": sum(lines[r] for r in game),
        "outside_cycle_files": len(free),
        "outside_cycle_lines": sum(lines[r] for r in free),
        "legacy_bank_lines": sum(lines[r] for r in legacy),
        "game_unsafe_sites": sum(len(re.findall(r"\bunsafe\s*(?:fn|\{|impl)", files[r])) for r in game),
        "couplings": coupling,
        "sm_game_ram": sm_game_ram_couplings(),
        "outside_cycle": free,
    }
    if "--json" in sys.argv:
        print(json.dumps(report, indent=1))
        return
    for crate, total in package_lines.items():
        print(f"{crate}: {total} lines")
    print(f"game/: {report['game_files']} files, {report['game_lines']} lines")
    print(f"outside backend cycle: {report['outside_cycle_files']} files, {report['outside_cycle_lines']} lines")
    print(f"legacy bank lines: {report['legacy_bank_lines']}")
    print(f"unsafe fns/blocks/impls in game/: {report['game_unsafe_sites']}")
    for name, value in coupling.items():
        print(f"  {name}: {value}")
    ram = report["sm_game_ram"]
    print(f"sm_game files still on compat RAM/registers: {ram['coupled_files']} of {ram['files']}")
    for name, value in ram["sites"].items():
        print(f"  {name}: {value} sites in {ram['site_files'][name]} files")


if __name__ == "__main__":
    main()
