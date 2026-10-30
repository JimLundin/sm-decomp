"""Bounded mutation check of actual detached-core assertions; GitHub CI only."""

import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
from actor_contract_mutations import check_actor_contracts, check_collision_contracts


LEADING_EDGE = """        room.square_radius(raw, axis)
            .wrapping_add(room.square_position(raw, axis))
            .wrapping_sub(1)"""
UPLOAD_SOURCE_CLEAR = """        host.session_mut()
            .presentation
            .visual_mut()
            .animated_tiles
            .slots[slot]
            .source = 0;"""
UPLOAD_ACK = "        host.upload(request);\n" + UPLOAD_SOURCE_CLEAR
MUTATIONS = (
    ("ignore_positive_beam_radius", "crates/sm_game/src/player_projectiles/collision.rs", LEADING_EDGE, "        position"),
    ("skip_collision_retirement", "crates/sm_game/src/player_projectiles/collision.rs", "    super::lifecycle::retire(room.0, raw);", "    let _ = raw;"),
    ("ignore_plant_position", "crates/sm_game/src/room_mechanisms/rules/plants.rs", "c.session_mut().player.x = mechanism!(c, slot, work0);", "let _ = mechanism!(c, slot, work0);"),
    ("ignore_new_beam", "crates/sm_game/src/player_projectiles/weapons.rs", "let kind = h.session().inventory.beams.active | 0x8000;", "let kind = 0x8000;"),
)
# New mutants require their named assertion, not just any panic in a check file.
NEW_MUTATIONS = (
    ("award_to_private_inventory_copy", "crates/sm_game/src/enemy_projectiles/drops.rs",
     "h.session_mut().inventory.restore(resource, amount);",
     "h.session().inventory.clone().restore(resource, amount);",
     "1053 actual drop award before sound"),
    ("discard_shared_shot_mark", "crates/sm_game/src/enemy_projectiles/contact.rs",
     "set_shot_direction(h, shot as isize, direction);",
     "let _ = direction;",
     "1052 shared shot mark"),
    ("stale_room_selected_after_spawn", "crates/sm_game/src/room_mechanisms/scheduler.rs",
     "let selected = h.session().mechanisms.selected;\n            let Some(next) = commands::run(h, identity, current.advance(2), selected) else {",
     "let selected = 2;\n            let Some(next) = commands::run(h, identity, current.advance(2), selected) else {",
     "1050 captured old program"),
)
# Visual checks run independently so only their intended panic may kill a mutant.
VISUAL_MUTATIONS = (
    ("omit_fractional_heat", "heat",
     "visual.composition.heat_fraction_before_sound",
     (("crates/sm_game/src/palette_effects/execution.rs",
       "let damage = (u32::from(fraction) | (u32::from(amount) << 16)).wrapping_add(0x4000);",
       "let damage = (u32::from(fraction) | (u32::from(amount) << 16)).wrapping_add(0);"),)),
    ("cache_heat_equipment_before_sound", "heat",
     "visual.composition.heat_equipment_reread",
     (("crates/sm_game/src/palette_effects/execution.rs", "        9298809 => {",
       "        9298809 => {\n            let mutation_equipment = host.session().inventory.equipment.active;"),
      ("crates/sm_game/src/palette_effects/execution.rs",
       "                let table = if host.session().inventory.equipment.active & 0x20 != 0 {",
       "                let table = if mutation_equipment & 0x20 != 0 {"),
      ("crates/sm_game/src/palette_effects/execution.rs",
       "                } else if host.session().inventory.equipment.active & 1 != 0 {",
       "                } else if mutation_equipment & 1 != 0 {"))),
    ("discard_palette_cross_owner_alias", "alias",
     "visual.composition.palette_timer_alias",
     (("crates/sm_game/src/palette_effects/execution.rs",
       "        None => host.set_palette_color_slice(displacement, value),",
       "        None => {},"),)),
    ("acknowledge_live_upload_selection", "upload_ack",
     "visual.composition.upload_captured_ack",
     (("crates/sm_game/src/animated_tiles/execution.rs",
       UPLOAD_ACK,
       "        host.upload(request);\n        let selected = usize::from(host.session().presentation.visual().animated_tiles.selected >> 1);\n        host.session_mut().presentation.visual_mut().animated_tiles.slots[selected].source = 0;"),)),
    ("acknowledge_failed_upload", "upload_failure",
     "visual.composition.upload_failure_source",
     (("crates/sm_game/src/animated_tiles/execution.rs",
       UPLOAD_ACK,
       UPLOAD_SOURCE_CLEAR + "\n        host.upload(request);"),)),
    ("discard_shared_palette_color", "eye",
     "visual.composition.animation_palette_after_eye",
     (("crates/sm_game/src/palette_effects/execution.rs", "            *color = value;",
       "            let _ = value;"),)),
    ("discard_eye_palette_producer", "eye",
     "visual.composition.eye_shared_colors",
     (("crates/sm_game/src/enemy_projectiles/setup.rs", "            ctx.set_palette_buffer((i >> 1) as isize, value);",
       "            let _ = value;"),)),
    ("replace_misc_carry_with_signed_branch", "samus_carry",
     "visual.composition.misc_unsigned_carry",
     (("crates/sm_game/src/samus_palette.rs", "        return carry(value >= 60);",
       "        return carry(value.wrapping_sub(60) & 0x8000 == 0);"),)),
    ("omit_selector2_final_misc", "samus_carry",
     "visual.composition.misc_unsigned_carry",
     (("crates/sm_game/src/samus_palette.rs",
       "    h.probe(\"samus.palette.tail\");\n    misc(h);\n    h.probe(\"samus.palette.return\");",
       "    h.probe(\"samus.palette.tail\");\n    h.probe(\"samus.palette.return\");"),)),
    ("replace_charge_carry_with_signed_branch", "samus_carry",
     "visual.composition.misc_charge_unsigned_carry",
     (("crates/sm_game/src/samus_palette.rs", "                return carry(charge >= 16);",
       "                return carry(charge.wrapping_sub(16) & 0x8000 == 0);"),)),
    ("replace_grapple_carry_with_signed_branch", "samus_carry",
     "visual.composition.misc_grapple_unsigned_carry",
     (("crates/sm_game/src/samus_palette.rs", "            return carry(grapple >= 0xc856);",
       "            return carry(grapple.wrapping_sub(0xc856) & 0x8000 == 0);"),)),
)
# Actual sprite faults use the same strict selected-case build and assertion rules.
SPRITE_MUTATIONS = (
    ('sprite_allocate_lowest_free', 'allocation',
     'sprite.mutation.allocation_captured_highest',
     (
         ('crates/sm_game/src/sprite_objects.rs',
          '        .rposition(|slot| slot.instruction == 0)',
          '        .position(|slot| slot.instruction == 0)'),
     )),
    ('sprite_population_clear_slot_zero', 'clears',
     'sprite.mutation.population_clear_extent',
     (
         ('crates/sm_game/src/sprite_objects.rs',
          '        if index != 0 {\n            slot.instruction = 0;',
          '        if true {\n            slot.instruction = 0;'),
     )),
    ('sprite_population_preserve_tails', 'clears',
     'sprite.mutation.population_clear_extent',
     (
         ('crates/sm_game/src/sprite_objects.rs',
          '    let tails = &mut session.sprite_compatibility;\n    tails.instruction.fill(0);\n    tails.timer.fill(0);\n    tails.appearance.fill(0);\n    tails.x.fill(0);\n    tails.x_fraction.fill(0);\n    tails.y.fill(0);\n    tails.y_fraction.fill(0);\n    tails.disable_flags.fill(0);\n',
          '    // Mutation: leave serialized compatibility tails unchanged.\n'),
     )),
    ('sprite_retire_clears_timers', 'clears',
     'sprite.mutation.retirement_preserves_columns',
     (
         ('crates/sm_game/src/sprite_objects.rs',
          'pub fn retire_all(session: &mut Session) {\n    for slot in &mut session.sprite_effects.active {\n        slot.instruction = 0;',
          'pub fn retire_all(session: &mut Session) {\n    for slot in &mut session.sprite_effects.active {\n        slot.instruction = 0;\n        slot.timer = 0;'),
     )),
    ('sprite_update_captured_timer_slot', 'live_selected',
     'sprite.update.live-selected',
     (
         ('crates/sm_game/src/sprite_objects.rs',
          '                    if timer & 0x8000 == 0 {\n                        write_selected_native!(host, timer, Timer, timer);',
          '                    if timer & 0x8000 == 0 {\n                        write_slot!(host, index, timer, Timer, timer);'),
     )),
    ('sprite_capture_content_before_diagnostic', 'live_selected',
     'sprite.content.post-diagnostic',
     (
         ('crates/sm_game/src/sprite_objects.rs',
          '                    let record = host.record(Source::Loaded, 0xb4, next);\n                    timer = record.native_word(0);',
          '                    let record = host.table(Source::Loaded, 0xb40000 | u32::from(next));\n                    let _diagnosed = host.record(Source::Loaded, 0xb4, next);\n                    timer = record.native_word(0);'),
     )),
    ('sprite_draw_ascending', 'draw_live',
     'sprite.mutation.draw_order_and_live_next',
     (
         ('crates/sm_game/src/sprite_objects.rs',
          '    for index in (0..32).rev() {',
          '    for index in 0..32 {'),
     )),
    ('sprite_draw_cached_future_slots', 'draw_live',
     'sprite.mutation.draw_order_and_live_next',
     (
         ('crates/sm_game/src/sprite_objects.rs',
          'pub fn draw(host: &mut impl Host) {\n    for index in (0..32).rev() {\n        if host.session().sprite_effects.active[index].instruction == 0 {\n            continue;\n        }\n        let x = host.session().sprite_effects.active[index]\n            .x\n            .whole\n            .wrapping_sub(host.inputs().camera_x);\n        if (x as i16).wrapping_add(16) < 0 || (i32::from(x) - 272) & 0x8000 == 0 {\n            continue;\n        }\n        let y = (host.session().sprite_effects.active[index].y.whole as i16)\n            .wrapping_sub(host.inputs().camera_y as i16);\n        if y < 0 || (i32::from(y) - 272) & 0x8000 == 0 {\n            continue;\n        }\n        let palette = host.session().sprite_effects.active[index].appearance & 0xe00;\n        let base_tile = host.session().sprite_effects.active[index].appearance & 0x1ff;\n        let cursor = host.session().sprite_effects.active[index].instruction;\n        let record = host.record(Source::Loaded, 0xb4, cursor);\n        let map = LegacySpriteMap(record.native_word(2));\n        host.submit_sprite(SpriteDraw {\n            map,\n            position: Point { x, y: y as u16 },\n            palette,\n            base_tile,\n        });\n        host.probe("sprite.draw.submitted");\n    }\n}',
          'pub fn draw(host: &mut impl Host) {\n    let mutation_slots = host.session().sprite_effects.active;\n    for index in (0..32).rev() {\n        if mutation_slots[index].instruction == 0 {\n            continue;\n        }\n        let x = mutation_slots[index]\n            .x\n            .whole\n            .wrapping_sub(host.inputs().camera_x);\n        if (x as i16).wrapping_add(16) < 0 || (i32::from(x) - 272) & 0x8000 == 0 {\n            continue;\n        }\n        let y = (mutation_slots[index].y.whole as i16)\n            .wrapping_sub(host.inputs().camera_y as i16);\n        if y < 0 || (i32::from(y) - 272) & 0x8000 == 0 {\n            continue;\n        }\n        let palette = mutation_slots[index].appearance & 0xe00;\n        let base_tile = mutation_slots[index].appearance & 0x1ff;\n        let cursor = mutation_slots[index].instruction;\n        let record = host.record(Source::Loaded, 0xb4, cursor);\n        let map = LegacySpriteMap(record.native_word(2));\n        host.submit_sprite(SpriteDraw {\n            map,\n            position: Point { x, y: y as u16 },\n            palette,\n            base_tile,\n        });\n        host.probe("sprite.draw.submitted");\n    }\n}'),
     )),
    ('sprite_drop_fractional_carry', 'draygon',
     'sprite.draygon42.carry-borrow',
     (
         ('crates/sm_game/src/sprite_effect_controllers.rs',
          '    let value = (u32::from(fraction) | (u32::from(whole) << 16)).wrapping_add(delta);',
          '    let value = (u32::from(fraction).wrapping_add(delta) & 0xffff)\n        | (u32::from(whole.wrapping_add((delta >> 16) as u16)) << 16);'),
     )),
    ('sprite_actor_cache_y_before_x_write', 'actor_alias',
     'sprite.actor.attach_live_y_alias',
     (
         ('crates/sm_game/src/sprite_effect_controllers.rs',
          '    let x = host.attachment_x(enemy);\n    write!(host, index, X, x.whole, x);\n    let y = host.attachment_y(enemy);',
          '    let x = host.attachment_x(enemy);\n    let y = host.attachment_y(enemy);\n    write!(host, index, X, x.whole, x);'),
     )),
    ('sprite_actor_cache_timer_before_clear', 'actor_alias',
     'sprite.actor.maw_live_first_timer',
     (
         ('crates/sm_game/src/sprite_effect_controllers.rs',
          'pub fn maw_appearance(\n    host: &mut impl Host,\n    handle: LegacySpriteHandle,\n    appearance: u16,\n    enemy: u16,\n) {',
          'pub fn maw_appearance(\n    host: &mut impl Host,\n    handle: LegacySpriteHandle,\n    appearance: u16,\n    enemy: u16,\n) {\n    let mutation_frozen_timer = host.attachment_frozen_timer(enemy);'),
         ('crates/sm_game/src/sprite_effect_controllers.rs',
          '    if host.attachment_frozen_timer(enemy) != 0 {',
          '    if mutation_frozen_timer != 0 {'),
     )),
    ('sprite_actor_cache_timer_before_freeze', 'actor_alias',
     'sprite.actor.maw_live_second_timer',
     (
         ('crates/sm_game/src/sprite_effect_controllers.rs',
          '    if host.attachment_frozen_timer(enemy) != 0 {\n        let old = read!(host, index, Appearance, appearance);\n        write!(host, index, Appearance, appearance, (old & 0xf1ff) | 0xc00);\n        let frozen_timer = host.attachment_frozen_timer(enemy);',
          '    if host.attachment_frozen_timer(enemy) != 0 {\n        let frozen_timer = host.attachment_frozen_timer(enemy);\n        let old = read!(host, index, Appearance, appearance);\n        write!(host, index, Appearance, appearance, (old & 0xf1ff) | 0xc00);'),
     )),
    ('sprite_maw_group_first_or_before_mask', 'maw_first_grouping',
     'sprite.maw.first_write_grouping',
     (
         ('crates/sm_game/src/sprite_effect_controllers.rs',
          '        appearance | (old & 0xf1ff)\n    );\n    if host.attachment_frozen_timer(enemy) != 0 {',
          '        (appearance | old) & 0xf1ff\n    );\n    if host.attachment_frozen_timer(enemy) != 0 {'),
     )),
    ('sprite_maw_group_final_or_before_mask', 'maw_final_grouping',
     'sprite.maw.final_restore_grouping',
     (
         ('crates/sm_game/src/sprite_effect_controllers.rs',
          '                appearance | (old & 0xf1ff)\n            );',
          '                (appearance | old) & 0xf1ff\n            );'),
     )),
)
# Actual gameplay flags faults; each uses a real composed producer/consumer selection.
FLAGS_MUTATIONS = (
    ('flags_event_write_discarded', 'plm_visual',
     'flags.plm.animation_event_alias',
     (
         ('crates/sm_game/src/progress.rs',
          '    mark(host, FlagColumn::Events, event);',
          '    prepare_flag_mask(host.compatibility_mut(), event);'),
     )),
    ('flags_event_keeps_stale_high_mask', 'plm_visual',
     'flags.scratch.full_mask',
     (
         ('crates/sm_game/src/progress.rs',
          '    compatibility.bitmask = 1u16 << (bit & 7);',
          '    compatibility.bitmask = (compatibility.bitmask & 0xff00) | (1u16 << (bit & 7));'),
     )),
    ('flags_boss_clears_high_mask', 'plm_visual',
     'flags.scratch.boss_low_only',
     (
         ('crates/sm_game/src/progress.rs',
          '    compatibility.bitmask = (compatibility.bitmask & 0xff00) | u16::from(low);',
          '    compatibility.bitmask = u16::from(low);'),
     )),
    ('flags_room_object_mark_uses_item_map', 'collections',
     'flags.plm.collection_identity',
     (
         ('crates/sm_game/src/progress.rs',
          '    mark(host, FlagColumn::RoomObjects, object);',
          '    mark(host, FlagColumn::Items, object);'),
     )),
    ('flags_rng_caches_prior_scratch', 'rng_scratch',
     'flags.eproj.rng_live_scratch',
     (
         ('crates/sm_game/src/enemy_projectiles/commands.rs',
          '        AT_D5E1 | AT_D69A => {\n            h.next_random();',
          '        AT_D5E1 | AT_D69A => {\n            let mutation_mask = h.session().flag_compatibility.bitmask;\n            h.next_random();'),
         ('crates/sm_game/src/enemy_projectiles/commands.rs',
          '                    (h.inputs().random >> 8) | (h.session().flag_compatibility.bitmask << 8)',
          '                    (h.inputs().random >> 8) | (mutation_mask << 8)'),
     )),
    ('flags_query_uses_prior_mask', 'eye_door',
     'flags.eproj.eye_door_current_mask',
     (
         ('crates/sm_game/src/progress.rs',
          '    let index = prepare_flag_mask(host.compatibility_mut(), bit);\n    read(host, column, index) & host.compatibility().bitmask as u8 != 0',
          '    let mutation_mask = host.compatibility().bitmask;\n    let index = prepare_flag_mask(host.compatibility_mut(), bit);\n    read(host, column, index) & mutation_mask as u8 != 0'),
     )),
    ('flags_door_adjacent_drops_external_byte', 'door511',
     'flags.plm.door511_adjacent',
     (
         ('crates/sm_game/src/progress.rs',
          '        None => host.door_adjacent_word_le(index as isize),',
          '        None => u16::from(bytes[usize::from(index)]),'),
     )),
    ('flags_door_adjacent_reads_before_mark', 'door511',
     'flags.plm.door511_adjacent',
     (
         ('crates/sm_game/src/progress.rs',
          'pub fn open_door_and_read_adjacent_flags(host: &mut (impl FlagHost + ?Sized), door: u16) -> u16 {\n    let index = mark(host, FlagColumn::Doors, door);\n    let bytes = &host.flags().opened_doors;\n    match bytes.get(usize::from(index)..usize::from(index) + 2) {\n        Some(pair) => u16::from_le_bytes([pair[0], pair[1]]),\n        None => host.door_adjacent_word_le(index as isize),\n    }\n}',
          'pub fn open_door_and_read_adjacent_flags(host: &mut (impl FlagHost + ?Sized), door: u16) -> u16 {\n    let index = prepare_flag_mask(host.compatibility_mut(), door);\n    let bytes = &host.flags().opened_doors;\n    let before = match bytes.get(usize::from(index)..usize::from(index) + 2) {\n        Some(pair) => u16::from_le_bytes([pair[0], pair[1]]),\n        None => host.door_adjacent_word_le(index as isize),\n    };\n    mark(host, FlagColumn::Doors, door);\n    before\n}'),
     )),
    ('flags_clamp_bit_index_to_events', 'aliases',
     'flags.alias.event_wide_index',
     (
         ('crates/sm_game/src/progress.rs',
          '    bit >> 3\n}',
          '    (bit >> 3) & 7\n}'),
     )),
    ('flags_boss_boundary_drops_high_byte', 'aliases',
     'flags.alias.boss_word_cross_owner',
     (
         ('crates/sm_game/src/progress.rs',
          '        None => host.boss_word(area, order),',
          '        None => host.boss_word(area, order) & 0xff,'),
     )),
)
VISUAL_SUCCESS = "visual composition case passed: "
VISUAL_PANIC = re.compile(
    r"(?:^|\n)thread '[^\n]+'(?: \(\d+\))? panicked at ([^\n]+):\n([^\n]*)\n"
)


