//! Eight focused player settlement checks; existing capture envelope.
//! Source-only. Root owns registration, build, fresh-host execution and capture.
use super::{Game, Snapshot, make_snapshot_with_player_owner, restore_snapshot};
use crate::{
    initial_state::InitialState, player_transitions_rust_observer as observer, rom::RomImage,
};

const IMAGE_LENGTH: usize = 0x100000;
const CAPACITY: usize = 2 * 1024 * 1024;
fn put(game: &mut Game, at: usize, value: u16) {
    game.ram().copy_in(at, &value.to_le_bytes());
}
fn offset(address: u32) -> usize {
    crate::rtl::lorom_offset(address) as usize
}
fn record(out: &mut Vec<u8>, tag: u8, bytes: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}
fn author(image: &RomImage, address: u32, bytes: &[u8], manifest: &mut Vec<u8>) {
    let at = offset(address);
    assert!(at + bytes.len() <= image.len());
    manifest.extend_from_slice(&address.to_le_bytes());
    manifest.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    manifest.extend_from_slice(bytes);
    for (index, value) in bytes.iter().copied().enumerate() {
        image.write(at + index, value);
    }
}

/// Caller supplies the same freshly initialized host required by existing captures.
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        if !matches!(id, 3900..=3907)
            || original.len() != 0x300000
            || game.g_cpu.is_null()
            || game.g_snes.is_null()
            || game.g_spc_player.is_null()
        {
            return Err("pose fixture requires admitted ID and initialized original host".into());
        }
        if !cfg!(target_endian = "little")
            || game.actor_identity_is_with_session()
            || !game.supports_legacy_persistence()
        {
            return Err("pose fixture ownership/endian/alignment prerequisite failed".into());
        }
        let loaded_before = game.loaded_rom.clone();
        let native_before = game.capture_native_rom_binding();
        let mut before = Box::new(Snapshot::INITIAL);
        {
            let snapshot_snes = game.g_snes;
            let snapshot_cpu = game.g_cpu;
            let snapshot_player = game.g_spc_player;
            let (backend, ram, _, _, _, owner) = game.ram_and_owned_snapshots();
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
        let cpu_before = *game.g_cpu;
        let cycles_before = (*game.g_snes).cpuCyclesLeft;
        let memory_ops_before = (*game.g_snes).cpuMemOps;
        let calling_before = game.g_calling_asm_from_c;
        let return_before = game.g_calling_asm_from_c_ret;
        let fail_before = game.g_fail;
        let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            game.ram().fill(0..game.ram().len(), 0);
            game.ram().write_byte(0x1ffff, 1);
            game.g_fail = false;
            let loaded = RomImage::zeroed(IMAGE_LENGTH);
            let native = RomImage::zeroed(IMAGE_LENGTH);
            let mut loaded_manifest = Vec::new();
            let mut native_manifest = Vec::new();
            use crate::ram_map::words as ram;
            // Minimal preserved beta seed: actual no-op movement identity,
            // actual palette tail and actual control restoration. No demo parser
            // or historical three-way preparation framework is installed.
            let mut poses = vec![0; 24];
            poses[8] = 8;
            poses[8 + 6] = 2;
            author(&loaded, 0x91b629, &poses, &mut loaded_manifest);
            author(&native, 0x91b629, &poses, &mut native_manifest);
            author(
                &loaded,
                0x91d727,
                &0x9000u16.to_ne_bytes(),
                &mut loaded_manifest,
            );
            let mut palette = Vec::new();
            for color in 0..16u16 {
                palette.extend_from_slice(&(0x2000 + color).to_le_bytes());
            }
            author(&native, 0x9b9000, &palette, &mut native_manifest);
            game.register_rom(loaded.clone());
            game.register_native_rom(native.clone());
            for (at, value) in [
                (ram::SAMUS_POSE, 1),
                (0xa1e, 8),
                (ram::SAMUS_PREV_POSE, 1),
                (ram::SAMUS_MOVEMENT_HANDLER, 0xe90e),
                (ram::FRAME_HANDLER_GAMMA, 0xe90e),
                (ram::FRAME_HANDLER_ALFA, 0xe8cd),
                (
                    ram::FRAME_HANDLER_BETA,
                    if id == 3906 {
                        0xe7f5
                    } else if id == 3907 {
                        0xe833
                    } else {
                        0xe725
                    },
                ),
                (ram::DEBUG_DISABLE_MINIMAP, 1),
                (ram::TIME_IS_FROZEN_FLAG, if id == 3903 { 1 } else { 0 }),
                (ram::SAMUS_ANIM_FRAME_TIMER, 2),
                (ram::SAMUS_ANIM_FRAME, 0),
                (ram::SAMUS_X_SPEED_DIVISOR, 0),
                (ram::TIMER_FOR_SHINE_TIMER, 10),
                (ram::SAMUS_HEALTH, 99),
                (ram::SAMUS_NEW_POSE, 0xffff),
                (ram::SAMUS_NEW_POSE_INTERRUPTED, 0xffff),
                (ram::SAMUS_NEW_POSE_TRANSITIONAL, 0xffff),
                (ram::SAMUS_Y_POS, 100),
                (ram::SAMUS_Y_RADIUS, 2),
                (ram::SAMUS_INVINCIBILITY_TIMER, 0),
                (ram::SAMUS_KNOCKBACK_TIMER, 0),
                (ram::JOYPAD1_LASTKEYS, 0x10),
                (ram::JOYPAD1_NEWKEYS, 0x20),
                (ram::JOYPAD1_INPUT_SAMUSFILTER, 0x40),
                (ram::JOYPAD1_NEWINPUT_SAMUSFILTER, 0x80),
                (ram::CONTROLLER1_INPUT_FOR_DEMO, 0x100),
                (ram::CONTROLLER1_NEW_INPUT_FOR_DEMO, 0x200),
                (ram::DEMO_BACKUP_PREV_CONTROLLER_INPUT, 0x400),
                (ram::DEMO_BACKUP_PREV_CONTROLLER_INPUT_NEW, 0x800),
                (ram::INTRO_SAMUS_DISPLAY, 0),
            ] {
                put(game, at, value);
            }
            put(
                game,
                ram::EQUIPPED_ITEMS,
                if id == 3901 {
                    1
                } else if id == 3902 {
                    0x21
                } else {
                    0
                },
            );
            put(
                game,
                ram::SAMUS_SUBUNIT_HEALTH,
                if id == 3905 { 0x10 } else { 0x2000 },
            );
            put(
                game,
                ram::SAMUS_PERIODIC_SUBDAMAGE,
                if id == 3905 { 0x20 } else { 1 },
            );
            put(
                game,
                ram::SAMUS_PERIODIC_DAMAGE,
                if id == 3904 {
                    0x8000
                } else if id == 3905 {
                    4
                } else {
                    0
                },
            );
            if id == 3905 {
                put(game, ram::SAMUS_HEALTH, 3);
            } else {
                put(game, ram::FX_TYPE, 4);
                put(game, ram::LAVA_ACID_Y_POS, 99);
                put(game, ram::GAME_TIME_FRAMES, 1);
                put(
                    game,
                    crate::ram_map::word_arrays::ATMOSPHERIC_GFX_FRAME_AND_TYPE,
                    0x400,
                );
            }
            let mut expected = (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>();
            fn set(bytes: &mut [u8], at: usize, value: u16) {
                bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            set(&mut expected, ram::SAMUS_ANIM_FRAME_TIMER, 1);
            set(&mut expected, 0xa72, 0x601);
            expected[0xc180..0xc1a0].copy_from_slice(&palette);
            if id != 3905 {
                set(&mut expected, ram::LIQUID_PHYSICS_TYPE, 2);
                set(
                    &mut expected,
                    ram::SAMUS_ANIM_FRAME_BUFFER,
                    if id == 3902 { 0 } else { 2 },
                );
            }
            let (whole, fraction) = match id {
                3900 => (97, 0x9fff),
                3901 => (98, 0x6000),
                3902 => (98, 0xc000),
                3905 => (0, 0),
                _ => (99, 0x2000),
            };
            set(&mut expected, ram::SAMUS_HEALTH, whole);
            set(&mut expected, ram::SAMUS_SUBUNIT_HEALTH, fraction);
            let (pending, subpending) = if id == 3904 {
                (0x8001, 0x8001)
            } else if id >= 3906 {
                (1, 0x8001)
            } else {
                (0, 0)
            };
            set(&mut expected, ram::SAMUS_PERIODIC_DAMAGE, pending);
            set(&mut expected, ram::SAMUS_PERIODIC_SUBDAMAGE, subpending);
            if id >= 3906 {
                for (at, value) in [
                    (ram::JOYPAD1_LASTKEYS, 0x100),
                    (ram::JOYPAD1_NEWKEYS, 0x200),
                    (ram::JOYPAD1_INPUT_SAMUSFILTER, 0x400),
                    (ram::JOYPAD1_NEWINPUT_SAMUSFILTER, 0x800),
                ] {
                    set(&mut expected, at, value);
                }
            }
            let mut output = b"PTRS1".to_vec();
            let mut identity = id.to_le_bytes().to_vec();
            identity.push(1); // Authored-data corrected-Rust comparison; not original-ROM oracle.
            record(&mut output, 0, &identity);
            // Existing authored-span tag8 is kept byte-for-byte structurally compatible.
            // Tag10 is the loaded span list; tag8 is native. Both use address/u32len/data.
            record(&mut output, 8, &native_manifest);
            record(&mut output, 10, &loaded_manifest);
            let mut allocation = (loaded.len() as u32).to_le_bytes().to_vec();
            allocation.extend_from_slice(&(native.len() as u32).to_le_bytes());
            record(&mut output, 12, &allocation);
            observer::begin(output, game, Vec::new());
            observer::observe(1, game);
            let invalid = invoke(id, game);
            if invalid != (id == 3904) {
                return Err("settlement wrong damage result".into());
            }
            if (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .ne(expected.iter().copied())
            {
                let at = (0..game.ram().len())
                    .map(|at| game.ram().read_byte(at))
                    .zip(&expected)
                    .position(|(a, b)| a != *b)
                    .unwrap();
                return Err(format!(
                    "settlement {id} RAM mismatch at {at:05x}: {:02x} != {:02x}",
                    game.ram().read_byte(at),
                    expected[at]
                ));
            }
            if game.g_fail || game.actor_identity_is_with_session() {
                return Err("settlement diagnostic or retained authority".into());
            }
            observer::scalar(3900, u16::from(invalid));
            observer::scalar(3903, 1);
            observer::observe(11, game);
            if game.actor_identity_is_with_session()
                || !game.capture_native_rom().same_allocation(&native)
                || !game
                    .loaded_rom
                    .as_ref()
                    .ok_or("lost loaded binding")?
                    .same_allocation(&loaded)
            {
                return Err("pose fixture retained owner or changed authored bindings".into());
            }
            let bytes = observer::finish().ok_or("missing pose trace")?;
            if bytes.len() > CAPACITY {
                return Err("pose trace exceeds existing capacity".into());
            }
            Ok(bytes)
        }));
        let _ = observer::finish();
        game.loaded_rom = loaded_before;
        game.restore_native_rom_binding(native_before);
        {
            let snapshot_snes = game.g_snes;
            let snapshot_cpu = game.g_cpu;
            let (backend, ram, _, _, _, _) = game.ram_and_owned_snapshots();
            restore_snapshot(
                &before,
                &mut *snapshot_snes,
                &mut *snapshot_cpu,
                ram,
                backend,
            )
        };
        before.audio.restore(game.g_snes);
        game.restore_native_player_snapshot(crate::game_state::PlayerSnapshotSource::Local(
            &before.player,
        ));
        *game.g_cpu = cpu_before;
        (*game.g_snes).cpuCyclesLeft = cycles_before;
        (*game.g_snes).cpuMemOps = memory_ops_before;
        queue_before.restore(game);
        game.g_calling_asm_from_c = calling_before;
        game.g_calling_asm_from_c_ret = return_before;
        game.g_fail = fail_before;
        result.unwrap_or_else(|_| Err("composed invocation unexpectedly panicked".into()))
    }
}

unsafe fn invoke(id: u32, game: &mut Game) -> bool {
    {
        if matches!(id, 3904 | 3905) {
            return direct_settlement(game);
        }
        crate::game::session_runtime::player_update_runtime::advance_frame(game);
        false
    }
}
fn direct_settlement(game: &mut Game) -> bool {
    let mut runtime = crate::game::session_runtime::Runtime::new(game);
    crate::game::player_transitions::finish_movement_and_palette(&mut runtime);
    let frozen = runtime.inputs().frozen;
    sm_game::samus_periodic_damage::apply(runtime.session_mut(), frozen).is_err()
}
