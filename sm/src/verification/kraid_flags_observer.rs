//! Actual Kraid phase observer; inert outside explicit candidate capture.
#![forbid(unsafe_code)]
use crate::{game_state::Game, kraid_flags_case as case};
use std::cell::RefCell;
thread_local! {static CAPTURE:RefCell<Option<(Vec<u8>,u8)>>=const{RefCell::new(None)};}
pub fn begin(id: u32) {
    CAPTURE.with(|c| {
        let mut out = b"NKOR1".to_vec();
        case::record(&mut out, 0, &id.to_le_bytes());
        *c.borrow_mut() = Some((out, 0));
    });
}
pub fn observe(stage: u8, game: &Game) {
    observe_result(stage, 0xffff, game);
}
pub fn observe_result(stage: u8, result: u16, game: &Game) {
    CAPTURE.with(|c| {
        if let Some((out, next)) = c.borrow_mut().as_mut() {
            assert_eq!(stage, *next, "actual Kraid phase order");
            case::state(out, stage, result, [0; 16], game);
            *next += 1;
        }
    });
}
pub fn take(count: u8) -> Result<Vec<u8>, String> {
    CAPTURE.with(|c| match c.borrow_mut().take() {
        Some((out, n)) if n == count => Ok(out),
        _ => Err("incomplete actual Kraid phases".into()),
    })
}
pub fn clear() {
    CAPTURE.with(|c| *c.borrow_mut() = None);
}
