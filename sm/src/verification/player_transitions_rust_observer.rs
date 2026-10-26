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
        "transitions.phase.2" => 2,
        "transitions.phase.3" => 3,
        "transitions.phase.4" => 4,
        "transitions.phase.5" => 5,
        "transitions.phase.6" => 6,
        "transitions.phase.7" => 7,
        "transitions.phase.8" => 8,
        "transitions.phase.9" => 9,
        "transitions.phase.10" => 10,
        "transitions.phase.20" => 20,
        "transitions.phase.21" => 21,
        "transitions.phase.22" => 22,
        "transitions.phase.23" => 23,
        "transitions.phase.24" => 24,
        "transitions.phase.25" => 25,
        "transitions.phase.26" => 26,
        "transitions.phase.30" => 30,
        "transitions.phase.31" => 31,
        "transitions.phase.32" => 32,
        "transitions.phase.210" => 210,
        "transitions.phase.211" => 211,
        "transitions.phase.220" => 220,
        "transitions.phase.221" => 221,
        "transitions.phase.226" => 226,
        "transitions.phase.227" => 227,
        "transitions.phase.228" => 228,
        "transitions.phase.229" => 229,
        "transitions.phase.230" => 230,
        "transitions.phase.231" => 231,
        "transitions.phase.232" => 232,
        "transitions.phase.233" => 233,
        "transitions.phase.234" => 234,
        "transitions.phase.235" => 235,
        "transitions.phase.236" => 236,
        "transitions.phase.237" => 237,
        "transitions.phase.238" => 238,
        "transitions.phase.239" => 239,
        "transitions.phase.240" => 240,
        "transitions.phase.2202" => 2202,
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
        "misc" => 4,
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
