//! Source bytes captured by the game library's own compilation.
//! The launcher checks this complete list before trusting Cargo's cached rlib.
//! New library source files must be listed here; missing entries fail closed.

pub static FILES: &[(&str, &[u8])] = &[
    ("sm/src/game/dialogue.rs", include_bytes!("game/dialogue.rs")),
    ("sm/src/game/dialogue_ram.rs", include_bytes!("game/dialogue_ram.rs")),
    ("sm/src/game/music.rs", include_bytes!("game/music.rs")),
    ("sm/src/game/plm_spawning.rs", include_bytes!("game/plm_spawning.rs")),
    (
        "sm/src/game/room_state.rs",
        include_bytes!("game/room_state.rs"),
    ),
    (
        "sm/src/game/progress_flags.rs",
        include_bytes!("game/progress_flags.rs"),
    ),
    (
        "sm/src/native_player_ram.rs",
        include_bytes!("native_player_ram.rs"),
    ),
    (
        "sm/src/initial_state.rs",
        include_bytes!("initial_state.rs"),
    ),
    ("sm/src/audio_upload.rs", include_bytes!("audio_upload.rs")),
    ("sm/src/apu_queue.rs", include_bytes!("apu_queue.rs")),
    ("sm/src/apu_ram.rs", include_bytes!("apu_ram.rs")),
    (
        "sm/src/game/launch_motion.rs",
        include_bytes!("game/launch_motion.rs"),
    ),
    (
        "sm/src/game/launch_motion_ram.rs",
        include_bytes!("game/launch_motion_ram.rs"),
    ),
    ("sm/src/snapshot_io.rs", include_bytes!("snapshot_io.rs")),
    ("sm/src/save_ram.rs", include_bytes!("save_ram.rs")),
    (
        "sm/src/game/mode7_transfers.rs",
        include_bytes!("game/mode7_transfers.rs"),
    ),
    (
        "sm/src/game/vram_read_queue.rs",
        include_bytes!("game/vram_read_queue.rs"),
    ),
    (
        "sm/src/game/decompression.rs",
        include_bytes!("game/decompression.rs"),
    ),
    (
        "sm/src/game/gameplay.rs",
        include_bytes!("game/gameplay.rs"),
    ),
    ("sm/src/game/prelude.rs", include_bytes!("game/prelude.rs")),
    (
        "sm/src/game/samus_pose.rs",
        include_bytes!("game/samus_pose.rs"),
    ),
    (
        "sm/src/game/beam_windows.rs",
        include_bytes!("game/beam_windows.rs"),
    ),
    (
        "sm/src/game/explosion_window.rs",
        include_bytes!("game/explosion_window.rs"),
    ),
    (
        "sm/src/game/projectile_flight.rs",
        include_bytes!("game/projectile_flight.rs"),
    ),
    (
        "sm/src/game/moving_platforms.rs",
        include_bytes!("game/moving_platforms.rs"),
    ),
    (
        "sm/src/audio_snapshot.rs",
        include_bytes!("audio_snapshot.rs"),
    ),
    ("sm/src/c_library.rs", include_bytes!("c_library.rs")),
    ("sm/src/config.rs", include_bytes!("config.rs")),
    ("sm/src/native_config.c", include_bytes!("native_config.c")),
    ("sm/src/native_config.h", include_bytes!("native_config.h")),
    ("sm/src/native_shader.c", include_bytes!("native_shader.c")),
    ("sm/src/native_shader.h", include_bytes!("native_shader.h")),
    (
        "sm/src/native_desktop.c",
        include_bytes!("native_desktop.c"),
    ),
    (
        "sm/src/native_desktop.h",
        include_bytes!("native_desktop.h"),
    ),
    ("sm/src/cpu_infra.rs", include_bytes!("cpu_infra.rs")),
    (
        "sm/src/game/callback_tables.rs",
        include_bytes!("game/callback_tables.rs"),
    ),
    (
        "sm/src/game/constants.rs",
        include_bytes!("game/constants.rs"),
    ),
    (
        "sm/src/game/enemy_layouts.rs",
        include_bytes!("game/enemy_layouts.rs"),
    ),
    (
        "sm/src/game/enemy_ram.rs",
        include_bytes!("game/enemy_ram.rs"),
    ),
    (
        "sm/src/game/intro_pages.rs",
        include_bytes!("game/intro_pages.rs"),
    ),
    (
        "sm/src/game/beam_firing.rs",
        include_bytes!("game/beam_firing.rs"),
    ),
    (
        "sm/src/game/room_backgrounds.rs",
        include_bytes!("game/room_backgrounds.rs"),
    ),
    (
        "sm/src/game/room_scrolls.rs",
        include_bytes!("game/room_scrolls.rs"),
    ),
    ("sm/src/game/sm_80.rs", include_bytes!("game/sm_80.rs")),
    ("sm/src/game/sm_81.rs", include_bytes!("game/sm_81.rs")),
    (
        "sm/src/game/sprite_drawing.rs",
        include_bytes!("game/sprite_drawing.rs"),
    ),
    ("sm/src/game/sm_82.rs", include_bytes!("game/sm_82.rs")),
    ("sm/src/game/sm_84.rs", include_bytes!("game/sm_84.rs")),
    (
        "sm/src/game/item_pickups.rs",
        include_bytes!("game/item_pickups.rs"),
    ),
    (
        "sm/src/game/item_pickups_ram.rs",
        include_bytes!("game/item_pickups_ram.rs"),
    ),
    (
        "sm/src/game/message_boxes.rs",
        include_bytes!("game/message_boxes.rs"),
    ),
    ("sm/src/game/sm_86.rs", include_bytes!("game/sm_86.rs")),
    (
        "sm/src/game/eproj_script.rs",
        include_bytes!("game/eproj_script.rs"),
    ),
    ("sm/src/game/sm_87.rs", include_bytes!("game/sm_87.rs")),
    (
        "sm/src/game/layer_blending.rs",
        include_bytes!("game/layer_blending.rs"),
    ),
    (
        "sm/src/game/liquid_effects.rs",
        include_bytes!("game/liquid_effects.rs"),
    ),
    (
        "sm/src/game/scrolling_sky.rs",
        include_bytes!("game/scrolling_sky.rs"),
    ),
    ("sm/src/game/sm_88.rs", include_bytes!("game/sm_88.rs")),
    ("sm/src/game/sm_89.rs", include_bytes!("game/sm_89.rs")),
    ("sm/src/game/sm_8b.rs", include_bytes!("game/sm_8b.rs")),
    ("sm/src/game/sm_8d.rs", include_bytes!("game/sm_8d.rs")),
    ("sm/src/game/sm_8f.rs", include_bytes!("game/sm_8f.rs")),
    ("sm/src/game/minimap.rs", include_bytes!("game/minimap.rs")),
    ("sm/src/game/sm_90.rs", include_bytes!("game/sm_90.rs")),
    (
        "sm/src/game/xray_background.rs",
        include_bytes!("game/xray_background.rs"),
    ),
    ("sm/src/game/sm_91.rs", include_bytes!("game/sm_91.rs")),
    (
        "sm/src/game/samus_animation.rs",
        include_bytes!("game/samus_animation.rs"),
    ),
    (
        "sm/src/game/projectile_animation.rs",
        include_bytes!("game/projectile_animation.rs"),
    ),
    (
        "sm/src/game/grapple_swing.rs",
        include_bytes!("game/grapple_swing.rs"),
    ),
    ("sm/src/game/sm_94.rs", include_bytes!("game/sm_94.rs")),
    ("sm/src/game/sm_9b.rs", include_bytes!("game/sm_9b.rs")),
    ("sm/src/game/sm_a0.rs", include_bytes!("game/sm_a0.rs")),
    ("sm/src/game/enemy_scheduling.rs", include_bytes!("game/enemy_scheduling.rs")),
    ("sm/src/game/scene_drawing.rs", include_bytes!("game/scene_drawing.rs")),
    (
        "sm/src/game/enemy_population.rs",
        include_bytes!("game/enemy_population.rs"),
    ),
    (
        "sm/src/game/enemy_population_ram.rs",
        include_bytes!("game/enemy_population_ram.rs"),
    ),
    ("sm/src/game/sm_a2.rs", include_bytes!("game/sm_a2.rs")),
    ("sm/src/game/sm_a3.rs", include_bytes!("game/sm_a3.rs")),
    ("sm/src/game/sm_a4.rs", include_bytes!("game/sm_a4.rs")),
    ("sm/src/game/sm_a5.rs", include_bytes!("game/sm_a5.rs")),
    ("sm/src/game/sm_a6.rs", include_bytes!("game/sm_a6.rs")),
    ("sm/src/game/sm_a7.rs", include_bytes!("game/sm_a7.rs")),
    ("sm/src/game/sm_a8.rs", include_bytes!("game/sm_a8.rs")),
    ("sm/src/game/sm_a9.rs", include_bytes!("game/sm_a9.rs")),
    ("sm/src/game/sm_aa.rs", include_bytes!("game/sm_aa.rs")),
    (
        "sm/src/game/mother_brain_visuals.rs",
        include_bytes!("game/mother_brain_visuals.rs"),
    ),
    ("sm/src/game/sm_b2.rs", include_bytes!("game/sm_b2.rs")),
    ("sm/src/game/sm_b3.rs", include_bytes!("game/sm_b3.rs")),
    ("sm/src/game/sprite_objects.rs", include_bytes!("game/sprite_objects.rs")),
    ("sm/src/game/waver.rs", include_bytes!("game/waver.rs")),
    ("sm/src/game_state.rs", include_bytes!("game_state.rs")),
    ("sm/src/rom.rs", include_bytes!("rom.rs")),
    ("sm/src/rom_patches.rs", include_bytes!("rom_patches.rs")),
    ("sm/src/recording.rs", include_bytes!("recording.rs")),
    ("sm/src/host.rs", include_bytes!("host.rs")),
    ("sm/src/lib.rs", include_bytes!("lib.rs")),
    ("sm/src/native_audio.rs", include_bytes!("native_audio.rs")),
    (
        "sm/src/platform/glsl_shader.rs",
        include_bytes!("platform/glsl_shader.rs"),
    ),
    (
        "sm/src/platform/opengl.rs",
        include_bytes!("platform/opengl.rs"),
    ),
    (
        "sm/src/platform/sdl_constants.rs",
        include_bytes!("platform/sdl_constants.rs"),
    ),
    (
        "sm/src/platform/texture_png.rs",
        include_bytes!("platform/texture_png.rs"),
    ),
    ("sm/src/ram_map.rs", include_bytes!("ram_map.rs")),
    ("sm/src/rtl.rs", include_bytes!("rtl.rs")),
    ("sm/src/snes/apu.rs", include_bytes!("snes/apu.rs")),
    ("sm/src/snes/cart.rs", include_bytes!("snes/cart.rs")),
    ("sm/src/snes/cpu.rs", include_bytes!("snes/cpu.rs")),
    ("sm/src/snes/dma.rs", include_bytes!("snes/dma.rs")),
    ("sm/src/snes/dsp.rs", include_bytes!("snes/dsp.rs")),
    ("sm/src/snes/input.rs", include_bytes!("snes/input.rs")),
    ("sm/src/snes/ppu.rs", include_bytes!("snes/ppu.rs")),
    (
        "sm/src/snes/registers.rs",
        include_bytes!("snes/registers.rs"),
    ),
    ("sm/src/snes/snes.rs", include_bytes!("snes/snes.rs")),
    (
        "sm/src/snes/snes_other.rs",
        include_bytes!("snes/snes_other.rs"),
    ),
    ("sm/src/snes/spc.rs", include_bytes!("snes/spc.rs")),
    ("sm/src/snes/backend.c", include_bytes!("snes/backend.c")),
    ("sm/src/spc_player.rs", include_bytes!("spc_player.rs")),
    ("sm/src/native_music.c", include_bytes!("native_music.c")),
    ("sm/src/native_music.h", include_bytes!("native_music.h")),
    ("sm/src/tracing.rs", include_bytes!("tracing.rs")),
    ("sm/src/types.rs", include_bytes!("types.rs")),
    ("sm/src/util.rs", include_bytes!("util.rs")),
    ("sm/src/native_util.c", include_bytes!("native_util.c")),
    ("sm/src/native_util.h", include_bytes!("native_util.h")),
    ("sm/src/verify_probe.rs", include_bytes!("verify_probe.rs")),
    (
        "sm/src/verify_source.rs",
        include_bytes!("verify_source.rs"),
    ),
    (
        "sm/src/word_transfers.rs",
        include_bytes!("word_transfers.rs"),
    ),
];