def visual_stderr_matches(stderr, label=None, upload_failure=False, assertion_source="visual_checks.rs"):
    text = stderr.replace("\r\n", "\n")
    expected = [("visual_host.rs", "intentional upload failure")] if upload_failure else []
    if label is not None:
        expected.append((assertion_source, "assertion `left == right` failed: " + label))
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


# Built once from the unmutated crate; mutations only target files under sm/src.
SM_GAME = []


def run(command, timeout, allow=(), externs=None):
    if command[0] == "rustc":
        flags = os.environ.get("CARGO_ENCODED_RUSTFLAGS")
        if flags is None:
            flags = os.environ.get("RUSTFLAGS", "").split()
        else:
            flags = flags.split("\x1f") if flags else []
        command = [*command, *(SM_GAME if externs is None else externs), *flags, "-D", "warnings", *allow]
    return subprocess.run(
        command, capture_output=True, text=True, encoding="utf-8", errors="replace",
        timeout=timeout, check=False,
    )


def apply_edit(name, copied, crates, relative, before, after):
    # crates/... paths edit the copied target packages; others edit sm/src/game.
    if relative.startswith("crates/"):
        target = crates / relative.removeprefix("crates/")
    else:
        target = copied / "game" / relative
    original = target.read_bytes().decode("utf-8")
    if "\r\n" in original:
        before, after = before.replace("\n", "\r\n"), after.replace("\n", "\r\n")
    if original.count(before) != 1:
        raise RuntimeError(f"{name}: mutation anchor must occur exactly once: {relative}")
    target.write_bytes(original.replace(before, after, 1).encode("utf-8"))


