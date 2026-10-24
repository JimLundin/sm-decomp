//! Actual corrected-Rust semantic observations; no CPU state is fabricated.
#![forbid(unsafe_code)]
use crate::{
    game::{session::Session, session_encoding},
    game_state::Game,
};
use std::cell::RefCell;
struct Trace {
    bytes: Vec<u8>,
    seed: Vec<u8>,
    excluded: Vec<(usize, usize)>,
}
thread_local! { static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) }; }
pub(crate) fn active() -> bool {
    TRACE.with(|s| s.borrow().is_some())
}
pub(crate) fn append(tag: u8, payload: &[u8]) {
    TRACE.with(|s| {
        if let Some(t) = s.borrow_mut().as_mut() {
            t.bytes.push(tag);
            t.bytes
                .extend_from_slice(&(payload.len() as u32).to_le_bytes());
            t.bytes.extend_from_slice(payload);
        }
    });
}
pub(crate) fn begin(bytes: Vec<u8>, game: &Game, excluded: Vec<(usize, usize)>) {
    TRACE.with(|s| {
        assert!(s.borrow().is_none(), "nested transition observer");
        *s.borrow_mut() = Some(Trace {
            bytes,
            seed: game.g_ram.to_vec(),
            excluded,
        });
    });
}
pub(crate) fn finish() -> Option<Vec<u8>> {
    TRACE.with(|s| s.borrow_mut().take().map(|t| t.bytes))
}
fn snapshot(event: u16, ram: &[u8], fail: bool) {
    if !active() {
        return;
    }
    assert!(!fail, "transition diagnostic");
    TRACE.with(|s| {
        let t = s.borrow();
        let t = t.as_ref().unwrap();
        for &(start, end) in &t.excluded {
            assert_eq!(
                &ram[start..end],
                &t.seed[start..end],
                "Rust changed CPU-only excluded bytes"
            );
        }
    });
    let mut payload = event.to_le_bytes().to_vec();
    payload.extend_from_slice(ram);
    payload.push(u8::from(fail));
    append(2, &payload);
}
pub(crate) fn observe(event: u16, game: &Game) {
    if !active() {
        return;
    }
    snapshot(event, &game.g_ram, game.g_fail);
}
pub(crate) fn projected(name: &str, state: &Session, game: &Game) {
    if !active() {
        return;
    }
    let event = match name {
        "transitions.suit.load" => 27,
        "transitions.suit.copy" => 28,
        _ => return,
    };
    let mut ram = game.g_ram.to_vec();
    session_encoding::encode(state, &mut ram);
    snapshot(event, &ram, game.g_fail);
}
pub(crate) fn word(game: &Game, at: usize) -> u16 {
    u16::from_le_bytes([game.g_ram[at], game.g_ram[at + 1]])
}
pub(crate) fn before(game: &Game, at: usize) -> u16 {
    if active() { word(game, at) } else { 0 }
}
pub(crate) fn publication(kind: u16, at: usize, before: u16, after: u16) {
    if !active() {
        return;
    }
    let mut p = kind.to_le_bytes().to_vec();
    p.extend_from_slice(&(at as u32).to_le_bytes());
    p.extend_from_slice(&before.to_le_bytes());
    p.extend_from_slice(&after.to_le_bytes());
    append(3, &p);
}
pub(crate) fn published(kind: u16, at: usize, before: u16, game: &Game) {
    if !active() {
        return;
    }
    publication(kind, at, before, word(game, at));
}
pub(crate) fn scalar(kind: u16, value: u16) {
    if !active() {
        return;
    }
    let mut p = kind.to_le_bytes().to_vec();
    p.extend_from_slice(&value.to_le_bytes());
    append(4, &p);
}
pub(crate) fn palette_event(kind: &str, argument: u16) {
    if !active() {
        return;
    }
    let code = match kind {
        "copy" => 1,
        "load" => 2,
        "command" => 3,
        _ => panic!("unexpected palette effect"),
    };
    let mut p = vec![code];
    p.extend_from_slice(&argument.to_le_bytes());
    append(5, &p);
}
pub(crate) fn sound(channel: usize, sound: u8, limit: u8) {
    if !active() {
        return;
    }
    append(6, &[u8::try_from(channel).unwrap(), limit, sound]);
}
