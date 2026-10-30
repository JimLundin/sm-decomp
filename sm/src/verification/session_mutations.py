"""Mutation check of the Session's RAM mappings; the mutants run in GitHub CI.

Each mutant perturbs one owner's or view's mapping between RAM and Session
fields: it shifts a field's address by 2, swaps two fields, or drops one byte
of a word. It must make each named sm_game test fail: unit tests, and the
state layout tests in crates/sm_game/tests/state_layout.rs. A surviving mutant, a
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
CODEC = "crates/sm_game/src/actors/codec.rs"
# sm_game unit tests (module paths) and crates/sm_game/tests/state_layout.rs
# tests (bare names). Every listed test must fail for a kill.
LANES = "session_encoding::player_lane_tests::player_lanes_project_from_their_fields"
RAW_LANES = "session_encoding::player_lane_tests::raw_transition_lanes_project_from_their_fields"
INVENTORY_VIEW = "inventory::tests::the_view_reads_and_stores_the_owned_inventory"
CAMERA = "camera_round_trips_every_field"
FLOW = "flow_round_trips_every_field"
MENU = "menu_state_round_trips_every_field"
ROOM_VIEW = "room_view_round_trips_every_field"
ROOM_ENTRY = "room_entry_scroll_round_trips_every_field"
INVENTORY = "inventory_round_trips_every_field"
LIQUID = "liquid_round_trips_every_field"
EPROJ = "enemy_projectile_table_round_trips_every_field"
FADES = "visual_fades_round_trip_every_field"
HDMA = "hdma_objects_round_trip_every_field"
ACTORS = "actor_extended_ram_round_trips_every_word"
BOTWOON = "botwoon_death_words_round_trip"
NO_OVERLAP = "no_two_owners_claim_the_same_byte"
RANDOM_RAM = "session_round_trips_random_ram"
LITTLE_ENDIAN = "projection_words_are_little_endian_byte_pairs"
INPUTS = "inputs_read_every_observation_from_its_address"
# The shared scalar-word closures carry every table owner below.
SCALAR_OWNERS = (LANES, CAMERA, FLOW, MENU, ROOM_ENTRY, INVENTORY, LIQUID, EPROJ, HDMA)


def shift(name, relative, before, after, *tests):
    return (name, relative, before, after, tests)


# name, file, anchor, replacement, the tests that must all fail
MUTATIONS = (
    # Player lanes (session_encoding's own unit tests).
    shift("shift_swing_length_delta", ENCODING,
          "$f(0x0d00,$($borrow)* $s.player.swing.length_delta);",
          "$f(0x0d02,$($borrow)* $s.player.swing.length_delta);", LANES),
    shift("shift_health", ENCODING,
          "$f(0x9c2,$($borrow)* $s.inventory.health.current);",
          "$f(0x9c4,$($borrow)* $s.inventory.health.current);", LANES, INVENTORY),
    shift("shift_interrupted_pose_request", ENCODING,
          "            0x0a2a,\n            $s.player.transitions.interrupted,",
          "            0x0a2c,\n            $s.player.transitions.interrupted,", RAW_LANES),
    shift("swap_grapple_flags_and_delay", ENCODING,
          "$f(0x0cf4,$($borrow)* $s.player.grapple_flags);\n        $f(0x0cf6,$($borrow)* $s.player.grapple_delay);",
          "$f(0x0cf4,$($borrow)* $s.player.grapple_delay);\n        $f(0x0cf6,$($borrow)* $s.player.grapple_flags);", LANES),
    shift("swap_swing_rates", ENCODING,
          "$f(0x0d26,$($borrow)* $s.player.swing.angular_velocity);\n        $f(0x0d28,$($borrow)* $s.player.swing.angle_acceleration);",
          "$f(0x0d26,$($borrow)* $s.player.swing.angle_acceleration);\n        $f(0x0d28,$($borrow)* $s.player.swing.angular_velocity);", LANES),
    # Shared scalar-word projection and store: one dropped byte, every table owner.
    shift("drop_scalar_high_byte_read", ENCODING,
          "        if at == base || at == base + 1 {\n            result = Some(value.to_le_bytes()[at - base]);",
          "        if at == base {\n            result = Some(value.to_le_bytes()[at - base]);", *SCALAR_OWNERS),
    shift("drop_scalar_high_byte_write", ENCODING,
          "        if at == base || at == base + 1 {\n            let mut bytes = word.to_le_bytes();\n            bytes[at - base] = value;",
          "        if at == base {\n            let mut bytes = word.to_le_bytes();\n            bytes[at - base] = value;", *SCALAR_OWNERS),
    # Camera.
    shift("shift_camera_ideal_layer1", ENCODING,
          "$f(ram::IDEAL_LAYER1_XPOS+axis*4,", "$f(ram::IDEAL_LAYER1_XPOS+2+axis*4,", CAMERA),
    shift("swap_camera_layer_blocks", ENCODING,
          "$f(ram::LAYER1_X_BLOCK+axis*2,$($borrow)* $c.layer1_block[axis]);\n            $f(ram::LAYER2_X_BLOCK+axis*2,$($borrow)* $c.layer2_block[axis]);",
          "$f(ram::LAYER1_X_BLOCK+axis*2,$($borrow)* $c.layer2_block[axis]);\n            $f(ram::LAYER2_X_BLOCK+axis*2,$($borrow)* $c.layer1_block[axis]);", CAMERA),
    # Flow.
    shift("shift_flow_demo_set", ENCODING,
          "$f(ram::DEMO_SET,$($borrow)* $c.demo_set);", "$f(ram::DEMO_SET+2,$($borrow)* $c.demo_set);", FLOW),
    shift("swap_flow_demo_timer_and_set", ENCODING,
          "$f(ram::DEMO_TIMER,$($borrow)* $c.demo_timer);\n        $f(ram::DEMO_SET,$($borrow)* $c.demo_set);",
          "$f(ram::DEMO_TIMER,$($borrow)* $c.demo_set);\n        $f(ram::DEMO_SET,$($borrow)* $c.demo_timer);", FLOW),
    # Menu state.
    shift("shift_menu_index", ENCODING,
          "$f(ram::MENU_INDEX,$($borrow)* $c.menu_index);", "$f(ram::MENU_INDEX+2,$($borrow)* $c.menu_index);", MENU),
    shift("swap_menu_animation_timers", ENCODING,
          "$f(ram::PAUSEMENU_LR_ANIMATION_TIMER,$($borrow)* $c.pausemenu_lr_animation_timer);\n        $f(ram::PAUSEMENU_ITEM_SELECTOR_ANIMATION_TIMER,$($borrow)* $c.pausemenu_item_selector_animation_timer);",
          "$f(ram::PAUSEMENU_LR_ANIMATION_TIMER,$($borrow)* $c.pausemenu_item_selector_animation_timer);\n        $f(ram::PAUSEMENU_ITEM_SELECTOR_ANIMATION_TIMER,$($borrow)* $c.pausemenu_lr_animation_timer);", MENU),
    # Room view: geometry words and the effect decode.
    shift("shift_room_width", ENCODING,
          "$f(ram::ROOM_WIDTH_IN_BLOCKS,$($borrow)* $c.width);", "$f(ram::ROOM_WIDTH_IN_BLOCKS+2,$($borrow)* $c.width);", ROOM_VIEW),
    shift("swap_room_width_and_height", ENCODING,
          "$f(ram::ROOM_WIDTH_IN_BLOCKS,$($borrow)* $c.width);\n        $f(ram::ROOM_HEIGHT_IN_BLOCKS,$($borrow)* $c.height);",
          "$f(ram::ROOM_WIDTH_IN_BLOCKS,$($borrow)* $c.height);\n        $f(ram::ROOM_HEIGHT_IN_BLOCKS,$($borrow)* $c.width);", ROOM_VIEW),
    shift("swap_room_statue_words", ENCODING,
          "        statue_animation: w2,\n        statue_finished: w3,",
          "        statue_animation: w3,\n        statue_finished: w2,", ROOM_VIEW),
    # Room entry and scrolling (the alias maps one word onto another field).
    shift("shift_room_transition_selector", ENCODING,
          "$f(0x099c,$($borrow)* $s.room.entry.transition_selector);",
          "$f(0x099e,$($borrow)* $s.room.entry.transition_selector);", ROOM_ENTRY),
    shift("alias_room_entry_direction_to_interlock", ENCODING,
          "$f(0x0791,$($borrow)* $s.room.entry.direction);",
          "$f(0x0791,$($borrow)* $s.room.entry.elevator_interlock);", ROOM_ENTRY),
    # Inventory: the encoding and the lease-free view.
    shift("swap_inventory_equipment", ENCODING,
          "$f(0x9a2,$($borrow)* $s.inventory.equipment.active);\n        $f(0x9a4,$($borrow)* $s.inventory.equipment.owned);",
          "$f(0x9a2,$($borrow)* $s.inventory.equipment.owned);\n        $f(0x9a4,$($borrow)* $s.inventory.equipment.active);", INVENTORY, LANES),
    shift("swap_inventory_view_loadout", "crates/sm_game/src/inventory.rs",
          "owned: word(Word::CollectedItems),\n            active: word(Word::EquippedItems),",
          "owned: word(Word::EquippedItems),\n            active: word(Word::CollectedItems),", INVENTORY_VIEW),
    # Liquid.
    shift("shift_liquid_surface_fraction", ENCODING,
          "$f(0x195c,$($borrow)* $s.room.liquid.surface.fraction);",
          "$f(0x195a,$($borrow)* $s.room.liquid.surface.fraction);", LIQUID),
    shift("swap_liquid_surface_words", ENCODING,
          "$f(0x195c,$($borrow)* $s.room.liquid.surface.fraction);\n        $f(0x195e,$($borrow)* $s.room.liquid.surface.whole);",
          "$f(0x195c,$($borrow)* $s.room.liquid.surface.whole);\n        $f(0x195e,$($borrow)* $s.room.liquid.surface.fraction);", LIQUID),
    # Enemy projectile table.
    shift("shift_enemy_projectile_graphics", ENCODING,
          "$f(0x19bb+index*2,", "$f(0x19bd+index*2,", EPROJ),
    shift("swap_enemy_projectile_definition_and_graphics", ENCODING,
          "$f(0x1997+index*2,$($borrow)* $s.enemy_projectiles.slots[index].definition);\n            $f(0x19bb+index*2,$($borrow)* $s.enemy_projectiles.slots[index].graphics);",
          "$f(0x1997+index*2,$($borrow)* $s.enemy_projectiles.slots[index].graphics);\n            $f(0x19bb+index*2,$($borrow)* $s.enemy_projectiles.slots[index].definition);", EPROJ),
    # Visual fades and colours.
    shift("shift_palette_fade_step", ENCODING,
          "$f(0xc400,$($borrow)* presentation_ref!", "$f(0xc406,$($borrow)* presentation_ref!", FADES),
    shift("swap_screen_fade_words", ENCODING,
          "$f(0x723,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).screen_fade.delay);\n        $f(0x725,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).screen_fade.counter);",
          "$f(0x723,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).screen_fade.counter);\n        $f(0x725,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).screen_fade.delay);", FADES),
    shift("drop_colour_high_byte_read", ENCODING,
          "        return Some(colors[(offset % 512) / 2].to_le_bytes()[offset % 2]);",
          "        return Some(colors[(offset % 512) / 2].to_le_bytes()[0]);", FADES),
    # HDMA objects.
    shift("shift_hdma_bank_slot", ENCODING,
          "$f(0x18c0+index*2,", "$f(0x18c2+index*2,", HDMA),
    shift("swap_hdma_channels_and_bank", ENCODING,
          "$f(0x18b4+index*2,$($borrow)* $s.hdma.objects[index].channels);\n            $f(0x18c0+index*2,$($borrow)* $s.hdma.objects[index].bank_slot);",
          "$f(0x18b4+index*2,$($borrow)* $s.hdma.objects[index].bank_slot);\n            $f(0x18c0+index*2,$($borrow)* $s.hdma.objects[index].channels);", HDMA),
    # Actor extended RAM.
    shift("shift_actor_extended_words", CODEC,
          "[(at - 0x7800) % 0x800 / 2]),", "[((at - 0x7800) % 0x800 / 2 + 1) % 0x400]),", ACTORS),
    shift("swap_actor_extended_blocks", CODEC,
          "$s.actors.extended.blocks[(at - 0x7800) / 0x800]", "$s.actors.extended.blocks[((at - 0x7800) / 0x800) ^ 1]", ACTORS),
    shift("drop_actor_word_high_byte_read", CODEC,
          "common_word!(s, at & !1, (&)).map(|word| word.to_le_bytes()[at & 1])",
          "common_word!(s, at & !1, (&)).map(|word| word.to_le_bytes()[0])", ACTORS),
    # Botwoon's death words.
    shift("shift_botwoon_tail_ready", ENCODING,
          "$f(0x803c, $($borrow)* $s.botwoon_death.tail_ready);", "$f(0x803e, $($borrow)* $s.botwoon_death.tail_ready);", BOTWOON),
    shift("swap_botwoon_pending_and_active", ENCODING,
          "$f(0x801e, $($borrow)* $s.botwoon_death.pending_hit);\n        $f(0x8020, $($borrow)* $s.botwoon_death.active);",
          "$f(0x801e, $($borrow)* $s.botwoon_death.active);\n        $f(0x8020, $($borrow)* $s.botwoon_death.pending_hit);", BOTWOON),
    shift("drop_botwoon_high_byte_read", ENCODING,
          "        if (base..base + 2).contains(&at) {\n            result = Some(word.to_le_bytes()[at - base]);\n        }\n    };\n    botwoon_death_words!(s, select, (&));",
          "        if (base..base + 1).contains(&at) {\n            result = Some(word.to_le_bytes()[at - base]);\n        }\n    };\n    botwoon_death_words!(s, select, (&));", BOTWOON),
    # Whole-Session properties.
    shift("encode_one_word_twice", ENCODING,
          "        $f(0x0d00,$($borrow)* $s.player.swing.length_delta);\n",
          "        $f(0x0d00,$($borrow)* $s.player.swing.length_delta);\n        $f(0x0d00,$($borrow)* $s.player.swing.length_delta);\n",
          NO_OVERLAP),
    shift("encode_skips_a_decoded_word", ENCODING,
          "    let mut emit = |at: usize, value: &u16| visit(at, *value);\n    scalar_words!(s,emit,(&));",
          "    let mut emit = |at: usize, value: &u16| if at != 0x0d00 { visit(at, *value) };\n    scalar_words!(s,emit,(&));",
          RANDOM_RAM),
    shift("projection_stores_big_endian", "crates/sm_game/src/projection.rs",
          "    for (lane, value) in value.to_le_bytes().into_iter().enumerate() {",
          "    for (lane, value) in value.to_be_bytes().into_iter().enumerate() {", LITTLE_ENDIAN),
    shift("shift_lava_observation", ENCODING,
          "        lava_y: w(0x1962),", "        lava_y: w(0x1964),", INPUTS),
)


def read(path):
    return path.read_bytes().decode("utf-8").replace("\r\n", "\n")


def check_anchors(root):
    stale = [f"{name}: {relative}: anchor occurs {read(root / relative).count(before)} times"
             for name, relative, before, _after, _tests in MUTATIONS
             if not (root / relative).is_file() or read(root / relative).count(before) != 1]
    if stale:
        raise RuntimeError("stale session mutation anchors:\n" + "\n".join(stale))
    print(f"session mutation anchors: {len(MUTATIONS)}/{len(MUTATIONS)} present exactly once", flush=True)


def run(command, timeout):
    return subprocess.run(command, capture_output=True, text=True, encoding="utf-8",
                          errors="replace", timeout=timeout, check=False)


def build(crates, work, name, sm_data, layout):
    """The sm_game unit-test binary, or the state layout test binary over a
    mutated sm_game library."""
    folder = work / name
    exe = ".exe" if os.name == "nt" else ""
    common = ["--extern", f"sm_data={sm_data}", "-L", f"dependency={work}", "-L", f"dependency={folder}", "-A", "unused"]
    if not layout:
        binary = folder / f"sm_game_tests{exe}"
        built = run(["rustc", "--edition=2024", "-O", "--test", "--crate-name", "sm_game",
                     str(crates / "sm_game" / "src" / "lib.rs"), "-o", str(binary), *common], 900)
    else:
        library = folder / "libsm_game.rlib"
        built = run(["rustc", "--edition=2024", "-O", "--crate-name", "sm_game", "--crate-type", "rlib",
                     str(crates / "sm_game" / "src" / "lib.rs"), "-o", str(library), *common], 900)
        binary = folder / f"state_layout{exe}"
        if built.returncode == 0:
            built = run(["rustc", "--edition=2024", "-O", "--test", str(crates / "sm_game" / "tests" / "state_layout.rs"),
                         "--extern", f"sm_game={library}", "-o", str(binary), *common], 900)
    if built.returncode != 0:
        raise RuntimeError(f"{name}: COMPILE ERROR; not a killed mutant\n{built.stderr}")
    return binary


def binaries(crates, work, name, sm_data, tests):
    """The unit and layout binaries this mutant's tests need, keyed by kind."""
    kinds = {"::" not in test for test in tests}
    (work / name).mkdir(parents=True, exist_ok=True)
    return {layout: build(crates, work, name, sm_data, layout) for layout in kinds}


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
        every = {test for *_, tests in MUTATIONS for test in tests}
        baseline = binaries(root / "crates", work, "baseline", sm_data, every)
        for name, relative, before, after, tests in MUTATIONS:
            for test in tests:
                good = run([str(baseline["::" not in test]), "--exact", test], 120)
                if good.returncode != 0 or f"test {test} ... ok" not in good.stdout:
                    raise RuntimeError(f"{name}: selected baseline test must pass\n{good.stdout}\n{good.stderr}")
            crates = work / name / "crates"
            shutil.copytree(root / "crates", crates)
            target = crates / relative.removeprefix("crates/")
            target.write_bytes(read(target).replace(before, after, 1).encode("utf-8"))
            mutant = binaries(crates, work, name, sm_data, tests)
            for test in tests:
                result = run([str(mutant["::" not in test]), "--exact", test], 120)
                if result.returncode != 101 or f"test {test} ... FAILED" not in result.stdout:
                    raise RuntimeError(f"{name}: SURVIVED; {test} must fail\n{result.stdout}\n{result.stderr}")
            print(f"{name}: KILLED by {', '.join(tests)}", flush=True)
    print(f"session mutation checks: {len(MUTATIONS)}/{len(MUTATIONS)} killed", flush=True)


if __name__ == "__main__":
    main()