def mutant_crates(source, work, name, edits):
    """A copy of the target packages when an edit reaches them, else None."""
    if not any(relative.startswith("crates/") for relative, _, _ in edits):
        return None
    crates = work / name / "crates"
    shutil.copytree(source.parent.parent / "crates", crates)
    return crates


def build_packages(crates, work, name, edits):
    """Rebuild the mutated packages (and every package above a mutated sm_data);
    link the untouched ones from the baseline libraries in `work`."""
    touched = {relative.split("/")[1] for relative, _, _ in edits if relative.startswith("crates/")}
    externs = ["-L", f"dependency={work / name}", "-L", f"dependency={work}"]
    for crate in ("sm_data", "sm_game", "sm_runtime"):
        if "sm_data" not in touched and crate not in touched:
            externs += ["--extern", f"{crate}={work / f'lib{crate}.rlib'}"]
            continue
        library = work / name / f"lib{crate}.rlib"
        built = run(["rustc", "--edition=2024", "-O", "--crate-name", crate, "--crate-type", "rlib",
                     str(crates / crate / "src" / "lib.rs"), "-o", str(library)], 180,
                    ("-A", "unused"), externs)
        if built.returncode != 0:
            raise RuntimeError(f"{name}: COMPILE ERROR in {crate}; not a killed mutant\n{built.stderr}")
        externs += ["--extern", f"{crate}={library}"]
    return externs


