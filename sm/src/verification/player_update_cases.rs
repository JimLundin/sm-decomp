//! Six actual beta callers through the existing initialized transition carrier.
//! Paired legacy paths suppress minimap; enabled minimap has separate byte contracts.
use super::{Game, Snapshot, make_snapshot_with_player_owner, restore_snapshot};
use crate::ram_map::words as w;
use crate::{
    initial_state::InitialState, player_transitions_rust_observer as observer, rom::RomImage,
};
fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
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
    g.ram().fill(0..g.ram().len(), 0);
    g.ram().write_byte(0x1ffff, 1);
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
        (w::SAMUS_POSE, if id == 4105 { 0x29 } else { 1 }),
        (0xa1e, 0x0908),
        (w::SAMUS_PREV_POSE, 1),
        (w::SAMUS_MOVEMENT_HANDLER, 0xa337),
        (
            w::FRAME_HANDLER_GAMMA,
            if id == 4104 {
                0xe37e
            } else if id == 4105 {
                0xe41b
            } else {
                0xe90e
            },
        ),
        (w::FRAME_HANDLER_ALFA, 0xe8cd),
        (w::FRAME_HANDLER_BETA, beta),
        (w::DEBUG_DISABLE_MINIMAP, 1),
        (w::SAMUS_CONTACT_DAMAGE_INDEX, 7),
        (w::SAMUS_ANIM_FRAME_TIMER, 2),
        (w::SAMUS_ANIM_FRAME, 0),
        (w::SPEED_ECHOES_INDEX, 0xffff),
        (w::TIMER_FOR_SHINE_TIMER, 10),
        (w::SAMUS_HEALTH, 99),
        (w::SAMUS_SUBUNIT_HEALTH, 0x2000),
        (w::SAMUS_PERIODIC_DAMAGE, 1),
        (w::SAMUS_PERIODIC_SUBDAMAGE, 0x8001),
        (w::SAMUS_NEW_POSE, 0xffff),
        (w::SAMUS_NEW_POSE_INTERRUPTED, 0xffff),
        (w::SAMUS_NEW_POSE_TRANSITIONAL, 0xffff),
        (w::SAMUS_Y_POS, 100),
        (w::SAMUS_Y_RADIUS, 2),
        (w::JOYPAD1_LASTKEYS, 0x10),
        (w::JOYPAD1_NEWKEYS, 0x20),
        (w::JOYPAD1_INPUT_SAMUSFILTER, 0x40),
        (w::JOYPAD1_NEWINPUT_SAMUSFILTER, 0x80),
        (w::CONTROLLER1_INPUT_FOR_DEMO, 0x100),
        (w::CONTROLLER1_NEW_INPUT_FOR_DEMO, 0x200),
        (w::DEMO_BACKUP_PREV_CONTROLLER_INPUT, 0x400),
        (w::DEMO_BACKUP_PREV_CONTROLLER_INPUT_NEW, 0x800),
    ] {
        put(g, at, value);
    }
    if id == 4105 {
        put(g, w::SAMUS_Y_SPEED, 4);
        put(g, w::SAMUS_Y_SUBSPEED, 0xf800);
        put(g, w::SAMUS_Y_SUBACCEL, 0x1000);
    }
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    fn set(bytes: &mut [u8], at: usize, value: u16) {
        bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    set(&mut expected, w::SAMUS_CONTACT_DAMAGE_INDEX, 0);
    set(
        &mut expected,
        w::SAMUS_ANIM_FRAME_TIMER,
        if id == 4105 { 15 } else { 1 },
    );
    if id == 4105 {
        set(&mut expected, w::SAMUS_Y_SPEED, 5);
        set(&mut expected, w::SAMUS_Y_SUBSPEED, 0x800);
        set(&mut expected, w::SAMUS_ANIM_FRAME, 4);
    }
    if id != 4103 {
        set(&mut expected, 0xa72, 0x601);
        expected[0xc180..0xc1a0].copy_from_slice(&palette);
    }
    if matches!(id, 4100 | 4104 | 4105) {
        set(&mut expected, w::SAMUS_HEALTH, 97);
        set(&mut expected, w::SAMUS_SUBUNIT_HEALTH, 0x9fff);
        set(&mut expected, w::SAMUS_PERIODIC_DAMAGE, 0);
        set(&mut expected, w::SAMUS_PERIODIC_SUBDAMAGE, 0);
    }
    if matches!(id, 4101 | 4102) {
        for (at, value) in [
            (w::JOYPAD1_LASTKEYS, 0x100),
            (w::JOYPAD1_NEWKEYS, 0x200),
            (w::JOYPAD1_INPUT_SAMUSFILTER, 0x400),
            (w::JOYPAD1_NEWINPUT_SAMUSFILTER, 0x800),
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
            || g.actor_identity_is_with_session()
            || observer::active()
        {
            return Err("player update ownership/endian/observer prerequisite".into());
        }
        let loaded_before = g.loaded_rom.clone();
        let native_before = g.capture_native_rom_binding();
        let fail_before = g.g_fail;
        let mut before = Box::new(Snapshot::INITIAL);
        {
            let snapshot_snes = g.g_snes;
            let snapshot_cpu = g.g_cpu;
            let snapshot_player = g.g_spc_player;
            let (backend, ram, _, _, _, owner) = g.ram_and_owned_snapshots();
            make_snapshot_with_player_owner(
                &mut before,
                &*snapshot_snes,
                &*snapshot_cpu,
                snapshot_player,
                owner,
                ram,
                backend,
            )
        };
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
            if (0..g.ram().len())
                .map(|at| g.ram().read_byte(at))
                .ne(expected.iter().copied())
            {
                let at = (0..g.ram().len())
                    .map(|at| g.ram().read_byte(at))
                    .zip(&expected)
                    .position(|(a, b)| a != *b)
                    .unwrap();
                return Err(format!(
                    "player update {id} RAM mismatch at {at:05x}: {:02x} != {:02x}",
                    g.ram().read_byte(at),
                    expected[at]
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
        {
            let snapshot_snes = g.g_snes;
            let snapshot_cpu = g.g_cpu;
            let (backend, ram, _, _, _, _) = g.ram_and_owned_snapshots();
            restore_snapshot(
                &before,
                &mut *snapshot_snes,
                &mut *snapshot_cpu,
                ram,
                backend,
            )
        };
        before.audio.restore(g.g_snes);
        g.restore_native_player_snapshot(crate::game_state::PlayerSnapshotSource::Local(
            &before.player,
        ));
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
