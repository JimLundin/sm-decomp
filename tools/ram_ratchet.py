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
BASELINE = {
    "sm_game_ram_sites": 2138,
    "sm_game_coupled_files": 98,
    "production_lines": 173506,
}
ADVISORY = {"production_lines"}


CFG_TEST = re.compile(r"^\s*#\[cfg\(test\)\]\s*$")
ATTRIBUTE = re.compile(r"^\s*#\[.*\]\s*$")
# Strings, character literals and line comments, which may hold unbalanced braces.
NOT_CODE = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'|//.*')


def test_lines(lines):
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


def production_lines():
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


def harness_lines():
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


def measure():
    ram = cycle_report.sm_game_ram_couplings()
    return {
        "sm_game_ram_sites": sum(ram["sites"].values()),
        "sm_game_coupled_files": ram["coupled_files"],
        "production_lines": production_lines(),
    }


def main():
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