def check(source, work, name, mutation=None, expected_assertion=None):
    # Every build starts from the untouched source tree, never another mutant.
    copied = work / name / "src"
    shutil.copytree(source, copied)
    edits = () if mutation is None else (mutation,)
    crates = mutant_crates(source, work, name, edits)
    for relative, before, after in edits:
        apply_edit(name, copied, crates, relative, before, after)
    externs = None if crates is None else build_packages(crates, work, name, edits)
    main = copied / "verification" / "player_detached" / "main.rs"
    binary = work / name / ("check.exe" if os.name == "nt" else "check")
    # A mutant that deletes the last use of an item leaves it unused; the
    # baseline stays strict and mutants are judged only by execution.
    allow = () if mutation is None else ("-A", "unused")
    compiled = run(["rustc", "--edition=2024", "-O", str(main), "-o", str(binary)], 180, allow, externs)
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


def check_visual(source, work, mutation, baseline_binary, family="visual"):
    name, case, label, edits = mutation
    environment = dict(os.environ, RUST_BACKTRACE="0")
    assertion_source = {"sprite": "sprite_checks.rs", "flags": "flag_checks.rs"}.get(family, "visual_checks.rs")
    upload_failure = family == "visual" and case == "upload_failure"
    def execute(binary):
        return subprocess.run(
            [str(binary), f"--{family}-mutation", case], capture_output=True,
            text=True, encoding="utf-8", errors="replace", timeout=30,
            check=False, env=environment,
        )
    baseline = execute(baseline_binary)
    success = {"sprite": "sprite composition case passed: ", "flags": "flags composition case passed: "}.get(family, VISUAL_SUCCESS) + case
    if baseline.returncode != 0 or baseline.stdout.strip() != success or not visual_stderr_matches(baseline.stderr, upload_failure=upload_failure, assertion_source=assertion_source):
        raise RuntimeError(f"{name}: selected baseline must pass\n{baseline.stdout}\n{baseline.stderr}")
    print(f"{name}: selected baseline PASS", flush=True)
    copied = work / name / "src"
    shutil.copytree(source, copied)
    crates = mutant_crates(source, work, name, edits)
    for relative, before, after in edits:
        apply_edit(name, copied, crates, relative, before, after)
    externs = None if crates is None else build_packages(crates, work, name, edits)
    binary = work / name / ("check.exe" if os.name == "nt" else "check")
    main = copied / "verification" / "player_detached" / "main.rs"
    # A mutant that deletes the last use of an item leaves it unused; the
    # baseline stays strict and mutants are judged only by execution.
    allow = () if mutation is None else ("-A", "unused")
    compiled = run(["rustc", "--edition=2024", "-O", str(main), "-o", str(binary)], 180, allow, externs)
    if compiled.returncode != 0:
        raise RuntimeError(f"{name}: COMPILE ERROR; not a killed mutant\n{compiled.stderr}")
    print(f"{name}: compile PASS", flush=True)
    result = execute(binary)
    if (result.returncode != 101 or result.stdout.strip()
            or not visual_stderr_matches(result.stderr, label, upload_failure, assertion_source)):
        raise RuntimeError(f"{name}: expected only named assertion, exit101, no success marker; "
                           f"survival/crash/unrelated panic is not a kill\n{result.stdout}\n{result.stderr}")
    print(f"{name}: KILLED by {label}", flush=True)


