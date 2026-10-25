//! Six actual beta callers through the existing initialized transition carrier.
//! Paired legacy paths suppress minimap; enabled minimap has separate byte contracts.
use super::{Game, MakeSnapshot, RestoreSnapshot, Snapshot};
use crate::ram_map::words as w;
use crate::{
    initial_state::InitialState, player_transitions_rust_observer as observer, rom::RomImage,
};
fn put(g: &mut Game, at: usize, value: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn record(out: &mut Vec<u8>, tag: u8, data: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
}
fn author(r: &RomImage, address: u32, data: &[u8], manifest: &mut Vec<u8>) {
    let at = crate::rtl::lorom_offset(address) as usize;
    assert!(at + data.len() <= r.len());
    manifest.extend_from_slice(&address.to_le_bytes());
    manifest.extend_from_slice(&(data.len() as u32).to_le_bytes());
    manifest.extend_from_slice(data);
    for (i, b) in data.iter().copied().enumerate() {
        r.write(at + i, b);
    }
}
fn seed(id: u32, g: &mut Game) -> (RomImage, RomImage, Vec<u8>, Vec<u8>, Vec<u8>) {
    g.g_ram.fill(0);
    g.g_ram[0x1ffff] = 1;
    g.g_fail = false;
    let loaded = RomImage::zeroed(0x100000);
    let native = RomImage::zeroed(0x100000);
    let mut lm = Vec::new();
    let mut nm = Vec::new();
    let mut poses = vec![0; 256 * 8];
    for pose in poses.chunks_exact_mut(8) {
        pose[0] = 8;
        pose[6] = 2;
    }
    author(&loaded, 0x91b629, &poses, &mut lm);
    author(&native, 0x91b629, &poses, &mut nm);
    author(&loaded, 0x91d727, &0x9000u16.to_le_bytes(), &mut lm);
    let mut palette = Vec::new();
    for color in 0..16u16 {
        palette.extend_from_slice(&(0x2000 + color).to_le_bytes());
    }
    author(&native, 0x9b9000, &palette, &mut nm);
    g.register_rom(loaded.clone());
    g.register_native_rom(native.clone());
    let beta = match id {
        4101 => 0xe7f5,
        4102 => 0xe833,
        4103 => 0xe8ec,
        _ => 0xe725,
    };
    for (at, value) in [
        (w::samus_pose, if id == 4105 { 0x29 } else { 1 }),
        (0xa1e, 0x0908),
        (w::samus_prev_pose, 1),
        (w::samus_movement_handler, 0xa337),
        (
            w::frame_handler_gamma,
            if id == 4104 {
                0xe37e
            } else if id == 4105 {
                0xe41b
            } else {
                0xe90e
            },
        ),
        (w::frame_handler_alfa, 0xe8cd),
        (w::frame_handler_beta, beta),
        (w::debug_disable_minimap, 1),
        (w::samus_contact_damage_index, 7),
        (w::samus_anim_frame_timer, 2),
        (w::samus_anim_frame, 0),
        (w::speed_echoes_index, 0xffff),
        (w::timer_for_shine_timer, 10),
        (w::samus_health, 99),
        (w::samus_subunit_health, 0x2000),
        (w::samus_periodic_damage, 1),
        (w::samus_periodic_subdamage, 0x8001),
        (w::samus_new_pose, 0xffff),
        (w::samus_new_pose_interrupted, 0xffff),
        (w::samus_new_pose_transitional, 0xffff),
        (w::samus_y_pos, 100),
        (w::samus_y_radius, 2),
        (w::joypad1_lastkeys, 0x10),
        (w::joypad1_newkeys, 0x20),
        (w::joypad1_input_samusfilter, 0x40),
        (w::joypad1_newinput_samusfilter, 0x80),
        (w::controller1_input_for_demo, 0x100),
        (w::controller1_new_input_for_demo, 0x200),
        (w::demo_backup_prev_controller_input, 0x400),
        (w::demo_backup_prev_controller_input_new, 0x800),
    ] {
        put(g, at, value);
    }
    if id == 4105 {
        put(g, w::samus_y_speed, 4);
        put(g, w::samus_y_subspeed, 0xf800);
        put(g, w::samus_y_subaccel, 0x1000);
    }
    let mut expected = g.g_ram.to_vec();
    fn set(bytes: &mut [u8], at: usize, value: u16) {
        bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    set(&mut expected, w::samus_contact_damage_index, 0);
    set(
        &mut expected,
        w::samus_anim_frame_timer,
        if id == 4105 { 15 } else { 1 },
    );
    if id == 4105 {
        set(&mut expected, w::samus_y_speed, 5);
        set(&mut expected, w::samus_y_subspeed, 0x800);
        set(&mut expected, w::samus_anim_frame, 4);
    }
    if id != 4103 {
        set(&mut expected, 0xa72, 0x601);
        expected[0xc180..0xc1a0].copy_from_slice(&palette);
    }
    if matches!(id, 4100 | 4104 | 4105) {
        set(&mut expected, w::samus_health, 97);
        set(&mut expected, w::samus_subunit_health, 0x9fff);
        set(&mut expected, w::samus_periodic_damage, 0);
        set(&mut expected, w::samus_periodic_subdamage, 0);
    }
    if matches!(id, 4101 | 4102) {
        for (at, value) in [
            (w::joypad1_lastkeys, 0x100),
            (w::joypad1_newkeys, 0x200),
            (w::joypad1_input_samusfilter, 0x400),
            (w::joypad1_newinput_samusfilter, 0x800),
        ] {
            set(&mut expected, at, value);
        }
    }
    (loaded, native, lm, nm, expected)
}
pub unsafe fn capture(id: u32, original: &[u8], g: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        if !(4100..=4105).contains(&id)
            || original.len() != 0x300000
            || g.g_cpu.is_null()
            || g.g_snes.is_null()
            || g.g_spc_player.is_null()
        {
            return Err("player update requires admitted ID and initialized host".into());
        }
        if !cfg!(target_endian = "little")
            || g.g_ram.as_ptr() as usize % core::mem::align_of::<u16>() != 0
            || g.actor_identity_is_with_session()
            || observer::active()
        {
            return Err("player update ownership/endian/observer prerequisite".into());
        }
        let loaded_before = g.loaded_rom.clone();
        let native_before = g.capture_native_rom_binding();
        let fail_before = g.g_fail;
        let mut before = Box::new(Snapshot::INITIAL);
        MakeSnapshot(&mut before, &*g.g_snes, &*g.g_cpu, g.g_spc_player);
        let cpu_before = *g.g_cpu;
        let cycles_before = (*g.g_snes).cpuCyclesLeft;
        let memory_ops_before = (*g.g_snes).cpuMemOps;
        let calling_before = g.g_calling_asm_from_c;
        let return_before = g.g_calling_asm_from_c_ret;
        let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(g);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (loaded, native, lm, nm, expected) = seed(id, g);
            let mut out = b"PTRS1".to_vec();
            let mut identity = id.to_le_bytes().to_vec();
            identity.push(1);
            record(&mut out, 0, &identity);
            record(&mut out, 8, &nm);
            record(&mut out, 10, &lm);
            record(&mut out, 12, &[0, 0, 0x10, 0, 0, 0, 0x10, 0]);
            observer::begin(out, g, Vec::new());
            observer::observe(1, g);
            let normal_transfers =
                crate::game::session_runtime::player_normal_transfers::begin(id, 1);
            crate::game::session_runtime::player_update_runtime::advance_frame(g);
            normal_transfers.finish();
            if g.g_ram.as_slice() != expected.as_slice() {
                let at = g
                    .g_ram
                    .iter()
                    .zip(&expected)
                    .position(|(a, b)| a != b)
                    .unwrap();
                return Err(format!(
                    "player update {id} RAM mismatch at {at:05x}: {:02x} != {:02x}",
                    g.g_ram[at], expected[at]
                ));
            }
            assert!(
                !g.g_fail && !g.actor_identity_is_with_session(),
                "actual beta returns authority"
            );
            observer::scalar(4100, 1);
            observer::observe(11, g);
            assert!(
                g.capture_native_rom().same_allocation(&native)
                    && g.loaded_rom
                        .as_ref()
                        .expect("Loaded binding")
                        .same_allocation(&loaded)
            );
            let bytes = observer::finish().ok_or("missing player update trace")?;
            if bytes.len() > 2 * 1024 * 1024 {
                return Err("player update trace exceeds existing capacity".into());
            }
            Ok(bytes)
        }));
        let _ = observer::finish();
        g.loaded_rom = loaded_before;
        g.restore_native_rom_binding(native_before);
        RestoreSnapshot(&before, &mut *g.g_snes, &mut *g.g_cpu);
        before.audio.restore(g.g_snes);
        before.player.restore(g.g_spc_player);
        *g.g_cpu = cpu_before;
        (*g.g_snes).cpuCyclesLeft = cycles_before;
        (*g.g_snes).cpuMemOps = memory_ops_before;
        queue_before.restore(g);
        g.g_calling_asm_from_c = calling_before;
        g.g_calling_asm_from_c_ret = return_before;
        g.g_fail = fail_before;
        result.unwrap_or_else(|_| Err("player update invocation unexpectedly panicked".into()))
    }
}
