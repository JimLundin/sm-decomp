#!/usr/bin/env python3
"""Fail when a change raises sm_game's RAM coupling or the production line count.

docs/REVIEW.md's Phase 2 ratchets: no landing may raise cycle_report's
sm_game_ram total sites or coupled files, and moves are line-neutral or
smaller in production lines (crates/*/src plus sm/src outside verification;
#[cfg(test)] items inside crates/*/src are tests, not production). The
scenario harness (crates/sm_tools, sm/src/verification/observation and
tools/scenario.py) is
verification, reported separately (REVIEW, lane S condition 1).
Within the same measurement scope the baseline below only moves down: run with --update after a landing that
lowers a measure, and commit the result with that landing.

The production line count is advisory for now (Jim, 2026-10-31: shipping
faster comes first; strict code-volume reduction comes later). It is still
reported, but only the RAM coupling measures fail the check.

The engine inventory (Jim, 2026-11-02) also counts what the sm_game measure
misses: address APIs (session_slot!, SlotAccess, native_word, write_slice_*),
direct .ram() calls and unresolved word/byte methods in the target packages, and
all RAM access in the legacy engine under sm/src/game. Completion means these
reach zero too. Address APIs and raw calls may not rise above main's baseline;
the other two are reported.

Calls into crate::legacy_overflow (cycle_report kind legacy_overflow) are the
original game's reads and stores past their tables, tagged
LEGACY(original-overflow) and listed in docs/LEGACY.md. They are not address
APIs (the module's own internals still count as their kinds); their count,
engine_legacy_overflow_sites, is reported only.

Run from the repository root: python3 tools/ram_ratchet.py [--update]
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cycle_report  # noqa: E402

# Lowered to the honest view-counting measure after batch 13t (2026-11-01;
# 3663, then 2319 before). First measured on main 91d0bcce. The
# production line baseline dropped by the 685 #[cfg(test)] lines it stopped
# counting (2026-10-31).
# Measurement-only rebaseline on main c7936fda9f8422358796cfdd62df096c88428545:
# include 303 executable RAM-backed cohort copy calls in 35 files. Four other
# session_cohort references are documentation links, not calls. The coupled
# union adds two files: 1842/99 becomes 2145/101. This is not the lower queued
# batch-13 count; lower it only after that landing. Production remains advisory.
# Measurement-only rebaseline on 65ae1cb61 (2026-11-03): count the 119 register
# writes in 6 files that store their WRAM display mirror ($51..$86) by RAM offset
# through raw work RAM (313 becomes 432); the ratchet saw only one RawRam import.
BASELINE = {
    "sm_game_ram_sites": 99,
    "sm_game_coupled_files": 17,
    "production_lines": 173506,
    # Engine inventory (Jim, 2026-11-02): address-shaped access is RAM access.
    # First measured on main 0115740f5 (ROM record reads excluded). Address APIs and raw calls are enforced:
    # converting counted sites into them is not a retirement, and the address
    # decoding behind them slows every replay. The lexical unresolved-method and
    # legacy-engine measures are reported only.
    "engine_ram_address_sites": 11,
    "engine_raw_ram_calls": 64,
    "engine_unresolved_ram_methods": 981,
    "legacy_engine_ram_sites": 515,
    # Original-game overflow paths (LEGACY(original-overflow)), reported only.
    "engine_legacy_overflow_sites": 61,
    # The Game boundary (Jim, 2026-11-03): each foreign call is a full Session
    # export and re-import; Game host impls are host traits served from the RAM
    # image; Session entries are leases opened from Game-hosted code. All three
    # reach zero when the Session drives the tick. Foreign calls and Game hosts
    # may not rise; lease moves can add entries, so those are reported.
    "foreign_calls": 173,
    "game_host_impls": 96,  # SpriteHost split into MapHost + SpriteWords (same coupling)
    "session_entries": 112,
    # Session word views keyed by RAM names (2026-11-03): sequence_word! and
    # gameplay_word! read and store typed Session-backed keys, not RAM, so
    # sm_game_ram_sites does not count them. They are still a word bag named
    # after RAM words rather than domain fields; reported only.
    "session_word_view_sites": 202,
}
ADVISORY = {"production_lines", "engine_unresolved_ram_methods", "legacy_engine_ram_sites",
            "engine_legacy_overflow_sites", "session_entries", "session_word_view_sites"}


CFG_TEST = re.compile(r"^\s*#\[cfg\(test\)\]\s*$")
ATTRIBUTE = re.compile(r"^\s*#\[.*\]\s*$")
# Strings, character literals and line comments, which may hold unbalanced braces.
NOT_CODE = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'|//.*')


def test_lines(lines: list[str]) -> int:
    """The lines of #[cfg(test)] items: from the attribute through the item's end."""
    count = 0
    index = 0
    while index < len(lines):
        if not CFG_TEST.match(lines[index]):
            index += 1
            continue
        end = index + 1
        while end < len(lines) and ATTRIBUTE.match(lines[end]):
            end += 1
        depth, opened = 0, False
        while end < len(lines):
            code = NOT_CODE.sub("", lines[end])
            depth += code.count("{") - code.count("}")
            opened = opened or "{" in code
            if (opened and depth == 0) or (not opened and code.rstrip().endswith(";")):
                break
            end += 1
        count += end + 1 - index
        index = end + 1
    return count


