//! Authored corrected-predecessor animation fixtures; no independent CPU claim.
//! Source-only lane. Root supplies the shared export and initialized host.
use crate::native_rom::NativeRomPointer;
use crate::{game_state::Game, player_transitions_rust_observer as observer};
use std::cell::Cell;
thread_local! {
    static CASE: Cell<Option<u32>> = const { Cell::new(None) };
}
fn put(game: &mut Game, at: usize, value: u16) {
    game.ram().copy_in(at, &value.to_le_bytes());
}
fn record(out: &mut Vec<u8>, tag: u8, bytes: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}
fn known(id: u32) -> bool {
    matches!(id, 100..=104 | 300..=304 | 400..=407
        | 500..=509 | 600..=605 | 700..=702
        | 900..=901 | 1000..=1001 | 1100..=1106)
}
/// Fresh initialized Game required. Captures actual source calls, no expected algorithm.
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    if !known(id) || original.len() != 0x300000 || !cfg!(target_endian = "little") {
        return Err("unsupported animation fixture or original payload/host".into());
    }
    if game.actor_identity_is_with_session()
        || !game.supports_legacy_persistence()
        || observer::active()
        || CASE.with(|state| state.get().is_some())
    {
        return Err("animation fixture requires canonical, idle naturally aligned host".into());
    }
    let image = game
        .loaded_rom
        .as_ref()
        .ok_or("missing loaded image")?
        .clone();
    let native = game.capture_native_rom();
    if game.g_snes.is_null() {
        return Err("missing initialized SNES".into());
    }
    let cart = unsafe { (*game.g_snes).cart };
    if cart.is_null() {
        return Err("missing initialized cartridge".into());
    }
    if image.len() != 0x400000
        || native.len() != 0x400000
        || image.as_mut_ptr() != native.as_mut_ptr()
        || unsafe {
            (*cart).rom.len() != 0x400000
                || (*cart).romSize != 0x400000
                || (*cart).rom.as_mut_ptr() != image.as_mut_ptr()
        }
    {
        return Err("requires unchanged original loader 4 MiB allocation".into());
    }
    let ram_before = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let fail_before = game.g_fail;
    let mut restored = Vec::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        game.ram().fill(0..game.ram().len(), 0);
        game.g_fail = false;
        let family = id / 100;
        let sub = id % 100;
        let mut pose = 2u16;
        let mut frame = 0u16;
        let mut timer = 0x1234u16;
        let mut steps = 1;
        // Sparse authored spans; every dereferenced program byte is explicitly set.
        // Full raw extent is guarded below even when only two bytes are selected.
        let program = if family == 9 {
            0x919001u32
        } else {
            0x919000u32
        };
        let mut bytes: Vec<(usize, Vec<u8>)> = vec![(0, vec![5]), (1, vec![7]), (2, vec![9])];
        for (at, value) in [
            (0x196e, 8),
            (0xa9c, 3),
            (0xa2c, 0x4567),
            (0xa32, 0x5678),
            (0xa58, 0x6789),
            (0xa60, 0x8888),
            (0xafa, 100),
            (0x195e, 0xffff),
            (0x1962, 0xffff),
            (0xb36, 1),
        ] {
            put(game, at, value);
        }
        // Preserve conspicuous FX slots/buffer across clock-only cases.
        game.ram().fill(0xad4..0xaf4, 0x5a);
        match family {
            1 => {
                pose = if sub == 1 {
                    0x4e
                } else if sub == 3 {
                    2
                } else {
                    0x4d
                };
                frame = 1;
                timer = 1;
                if sub == 2 {
                    put(game, 0xb36, 2);
                }
                if sub == 4 {
                    steps = 2;
                }
            }
            3 => {
                timer = [0, 1, 0x8000, 0xffff, 1][sub as usize];
                frame = if sub == 4 { 0xffff } else { 0 };
            }
            4 => match sub {
                0..=2 => {
                    put(game, 0x9c2, [29, 30, 0x8000][sub as usize]);
                    bytes.push((0, vec![0x86, 7]));
                }
                3 => bytes.push((0, vec![0x87, 7])),
                4 => {
                    frame = 2;
                    bytes.push((2, vec![0x8e, 1]));
                }
                5 => {
                    bytes.push((0, vec![0x8e, 1]));
                    bytes.push((0xffff, vec![11]));
                }
                6 => {
                    frame = 2;
                    bytes.push((2, vec![0x8f]));
                }
                _ => {
                    frame = 0xffff;
                    bytes.push((0xffff, vec![0x87]));
                }
            },
            5 => {
                let pending = [0x4b, 0x4c, 0x19, 0x1a, 2][(sub % 5) as usize];
                put(game, 0xa28, pending);
                if sub >= 5 {
                    put(game, 0xa60, 0xe91d);
                }
                bytes.push((0, vec![0x88, 2]));
            }
            6 => {
                put(game, 0x9a2, if sub >= 3 { 0x0100 } else { 0 });
                put(game, 0xb2c, u16::from(sub % 3 == 1));
                put(game, 0xb2e, u16::from(sub % 3 == 2));
                bytes.push((0, vec![0x89, 2, 1, 0x11, 0x12, 0x13, 0x14]));
            }
            7 => {
                put(game, 0xb2c, u16::from(sub == 1));
                put(game, 0xb2e, u16::from(sub == 2));
                bytes.push((0, vec![0x8a, 0x21, 0x22]));
            }
            9 | 10 => {
                put(game, 0x9a2, if sub == 0 { 0x0100 } else { 0 });
                bytes.push((0, vec![0x8c, 2, 1, 0x31, 0x32]));
            }
            11 => {
                bytes.push((
                    0,
                    if sub == 0 {
                        vec![0x8d, 0x33]
                    } else {
                        vec![0x7f + sub as u8]
                    },
                ));
            }
            _ => unreachable!(),
        }
        put(game, 0xa1c, pose);
        put(game, 0xa96, frame);
        put(game, 0xa94, timer);
        // has_momentum=0 prevents every speed-table or speed-sound closure.
        if observer::word(game, 0xb3c) != 0 {
            return Err("momentum eligibility".into());
        }
        let physical = crate::rtl::lorom_offset(program) as usize;
        let extent = if id == 405 || id == 407 { 0x10000 } else { 64 };
        if physical
            .checked_add(extent)
            .is_none_or(|end| end > 0x300000 || end > native.len())
        {
            return Err("raw program extent outside original payload allocation".into());
        }
        let mut output = b"PTRS1".to_vec();
        let mut identity = id.to_le_bytes().to_vec();
        identity.push(2); // authored predecessor family, not the original11 CPU mapping
        record(&mut output, 0, &identity);
        let mut authored = Vec::new();
        let mut write = |address: u32, at: usize, values: &[u8]| -> Result<(), String> {
            if at
                .checked_add(values.len())
                .is_none_or(|end| end > 0x300000 || end > image.len())
            {
                return Err("authored span outside original prefix".into());
            }
            authored.extend_from_slice(&address.to_le_bytes());
            authored.extend_from_slice(&(values.len() as u32).to_le_bytes());
            authored.extend_from_slice(values);
            for (i, &value) in values.iter().enumerate() {
                restored.push((at + i, image.read(at + i)));
                image.write(at + i, value);
            }
            Ok(())
        };
        for (at, values) in &bytes {
            // Address records use physical LoROM projection for cross-bank offsets.
            let offset = physical + at;
            let address =
                0x800000 | (((offset / 0x8000) as u32) << 16) | 0x8000 | (offset % 0x8000) as u32;
            write(address, offset, values)?;
        }
        let pointer_address = 0x91b010 + u32::from(pose) * 2;
        write(
            pointer_address,
            crate::rtl::lorom_offset(pointer_address) as usize,
            &(program as u16).to_le_bytes(),
        )?;
        record(&mut output, 8, &authored);
        record(&mut output, 9, &[]); // corrected predecessor: no CPU-only exclusions
        CASE.with(|state| state.set(Some(id)));
        observer::begin(output, game, Vec::new());
        observer::observe(200, game);
        {
            use {
                crate::game::{
                    player_transitions::{self, state::AnimationStep},
                    session_runtime::Runtime,
                },
                sm_game::session_access::SessionAccess,
            };
            let mut host = Runtime::new(game);
            for step in 0..steps {
                match family {
                    1 | 3 => {
                        player_transitions::advance_animation(&mut host, AnimationStep::Gameplay)
                    }
                    _ => player_transitions::verify_interpret(&mut host),
                }
                host.probe(if step == 0 {
                    "transitions.phase.210"
                } else {
                    "transitions.phase.211"
                });
            }
        }
        if game.capture_native_rom().as_mut_ptr() != native.as_mut_ptr()
            || game
                .loaded_rom
                .as_ref()
                .is_none_or(|loaded| loaded.as_mut_ptr() != image.as_mut_ptr())
            || game.actor_identity_is_with_session()
        {
            return Err("animation fixture unexpectedly changed ownership/binding".into());
        }
        let out = observer::finish().ok_or("missing animation trace")?;
        if out.len() > 2 * 1024 * 1024 {
            return Err("animation trace exceeds export capacity".into());
        }
        Ok(out)
    }));
    CASE.with(|state| state.set(None));
    let _ = observer::finish();
    // Reverse order also correctly restores deliberately overlapping authored bytes.
    for (at, value) in restored.into_iter().rev() {
        image.write(at, value);
    }
    game.ram().copy_in(0, &ram_before);
    game.g_fail = fail_before;
    result.unwrap_or_else(|_| Err("eligible animation predecessor panicked".into()))
}
