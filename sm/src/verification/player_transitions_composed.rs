//! Three composed finish_movement contracts; existing capture envelope.
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
        if !matches!(id, 3700..=3702)
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
            let mut nposes = vec![0; 256 * 8];
            let mut lposes = vec![0; 256 * 8];
            for pose in 0..256 {
                nposes[pose * 8] = 8;
                nposes[pose * 8 + 6] = 2;
                lposes[pose * 8] = 4;
                lposes[pose * 8 + 1] = 7;
                lposes[pose * 8 + 6] = 2;
            }
            nposes[0x37 * 8 + 1] = 15;
            nposes[0x4b * 8 + 1] = 2;
            nposes[0x19 * 8 + 1] = 3;

            let mut programs = Vec::new();
            for _ in 0..256 {
                programs.extend_from_slice(&0x9000u16.to_ne_bytes());
            }
            author(&loaded, 0x91b629, &lposes, &mut loaded_manifest);
            author(&loaded, 0x91b010, &programs, &mut loaded_manifest);
            author(&native, 0x91b629, &nposes, &mut native_manifest);
            author(&native, 0x919000, &[7; 8], &mut native_manifest);
            if core::mem::align_of::<crate::types::SamusPoseParams>() != 1
                || core::mem::size_of::<crate::types::SamusPoseParams>() != 8
            {
                return Err("unexpected pose record layout".into());
            }
            game.register_rom(loaded.clone());
            game.register_native_rom(native.clone());
            let (current, previous) = (2, 3);
            for (at, value) in [
                (0xa1c, current),
                (0xa20, previous),
                (0xa1e, 8),
                (0xa22, 4),
                (0xa24, 0x5678),
                (0xa26, 0x9abc),
                (0xa28, 0xffff),
                (0xa2a, 0xffff),
                (0xa2c, 0xffff),
                (0xa2e, 0),
                (0xa30, 0),
                (0xa32, 0),
                (0xa34, 0x1111),
                (0xa36, 0x2222),
                (0xa60, 0xe913),
                (0xaf6, 100),
                (0xafa, 100),
                (0xafc, 0x1234),
                (0xb14, 200),
                (0xb16, 0x5678),
                (0xb00, 2),
                (0xafe, 2),
                (0xa66, 2),
                (0xa94, 0x3141),
                (0xa96, 0x5926),
                (0xa9c, 0x11),
                (0xa9a, 2),
                (0xa9e, 0xabcd),
                (0xaa0, 0xbeef),
                (0xdc6, 0x4321),
                (0x195e, 0xffff),
                (0x1962, 0xffff),
                (0x9a2, 0),
            ] {
                put(game, at, value);
            }
            put(game, crate::ram_map::words::ROOM_WIDTH_IN_BLOCKS, 16);
            put(game, crate::ram_map::words::ROOM_HEIGHT_IN_BLOCKS, 16);
            put(game, 0xa94, 2);
            put(game, 0xdc6, 0);
            if id == 3700 {
                for (at, value) in [(0xa28, 5), (0xa2a, 4), (0xa2c, 6), (0xa32, 3)] {
                    put(game, at, value);
                }
            } else if id == 3701 {
                put(game, 0xa28, 0x37);
                put(game, 0xa2e, 0xffff);
            } else {
                use crate::ram_map::words as w;
                put(game, w::FX_TYPE, 6);
                put(game, w::FX_Y_POS, 70);
                put(game, w::LIQUID_PHYSICS_TYPE, 1);
                put(game, w::NMI_FRAME_COUNTER_WORD, 0);
                put(game, w::SAMUS_X_POS, 0x120);
            }
            let mut expected = (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>();
            expected_word(&mut expected, 0xa94, if id == 3700 { 11 } else { 1 });
            expected_word(&mut expected, 0xa9c, if id == 3702 { 3 } else { 2 });
            expected_word(&mut expected, 0xa1c, if id == 3700 { 6 } else { 3 });
            expected_word(&mut expected, 0xa1e, 8);
            expected_word(&mut expected, 0xa24, 3);
            expected_word(&mut expected, 0xa26, 4);
            expected_word(&mut expected, 0xa20, if id == 3700 { 6 } else { 3 });
            expected_word(&mut expected, 0xa22, 8);
            if id == 3700 {
                expected_word(&mut expected, 0xa96, 1);
                expected_word(&mut expected, 0xa9a, 1);
            } else {
                for (at, value) in [(0xa2e, 7), (0xa34, 0), (0xa36, 0), (0xa9a, 0)] {
                    expected_word(&mut expected, at, value);
                }
            }
            if id == 3702 {
                for (at, value) in [
                    (0xaf6, 0x3456),
                    (0xa2a, 0x37),
                    (0xa30, 0xffff),
                    (0x5e5, 0x111),
                    (0xad8, 3),
                    (0xae0, 0x120),
                    (0xae8, 104),
                    (0xaf0, 0x500),
                ] {
                    expected_word(&mut expected, at, value);
                }
                expected[0x654] = 6;
                expected[0x666] = 15;
                expected[0x647] = 1;
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
            MODE.with(|mode| mode.set(if id == 3702 { 1 } else { 3 }));
            invoke(game);
            if id == 3702 && MODE.with(|mode| mode.get()) != 2 {
                return Err("missing actual random intervention".into());
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
                    "composed {id} full RAM differs at {at:05x}: actual {:02x}, expected {:02x}",
                    game.ram().read_byte(at),
                    expected[at]
                ));
            }
            if game.g_fail {
                return Err("composed call reported diagnostic".into());
            }
            observer::scalar(3702, 1);
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
        MODE.with(|mode| mode.set(0));
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

thread_local! { static MODE: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }
/// Only the three composed fixture invocations enable matching clock observations.
pub(crate) fn active() -> bool {
    MODE.with(|mode| mode.get() != 0)
}
fn expected_word(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn word(game: &Game, at: usize) -> u16 {
    {
        let _ = &[(); 0x20000][at..at + 2];
        let mut bytes = [0; 2];
        game.ram().copy_out(at, &mut bytes);
        u16::from_le_bytes(bytes)
    }
}
/// Hook immediately after actual NextRandom publishes its result; dormant for all other captures.
pub(crate) fn random_after(game: &mut Game) {
    if MODE.with(|mode| mode.get()) != 1 {
        return;
    }
    assert!(
        !game.actor_identity_is_with_session(),
        "foreign effect must own RAM"
    );
    for (at, value) in [
        (0xaf6, 0x120),
        (0xad8, 3),
        (0xae0, 0x120),
        (0xae8, 104),
        (0xaf0, 0x500),
        (0x5e5, 0x111),
    ] {
        assert_eq!(
            word(game, at),
            value,
            "actual bubble/random publication at {at:04x}"
        );
    }
    observer::observe(3700, game);
    put(game, 0xaf6, 0x3456);
    put(game, 0xa2a, 0x37);
    put(game, 0xa30, 0xffff);
    MODE.with(|mode| mode.set(2));
    observer::observe(3701, game);
}
unsafe fn invoke(game: &mut Game) {
    let mut runtime = crate::game::session_runtime::Runtime::new(game);
    crate::game::player_transitions::finish_movement(&mut runtime);
}
