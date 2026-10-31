//! Eleven eligible corrected legacy Rust calls, compared with frozen CPU output.
use super::{Game, Snapshot, make_snapshot_with_player_owner, restore_snapshot};
use crate::native_rom::NativeRomPointer;
use crate::{initial_state::InitialState, player_transitions_rust_observer as observer};
pub const CAPACITY: usize = 2 * 1024 * 1024;
const STACK_TOP: u16 = 0x1fff;
fn offset(address: u32) -> usize {
    crate::rtl::lorom_offset(address) as usize
}
fn stack_bytes(id: u32) -> u16 {
    match id {
        1 | 2 => 13,
        4 => 9,
        5 | 6 => 3,
        _ => 7,
    }
}
fn put(game: &mut Game, at: usize, value: u16) {
    game.ram().copy_in(at, &value.to_le_bytes());
}
fn record(out: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
}
fn span(image: &crate::rom::RomImage, address: u32, length: usize) -> Result<(), String> {
    if address & 0x8000 == 0
        || offset(address)
            .checked_add(length)
            .is_none_or(|end| end > 0x300000 || end > image.len())
    {
        return Err(format!("ineligible native span {address:06x}"));
    }
    Ok(())
}
fn eligibility(id: u32, game: &Game, image: &crate::rom::RomImage) -> Result<(), String> {
    if !cfg!(target_endian = "little") {
        return Err("requires natural u16 RAM alignment and supported little-endian host".into());
    }
    if game.actor_identity_is_with_session() || !game.supports_legacy_persistence() {
        return Err("requires available canonical actor/Session ownership".into());
    }
    // Byte-aligned SamusPoseParams copies its full eight-byte record. This
    // bounded original pose table covers every fixture's before/after poses.
    span(image, 0x91b629, 0xbe11 - 0xb629)?;
    span(image, 0x91f9c2, 0x32)?;
    span(image, 0x9b9400, 32)?;
    span(image, 0x91b010, 0x200)?;
    if core::mem::align_of::<crate::types::SamusPoseParams>() != 1
        || core::mem::size_of::<crate::types::SamusPoseParams>() != 8
    {
        return Err("unexpected pose record representation".into());
    }
    for pose in [0usize, 1, 2, 0x4d, 0x49, 0x4a, 0xc7, 0x25, 0x26] {
        let base = offset(0x91b629) + pose * 8;
        if image.read(base + 1) >= 28 || (matches!(pose, 1 | 2) && image.read(base + 3) >= 10) {
            return Err("ineligible pose callback or direction index".into());
        }
    }
    for (pose, movement) in [(1usize, 0), (2, 0), (0xc7, 0x1b), (0x25, 0xe), (0x26, 0xe)] {
        if image.read(offset(0x91b629) + pose * 8 + 1) != movement {
            return Err("unexpected selected pose movement".into());
        }
    }
    for pose in [1usize, 2] {
        let direction = usize::from(image.read(offset(0x91b629) + pose * 8 + 3));
        let turned = usize::from(image.read(offset(0x91f9c2) + direction));
        span(image, 0x91b629 + u32::try_from(turned * 8).unwrap(), 8)?;
        if image.read(offset(0x91b629) + turned * 8 + 1) != 14 {
            return Err("turning path eligibility failed".into());
        }
    }
    if id >= 7 {
        let program = if id == 10 { 0x91b401 } else { 0x91b400 };
        span(image, program, 6)?;
        if observer::word(game, 0xb3c) != 0
            || observer::word(game, 0xa96) != 0
            || observer::word(game, 0xa1c) != 1
        {
            return Err("animation path eligibility failed".into());
        }
        let pointer_at = offset(0x91b012);
        let pointer = u16::from_ne_bytes([image.read(pointer_at), image.read(pointer_at + 1)]);
        if u32::from(pointer) != (program & 0xffff) {
            return Err("authored program pointer mismatch".into());
        }
        if image.read(offset(program)) != 0x8c {
            return Err("authored opcode eligibility failed".into());
        }
    }
    if id == 4
        && (observer::word(game, 0xcd0) != 0
            || observer::word(game, 0xa68) != 1
            || game.ram().read_byte(0xa23) != 2)
    {
        return Err("windup path eligibility failed".into());
    }
    if matches!(id, 5 | 6) && (observer::word(game, 0x9e4) != 0 || game.ram().read_byte(0xa23) != 0)
    {
        return Err("moonwalk path eligibility failed".into());
    }
    Ok(())
}
/// Root runs once per freshly initialized child, never on uncorrected Rust.
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        if id >= 11
            || original.len() != 0x300000
            || game.g_cpu.is_null()
            || game.g_snes.is_null()
            || game.g_spc_player.is_null()
        {
            return Err("requires valid case and initialized original host".into());
        }
        let image = game
            .loaded_rom
            .as_ref()
            .ok_or("missing loaded image")?
            .clone();
        let native = game.capture_native_rom();
        let cart = (*game.g_snes).cart;
        if cart.is_null()
            || image.as_mut_ptr() != (*cart).rom.as_mut_ptr()
            || image.as_mut_ptr() != native.as_mut_ptr()
        {
            return Err("loaded/native/cartridge allocation mismatch".into());
        }
        // snes_loadRom rounds the original 3 MiB payload to a 4 MiB mirrored
        // cartridge. Validate that exact loader extent, not payload equality.
        let allocated_length = original.len().next_power_of_two();
        if image.len() != allocated_length
            || native.len() != allocated_length
            || (*cart).rom.len() != allocated_length
            || (*cart).rom_size as usize != allocated_length
        {
            return Err("requires exact loader cartridge allocation extent".into());
        }
        // Independent original-data equality; never copy original code into fixtures.
        for (address, length) in [
            (0x91b629, 0xbe11 - 0xb629),
            (0x91f9c2, 0x32),
            (0x9b9400, 32),
            (0x91b010, 0x200),
        ] {
            span(&image, address, length)?;
            let at = offset(address);
            if (0..length).any(|i| image.read(at + i) != original[at + i]) {
                return Err(format!("original fixture data differs at {address:06x}"));
            }
        }
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
        let mut restored_bytes = Vec::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            game.ram().fill(0..game.ram().len(), 0);
            game.ram().write_byte(0x1ffff, 1);
            game.g_fail = false;
            let (pose, facing_type, old_pose, old_type, items) = match id {
                0 => (1, 0x7654, 0, 0, 8),
                1 => (2, 0x7654, 0, 3, 8),
                2 => (2, 0x7654, 0, 0x14, 8),
                3 => (2, 0x7654, 0, 3, 0),
                4 => (0x4d, 0x0208, 0, 2, 0),
                5 => (0x49, 0x1008, 1, 0, 0),
                6 => (0x4a, 0x1004, 2, 0, 0),
                7 => (1, 8, 0, 0, 8),
                8 | 10 => (1, 8, 0, 0, 0x2000),
                _ => (1, 8, 0, 0, 0x20),
            };
            for (at, value) in [
                (0xa1c, pose),
                (0xa1e, facing_type),
                (0xa20, old_pose),
                (0x9a2, items),
                (0xa68, u16::from(id == 4)),
                (0xafa, 0x140),
                (0xb14, 0x130),
                (0xa2c, 0x3456),
                (0xa32, 0x4567),
                (0xa94, 0x5678),
                (0xa96, 0),
                (0xa9c, 0x6789),
            ] {
                put(game, at, value);
            }
            game.ram().write_byte(0xa23, old_type);
            game.ram().fill(0xc180..0xc1a0, 0x5a);
            // These seeds make the DP scratch exclusions independently visible.
            game.ram().copy_in(0, &[0xa5, 0x5a, 0x7e]);
            put(game, 0x12, 0x6b7c);
            let mut output = b"PTRS1".to_vec();
            let mut identity = id.to_le_bytes().to_vec();
            identity.push(u8::from(id >= 7));
            record(&mut output, 0, &identity);
            if id >= 7 {
                // Real logical data placement, not native allocation realignment.
                let address: u32 = if id == 10 { 0x91b401 } else { 0x91b400 };
                let pointer = (address as u16).to_le_bytes();
                let stream = [0x8c, 0x08, 0x20, 0x11, 0x22, 0x5a];
                let mut authored = Vec::new();
                for (at, bytes) in [(0x91b012, pointer.as_slice()), (address, stream.as_slice())] {
                    authored.extend_from_slice(&at.to_le_bytes());
                    authored.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                    authored.extend_from_slice(bytes);
                    for (i, value) in bytes.iter().enumerate() {
                        let at = offset(at) + i;
                        restored_bytes.push((at, image.read(at)));
                        image.write(at, *value);
                    }
                }
                record(&mut output, 8, &authored);
            }
            let mut exclusions = Vec::new();
            let mut exclude = |start: u32, end: u32, reason: u8| {
                exclusions.extend_from_slice(&start.to_le_bytes());
                exclusions.extend_from_slice(&end.to_le_bytes());
                exclusions.push(reason);
            };
            exclude(u32::from(STACK_TOP - stack_bytes(id) + 1), 0x2000, 1);
            if id >= 7 {
                exclude(0, 3, 2);
                exclude(0x12, 0x14, 2);
            }
            record(&mut output, 9, &exclusions);
            eligibility(id, game, &native)?;
            let mut excluded = vec![(usize::from(STACK_TOP - stack_bytes(id) + 1), 0x2000)];
            if id >= 7 {
                excluded.extend([(0, 3), (0x12, 0x14)]);
            }
            observer::begin(output, game, excluded);
            observer::observe(1, game);
            {
                let mut host = crate::game::session_runtime::Runtime::new(game);
                match id {
                    0..=3 => crate::game::player_transitions::refresh_pose_rules(&mut host),
                    4..=6 => crate::game::player_transitions::verify_normalize(&mut host),
                    _ => crate::game::player_transitions::verify_interpret(&mut host),
                }
            }
            observer::observe(11, game);
            if game.actor_identity_is_with_session()
                || game.capture_native_rom().as_mut_ptr() != native.as_mut_ptr()
                || game.loaded_rom.as_ref().unwrap().as_mut_ptr() != image.as_mut_ptr()
            {
                return Err("Rust call retained ownership or replaced fixture allocation".into());
            }
            let bytes = observer::finish().ok_or("missing Rust trace")?;
            if bytes.len() > CAPACITY {
                return Err("Rust trace capacity exceeded".into());
            }
            Ok(bytes)
        }));
        // Disarm on success, returned failure, or panic before restoring host.
        let _ = observer::finish();
        for (at, byte) in restored_bytes {
            image.write(at, byte);
        }
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
        result.unwrap_or_else(|_| Err("defined predecessor invocation panicked".into()))
    }
}