def production_lines() -> int:
    total = 0
    for top in ["crates", os.path.join("sm", "src")]:
        for root, dirs, names in os.walk(top):
            parts = root.replace("\\", "/").split("/")
            if top == "crates" and "src" not in parts:
                continue
            if "verification" in parts or "sm_tools" in parts:
                continue
            for name in names:
                if name.endswith(".rs"):
                    with open(os.path.join(root, name), "rb") as source:
                        text = source.read().decode("utf-8")
                    total += text.count("\n")
                    if top == "crates":
                        total -= test_lines(text.split("\n"))
    return total


def harness_lines() -> int:
    """Lines of the scenario harness, which count as verification."""
    total = 0
    for top in [os.path.join("crates", "sm_tools", "src"),
                os.path.join("sm", "src", "verification", "observation")]:
        for root, dirs, names in os.walk(top):
            for name in names:
                if name.endswith(".rs"):
                    with open(os.path.join(root, name), "rb") as source:
                        total += source.read().decode("utf-8").count("\n")
    # The scenario model, its lock and cut.
    with open(os.path.join("tools", "scenario.py"), "rb") as source:
        total += source.read().decode("utf-8").count("\n")
    return total


def engine_inventory() -> dict[str, int]:
    """Address-shaped and raw RAM access the sm_game measure does not see.

    cycle_report's source inventory over the engine: the target packages
    (crates/*/src outside verification and sm_tools) and the legacy engine
    (sm/src/game outside compat and verification). Address APIs such as
    session_slot!, SlotAccess and native_word still name SNES RAM addresses even
    when typed Session storage answers them, so they count as RAM access.
    """
    kinds: dict[str, dict[str, int]] = {"target": {}, "legacy_engine": {}}
    for top in ("crates", os.path.join("sm", "src", "game")):
        for root, _, names in os.walk(top):
            for name in names:
                if not name.endswith(".rs"):
                    continue
                path = os.path.join(root, name).replace("\\", "/")
                if path.startswith("crates/") and "/src/" not in path:
                    continue
                counts = kinds.get(cycle_report.inventory_scope(path)[0])
                if counts is None:
                    continue
                with open(path, encoding="utf-8", errors="replace") as source:
                    text = source.read()
                for record in cycle_report.inventory_sites(text):
                    counts[record[2]] = counts.get(record[2], 0) + 1
    target, legacy = kinds["target"], kinds["legacy_engine"]
    access = ("raw_ram_call", "raw_base", "ram_macro", "ram_view", "address_api", "cohort_copy")
    return {
        "engine_ram_address_sites": target.get("address_api", 0),
        "engine_raw_ram_calls": target.get("raw_ram_call", 0) + target.get("raw_base", 0),
        "engine_unresolved_ram_methods": target.get("unresolved_method", 0),
        "legacy_engine_ram_sites": sum(legacy.get(kind, 0) for kind in access),
        "engine_legacy_overflow_sites": target.get("legacy_overflow", 0),
    }


