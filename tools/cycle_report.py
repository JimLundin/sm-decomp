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
    # The menu prelude's views of Session-projected words (menu_state's map,
    # save selection, options and death words, the power bomb's, and
    # session_words) read and store the host's projected bytes too.
    "Session word views (map_screen!/save_selection!/options_menu!/...)": re.compile(
        r"\b(?:map_screen|save_selection|options_menu|death_sequence|power_bomb|session_word)!"
        r"\s*[({](?!\s*\$)"),
    "RAM word view calls (flow/inventory/scroll/message_box/room_view/...)": re.compile(
        r"\b(?:flow::(?:set|current)|inventory::(?:get|set|current)|scroll::(?:get|set)"
        r"|message_box::(?:get|set)|room_view::(?:set|set_effect|current|effects|enter_header)"
        r"|camera::current|menu_state::(?:get|set|current|get_byte|set_byte)"
        r"|screen_fade::(?:current|set_both|set_counter|set_low)|melt::(?:get|set)"
        r"|boss_ram::(?:get|set|get_at|set_at|fill)|LiquidWords::(?:read|write))\((?!\s*&?\*?\$)"),
    # Typed copies still load/store RAM-addressed words through projection::word.
    # Imports and aliases are resolved below; counting only this spelling would
    # let a use declaration hide the same coupling.
    "RAM-backed cohort copies (session_cohort::load/with/with_projected)": re.compile(
        r"\bsession_cohort\s*::\s*(?:load|with|with_projected)\b\s*(?=\(|::\s*<)"),
}
COHORT_LABEL = "RAM-backed cohort copies (session_cohort::load/with/with_projected)"


def cohort_sites(text):
    """Count direct calls and imported module/function spellings once per site."""
    # Comments and literals cannot execute a copy. Preserve newlines for uses.
    text = re.sub(r'//[^\n]*|/\*.*?\*/|r(\#*)".*?"\1|"(?:\\.|[^"\\])*"',
                  " ", text, flags=re.S)
    modules = {"session_cohort"}
    functions = set()
    imports = []

    def expand(tree, prefix=""):
        depth, start = 0, 0
        parts = []
        for index, char in enumerate(tree):
            depth += (char == "{") - (char == "}")
            if char == "," and depth == 0:
                parts.append(tree[start:index])
                start = index + 1
        parts.append(tree[start:])
        for part in parts:
            part = part.strip()
            if not part:
                continue
            if "{" in part:
                base, nested = part.split("{", 1)
                expand(nested.rsplit("}", 1)[0], prefix + base.strip())
            else:
                binding = re.split(r"\s+as\s+", part)
                path = re.sub(r"\s+", "", prefix + binding[0])
                name = binding[1].strip() if len(binding) == 2 else path.split("::")[-1]
                imports.append((path, name))

    for match in re.finditer(r"\buse\s+([^;]+);", text):
        expand(match.group(1))
    # Resolve alias chains, nested use trees, {self as ...}, and glob imports.
    changed = True
    while changed:
        before = (len(modules), len(functions))
        for path, name in imports:
            parts = path.split("::")
            if parts[-1] in modules or (len(parts) > 1 and parts[-1] == "self" and parts[-2] in modules):
                modules.add(name if name != "self" else parts[-2])
            elif len(parts) > 1 and parts[-2] in modules:
                if parts[-1] == "*":
                    functions.update(("load", "with", "with_projected"))
                elif parts[-1] in ("load", "with", "with_projected"):
                    functions.add(name)
            elif parts[-1] in functions:
                functions.add(name)
        changed = before != (len(modules), len(functions))
    module = "|".join(re.escape(name) for name in sorted(modules))
    paths = rf"\b(?:{module})\s*::\s*(?:load|with|with_projected)\b"
    if functions:
        paths += r"|(?<![\w:.])(?:" + "|".join(re.escape(name) for name in sorted(functions)) + r")\b"
    return len(re.findall(rf"(?:{paths})\s*(?=\(|::\s*<)", text))


