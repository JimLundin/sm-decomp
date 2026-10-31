//! Authored PT21/PT22 fixtures over the actual corrected predecessor entries.
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
        if !matches!(id, 2100..=2102 | 2200..=2202)
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
            if loaded.same_allocation(&native) {
                return Err("authored L/N allocations must be distinct".into());
            }
            let mut loaded_manifest = Vec::new();
            let mut native_manifest = Vec::new();
            let (current_radius, previous_radius) = if id == 2102 { (4, 9) } else { (9, 4) };
            author(
                &loaded,
                0x91b629 + 2 * 8,
                &[8, 0, 0, 0, 0, 0, current_radius, 0],
                &mut loaded_manifest,
            );
            author(
                &loaded,
                0x91b629 + 3 * 8,
                &[4, 0, 0, 0, 0, 0, previous_radius, 0],
                &mut loaded_manifest,
            );
            author(
                &native,
                0x91b629 + 2 * 8,
                &[8, 0, 0, 0, 0, 0, 33, 0],
                &mut native_manifest,
            );
            author(
                &native,
                0x91b629 + 3 * 8,
                &[4, 0, 0, 0, 0, 0, 1, 0],
                &mut native_manifest,
            );
            game.register_rom(loaded.clone());
            game.register_native_rom(native.clone());
            for (at, value) in [
                (0xa1c, 2),
                (0xa20, 3),
                (0xa1e, 0x3412),
                (0xa22, 0x7856),
                (0xa24, 0xabcd),
                (0xa26, 0xef90),
                (0xafa, if id == 2101 { 2 } else { 100 }),
                (0xb14, if id == 2101 { 3 } else { 200 }),
                (0xafc, 0x1234),
                (0xb16, 0x5678),
                (0xa94, 0x3141),
                (0xa96, 0x5926),
            ] {
                put(game, at, value);
            }
            if id >= 2200 {
                game.ram().copy_in(
                    0xa1c,
                    &[
                        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
                    ],
                );
                if id == 2201 {
                    // Same pose word, deliberately distinct packed movement/facing.
                    game.ram().copy_in(0xa20, &[0x11, 0x22]);
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
            {
                use {
                    crate::game::{player_transitions, session_runtime::Runtime},
                    sm_game::session_access::SessionAccess,
                };
                let mut host = Runtime::new(game);
                if id < 2200 {
                    player_transitions::align_bottom_with_previous_pose(&mut host);
                } else {
                    player_transitions::record_pose_history(host.session_mut());
                    if id == 2202 {
                        host.probe("transitions.phase.2202");
                        player_transitions::record_pose_history(host.session_mut());
                    }
                }
            }
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
        result.unwrap_or_else(|_| Err("PT21/PT22 defined invocation unexpectedly panicked".into()))
    }
}