def game_boundary() -> dict[str, int]:
    """Calls across the Game boundary: foreign calls (crates and sm/src), host
    traits implemented for Game (sm/src), and leases opened from Game-hosted
    sm_game code. Line comments are ignored."""
    foreign = re.compile(r"\.foreign\s*\(")
    host = re.compile(r"^\s*impl\b[^{;]*\bfor\s+Game\b")
    entry = re.compile(r"\.runtime\s*\(\s*\)")
    # Built with fromkeys: --update rewrites `"name": <n>` literals, which a
    # dict display here would match.
    counts = dict.fromkeys(("foreign_calls", "game_host_impls", "session_entries"), 0)
    for top in ("crates", os.path.join("sm", "src")):
        for root, _, names in os.walk(top):
            for name in names:
                if not name.endswith(".rs"):
                    continue
                path = os.path.join(root, name).replace("\\", "/")
                with open(path, encoding="utf-8", errors="replace") as source:
                    lines = [line.split("//", 1)[0] for line in source]
                counts["foreign_calls"] += sum(len(foreign.findall(line)) for line in lines)
                if path.startswith("sm/src/"):
                    counts["game_host_impls"] += sum(1 for line in lines if host.match(line))
                if path.startswith("crates/sm_game/src/"):
                    counts["session_entries"] += sum(len(entry.findall(line)) for line in lines)
    return counts


def session_word_view_sites() -> int:
    """Call sites of the RAM-named Session word views in sm_game, outside
    comments and the macros' own recursion."""
    view = re.compile(r"(?<!\$crate::)\b(?:sequence_word|gameplay_word)!\s*[({]")
    total = 0
    for root, _, names in os.walk(os.path.join("crates", "sm_game", "src")):
        for name in names:
            if name.endswith(".rs"):
                with open(os.path.join(root, name), encoding="utf-8", errors="replace") as source:
                    text = re.sub(r"//[^\n]*|/\*.*?\*/", " ", source.read(), flags=re.S)
                total += len(view.findall(text))
    return total


def measure() -> dict[str, int]:
    ram = cycle_report.sm_game_ram_couplings()
    return {
        "sm_game_ram_sites": sum(ram["sites"].values()),
        "sm_game_coupled_files": ram["coupled_files"],
        "production_lines": production_lines(),
        **engine_inventory(),
        **game_boundary(),
        "session_word_view_sites": session_word_view_sites(),
    }


def main() -> None:
    now = measure()
    over = []
    for name, base in BASELINE.items():
        print(f"{name}: {now[name]} (baseline {base}, delta {now[name] - base:+d})")
        if now[name] > base:
            if name in ADVISORY:
                print(f"advisory: {name} is over its baseline")
            else:
                over.append(name)
    print(f"scenario harness lines (verification, reported separately): {harness_lines()}")
    if "--update" in sys.argv:
        path = os.path.abspath(__file__)
        text = open(path, encoding="utf-8").read()
        for name, base in BASELINE.items():
            lowered = min(base, now[name])
            text = re.sub(rf'("{name}": )\d+', rf"\g<1>{lowered}", text)
        open(path, "w", encoding="utf-8", newline="\n").write(text)
        print("baseline updated (only lowered measures change)")
    if over:
        print("ratchet raised: " + ", ".join(over))
        sys.exit(1)


if __name__ == "__main__":
    main()