def sm_game_ram_couplings(source_root=os.path.join("crates", "sm_game", "src")):
    counts, files = {}, {}
    sources = {}
    for root, _, names in os.walk(source_root):
        for name in names:
            if name.endswith(".rs"):
                with open(os.path.join(root, name), encoding="utf-8", errors="replace") as source:
                    sources[os.path.join(root, name)] = source.read()
    for label, pattern in RAM_COUPLINGS.items():
        hits = {path: cohort_sites(text) if label == COHORT_LABEL else len(pattern.findall(text))
                for path, text in sources.items()}
        counts[label] = sum(hits.values())
        files[label] = sum(1 for n in hits.values() if n)
    coupled = sum(1 for text in sources.values()
                  if cohort_sites(text) or any(p.search(text) for label, p in RAM_COUPLINGS.items()
                                               if label != COHORT_LABEL))
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
    if "--ram-history" in sys.argv:
        ram_history_main(sys.argv[sys.argv.index("--ram-history") + 1:])
        return
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


# RAM inventory is advisory. Keep RAM_COUPLINGS and the existing ratchet intact.
RAM_INVENTORY_VERSION = "ram-source-v1"
RAM_INVENTORY_RULES = {
    "raw_ram_call": "RawRam method on an explicit .ram() receiver or file-local alias; lexical evidence, not type checking",
    "raw_base": "g_ram identifier or simple let alias; reference, not a resolved dereference/operation",
    "ram_macro": "word/checked_word/ram_word/slot/word_transfers invocations, outside macro definitions",
    "ram_view": "Existing named RAM/view call patterns; may project owned fields at runtime",
    "cohort_copy": "session_cohort load/with/with_projected calls, including file-local import aliases",
    "address_api": "SessionRam/SlotAccess/native-order methods, .word/.byte with imported RAM selector namespace, and session_slot macro; may use typed storage, never added to raw calls",
    "unresolved_method": "word/byte/read/write/copy method without explicit raw receiver or known address API spelling",
    "backend_reference": "Game/raw/interface/native/compat symbols and import aliases; references, not calls",
}
RAM_BLINDSPOTS = [
    "Lexical source inventory, not Rust name/type resolution or expanded operation counts; cfg branches counted regardless of build.",
    "SessionRam, SlotAccess and named views may select typed Session or RAM at runtime; no dynamic ownership inferred.",
    "File-local imports/simple let aliases only; shadowing, reexports across files, glob imports and function pointers unresolved.",
    "Macro definitions excluded from call totals; macro expansion multiplicity and custom/generated macros unresolved.",
    "Address construction, pointer dereferences hidden by arbitrary aliases/helpers, native foreign writers and external code unresolved.",
    "Paths define domains, not gameplay reach or semantic ownership; file moves between scopes require review.",
    "Zero matched candidates is not proof of zero engine RAM dependencies or complete gameplay coverage.",
]


def rust_inventory_code(text):
    """Mask non-code, #[cfg(test)] items and macro definitions without moving offsets."""
    # Nested Rust comments and raw/byte strings need more than a comment regex.
    token = re.compile(r'//[^\n]*|/\*|b?r(?P<h>\#*)"|b?"(?:\\.|[^"\\])*"|b?\'(?:\\.|[^\\\'])\'')
    chars, index = list(text), 0

    def mask(start, end):
        chars[start:end] = ["\n" if c == "\n" else " " for c in text[start:end]]

    while match := token.search(text, index):
        end = match.end()
        if match.group(0) == "/*":
            depth = 1
            while end < len(text) and depth:
                if text.startswith("/*", end):
                    depth, end = depth + 1, end + 2
                elif text.startswith("*/", end):
                    depth, end = depth - 1, end + 2
                else:
                    end += 1
        elif match.group("h") is not None:
            close = '"' + match.group("h")
            found = text.find(close, end)
            end = len(text) if found < 0 else found + len(close)
        mask(match.start(), end)
        index = end
    code = "".join(chars)
    # Balanced braces after a test attribute/macro declaration; no macro expansion.
    starts = list(re.finditer(r'#\[cfg\(test\)\]|\bmacro_rules!\s*\w+', code))
    for match in starts:
        start = match.end()
        opening = re.search(r'[\{;]', code[start:])
        if opening is None:
            continue
        end = start + opening.start()
        if code[end] == "{":
            depth, end = 1, end + 1
            while end < len(code) and depth:
                depth += (code[end] == "{") - (code[end] == "}")
                end += 1
        else:
            end += 1
        mask(match.start(), end)
    return "".join(chars)


