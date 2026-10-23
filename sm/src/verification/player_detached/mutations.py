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
# New mutants require their named assertion, not just any panic in a check file.
NEW_MUTATIONS = (
    ("award_to_private_inventory_copy", "enemy_projectiles/drops.rs",
     "h.session_mut().inventory.restore(resource, amount);",
     "h.session().inventory.clone().restore(resource, amount);",
     "1053 actual drop award before sound"),
    ("discard_shared_shot_mark", "enemy_projectiles/contact.rs",
     "set_shot_direction(h, shot as isize, direction);",
     "let _ = direction;",
     "1052 shared shot mark"),
    ("stale_room_selected_after_spawn", "room_mechanisms/scheduler.rs",
     "let selected = h.session().mechanisms.selected;\n            let Some(next) = commands::run(h, identity, current.advance(2), selected) else {",
     "let selected = 2;\n            let Some(next) = commands::run(h, identity, current.advance(2), selected) else {",
     "1050 captured old program"),
)
# Visual checks run independently so only their intended panic may kill a mutant.
VISUAL_MUTATIONS = (
    ("omit_fractional_heat", "heat",
     "visual.composition.heat_fraction_before_sound",
     (("palette_effects/execution.rs",
       "let damage = (u32::from(fraction) | (u32::from(amount) << 16)).wrapping_add(0x4000);",
       "let damage = (u32::from(fraction) | (u32::from(amount) << 16)).wrapping_add(0);"),)),
    ("cache_heat_equipment_before_sound", "heat",
     "visual.composition.heat_equipment_reread",
     (("palette_effects/execution.rs", "        9298809 => {",
       "        9298809 => {\n            let mutation_equipment = host.session().inventory.equipment.active;"),
      ("palette_effects/execution.rs",
       "                let table = if host.session().inventory.equipment.active & 0x20 != 0 {",
       "                let table = if mutation_equipment & 0x20 != 0 {"),
      ("palette_effects/execution.rs",
       "                } else if host.session().inventory.equipment.active & 1 != 0 {",
       "                } else if mutation_equipment & 1 != 0 {"))),
    ("discard_palette_cross_owner_alias", "alias",
     "visual.composition.palette_timer_alias",
     (("palette_effects/execution.rs",
       "        None => host.set_palette_color_slice(displacement, value),",
       "        None => {},"),)),
    ("acknowledge_live_upload_selection", "upload_ack",
     "visual.composition.upload_captured_ack",
     (("animated_tiles/execution.rs",
       "        host.upload(request);\n        host.session_mut().visual.animated_tiles.slots[slot].source = 0;",
       "        host.upload(request);\n        let selected = usize::from(host.session().visual.animated_tiles.selected >> 1);\n        host.session_mut().visual.animated_tiles.slots[selected].source = 0;"),)),
    ("acknowledge_failed_upload", "upload_failure",
     "visual.composition.upload_failure_source",
     (("animated_tiles/execution.rs",
       "        host.upload(request);\n        host.session_mut().visual.animated_tiles.slots[slot].source = 0;",
       "        host.session_mut().visual.animated_tiles.slots[slot].source = 0;\n        host.upload(request);"),)),
    ("discard_shared_palette_color", "eye",
     "visual.composition.animation_palette_after_eye",
     (("palette_effects/execution.rs", "            *color = value;",
       "            let _ = value;"),)),
    ("discard_eye_palette_producer", "eye",
     "visual.composition.eye_shared_colors",
     (("enemy_projectiles/setup.rs", "            ctx.set_palette_buffer((i >> 1) as isize, value);",
       "            let _ = value;"),)),
    ("replace_misc_carry_with_signed_branch", "samus_carry",
     "visual.composition.misc_unsigned_carry",
     (("samus_palette.rs", "        return carry(value >= 60);",
       "        return carry(value.wrapping_sub(60) & 0x8000 == 0);"),)),
    ("omit_selector2_final_misc", "samus_carry",
     "visual.composition.misc_unsigned_carry",
     (("samus_palette.rs",
       "    h.probe(\"samus.palette.tail\");\n    misc(h);\n    h.probe(\"samus.palette.return\");",
       "    h.probe(\"samus.palette.tail\");\n    h.probe(\"samus.palette.return\");"),)),
    ("replace_charge_carry_with_signed_branch", "samus_carry",
     "visual.composition.misc_charge_unsigned_carry",
     (("samus_palette.rs", "                return carry(charge >= 16);",
       "                return carry(charge.wrapping_sub(16) & 0x8000 == 0);"),)),
    ("replace_grapple_carry_with_signed_branch", "samus_carry",
     "visual.composition.misc_grapple_unsigned_carry",
     (("samus_palette.rs", "            return carry(grapple >= 0xc856);",
       "            return carry(grapple.wrapping_sub(0xc856) & 0x8000 == 0);"),)),
)
VISUAL_SUCCESS = "visual composition case passed: "
VISUAL_PANIC = re.compile(
    r"(?:^|\n)thread '[^\n]+'(?: \(\d+\))? panicked at ([^\n]+):\n([^\n]*)\n"
)


