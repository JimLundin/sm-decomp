//! Actual save stages for captures and explicitly requested replay probes.
#![forbid(unsafe_code)]
use crate::{flags_save_case as case, game_state::Game};
use std::cell::RefCell;
thread_local! {static CAPTURE:RefCell<Option<(Vec<u8>,u8)>>=const {RefCell::new(None)};}
pub fn begin(id: u32) {
    CAPTURE.with(|capture| {
        let mut output = b"FSOR1".to_vec();
        case::record(&mut output, 0, &id.to_le_bytes());
        *capture.borrow_mut() = Some((output, 0));
    });
}
pub fn observe(stage: u8, game: &Game) {
    if stage == 6 {
        // All in-memory save data and both checksum pairs have been written.
        crate::verify_probe::hit("save.sram.committed");
    }
    CAPTURE.with(|capture| {
        if let Some((output, next)) = capture.borrow_mut().as_mut() {
            assert_eq!(stage, *next, "actual save observer stage order");
            case::state(output, stage, [0; 16], game);
            *next += 1;
        }
    });
}
pub fn take() -> Result<Vec<u8>, String> {
    CAPTURE.with(|capture| match capture.borrow_mut().take() {
        Some((output, 7)) => Ok(output),
        _ => Err("actual save did not complete all seven stages".into()),
    })
}
pub fn clear() {
    CAPTURE.with(|capture| *capture.borrow_mut() = None);
}