def inventory_imports(code):
    """Expand aliases without losing bindings (use_paths serves a graph instead)."""
    imports = []

    def expand(tree, prefix=""):
        depth, start, parts = 0, 0, []
        for index, char in enumerate(tree):
            depth += (char == "{") - (char == "}")
            if char == "," and depth == 0:
                parts.append(tree[start:index])
                start = index + 1
        for part in parts + [tree[start:]]:
            part = part.strip()
            if not part:
                continue
            if "{" in part:
                base, nested = part.split("{", 1)
                expand(nested.rsplit("}", 1)[0], prefix + base.strip())
            else:
                binding = re.split(r"\s+as\s+", part)
                path = re.sub(r"\s+", "", prefix + binding[0])
                imports.append((path, binding[1].strip() if len(binding) == 2 else path.split("::")[-1]))

    for match in re.finditer(r"\buse\s+([^;]+);", code):
        expand(match.group(1))
    return imports


def inventory_sites(text):
    """One record per source token offset. Kinds never share an offset."""
    code = rust_inventory_code(text)
    imports = inventory_imports(code)
    modules, copies = {"session_cohort"}, set()
    spellings = {name: name for name in ("word", "checked_word", "ram_word", "ram_checked_word", "slot", "word_transfers", "boss", "scroll", "inventory", "melt", "menu", "cinematic", "map_screen", "save_selection", "options_menu", "death_sequence", "power_bomb", "session_word", "session_slot", "register_writes", "SlotAccess")}
    ram_namespaces = set()
    dependencies = {"Game", "RawRam", "CompatibilityRam", "SessionRam", "SlotAccess", "WorkRamAccess", "ram_map", "g_ram"}
    raw_aliases = {"g_ram"}
    for _ in range(len(imports) + 1):
        before = (len(modules), len(copies), len(dependencies), len(spellings), len(ram_namespaces))
        for path, alias in imports:
            parts = path.split("::")
            leaf = parts[-1]
            if leaf in modules or (leaf == "self" and len(parts) > 1 and parts[-2] in modules):
                modules.add(alias if alias != "self" else parts[-2])
            elif len(parts) > 1 and parts[-2] in modules and leaf in ("load", "with", "with_projected"):
                copies.add(alias)
            elif leaf in copies:
                copies.add(alias)
            if leaf in dependencies:
                dependencies.add(alias)
            if leaf in spellings:
                spellings[alias] = spellings[leaf]
            if "ram" in parts and leaf in ("words", "word_arrays", "bytes"):
                ram_namespaces.add(alias)
            if leaf in ram_namespaces:
                ram_namespaces.add(alias)
            if leaf == "g_ram":
                raw_aliases.add(alias)
        if before == (len(modules), len(copies), len(dependencies), len(spellings), len(ram_namespaces)):
            break
    # Imports are dependency evidence, not executable call sites.
    calls = re.sub(r"\buse\s+[^;]+;", lambda m: " " * len(m.group()), code)
    ram_receivers = set()
    for _ in range(8):
        before = (len(ram_receivers), len(raw_aliases))
        for match in re.finditer(r"\blet\s+(?:mut\s+)?(\w+)\s*=\s*([^;]+);", calls):
            name, value = match.groups()
            if re.fullmatch(r"\s*\w+\.ram\(\)\s*", value) or value.strip() in ram_receivers:
                ram_receivers.add(name)
            if value.strip() in raw_aliases:
                raw_aliases.add(name)
        if before == (len(ram_receivers), len(raw_aliases)):
            break
    records = {}

    def add(start, end, kind, symbol):
        # Specific call categories run first; dependency matches cannot double count.
        if start not in records:
            records[start] = (start, end, kind, symbol)

    macros = {"word", "checked_word", "ram_word", "ram_checked_word", "slot", "word_transfers"}
    views = {"boss", "scroll", "inventory", "melt", "menu", "cinematic", "map_screen", "save_selection", "options_menu", "death_sequence", "power_bomb", "session_word"}
    for match in re.finditer(r"\b(\w+)\s*!\s*[({]", calls):
        name = spellings.get(match.group(1), match.group(1))
        if name in macros | views | {"session_slot", "register_writes"}:
            kind = "ram_macro" if name in macros else "address_api" if name == "session_slot" else "ram_view"
            add(match.start(1), match.end(1), kind, name)
    path_call = re.compile(r"\b(\w+(?:\s*::\s*\w+)*)\s*(?=\(|::\s*<)")
    for match in path_call.finditer(calls):
        path = re.sub(r"\s+", "", match.group(1))
        parts = path.split("::")
        leaf = parts[-1]
        if (len(parts) > 1 and parts[-2] in modules and leaf in ("load", "with", "with_projected")) or (len(parts) == 1 and leaf in copies):
            add(match.start(), match.end(1), "cohort_copy", path)
        elif any(spellings.get(part) == "SlotAccess" for part in parts) and leaf in ("slot_word", "set_slot_word", "slot_byte", "set_slot_byte"):
            add(match.start(), match.end(1), "address_api", path)
    # Preserve the existing view taxonomy, but do not count type-only RawRam references as calls.
    for label, pattern in RAM_COUPLINGS.items():
        if label == COHORT_LABEL or label.startswith("raw RAM hosts"):
            continue
        for match in pattern.finditer(calls):
            add(match.start(), match.end(), "ram_view", match.group().split("(")[0].strip())
    raw_methods = {"read_byte", "write_byte", "copy_in", "copy_out", "fill", "copy_within"}
    address_methods = {"native_word", "lease_native_word", "write_word_le", "write_word_native", "write_slice_le", "write_slice_native", "slot_word", "set_slot_word", "slot_byte", "set_slot_byte", "compatibility_byte", "compatibility_write_byte", "projected_slot_byte", "store_projected_slot_byte"}
    generic = {"word", "byte", "set_byte", "read_byte", "write_byte", "read_word", "write_word", "write_slice", "copy_in", "copy_out", "copy_within"}
    for match in re.finditer(r"\.\s*(\w+)\s*\(", calls):
        method = match.group(1)
        if method not in raw_methods | address_methods | generic:
            continue
        prefix = calls[max(0, match.start() - 180):match.start()].rstrip()
        raw = re.search(r"\.ram\s*\(\s*\)\s*$", prefix)
        receiver = re.search(r"\b(\w+)\s*$", prefix)
        raw = bool(raw or (receiver and receiver.group(1) in ram_receivers))
        if raw and method in raw_methods:
            kind = "raw_ram_call"
        elif method in address_methods or (method in ("word", "byte", "set_byte", "write_slice") and any(re.match(r"\s*" + re.escape(name) + r"\s*::", calls[match.end():]) for name in ram_namespaces)):
            kind = "address_api"
        else:
            # .word(W::...) and all generic word calls remain receiver-dependent.
            kind = "unresolved_method"
        add(match.start(1), match.end(1), kind, method)
    for match in re.finditer(r"\b\w+\b", calls):
        name = match.group()
        if name in raw_aliases:
            add(match.start(), match.end(), "raw_base", name)
    for match in re.finditer(r"\b\w+\b", code):
        name = match.group()
        if name in dependencies or name.startswith(("native_", "compatibility_")):
            add(match.start(), match.end(), "backend_reference", name)
    return sorted(records.values())