def visual_stderr_matches(stderr, label=None, upload_failure=False):
    text = stderr.replace("\r\n", "\n")
    expected = [("visual_host.rs", "intentional upload failure")] if upload_failure else []
    if label is not None:
        expected.append(("visual_checks.rs", "assertion `left == right` failed: " + label))
    panics = list(VISUAL_PANIC.finditer(text))
    if len(panics) != len(expected):
        return False
    if not panics:
        return not text.strip()
    if text[:panics[0].start()].strip():
        return False
    note = "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
    for index, (panic, (source, wanted)) in enumerate(zip(panics, expected)):
        location, message = panic.groups()
        if (not re.search(r"(?:^|/)" + re.escape(source) + r":\d+:\d+$", location.replace("\\", "/"))
                or message != wanted):
            return False
        end = panics[index + 1].start() if index + 1 < len(panics) else len(text)
        lines = [line.strip() for line in text[panic.end():end].splitlines()
                 if line.strip() and line.strip() != note]
        if wanted.startswith("assertion `left == right` failed:"):
            if len(lines) != 2 or not lines[0].startswith("left: ") or not lines[1].startswith("right: "):
                return False
        elif lines:
            return False
    return True


SUCCESS = "player/animation/palette composed core checks passed"
ASSERTION = re.compile(
    r"panicked at [^\r\n]*[\\/](?:player_checks|composition_checks|enemy_checks)\.rs:(\d+):\d+:\r?\n"
    r"assertion `left == right` failed"
)


def run(command, timeout):
    return subprocess.run(
        command, capture_output=True, text=True, encoding="utf-8", errors="replace",
        timeout=timeout, check=False,
    )


def check(source, work, name, mutation=None, expected_assertion=None):
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
        return binary
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
    if expected_assertion is not None and expected_assertion not in result.stderr:
        raise RuntimeError(f"{name}: EXECUTION ERROR; wrong equality assertion, expected {expected_assertion}\n{result.stderr}")
    print(f"{name}: KILLED by gameplay assertion at line {assertion.group(1)} equality assertion", flush=True)


def check_visual(source, work, mutation, baseline_binary):
    name, case, label, edits = mutation
    environment = dict(os.environ, RUST_BACKTRACE="0")
    def execute(binary):
        return subprocess.run(
            [str(binary), "--visual-mutation", case], capture_output=True,
            text=True, encoding="utf-8", errors="replace", timeout=30,
            check=False, env=environment,
        )
    baseline = execute(baseline_binary)
    success = VISUAL_SUCCESS + case
    if baseline.returncode != 0 or baseline.stdout.strip() != success or not visual_stderr_matches(baseline.stderr, upload_failure=case == "upload_failure"):
        raise RuntimeError(f"{name}: selected baseline must pass\n{baseline.stdout}\n{baseline.stderr}")
    print(f"{name}: selected baseline PASS", flush=True)
    copied = work / name / "src"
    shutil.copytree(source, copied)
    for relative, before, after in edits:
        target = copied / "game" / relative
        original = target.read_text(encoding="utf-8")
        if original.count(before) != 1:
            raise RuntimeError(f"{name}: mutation anchor must occur exactly once: {relative}")
        target.write_text(original.replace(before, after, 1), encoding="utf-8")
    binary = work / name / ("check.exe" if os.name == "nt" else "check")
    main = copied / "verification" / "player_detached" / "main.rs"
    compiled = run(["rustc", "--edition=2024", "-O", str(main), "-o", str(binary)], 180)
    if compiled.returncode != 0:
        raise RuntimeError(f"{name}: COMPILE ERROR; not a killed mutant\n{compiled.stderr}")
    print(f"{name}: compile PASS", flush=True)
    result = execute(binary)
    if (result.returncode != 101 or result.stdout.strip()
            or not visual_stderr_matches(result.stderr, label, case == "upload_failure")):
        raise RuntimeError(f"{name}: expected only named assertion, exit101, no success marker; "
                           f"survival/crash/unrelated panic is not a kill\n{result.stdout}\n{result.stderr}")
    print(f"{name}: KILLED by {label}", flush=True)


def main():
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("mutation builds and execution are restricted to GitHub CI")
    # Installed at sm/src/verification/player_detached/mutations.py.
    source = Path(__file__).resolve().parents[2]
    if not (source / "game" / "projectile_room.rs").is_file():
        raise RuntimeError("install this runner alongside the repo detached harness")
    with tempfile.TemporaryDirectory(prefix="player-detached-mutations-") as directory:
        work = Path(directory)
        baseline_binary = check(source, work, "baseline")
        for name, relative, before, after in MUTATIONS:
            check(source, work, name, (relative, before, after))
        for name, relative, before, after, marker in NEW_MUTATIONS:
            check(source, work, name, (relative, before, after), marker)
        for mutation in VISUAL_MUTATIONS:
            check_visual(source, work, mutation, baseline_binary)
    print(f"mutation checks: {len(MUTATIONS)+len(NEW_MUTATIONS)+len(VISUAL_MUTATIONS)}/{len(MUTATIONS)+len(NEW_MUTATIONS)+len(VISUAL_MUTATIONS)} killed; baseline passed", flush=True)


if __name__ == "__main__":
    main()
