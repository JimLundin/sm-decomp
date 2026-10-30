//! Five actual beta/intro caller comparisons; existing capture envelope.
//! Source-only. Root owns registration, build, fresh-host execution and capture.
use super::{Game, MakeSnapshotWithPlayerOwner, RestoreSnapshot, Snapshot};
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
        if !matches!(id, 3800..=3804)
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
            MakeSnapshotWithPlayerOwner(
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
                (ram::samus_pose, 1),
                (0xa1e, 8),
                (ram::samus_prev_pose, 1),
                (ram::samus_movement_handler, 0xe90e),
                (ram::frame_handler_gamma, 0xe90e),
                (ram::frame_handler_alfa, 0xe8cd),
                (
                    ram::frame_handler_beta,
                    if id == 3800 { 0xe7f5 } else { 0xe833 },
                ),
                (ram::debug_disable_minimap, 1),
                (ram::time_is_frozen_flag, 1),
                (ram::samus_anim_frame_timer, 2),
                (ram::samus_anim_frame, 0),
                (ram::samus_x_speed_divisor, 0),
                (ram::timer_for_shine_timer, 10),
                (ram::samus_health, 99),
                (ram::samus_new_pose, 0xffff),
                (ram::samus_new_pose_interrupted, 0xffff),
                (ram::samus_new_pose_transitional, 0xffff),
                (ram::samus_y_pos, 100),
                (ram::samus_y_radius, 2),
                (ram::samus_invincibility_timer, 3),
                (ram::samus_knockback_timer, 4),
                (ram::joypad1_lastkeys, 0x10),
                (ram::joypad1_newkeys, 0x20),
                (ram::joypad1_input_samusfilter, 0x40),
                (ram::joypad1_newinput_samusfilter, 0x80),
                (ram::controller1_input_for_demo, 0x100),
                (ram::controller1_new_input_for_demo, 0x200),
                (ram::demo_backup_prev_controller_input, 0x400),
                (ram::demo_backup_prev_controller_input_new, 0x800),
                (
                    ram::cinematic_var15,
                    match id {
                        3803 => 1,
                        3804 => 0x8000,
                        _ => 0,
                    },
                ),
            ] {
                put(game, at, value);
            }
            if id == 3801 {
                put(game, ram::hdma_var_1, 0xa55a);
            }
            let original_ram = (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>();
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
            ACTIVE.with(|active| active.set(true));
            invoke(id, game);
            ACTIVE.with(|active| active.set(false));
            if game.actor_identity_is_with_session() {
                return Err("actual caller retained Session authority".into());
            }
            if id == 3802 {
                if (0..game.ram().len())
                    .map(|at| game.ram().read_byte(at))
                    .ne(original_ram.iter().copied())
                {
                    return Err("disabled intro changed RAM".into());
                }
            } else {
                let restored = [
                    ram::joypad1_lastkeys,
                    ram::joypad1_newkeys,
                    ram::joypad1_input_samusfilter,
                    ram::joypad1_newinput_samusfilter,
                ]
                .map(|at| word(game, at));
                let expected = if id == 3801 {
                    [0x100, 0x200, 0x5a00, 0x8a5]
                } else {
                    [0x100, 0x200, 0x400, 0x800]
                };
                if restored != expected
                    || word(game, ram::samus_anim_frame_timer) != 1
                    || word(game, ram::samus_anim_frame) != 0
                    || word(game, ram::input_to_pose_calc) != 0
                    || {
                        let mut bytes = [0; 0xc1a0 - (0xc180)];
                        game.ram().copy_out(0xc180, &mut bytes);
                        bytes.as_slice() != palette.as_slice()
                    }
                {
                    return Err("actual beta failed control/animation/palette completion".into());
                }
                let (invincibility, knockback) = if id >= 3803 { (2, 3) } else { (3, 4) };
                if word(game, ram::samus_invincibility_timer) != invincibility
                    || word(game, ram::samus_knockback_timer) != knockback
                {
                    return Err("actual intro timer tail mismatch".into());
                }
            }
            if game.g_fail {
                return Err("actual caller reported diagnostic".into());
            }
            observer::scalar(3800, 1);
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
        ACTIVE.with(|active| active.set(false));
        let _ = observer::finish();
        game.loaded_rom = loaded_before;
        game.restore_native_rom_binding(native_before);
        {
            let snapshot_snes = game.g_snes;
            let snapshot_cpu = game.g_cpu;
            let (backend, ram, _, _, _, _) = game.ram_and_owned_snapshots();
            RestoreSnapshot(
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

thread_local! { static ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
/// Only qualifies actual legacy phase220 observation; does not emit an event.
pub(crate) fn active() -> bool {
    ACTIVE.with(std::cell::Cell::get)
}

fn word(game: &Game, at: usize) -> u16 {
    {
        let _ = &[(); 0x20000][at..at + 2];
        let mut bytes = [0; 2];
        game.ram().copy_out(at, &mut bytes);
        u16::from_le_bytes(bytes)
    }
}
unsafe fn invoke(id: u32, game: &mut Game) {
    match id {
        3800 | 3801 => crate::game::session_runtime::player_update_runtime::advance_frame(game),
        _ => sm_game::cinematic_states::handle_samus_during_intro(game),
    }
}