def inventory_scope(path):
    """Physical classification retained across the target-package migration."""
    parts = path.split("/")
    if "verification" in parts or "verify" in parts or parts[-1].startswith("verify") or "sm_tools" in parts:
        return "verification", "verification"
    if path.startswith("crates/"):
        return "target", "/".join(parts[1:2] + parts[3:4])
    if path.startswith("sm/src/game/") and "/compat/" not in path:
        return "legacy_engine", "legacy/" + parts[3].removesuffix(".rs")
    return "compat_oracle", "/".join(parts[2:3])


def ram_history_main(argv):
    """Cheap immutable-pin source measurements, extending this existing report."""
    import argparse
    import ast
    import csv
    import gzip
    import hashlib
    import subprocess
    from datetime import datetime, timezone
    from pathlib import Path

    parser = argparse.ArgumentParser(description="RAM source inventory; lexical sites are not resolved operation counts")
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--pins", type=Path, required=True, help="CSV or Markdown fenced CSV: pin,status,observed_at,evidence; optional label. status=landed/pending")
    parser.add_argument("--output", type=Path, required=True, help="Owned artifact directory; never writes the checkout")
    args = parser.parse_args(argv)
    if args.repo.resolve() == args.output.resolve() or args.repo.resolve() in args.output.resolve().parents:
        parser.error("--output must be outside the checkout")
    args.output.mkdir(parents=True, exist_ok=True)
    manifest = args.pins.read_text(encoding="utf-8-sig")
    if args.pins.suffix.lower() == ".md":
        manifests = re.findall(r"```csv\s*\n(pin,status,observed_at,evidence,label\n.*?)\n```", manifest, re.S)
        if len(manifests) != 1:
            parser.error("Markdown input must have one fenced RAM pins CSV (otherwise use a dedicated CSV)")
        manifest = manifests[0]
    rows = list(csv.DictReader(manifest.splitlines()))
    if not rows:
        parser.error("--pins must contain at least one dated source record")
    measured_at = datetime.now(timezone.utc).isoformat()
    tool_digest = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    rules_digest = hashlib.sha256(json.dumps(RAM_INVENTORY_RULES, sort_keys=True).encode()).hexdigest()
    fixed_ratchet_digest = hashlib.sha256(repr([(k, p.pattern) for k, p in RAM_COUPLINGS.items()]).encode()).hexdigest()

    def git(*arguments):
        return subprocess.check_output(["git", *arguments], cwd=args.repo)

    # Cache totals by blob + scope with the tool fingerprint. Sites stream to CSV.
    cache_path = args.output / "blob-cache.csv.gz"
    cache = {}
    if cache_path.exists():
        for cached in csv.DictReader(gzip.open(cache_path, "rt", encoding="utf-8", newline="")):
            if cached["tool_sha256"] == tool_digest:
                cache[(cached["blob"], cached["path"])] = cached
    history, domains, pin_reports = [], [], []
    site_file = args.output / "sites.csv.gz"
    with gzip.open(site_file, "wt", encoding="utf-8", newline="") as output:
        sites_writer = csv.writer(output, lineterminator="\n")
        sites_writer.writerow(["pin", "blob", "scope", "domain", "path", "line", "column", "offset", "kind", "symbol"])
        for record in rows:
            if not re.fullmatch(r"[0-9a-f]{40}", record["pin"]):
                parser.error("pin must be a complete immutable Git commit hash")
            pin = git("rev-parse", "--verify", record["pin"] + "^{commit}").decode().strip()
            if record["status"] not in ("landed", "pending"):
                parser.error("pin status must be landed or pending")
            observed = datetime.fromisoformat(record["observed_at"].replace("Z", "+00:00"))
            if observed.tzinfo is None or not record["evidence"]:
                parser.error("observed_at needs a timezone and evidence must name its source")
            commit_at = git("show", "-s", "--format=%cI", pin).decode().strip()
            native_source = git("show", pin + ":tools/cycle_report.py").decode()
            native_hash = hashlib.sha256(native_source.encode()).hexdigest()
            # Execute only the original immutable measurement definitions, never main.
            wanted = {"RAM_COUPLINGS", "COHORT_LABEL"}
            definitions = [node for node in ast.parse(native_source).body
                           if (isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id in wanted for t in node.targets))
                           or (isinstance(node, ast.FunctionDef) and node.name == "cohort_sites")]
            namespace = {"re": re}
            exec(compile(ast.Module(body=definitions, type_ignores=[]), "<pinned-ratchet>", "exec"), namespace)
            native_patterns = namespace.get("RAM_COUPLINGS", {})
            totals, per_domain = Counter(), defaultdict(Counter)
            fixed_sites = fixed_files = native_sites = native_files = copies = 0
            tree = git("ls-tree", "-r", "-z", pin, "sm/src", "crates").split(b"\0")
            with subprocess.Popen(["git", "cat-file", "--batch"], cwd=args.repo, stdin=subprocess.PIPE, stdout=subprocess.PIPE) as batch:
                for entry in filter(None, tree):
                    meta, encoded_path = entry.split(b"\t", 1)
                    path = encoded_path.decode()
                    if not path.endswith(".rs") or (path.startswith("crates/") and "/src/" not in path):
                        continue
                    blob = meta.split()[2].decode()
                    batch.stdin.write(blob.encode() + b"\n")
                    batch.stdin.flush()
                    header = batch.stdout.readline().split()
                    data = batch.stdout.read(int(header[-1]))
                    if batch.stdout.read(1) != b"\n":
                        raise ValueError("bad Git blob framing")
                    text = data.decode("utf-8")
                    scope, domain = inventory_scope(path)
                    key = (blob, path)
                    if key not in cache:
                        found = inventory_sites(text)
                        counts = Counter(site[2] for site in found)
                        packed = json.dumps(found, separators=(",", ":"))
                        cache[key] = {"tool_sha256": tool_digest, "blob": blob, "path": path, "sites": packed}
                    else:
                        found = json.loads(cache[key]["sites"])
                        counts = Counter(site[2] for site in found)
                    for kind, count in counts.items():
                        totals[(scope, kind)] += count
                        per_domain[(scope, domain)][kind] += count
                    for start, end, kind, symbol in found:
                        line = text.count("\n", 0, start) + 1
                        column = start - text.rfind("\n", 0, start)
                        sites_writer.writerow([pin, blob, scope, domain, path, line, column, start, kind, symbol])
                    if path.startswith("crates/sm_game/src/"):
                        fixed = sum(cohort_sites(text) if label == COHORT_LABEL else len(pattern.findall(text)) for label, pattern in RAM_COUPLINGS.items())
                        native = sum(namespace["cohort_sites"](text) if label == namespace.get("COHORT_LABEL") else len(pattern.findall(text)) for label, pattern in native_patterns.items())
                        fixed_sites += fixed
                        fixed_files += bool(fixed)
                        native_sites += native
                        native_files += bool(native)
                        copies += cohort_sites(text)
                batch.stdin.close()
                if batch.wait() != 0:
                    raise ValueError("git cat-file failed")
            summary = dict(pin=pin, status=record["status"], label=record.get("label", ""), observed_at=record["observed_at"], commit_at=commit_at, evidence=record["evidence"], measured_at=measured_at,
                           inventory_version=RAM_INVENTORY_VERSION, tool_sha256=tool_digest, rules_sha256=rules_digest,
                           native_ratchet_tool_sha256=native_hash, fixed_ratchet_patterns_sha256=fixed_ratchet_digest,
                           native_ratchet_patterns_sha256=hashlib.sha256(repr([(k, p.pattern) for k, p in native_patterns.items()]).encode()).hexdigest(),
                           ratchet_native_sites=native_sites, ratchet_fixed_sites=fixed_sites, ratchet_fixed_files=fixed_files, ratchet_native_files=native_files, cohort_calls=copies)
            for scope in ("target", "legacy_engine", "compat_oracle", "verification"):
                for kind in RAM_INVENTORY_RULES:
                    summary[scope + "_" + kind] = totals[scope, kind]
            for (scope, domain), counts in sorted(per_domain.items()):
                domains.append(dict(pin=pin, scope=scope, domain=domain, **{kind: counts[kind] for kind in RAM_INVENTORY_RULES}))
            history.append(summary)
            pin_reports.append(f"{record.get('label', pin[:12])}: {native_sites} native / {fixed_sites} fixed ratchet, {fixed_files} files; raw calls {totals['target', 'raw_ram_call']} target + {totals['legacy_engine', 'raw_ram_call']} legacy")
    for name, records in (("history.csv", history), ("domains.csv", domains), ("blob-cache.csv.gz", list(cache.values()))):
        opener = gzip.open if name.endswith(".gz") else open
        with opener(args.output / name, "wt", encoding="utf-8", newline="") as output:
            writer = csv.DictWriter(output, fieldnames=list(records[0]), lineterminator="\n")
            writer.writeheader()
            writer.writerows(records)
    landed = sorted((r for r in history if r["status"] == "landed"), key=lambda r: datetime.fromisoformat(r["observed_at"].replace("Z", "+00:00")))
    report = ["# RAM source burndown", "", f"Measured {measured_at}; definition `{RAM_INVENTORY_VERSION}`, tool `{tool_digest}`.", "",
              "Each site is a lexical source candidate/reference, not a resolved RAM operation. Typed Session field access is excluded. Address APIs can use typed Session; never sum them into raw RAM calls.", "",
              "Native ratchet uses each pin's original definitions; fixed ratchet applies this tool's patterns to every pin. A definition change is not a code elimination. Raw-call curve includes target plus sm/src/game so migration between those paths earns no saving. Compat/oracle and verification remain separate.", "",
              "| State | Pin | Observed UTC | Native ratchet | Fixed ratchet | Raw calls: target + legacy | Address APIs: target + legacy |", "| --- | --- | --- | ---: | ---: | ---: | ---: |"]
    for r in history:
        report.append(f"| {r['status']} {r['label']} | `{r['pin']}` | {r['observed_at']} | {r['ratchet_native_sites']} | {r['ratchet_fixed_sites']} | {r['target_raw_ram_call']} + {r['legacy_engine_raw_ram_call']} | {r['target_address_api']} + {r['legacy_engine_address_api']} |")
    if len(landed) > 1:
        first, last = landed[0], landed[-1]
        hours = (datetime.fromisoformat(last["observed_at"].replace("Z", "+00:00")) - datetime.fromisoformat(first["observed_at"].replace("Z", "+00:00"))).total_seconds() / 3600
        delta = first["ratchet_fixed_sites"] - last["ratchet_fixed_sites"]
        rate = f"{delta / hours:.2f}/hour" if hours > 0 else "rate unavailable: equal observation timestamps"
        raw_delta = first["target_raw_ram_call"] + first["legacy_engine_raw_ram_call"] - last["target_raw_ram_call"] - last["legacy_engine_raw_ram_call"]
        report += ["", f"Landed measured interval: {hours:.3f} hours; fixed-ratchet net removal {delta} sites ({rate}); explicit raw-receiver call candidates net removal {raw_delta}. This is an observed landing rate, not a forecast or source/build/replay speed."]
        report += ["", "```mermaid", "xychart-beta", '  title "Landed fixed-definition sm_game ratchet"', "  x-axis [" + ", ".join('"' + (r["label"] or r["pin"][:8]) + '"' for r in landed) + "]", "  y-axis \"Matched sites\" 0 --> " + str(max(1, max(r["ratchet_fixed_sites"] for r in landed))), "  line [" + ", ".join(str(r["ratchet_fixed_sites"]) for r in landed) + "]", "```"]
        report += ["", "```mermaid", "xychart-beta", '  title "Landed lexical raw RAM calls (target + legacy engine)"', "  x-axis [" + ", ".join('"' + (r["label"] or r["pin"][:8]) + '"' for r in landed) + "]", "  y-axis \"Candidate calls\" 0 --> " + str(max(1, max(r["target_raw_ram_call"] + r["legacy_engine_raw_ram_call"] for r in landed))), "  line [" + ", ".join(str(r["target_raw_ram_call"] + r["legacy_engine_raw_ram_call"]) for r in landed) + "]", "```", "", "X positions are landing events, not evenly spaced elapsed time; timestamps are in the table/CSV."]
    report += ["", "Source/review/build/replay/CI queue durations: not measured by Git timestamps; no timing claims inferred.", "", "## Latest landed remaining inventory", "", "| Scope | Raw calls | Raw bases | RAM macros | RAM views | Cohort copies | Address APIs | Unresolved methods | Backend references |", "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"]
    if landed:
        latest = landed[-1]
        for scope in ("target", "legacy_engine", "compat_oracle", "verification"):
            report.append("| " + scope + " | " + " | ".join(str(latest[scope + "_" + kind]) for kind in RAM_INVENTORY_RULES) + " |")
    report += ["", "The raw-call curve alone omits named macros, views and copies. These columns have different meanings and must not be summed into a resolved-operation total. Detailed remaining counts by path domain are in `domains.csv`.", "", "## Definitions and evidence", "", "`sites.csv.gz` gives immutable pin/blob, scope/domain, file, line/column/offset, exclusive kind and symbol. Deduplication identity is (pin,file,offset); most specific call kind wins, then dependency references. Different tokens in one statement can count separately. `domains.csv` gives path-defined remaining counts; `history.csv` is dated machine-readable evidence. Blob cache is local tooling output, not historical authority.", ""]
    report += [f"- `{kind}`: {meaning}." for kind, meaning in RAM_INVENTORY_RULES.items()]
    report += ["", "## Unresolved scope", ""] + ["- " + item for item in RAM_BLINDSPOTS]
    (args.output / "report.md").write_text("\n".join(report) + "\n", encoding="utf-8", newline="\n")
    print("\n".join(pin_reports))
    print(args.output / "report.md")


if __name__ == "__main__":
    main()