def all_edits():
    """Every suite's (mutation name, relative path, anchor) edits."""
    import actor_contract_mutations as contracts
    for name, relative, before, _after in (*MUTATIONS, *((n, r, b, a) for n, r, b, a, _ in NEW_MUTATIONS)):
        yield name, relative, before
    for suite in (VISUAL_MUTATIONS, SPRITE_MUTATIONS, FLAGS_MUTATIONS,
                  contracts.ACTOR_CONTRACT_MUTATIONS, contracts.COLLISION_CONTRACT_MUTATIONS):
        for name, _selected, _label, edits in suite:
            for relative, before, _after in edits:
                yield name, relative, before


def check_anchors(source):
    """Fail when a mutation's file or anchor no longer exists exactly once."""
    stale = []
    count = 0
    for name, relative, before in all_edits():
        count += 1
        if relative.startswith("crates/"):
            target = source.parent.parent / relative
        else:
            target = source / "game" / relative
        text = target.read_bytes().decode("utf-8").replace("\r\n", "\n") if target.is_file() else None
        if text is None or text.count(before) != 1:
            found = "missing file" if text is None else f"anchor occurs {text.count(before)} times"
            stale.append(f"{name}: {relative}: {found}")
    if stale:
        raise RuntimeError("stale mutation anchors:\n" + "\n".join(stale))
    print(f"mutation anchors: {count}/{count} present exactly once", flush=True)


