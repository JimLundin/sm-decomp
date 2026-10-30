//! Scoped verifier-only environment changes at actual legacy callback edges.
//! The display and native renderer still execute their maintained algorithms.
use crate::{game_state::Game, rom::RomImage};
use std::cell::RefCell;

struct MessageTrace {
    calls: usize,
    bytes: Vec<u8>,
}
struct NativeTrace {
    replacement: RomImage,
    addresses: Vec<u16>,
}
thread_local! {
    static MESSAGE: RefCell<Option<MessageTrace>> = const { RefCell::new(None) };
    static NATIVE: RefCell<Option<NativeTrace>> = const { RefCell::new(None) };
}

pub(super) fn begin_message() {
    MESSAGE.with(|state| {
        assert!(state.borrow().is_none());
        *state.borrow_mut() = Some(MessageTrace {
            calls: 0,
            bytes: Vec::new(),
        });
    });
}
pub(super) fn finish_message() -> Vec<u8> {
    MESSAGE.with(|state| {
        let trace = state.borrow_mut().take().expect("armed message trace");
        assert!(trace.calls > 1, "actual message must yield and resume");
        trace.bytes
    })
}

/// Called inside the scheduler's existing released-owner display closure.
/// Each edge records613 before changing the environment; the actual display
/// then runs and613 is inspected again before returning to the scheduler.
pub(crate) unsafe fn display(request: u16, game: &mut Game) -> u8 {
    let marker = MESSAGE.with(|state| {
        let mut state = state.borrow_mut();
        state.as_mut().map(|trace| {
            let before = game.ram().read_byte(0x613);
            assert_eq!(
                before,
                if trace.calls == 0 { 0 } else { 1 },
                "local Pass leaked into callback publication"
            );
            let marker = if trace.calls == 0 { 0xa5 } else { 0xa6 };
            trace.bytes.extend_from_slice(&[before, marker]);
            trace.bytes.extend_from_slice(&request.to_le_bytes());
            trace.calls += 1;
            game.ram().write_byte(0x613, marker);
            marker
        })
    });
    let returned = sm_game::message_boxes::display(request, game);
    if let Some(marker) = marker {
        assert_eq!(
            game.ram().read_byte(0x613),
            marker,
            "display changed caller continuation"
        );
        MESSAGE.with(|state| {
            state
                .borrow_mut()
                .as_mut()
                .unwrap()
                .bytes
                .extend_from_slice(&[game.ram().read_byte(0x613), returned]);
        });
    }
    returned
}

pub(super) fn begin_native(replacement: RomImage) {
    NATIVE.with(|state| {
        assert!(state.borrow().is_none());
        *state.borrow_mut() = Some(NativeTrace {
            replacement,
            addresses: Vec::new(),
        });
    });
}
pub(super) fn finish_native() -> Vec<u16> {
    NATIVE.with(|state| {
        state
            .borrow_mut()
            .take()
            .expect("armed native trace")
            .addresses
    })
}

/// Runs immediately before real RomPtr at the existing foreign boundary.
/// Allocation replacement is environmental input; no stream value is mocked.
pub(crate) fn native_read(bank: u8, address: u16, game: &mut Game) {
    NATIVE.with(|state| {
        if let Some(trace) = state.borrow_mut().as_mut() {
            assert_eq!(bank, 0x84);
            trace.addresses.push(address);
            if trace.addresses.len() == 2 {
                game.register_native_rom(trace.replacement.clone());
            }
        }
    });
}
