//! Scoped verifier-only environment changes at actual legacy callback edges.
//! The display and native renderer still execute their maintained algorithms.
#[cfg(sm_verify_dll)]
use crate::game_state::Game;
use crate::rom::RomImage;
use std::cell::RefCell;

struct MessageTrace {
    calls: usize,
    bytes: Vec<u8>,
}
struct NativeTrace {
    /// Installed by the verifier's native read hook.
    #[cfg(sm_verify_dll)]
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

/// Called inside the scheduler's existing released-owner display call, on its
/// lease. Each edge records $0613 before changing the environment; the actual
/// display then runs and $0613 is inspected again before returning to the
/// scheduler. The byte is the lease's.
#[cfg(sm_verify_dll)]
pub(crate) fn display(request: u16, runtime: &mut crate::game::session_runtime::Runtime<'_>) -> u8 {
    let marker = MESSAGE.with(|state| {
        let mut state = state.borrow_mut();
        state.as_mut().map(|trace| {
            let before = runtime.byte(0x613);
            assert_eq!(
                before,
                if trace.calls == 0 { 0 } else { 1 },
                "local Pass leaked into callback publication"
            );
            let marker = if trace.calls == 0 { 0xa5 } else { 0xa6 };
            trace.bytes.extend_from_slice(&[before, marker]);
            trace.bytes.extend_from_slice(&request.to_le_bytes());
            trace.calls += 1;
            runtime.set_byte(0x613, marker);
            marker
        })
    });
    let returned = sm_game::message_boxes::display(request, runtime);
    if let Some(marker) = marker {
        assert_eq!(
            runtime.byte(0x613),
            marker,
            "display changed caller continuation"
        );
        MESSAGE.with(|state| {
            state
                .borrow_mut()
                .as_mut()
                .unwrap()
                .bytes
                .extend_from_slice(&[runtime.byte(0x613), returned]);
        });
    }
    returned
}

pub(super) fn begin_native(replacement: RomImage) {
    NATIVE.with(|state| {
        assert!(state.borrow().is_none());
        *state.borrow_mut() = Some(NativeTrace {
            #[cfg(sm_verify_dll)]
            replacement,
            addresses: Vec::new(),
        });
    });
    // Other builds have no read hook to install the replacement.
    #[cfg(not(sm_verify_dll))]
    drop(replacement);
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

/// Whether a case traces native draw reads; only then do they reach the Game.
#[cfg(sm_verify_dll)]
pub(crate) fn native_read_traced() -> bool {
    NATIVE.with(|state| state.borrow().is_some())
}

/// The native read hook on the scheduler's lease: traced, it runs on the game,
/// which the lease exports to; untraced, the observer does nothing and the
/// lease keeps the foreign call's observation refresh.
#[cfg(sm_verify_dll)]
pub(crate) fn native_read_on_lease(
    runtime: &mut crate::game::session_runtime::Runtime<'_>,
    bank: u8,
    address: u16,
) {
    if native_read_traced() {
        runtime.foreign(|game| native_read(bank, address, game));
    } else {
        runtime.refresh_player_observations();
    }
}
/// Runs immediately before real RomPtr at the existing foreign boundary.
/// Allocation replacement is environmental input; no stream value is mocked.
#[cfg(sm_verify_dll)]
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
