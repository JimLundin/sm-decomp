//! Actual corrected-Rust semantic observations; no CPU state is fabricated.
#![forbid(unsafe_code)]
use crate::{
    game::{session::Session, session_encoding},
    game_state::Game,
};
pub(crate) use sm_game::transition_trace::{
    active, append, finish, palette_event, publication, scalar, sound,
};
pub(crate) fn begin(bytes: Vec<u8>, game: &Game, excluded: Vec<(usize, usize)>) {
    sm_game::transition_trace::start(
        bytes,
        || {
            (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<u8>>()
        },
        excluded,
    );
}
fn snapshot(event: u16, ram: &[u8], fail: bool) {
    if !active() {
        return;
    }
    assert!(!fail, "transition diagnostic");
    sm_game::transition_trace::check_excluded(ram);
    let mut payload = event.to_le_bytes().to_vec();
    payload.extend_from_slice(ram);
    payload.push(u8::from(fail));
    append(2, &payload);
}
pub(crate) fn observe(event: u16, game: &Game) {
    if !active() {
        return;
    }
    snapshot(
        event,
        &(0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<u8>>(),
        game.g_fail,
    );
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
    let mut ram = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<u8>>();
    session_encoding::encode(state, |at, span| {
        ram[at..at + span.len()].copy_from_slice(span)
    });
    snapshot(event, &ram, game.g_fail);
}
pub(crate) fn word(game: &Game, at: usize) -> u16 {
    u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}