def main():
    if "--check-anchors" in sys.argv[1:]:
        check_anchors(Path(__file__).resolve().parents[2])
        return
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("mutation builds and execution are restricted to GitHub CI")
    # Installed at sm/src/verification/player_detached/mutations.py.
    source = Path(__file__).resolve().parents[2]
    if not (source / "verification" / "player_detached" / "main.rs").is_file():
        raise RuntimeError("install this runner alongside the repo detached harness")
    with tempfile.TemporaryDirectory(prefix="player-detached-mutations-") as directory:
        work = Path(directory)
        # This same runner compiles every detached baseline and selected mutant.
        SM_GAME.extend(["-L", f"dependency={work}"])
        for crate in ("sm_data", "sm_game", "sm_runtime"):
            library = work / f"lib{crate}.rlib"
            built = run(["rustc", "--edition=2024", "-O", "--crate-name", crate,
                         "--crate-type", "rlib", str(source.parent.parent / "crates" / crate / "src" / "lib.rs"),
                         "-o", str(library)], 180)
            if built.returncode != 0:
                raise RuntimeError(f"{crate} library build failed\n{built.stderr}")
            SM_GAME.extend(["--extern", f"{crate}={library}"])
        baseline_binary = check(source, work, "baseline")
        for name, relative, before, after in MUTATIONS:
            check(source, work, name, (relative, before, after))
        for name, relative, before, after, marker in NEW_MUTATIONS:
            check(source, work, name, (relative, before, after), marker)
        for mutation in VISUAL_MUTATIONS:
            check_visual(source, work, mutation, baseline_binary)
        for mutation in SPRITE_MUTATIONS:
            check_visual(source, work, mutation, baseline_binary, family="sprite")
        for mutation in FLAGS_MUTATIONS:
            check_visual(source, work, mutation, baseline_binary, family="flags")
        actor_contract_count = check_actor_contracts(source, work, run)
        collision_contract_count = check_collision_contracts(source, work, run)
    print(f"mutation checks: {len(MUTATIONS)+len(NEW_MUTATIONS)+len(VISUAL_MUTATIONS)+len(SPRITE_MUTATIONS)+len(FLAGS_MUTATIONS)}/{len(MUTATIONS)+len(NEW_MUTATIONS)+len(VISUAL_MUTATIONS)+len(SPRITE_MUTATIONS)+len(FLAGS_MUTATIONS)} killed; baseline passed", flush=True)
    print(f"actor contract mutation checks: {actor_contract_count}/{actor_contract_count} killed; selected baselines passed", flush=True)
    print(f"collision contract mutation checks: {collision_contract_count}/{collision_contract_count} killed; all 15 collision tests and selected baselines passed", flush=True)


if __name__ == "__main__":
    main()
