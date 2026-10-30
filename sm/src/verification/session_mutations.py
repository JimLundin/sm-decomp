"""Mutation check of the Session's RAM mappings; the mutants run in GitHub CI.

Each mutant perturbs one owner's or view's mapping between RAM and Session
fields: it shifts a field's address by 2, swaps two fields, or drops one byte
of a word. It must make a named sm_game unit test fail. A surviving mutant, a
compile error or a failure in any other test is not a kill.

    python session_mutations.py                  run every mutant (GitHub CI only)
    python session_mutations.py --check-anchors  check every anchor exists, no build
"""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ENCODING = "crates/sm_game/src/session_encoding.rs"
LANES = "session_encoding::player_lane_tests::player_lanes_project_from_their_fields"
RAW_LANES = "session_encoding::player_lane_tests::raw_transition_lanes_project_from_their_fields"
INVENTORY = "inventory::tests::the_view_reads_and_stores_the_owned_inventory"

# name, file, anchor, replacement, the test that must fail
MUTATIONS = (
    ("shift_swing_length_delta", ENCODING,
     "$f(0x0d00,$($borrow)* $s.player.swing.length_delta);",
     "$f(0x0d02,$($borrow)* $s.player.swing.length_delta);", LANES),
    ("shift_health", ENCODING,
     "$f(0x9c2,$($borrow)* $s.inventory.health.current);",
     "$f(0x9c4,$($borrow)* $s.inventory.health.current);", LANES),
    ("shift_interrupted_pose_request", ENCODING,
     "            0x0a2a,\n            $s.player.transitions.interrupted,",
     "            0x0a2c,\n            $s.player.transitions.interrupted,", RAW_LANES),
    ("swap_grapple_flags_and_delay", ENCODING,
     "$f(0x0cf4,$($borrow)* $s.player.grapple_flags);\n        $f(0x0cf6,$($borrow)* $s.player.grapple_delay);",
     "$f(0x0cf4,$($borrow)* $s.player.grapple_delay);\n        $f(0x0cf6,$($borrow)* $s.player.grapple_flags);", LANES),
    ("swap_swing_rates", ENCODING,
     "$f(0x0d26,$($borrow)* $s.player.swing.angular_velocity);\n        $f(0x0d28,$($borrow)* $s.player.swing.angle_acceleration);",
     "$f(0x0d26,$($borrow)* $s.player.swing.angle_acceleration);\n        $f(0x0d28,$($borrow)* $s.player.swing.angular_velocity);", LANES),
    ("swap_inventory_view_loadout", "crates/sm_game/src/inventory.rs",
     "owned: word(Word::CollectedItems),\n            active: word(Word::EquippedItems),",
     "owned: word(Word::EquippedItems),\n            active: word(Word::CollectedItems),", INVENTORY),
    ("drop_scalar_high_byte_read", ENCODING,
     "        if at == base || at == base + 1 {\n            result = Some(value.to_le_bytes()[at - base]);",
     "        if at == base {\n            result = Some(value.to_le_bytes()[at - base]);", LANES),
    ("drop_scalar_high_byte_write", ENCODING,
     "        if at == base || at == base + 1 {\n            let mut bytes = word.to_le_bytes();\n            bytes[at - base] = value;",
     "        if at == base {\n            let mut bytes = word.to_le_bytes();\n            bytes[at - base] = value;", LANES),
)


def read(path):
    return path.read_bytes().decode("utf-8").replace("\r\n", "\n")


def check_anchors(root):
    stale = [f"{name}: {relative}: anchor occurs {read(root / relative).count(before)} times"
             for name, relative, before, _after, _test in MUTATIONS
             if not (root / relative).is_file() or read(root / relative).count(before) != 1]
    if stale:
        raise RuntimeError("stale session mutation anchors:\n" + "\n".join(stale))
    print(f"session mutation anchors: {len(MUTATIONS)}/{len(MUTATIONS)} present exactly once", flush=True)


def run(command, timeout):
    return subprocess.run(command, capture_output=True, text=True, encoding="utf-8",
                          errors="replace", timeout=timeout, check=False)


def build(crates, work, name, sm_data):
    binary = work / name / ("sm_game_tests.exe" if os.name == "nt" else "sm_game_tests")
    built = run(["rustc", "--edition=2024", "-O", "--test", "--crate-name", "sm_game",
                 str(crates / "sm_game" / "src" / "lib.rs"), "--extern", f"sm_data={sm_data}",
                 "-L", f"dependency={work}", "-o", str(binary), "-A", "unused"], 600)
    if built.returncode != 0:
        raise RuntimeError(f"{name}: COMPILE ERROR; not a killed mutant\n{built.stderr}")
    return binary


def main():
    root = Path(__file__).resolve().parents[3]
    check_anchors(root)
    if "--check-anchors" in sys.argv[1:]:
        return
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("session mutation builds and execution are restricted to GitHub CI")
    with tempfile.TemporaryDirectory(prefix="session-mutations-") as directory:
        work = Path(directory)
        sm_data = work / "libsm_data.rlib"
        built = run(["rustc", "--edition=2024", "-O", "--crate-name", "sm_data", "--crate-type", "rlib",
                     str(root / "crates" / "sm_data" / "src" / "lib.rs"), "-o", str(sm_data)], 600)
        if built.returncode != 0:
            raise RuntimeError(f"sm_data build failed\n{built.stderr}")
        (work / "baseline").mkdir()
        baseline = build(root / "crates", work, "baseline", sm_data)
        for name, relative, before, after, test in MUTATIONS:
            good = run([str(baseline), "--exact", test], 60)
            if good.returncode != 0 or f"test {test} ... ok" not in good.stdout:
                raise RuntimeError(f"{name}: selected baseline test must pass\n{good.stdout}\n{good.stderr}")
            crates = work / name / "crates"
            shutil.copytree(root / "crates", crates)
            target = crates / relative.removeprefix("crates/")
            original = read(target)
            target.write_bytes(original.replace(before, after, 1).encode("utf-8"))
            binary = build(crates, work, name, sm_data)
            result = run([str(binary), "--exact", test], 60)
            if result.returncode != 101 or f"test {test} ... FAILED" not in result.stdout:
                raise RuntimeError(f"{name}: SURVIVED or failed elsewhere; {test} must fail\n"
                                   f"{result.stdout}\n{result.stderr}")
            print(f"{name}: KILLED by {test}", flush=True)
    print(f"session mutation checks: {len(MUTATIONS)}/{len(MUTATIONS)} killed", flush=True)


if __name__ == "__main__":
    main()
