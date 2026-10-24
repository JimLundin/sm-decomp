//! Synthetic predecessor fixtures. No copied gameplay algorithms or ROM-derived data.
use crate::ram_map::{word_arrays as a, words as w};
use crate::{
    game_state::Game,
    rom::RomImage,
    types::{EnemyData as E, EnemyDef as D, EnemySpawnData as S},
};
use core::mem::offset_of;
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) const CASES: u32 = 35;
pub(crate) fn offset(address: u32) -> usize {
    crate::rtl::lorom_offset(address) as usize
}
pub(crate) fn put(g: &mut Game, at: usize, v: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&v.to_ne_bytes());
}
pub(crate) fn get(g: &Game, at: usize) -> u16 {
    u16::from_ne_bytes(g.g_ram[at..at + 2].try_into().unwrap())
}
fn ew(g: &mut Game, k: u16, field: usize, v: u16) {
    put(g, 0xf78 + usize::from(k) + field, v);
}
fn rw(r: &RomImage, address: u32, v: u16) {
    for (i, b) in v.to_ne_bytes().into_iter().enumerate() {
        r.write(offset(address) + i, b);
    }
}
fn def(r: &RomImage, address: u32, parts: u16) {
    for (field, v) in [
        (offset_of!(D, x_radius), 13),
        (offset_of!(D, y_radius), 17),
        (offset_of!(D, health), 1234),
        (offset_of!(D, bank), 0x07a2),
        (offset_of!(D, ai_init), 0x804c),
        (offset_of!(D, num_parts), parts),
        (offset_of!(D, main_ai), 0x804c),
        (offset_of!(D, tile_data_size), 0x60),
        (offset_of!(D, name_ptr), 0x9000),
    ] {
        rw(r, address + field as u32, v);
    }
    r.write(offset(address) + offset_of!(D, layer), 3);
}
fn population(r: &RomImage, address: u32, definition: u16, x: u16) {
    for (i, v) in [definition, x, 0xfff0, 0x9000, 0x2400, 0, 0x6789, 0xabcd]
        .into_iter()
        .enumerate()
    {
        rw(r, address + 2 * i as u32, v);
    }
}
pub(crate) fn snapshot(out: &mut Vec<u8>, g: &Game) {
    out.extend_from_slice(&g.g_ram);
    for v in [
        g.eproj_spawn_rect.x,
        g.eproj_spawn_rect.y,
        g.eproj_spawn_rect.w,
        g.eproj_spawn_rect.h,
        g.eproj_spawn_r22,
        g.eproj_spawn_pt.x,
        g.eproj_spawn_pt.y,
        g.eproj_spawn_varE24,
    ] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.push(u8::from(g.g_fail));
}
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Install,
    PrepareInstall,
    Spawn,
    CallerSpawn,
    Record,
    Respawn,
    Animation,
    CallerAnimation,
}
pub(crate) fn run(op: Operation, g: &mut Game) -> Option<u16> {
    unsafe {
        match op {
            Operation::Install => crate::game::actors::reference_population_animation::transfer(g),
            Operation::PrepareInstall => {
                crate::game::actors::reference_population_animation::prepare(g);
                // Content changes after actual preparation, before actual installation.
                let image = g.loaded_rom.as_ref().unwrap();
                population(image, 0xa18000, 0x9100, 0x2468);
                rw(image, 0xa18010, 0xffff);
                image.write(offset(0xa18012), 9);
                put(g, w::enemy_tile_vram_src, 0xfffe);
                crate::game::actors::reference_population_animation::transfer(g);
            }
            Operation::Spawn => {
                return Some(crate::game::actors::reference_population_animation::spawn(
                    0xa1, 0x8000, g,
                ));
            }
            Operation::CallerSpawn => crate::game::sm_a6::CeresRidley_Func_25(g),
            Operation::Record => crate::game::actors::reference_population_animation::record(64, g),
            Operation::Respawn => {
                crate::game::actors::reference_population_animation::respawn(64, g)
            }
            Operation::Animation => {
                crate::game::actors::reference_population_animation::animation(g)
            }
            Operation::CallerAnimation => {
                crate::game::actors::reference_population_animation::advance(g)
            }
        }
    }
    None
}
fn script(r: &RomImage, address: u32, values: &[u16]) {
    for (i, v) in values.iter().enumerate() {
        rw(r, address + 2 * i as u32, *v);
    }
}
fn fixture(id: u32, g: &mut Game, loaded: &RomImage, native: &RomImage) -> Operation {
    // All bytes initialized; sentinel pattern exposes unintended unrelated writes.
    for (i, b) in g.g_ram.iter_mut().enumerate() {
        *b = (i.wrapping_mul(37) ^ (i >> 8)) as u8;
    }
    g.g_fail = false;
    // Native allocations remain full, distinct and lifetime-owned. No short raw pointers.
    for r in [loaded, native] {
        def(r, 0xa09000, 1);
        def(r, 0xa09100, 1);
        def(r, 0xa01000, 1);
        for (i, v) in [0x4142, 0x4344, 0x4546, 0x4748, 0x494a, 0xdead, 0x4b4c]
            .into_iter()
            .enumerate()
        {
            rw(r, 0xb49000 + i as u32 * 2, v);
        }
        for i in 0..5 {
            population(
                r,
                0xa18000 + i * 16,
                if i & 1 == 0 { 0x9000 } else { 0x9100 },
                0x1234 + i as u16,
            );
        }
        rw(r, 0xa18020, 0xffff);
        r.write(offset(0xa18022), 7);
        rw(r, 0xb48000, 0x9100);
        rw(r, 0xb48002, 3);
        rw(r, 0xb48004, 0x9000);
        rw(r, 0xb48006, 5);
        rw(r, 0xb48008, 0xffff);
        population(r, 0xa6aa2f, 0x9000, 0x3456);
        population(r, 0xa6aa3f, 0x9100, 0x4567);
    }
    // Distinguish allocations without changing harmless native callback identity.
    rw(native, 0xa09000 + offset_of!(D, health) as u32, 4321);
    put(g, w::room_enemy_population_ptr, 0x8000);
    put(g, w::room_enemy_tilesets_ptr, 0x8000);
    put(g, w::cur_enemy_index, 0x140);
    put(g, w::enemy_tile_vram_src, 0xfffe);
    for i in 0..32 {
        ew(g, i * 64, offset_of!(E, enemy_ptr), 0);
    }
    for (i, v) in [0x9000, 0x9100, 0, 0].into_iter().enumerate() {
        put(g, a::enemy_def_ptr + 2 * i, v);
        put(g, a::enemy_gfxdata_tiles_index + 2 * i, 0x20 + i as u16);
        put(g, a::enemy_gfxdata_vram_ptr + 2 * i, 0x300 + i as u16);
    }
    match id {
        0 => {
            rw(loaded, 0xa18000, 0xffff);
            Operation::Install
        }
        1 => Operation::Install,
        2 => {
            rw(loaded, 0xb48000, 0xffff);
            put(g, w::enemy_tile_load_data_write_pos, 0);
            Operation::PrepareInstall
        }
        3 => {
            rw(loaded, 0xb48000, 0xffff);
            Operation::Install
        }
        4..=7 => {
            rw(
                loaded,
                0xa09000 + offset_of!(D, num_parts) as u32,
                (id - 4) as u16,
            );
            Operation::Spawn
        }
        8 => {
            rw(loaded, 0xa09000 + offset_of!(D, num_parts) as u32, 3);
            for k in [64, 192, 320] {
                ew(g, k, 0, 0xbeef);
            }
            Operation::Spawn
        }
        9 => {
            for k in 0..32 {
                ew(g, k * 64, 0, 0xbeef);
            }
            Operation::Spawn
        }
        10 => {
            for k in 0..31 {
                ew(g, k * 64, 0, 0xbeef);
            }
            Operation::Spawn
        }
        11 => {
            population(loaded, 0xa18000, 0x1000, 0x1234);
            Operation::Spawn
        }
        12 => Operation::CallerSpawn,
        13 => Operation::Spawn,
        14 | 15 => {
            ew(g, 64, 0, 0x9000);
            ew(g, 64, offset_of!(E, x_pos), 0x1357);
            ew(g, 64, offset_of!(E, y_pos), 0x2468);
            if id == 15 {
                rw(loaded, 0xa09000 + offset_of!(D, name_ptr) as u32, 0);
            }
            Operation::Record
        }
        16..=21 => {
            // Slot64 uses population base + (64>>2). Saved mutable fields are intentionally distinct.
            population(
                loaded,
                0xa18010,
                if matches!(id, 17 | 18 | 20) {
                    0x1000
                } else {
                    0x9000
                },
                0x3579,
            );
            put(g, 0x7040 + offset_of!(S, palette_index), 0x0a00);
            put(g, 0x7040 + offset_of!(S, vram_tiles_index), 0x345);
            ew(g, 64, offset_of!(E, x_subpos), 0xabcd);
            ew(g, 64, offset_of!(E, y_subpos), 0x9876);
            ew(g, 64, offset_of!(E, ai_handler_bits), 0x1234);
            ew(g, 64, offset_of!(E, shake_timer), 0x5678);
            ew(g, 64, offset_of!(E, spritemap_pointer), 0x9abc);
            if id == 19 {
                put(g, 0x7040 + offset_of!(S, palette_index), 0x600);
                put(g, 0x7040 + offset_of!(S, vram_tiles_index), 0x77);
            }
            if id == 21 {
                put(g, 0x7040 + offset_of!(S, x_pos), 0xbeef);
                put(g, 0x7040 + offset_of!(S, cause_of_death), 0x1234);
            }
            Operation::Respawn
        }
        22..=34 => {
            put(g, w::cur_enemy_index, 0);
            ew(g, 0, 0, 0x9000);
            ew(g, 0, offset_of!(E, bank), 0x00a2);
            ew(g, 0, offset_of!(E, ai_handler_bits), 0);
            ew(g, 0, offset_of!(E, instruction_timer), 1);
            ew(g, 0, offset_of!(E, current_instruction), 0x9000);
            ew(g, 0, offset_of!(E, extra_properties), 0x8120);
            script(loaded, 0xa29000, &[5, 0x9abc]);
            match id {
                22 => ew(g, 0, offset_of!(E, ai_handler_bits), 4),
                23 => ew(g, 0, offset_of!(E, instruction_timer), 0),
                24 => ew(g, 0, offset_of!(E, instruction_timer), 2),
                26 => script(loaded, 0xa29000, &[0x80ed, 0]),
                27 => {
                    ew(g, 0, offset_of!(E, bank), 0xa5);
                    script(loaded, 0xa59000, &[0x812f]);
                }
                28 => {
                    ew(g, 0, offset_of!(E, bank), 0xa5);
                    script(loaded, 0xa59000, &[0x813a, 9]);
                }
                29 | 31 => {
                    ew(g, 0, offset_of!(E, bank), 0xaa);
                    put(g, w::vram_write_queue_tail, 3);
                    // AA814B + seven packed operand bytes, then ordinary frame header.
                    let bytes = [
                        0x4b, 0x81, 0x34, 0x12, 0x78, 0x56, 0xa9, 0xbc, 0x9a, 7, 0, 0x68, 0x24,
                    ];
                    for (i, b) in bytes.into_iter().enumerate() {
                        loaded.write(offset(0xaa9000) + i, b);
                    }
                }
                32 => {
                    script(loaded, 0xa29000, &[0x8173, 0x8123, 0x2468, 7, 0x1357]);
                    ew(g, 64, offset_of!(E, bank), 0xa2);
                    ew(g, 64, offset_of!(E, timer), 0x7777);
                }
                33 => {
                    put(g, w::first_free_enemy_index, 64);
                    put(g, w::enemy_index_to_shake, 0xffff);
                    put(g, a::active_enemy_indexes, 0);
                    put(g, a::active_enemy_indexes + 2, 0xffff);
                    put(g, w::debug_time_frozen_for_enemies, 0);
                    put(g, w::time_is_frozen_flag, 0);
                    ew(g, 0, offset_of!(E, properties), 0x2500);
                    ew(g, 0, offset_of!(E, extra_properties), 0);
                    ew(g, 0, offset_of!(E, flash_timer), 0);
                    ew(g, 0, offset_of!(E, frozen_timer), 0);
                    // Existing sprite owner no-op update: disable every sprite instruction.
                    crate::game::sprite_effect_runtime::clear_sprites(g);
                }
                34 => {
                    ew(g, 0, offset_of!(E, bank), 0x80);
                    ew(g, 0, offset_of!(E, current_instruction), 0x8000);
                    script(loaded, 0x808000, &[3, 0x4321]);
                }
                _ => {}
            }
            if id == 33 {
                Operation::CallerAnimation
            } else {
                Operation::Animation
            }
        }
        _ => unreachable!(),
    }
}
pub(crate) fn capture_case(
    id: u32,
    run: fn(Operation, &mut Game) -> Option<u16>,
) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let native = RomImage::zeroed(0x200000);
        let loaded = RomImage::zeroed(0x200000);
        let mut g = Game::new();
        g.register_native_rom(native.clone());
        g.loaded_rom = Some(loaded.clone());
        let op = fixture(id, &mut g, &loaded, &native);
        if matches!(id, 30 | 31) {
            let len = if id == 30 {
                offset(0xa29000) + 2
            } else {
                offset(0xaa9000) + 7
            };
            let short = RomImage::zeroed(len);
            for i in 0..len {
                short.write(i, loaded.read(i));
            }
            g.loaded_rom = Some(short);
        }
        let mut out = b"APAN1".to_vec();
        out.extend_from_slice(&id.to_le_bytes());
        snapshot(&mut out, &g);
        let before = g.g_ram.to_vec();
        crate::actor_population_animation_observer::begin(id);
        let actual = catch_unwind(AssertUnwindSafe(|| run(op, &mut g)));
        let trace = crate::actor_population_animation_observer::finish();
        let expected_panic = matches!(id, 11 | 17 | 18 | 20 | 30 | 31);
        match actual {
            Ok(value) => {
                assert!(!expected_panic, "expected directed failure");
                out.push(0);
                out.push(u8::from(value.is_some()));
                out.extend_from_slice(&u64::from(value.unwrap_or(0)).to_le_bytes());
                if (4..=10).contains(&id) {
                    let expected = match id {
                        7 => 64,
                        8 => 320,
                        9 => 0xffff,
                        10 => 1984,
                        _ => 0,
                    };
                    assert_eq!(value, Some(expected), "multipart return/one-fewer stop");
                }
            }
            Err(payload) => {
                assert!(expected_panic, "unexpected predecessor panic");
                let message = payload
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .expect("string failure");
                let expected = match id {
                    _ => {
                        let len = match id {
                            11 | 17 => offset(0xa01000) + offset_of!(D, x_radius),
                            18 => offset(0xa01000) + offset_of!(D, y_radius),
                            20 => offset(0xa01000) + offset_of!(D, layer),
                            30 => offset(0xa29000) + 2,
                            31 => offset(0xaa9000) + 7,
                            _ => unreachable!(),
                        };
                        format!("index out of bounds: the len is {len} but the index is {len}")
                    }
                };
                assert_eq!(message, expected, "exact failure source");
                out.push(1);
                out.extend_from_slice(&(message.len() as u32).to_le_bytes());
                out.extend_from_slice(message.as_bytes());
            }
        }
        if (16..=21).contains(&id) {
            for field in [
                offset_of!(E, x_subpos),
                offset_of!(E, y_subpos),
                offset_of!(E, ai_handler_bits),
                offset_of!(E, shake_timer),
                offset_of!(E, spritemap_pointer),
            ] {
                let at = 0xfb8 + field;
                assert_eq!(
                    &g.g_ram[at..at + 2],
                    &before[at..at + 2],
                    "preserved respawn field"
                );
            }
        }
        if matches!(id, 14 | 15) {
            let at = 0x7040 + offset_of!(S, name) + 12;
            assert_eq!(
                &g.g_ram[at..at + 6],
                &before[at..at + 6],
                "saved name tail preserved"
            );
        }
        if id == 11 {
            assert_eq!(get(&g, 0xf78), 0, "no definition before configure succeeds");
        }
        if id == 32 {
            assert_eq!(
                get(&g, 0xfb8 + offset_of!(E, timer)),
                0x2468,
                "live selector for second opcode"
            );
            assert_eq!(
                get(&g, 0xf78 + offset_of!(E, spritemap_pointer)),
                0x1357,
                "captured image and actor for frame"
            );
        }
        assert_eq!(
            g.g_fail,
            matches!(id, 11 | 17 | 18 | 20),
            "exact low-window diagnostic status"
        );
        snapshot(&mut out, &g);
        out.extend_from_slice(&(trace.len() as u32).to_le_bytes());
        out.extend(trace);
        out
    }));
    crate::actor_population_animation_observer::clear();
    match result {
        Ok(v) => Some(v),
        Err(_) => {
            eprintln!("actor population/animation case {id} assertion failed");
            None
        }
    }
}
