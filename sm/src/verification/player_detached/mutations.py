"""Bounded mutation check of actual detached-core assertions; GitHub CI only."""

import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


LEADING_EDGE = """        owner
            .field(slot, axis, Field::Radius)
            .wrapping_add(owner.field(slot, axis, Field::Position))
            .wrapping_sub(1)"""
MUTATIONS = (
    ("ignore_positive_beam_radius", "projectile_room.rs", LEADING_EDGE, "        position"),
    ("skip_collision_retirement", "projectile_room.rs", "    (context.retire(owner, slot), 1)", "    (owner, 1)"),
    ("ignore_plant_position", "room_mechanisms/rules/plants.rs", "c.session_mut().player.x = mechanism!(c, slot, work0);", "let _ = mechanism!(c, slot, work0);"),
    ("ignore_new_beam", "player_projectiles/weapons.rs", "let kind = h.session().inventory.beams.active | 0x8000;", "let kind = 0x8000;"),
)
SUCCESS = "player/animation/palette composed core checks passed"
ASSERTION = re.compile(
    r"panicked at [^\r\n]*[\\/](?:player_checks|composition_checks)\.rs:(\d+):\d+:\r?\n"
    r"assertion `left == right` failed"
)


def run(command, timeout):
    return subprocess.run(
        command, capture_output=True, text=True, encoding="utf-8", errors="replace",
        timeout=timeout, check=False,
    )


def check(source, work, name, mutation=None):
    # Every build starts from the untouched source tree, never another mutant.
    copied = work / name / "src"
    shutil.copytree(source, copied)
    if mutation is not None:
        relative, before, after = mutation
        target = copied / "game" / relative
        original = target.read_text(encoding="utf-8")
        if original.count(before) != 1:
            raise RuntimeError(f"{name}: mutation target must occur exactly once")
        target.write_text(original.replace(before, after, 1), encoding="utf-8")
    main = copied / "verification" / "player_detached" / "main.rs"
    binary = work / name / ("check.exe" if os.name == "nt" else "check")
    compiled = run(["rustc", "--edition=2024", "-O", str(main), "-o", str(binary)], 180)
    if compiled.returncode != 0:
        raise RuntimeError(f"{name}: COMPILE ERROR; not a killed mutant\n{compiled.stderr}")
    print(f"{name}: compile PASS", flush=True)
    result = run([str(binary)], 30)
    if mutation is None:
        if result.returncode != 0 or SUCCESS not in result.stdout:
            raise RuntimeError(f"baseline must pass\n{result.stdout}\n{result.stderr}")
        print("baseline: execution PASS", flush=True)
        return
    if result.returncode == 0 and SUCCESS in result.stdout:
        raise RuntimeError(f"{name}: SURVIVED; gameplay checks passed")
    assertion = ASSERTION.search(result.stderr)
    # Rust assertion panics exit 101. Compile failures, timeouts, native crashes,
    # unsupported-effect panics and missing checks must never count as kills.
    if result.returncode != 101 or assertion is None or SUCCESS in result.stdout:
        raise RuntimeError(
            f"{name}: EXECUTION ERROR; expected gameplay equality assertion panic (exit 101), "
            f"got exit {result.returncode}\n{result.stdout}\n{result.stderr}"
        )
    print(f"{name}: KILLED by gameplay assertion at line {assertion.group(1)} equality assertion", flush=True)


def main():
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("mutation builds and execution are restricted to GitHub CI")
    # Installed at sm/src/verification/player_detached/mutations.py.
    source = Path(__file__).resolve().parents[2]
    if not (source / "game" / "projectile_room.rs").is_file():
        raise RuntimeError("install this runner alongside the repo detached harness")
    with tempfile.TemporaryDirectory(prefix="player-detached-mutations-") as directory:
        work = Path(directory)
        check(source, work, "baseline")
        for name, relative, before, after in MUTATIONS:
            check(source, work, name, (relative, before, after))
    print(f"mutation checks: {len(MUTATIONS)}/{len(MUTATIONS)} killed; baseline passed", flush=True)


if __name__ == "__main__":
    main()
