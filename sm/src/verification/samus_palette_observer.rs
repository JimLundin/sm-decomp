//! Verification-only observations of actual Samus gameplay and aggregate Runtime.
#![forbid(unsafe_code)]
use crate::game::{samus_palette, session::Session, session_encoding, session_runtime::Runtime};
use crate::game_state::Game;
use std::cell::{Cell, RefCell};
thread_local! { static TRACE: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) }; }
thread_local! {
    static ACTIVE_CASE: Cell<Option<u32>> = const { Cell::new(None) };
    static INJECTED: Cell<bool> = const { Cell::new(false) };
}
/// Four explicitly injected callback-transition cases, after actual charge
/// processing and immediately before selector dispatch. Not production policy.
pub(crate) fn inject(name: &str, state: &mut Session) {
    if name != "samus.palette.dispatch" {
        return;
    }
    let Some(id @ 29..=32) = ACTIVE_CASE.with(Cell::get) else {
        return;
    };
    assert!(
        !INJECTED.with(|flag| flag.replace(true)),
        "duplicate visual injection"
    );
    let index = if id == 30 || id == 32 { 9 } else { 0 };
    let timer = if id <= 30 { 2 } else { 1 };
    state.player.special_palette_flags = 0x8000;
    state.projectiles.controls.charge_palette = index;
    state.player.palette.timer = timer;
    let mut payload = 0x91d712u32.to_le_bytes().to_vec();
    for (address, value) in [(0xa4au32, 0x8000u16), (0xb62, index), (0xad0, timer)] {
        payload.extend_from_slice(&address.to_le_bytes());
        payload.extend_from_slice(&value.to_le_bytes());
    }
    append(9, &payload);
}
fn append(tag: u8, payload: &[u8]) {
    TRACE.with(|slot| {
        if let Some(bytes) = slot.borrow_mut().as_mut() {
            bytes.push(tag);
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(payload);
        }
    });
}
pub(crate) fn event(kind: &str, argument: u16) {
    #[cfg(sm_verify_dll)]
    crate::player_transitions_rust_observer::palette_event(kind, argument);
    if !TRACE.with(|slot| slot.borrow().is_some()) {
        return;
    }
    if kind == "misc" {
        append(6, &[]);
        return;
    }
    let code = match kind {
        "copy" => 1,
        "load" => 2,
        "command" => 3,
        _ => panic!("unknown visual oracle event {kind}"),
    };
    append(7, &[code, argument as u8, (argument >> 8) as u8]);
}
pub(crate) fn sound(channel: usize, sound: u8, limit: u8) {
    #[cfg(sm_verify_dll)]
    crate::player_transitions_rust_observer::sound(channel, sound, limit);
    append(8, &[u8::try_from(channel).unwrap(), limit, sound]);
}
pub(crate) fn observe(name: &str, state: &Session, game: &Game) {
    #[cfg(sm_verify_dll)]
    crate::player_transitions_rust_observer::projected(name, state, game);
    if !TRACE.with(|slot| slot.borrow().is_some()) {
        return;
    }
    let (tag, carry) = match name {
        "samus.palette.entry" => (1, None),
        "samus.palette.carry.clear" => (2, Some(0)),
        "samus.palette.carry.set" => (2, Some(1)),
        "samus.palette.tail" => (4, None),
        "samus.palette.return" => (5, None),
        _ => return,
    };
    // Project to a copy only; observation cannot trigger a foreign interval.
    let mut bytes = game.g_ram.to_vec();
    session_encoding::encode(state, &mut bytes);
    if let Some(carry) = carry {
        bytes.insert(0, carry);
    }
    append(tag, &bytes);
}
fn put(game: &mut Game, at: usize, value: u16) {
    game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
/// Frozen29 inputs plus four appended post-charge injections; no ROM bytes.
fn seed(id: u32, game: &mut Game) -> Result<(), String> {
    let (flags, hurt, grapple, movement, flare, shoot) = match id {
        0..=11 => (
            0,
            [0, 1, 2, 6, 7, 38, 58, 59, 60, 0x803a, 0x803b, 0xffff][id as usize],
            0xc4f0,
            0,
            0,
            0,
        ),
        12..=14 => ([1, 2, 0x7fff][(id - 12) as usize], 0, 0xc4f0, 0, 0, 0),
        15..=16 => (0, 39, 0xc4f0, [3, 0x14][(id - 15) as usize], 0, 0),
        17..=24 => (
            0,
            39,
            0xc4f0,
            0,
            [15, 16, 0x800f, 0x8010][((id - 17) / 2) as usize],
            if id & 1 == 0 { 0x40 } else { 0 },
        ),
        25..=28 => (
            0,
            39,
            [0xc855, 0xc856, 0x4855, 0x4856][(id - 25) as usize],
            0,
            0,
            0,
        ),
        29..=32 => (0, 0, 0xc4f0, 0, 0, 0),
        _ => return Err("unknown selector2 case".into()),
    };
    game.g_ram.fill(0);
    game.g_ram[0x1ffff] = 1;
    for (at, value) in [
        (0xacc, 2),
        (0xa4a, flags),
        (0xa48, hurt),
        (0xd32, grapple),
        (0xa1f, movement),
        (0xcd0, flare),
        (0x8b, shoot),
        (0x9b2, 0x40),
        (0xa44, 0xe913),
        (0x998, 8),
    ] {
        put(game, at, value);
    }
    Ok(())
}
/// Caller initializes original-ROM content/audio and invokes once in an isolated
/// process. Returned ROM-derived observations must stay outside the repository.
pub(crate) fn capture(id: u32, game: &mut Game) -> Result<Vec<u8>, String> {
    seed(id, game)?;
    TRACE.with(|slot| {
        assert!(slot.borrow().is_none());
        *slot.borrow_mut() = Some(b"SPCR1".to_vec());
    });
    ACTIVE_CASE.with(|value| value.set(Some(id)));
    INJECTED.with(|value| value.set(false));
    append(0, &id.to_le_bytes());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        samus_palette::advance(&mut Runtime::new(game))
    }));
    let bytes = TRACE.with(|slot| slot.borrow_mut().take().unwrap());
    ACTIVE_CASE.with(|value| value.set(None));
    let injected = INJECTED.with(|value| value.replace(false));
    result.map_err(|_| "candidate Samus advance panicked".to_owned())?;
    if injected != (id >= 29) {
        return Err("missing visual callback-transition injection".into());
    }
    Ok(bytes)
}
