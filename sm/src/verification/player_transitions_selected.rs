//! Six selected existing PT27/PT30 seeds; root builds and executes.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::native_rom::NativeRomPointer;
use crate::{
    game_state::Game,
    player_transitions_rust_observer as trace,
    ram_map::{word_arrays as a, words as w},
    rom::RomImage,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

pub const IDS: &[u32] = &[2759, 2763, 3004, 3036, 3062, 3086];
fn expected(id: u32) -> [u16; 2] {
    match id {
        3004 => [6, 0xdfff],
        3036 => [2, 0x3fff],
        3062 => [2, 0],
        3086 => [0, 0x1000],
        _ => unreachable!(),
    }
}
const WRITABLE_WORDS: [usize; 5] = [
    w::samus_y_speed,
    w::samus_y_subspeed,
    w::grapple_walljump_timer,
    w::reached_ceres_elevator_fade_timer,
    w::samus_y_dir,
];
fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes({
        let mut bytes = [0; 2];
        g.ram().copy_out(at, &mut bytes);
        bytes
    })
}
fn seed(g: &mut Game) -> (RomImage, RomImage) {
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    let loaded = RomImage::zeroed(0x200000);
    let native = RomImage::zeroed(0x200000);
    for pose in [2usize] {
        loaded.write(
            crate::rom::lorom_offset(0x91b629) as usize + pose * 8 + 6,
            2,
        );
        native.write(
            crate::rom::lorom_offset(0x91b629) as usize + pose * 8 + 6,
            7,
        );
    }
    g.register_rom(loaded.clone());
    g.register_native_rom(native.clone());
    for (at, v) in [
        (w::samus_pose, 2),
        (w::samus_x_pos, 100),
        (w::samus_y_pos, 100),
        (w::samus_x_radius, 2),
        (w::samus_y_radius, 2),
        (w::samus_anim_frame_timer, 2),
        (w::fx_y_pos, 0x8000),
        (w::lava_acid_y_pos, 0x8000),
        (w::samus_health, 71),
        (w::speed_echoes_index, 0xffff),
        (w::room_width_in_blocks, 16),
        (w::room_height_in_blocks, 16),
        (w::samus_prev_pose, 0x81),
        (w::samus_y_speed, 5),
    ] {
        put(g, at, v);
    }
    g.ram().write_byte(0xa1e, 8);
    g.ram().write_byte(0xa1f, 0);
    g.ram().write_byte(0xa23, 3);
    for slot in 0..4 {
        put(
            g,
            a::atmospheric_gfx_anim_timer + slot * 2,
            0x1111 + slot as u16,
        );
        put(g, a::atmospheric_gfx_x_pos + slot * 2, 0x2222 + slot as u16);
        put(g, a::atmospheric_gfx_y_pos + slot * 2, 0x3333 + slot as u16);
        put(
            g,
            a::atmospheric_gfx_frame_and_type + slot * 2,
            0x6600 + slot as u16,
        );
    }
    (loaded, native)
}
fn setup_jump(sub: u32, g: &mut Game) {
    let environment = (sub / 8) % 3;
    let bits = sub % 8;
    let equipment = if bits & 1 != 0 { 0x20 } else { 0 }
        | if bits & 2 != 0 { 0x100 } else { 0 }
        | if bits & 4 != 0 { 0x2000 } else { 0 };
    put(g, w::equipped_items, equipment);
    if environment == 1 {
        put(g, w::fx_y_pos, 90);
    } else if environment == 2 {
        put(g, w::lava_acid_y_pos, 90);
    }
    for (at, v) in [
        (w::samus_x_extra_run_speed, 5),
        (w::samus_x_extra_run_subspeed, 0xffff),
        (w::grapple_walljump_timer, 0xa55a),
        (w::reached_ceres_elevator_fade_timer, 0x5aa5),
    ] {
        put(g, at, v);
    }
}
fn seed_request(id: u32, g: &mut Game) {
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    for (at, value) in [
        (0xa1c, 1),
        (0xa20, 2),
        (0xa1e, 4),
        (0xaf6, 0x140),
        (0xafa, 0x180),
        (0xb10, 0x100),
        (0xb14, 0x1c0),
        (0xa28, 0x3456),
        (0xa2a, 0x4567),
        (0xa2c, 0x5678),
        (0xa2e, 0x6789),
        (0xa30, 0x789a),
        (0xa32, 0x1234),
        (0x18a8, 0x12),
        (0xa94, 0xfffe),
        (0xa9c, 5),
        (0x9a2, 0x20),
        (0x79f, 6),
    ] {
        put(g, at, value);
    }
    if id == 2759 {
        put(g, 0xa56, 0x100);
        put(g, 0xa1e, 0x1c04);
    } else {
        put(g, 0xa52, 1);
        put(g, 0xa32, 3);
    }
}
fn check_request(id: u32, before: &[u8], g: &Game) {
    let mut expected = before.to_vec();
    if id == 2763 {
        expected[0xa32..0xa34].copy_from_slice(&8u16.to_le_bytes());
    }
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "selected request complete RAM outcome"
    );
    assert!(!g.g_fail);
    assert!(!g.actor_identity_is_with_session());
}
unsafe fn invoke(id: u32, g: &mut Game) {
    use crate::game::player_transitions::state::JumpKind;
    use crate::game::{player_transitions as core, session_runtime::Runtime};
    let mut host = Runtime::new(g);
    match id {
        2759 | 2763 => core::verify_hit_and_bomb(&mut host),
        3004 => core::initialize_jump(&mut host, JumpKind::Normal),
        3036 => core::initialize_jump(&mut host, JumpKind::Wall),
        3062 => core::initialize_jump(&mut host, JumpKind::Knockback),
        3086 => core::initialize_jump(&mut host, JumpKind::Bomb),
        _ => unreachable!(),
    }
}
fn check_outcome(id: u32, before: &[u8], g: &Game, loaded: &RomImage, native: &RomImage) {
    let expected = expected(id);
    assert_eq!(word(g, w::samus_y_speed), expected[0], "PT30 whole speed");
    assert_eq!(
        word(g, w::samus_y_subspeed),
        expected[1],
        "PT30 fractional speed"
    );
    assert_eq!(word(g, w::grapple_walljump_timer), 0, "PT30 grapple timer");
    assert_eq!(
        word(g, w::reached_ceres_elevator_fade_timer),
        0,
        "PT30 Ceres timer"
    );
    assert_eq!(word(g, w::samus_y_dir), 1, "PT30 upward direction");
    for (at, &old) in before.iter().take(g.ram().len()).enumerate() {
        let new = g.ram().read_byte(at);
        if !WRITABLE_WORDS
            .iter()
            .any(|&start| at == start || at == start + 1)
        {
            assert_eq!(old, new, "PT30 unexpected RAM write at {at:#x}");
        }
    }
    assert!(!g.g_fail, "PT30 unexpected diagnostic");
    assert!(!g.actor_identity_is_with_session(), "PT30 leaked owner");
    assert_eq!(
        g.loaded_rom
            .as_ref()
            .expect("PT30 loaded binding missing")
            .as_mut_ptr(),
        loaded.as_mut_ptr(),
        "PT30 loaded binding"
    );
    assert_eq!(
        g.capture_native_rom().as_mut_ptr(),
        native.as_mut_ptr(),
        "PT30 native binding"
    );
}
/// Root dispatcher admits exactly IDS; no other family paths are registered.
pub fn capture(id: u32, original: &[u8], g: &mut Game) -> Result<Vec<u8>, String> {
    if !IDS.contains(&id)
        || original.len() != 0x300000
        || g.g_snes.is_null()
        || g.g_cpu.is_null()
        || g.g_spc_player.is_null()
    {
        return Err("PT30 requires admitted ID and initialized host".into());
    }
    if !cfg!(target_endian = "little") || g.actor_identity_is_with_session() {
        return Err("PT30 natural RAM alignment/ownership prerequisite".into());
    }
    if g.loaded_rom.is_none() || trace::active() || !g.supports_legacy_persistence() {
        return Err("PT30 requires idle observer and canonical persistence".into());
    }
    let old_loaded = g.loaded_rom.clone();
    let old_native = g.capture_native_rom_binding();
    let old_ram = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let old_fail = g.g_fail;
    let result = catch_unwind(AssertUnwindSafe(|| {
        let request = id < 3000;
        let (loaded, native) = if request {
            seed_request(id, g);
            (
                g.loaded_rom.as_ref().expect("loaded host").clone(),
                g.capture_native_rom(),
            )
        } else {
            let images = seed(g);
            setup_jump(id - 3000, g);
            images
        };
        let before = (0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>();
        let mut out = b"PTRS1".to_vec();
        let mut identity = id.to_le_bytes().to_vec();
        identity.push(u8::from(!request));
        out.push(0);
        out.extend_from_slice(&5u32.to_le_bytes());
        out.extend_from_slice(&identity);
        if !request {
            out.push(9);
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        trace::begin(out, g, Vec::new());
        trace::observe(1, g);
        unsafe {
            invoke(id, g);
        }
        if request {
            check_request(id, &before, g);
        } else {
            check_outcome(id, &before, g, &loaded, &native);
            trace::scalar(30000, 1);
        }
        assert!(!g.actor_identity_is_with_session());
        assert!(g.loaded_rom.as_ref().unwrap().same_allocation(&loaded));
        assert!(g.capture_native_rom().same_allocation(&native));
        trace::observe(11, g);
        let out = trace::finish().ok_or_else(|| "PT30 missing capture".to_owned())?;
        assert_eq!(
            out.len(),
            if request { 262175 } else { 262189 },
            "selected finite completion length"
        );
        Ok(out)
    }));
    let _ = trace::finish();
    g.loaded_rom = old_loaded;
    g.restore_native_rom_binding(old_native);
    g.ram().copy_in(0, &old_ram);
    g.g_fail = old_fail;
    result.unwrap_or_else(|_| Err("PT30 invocation/outcome failed (not a fidelity result)".into()))
}
