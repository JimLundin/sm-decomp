//! Actual save stages for captures and explicitly requested replay probes.
#![forbid(unsafe_code)]
use crate::flags_save_case as case;
use std::cell::RefCell;
thread_local! {static CAPTURE:RefCell<Option<(Vec<u8>,u8)>>=const {RefCell::new(None)};}
pub fn begin(id: u32) {
    CAPTURE.with(|capture| {
        let mut output = b"FSOR1".to_vec();
        case::record(&mut output, 0, &id.to_le_bytes());
        *capture.borrow_mut() = Some((output, 0));
    });
}
/// The save stage over a RAM image, which a Session lease projects; it is
/// taken only while a capture is active.
#[cfg(sm_verify_dll)]
pub fn observe_image(stage: u8, ram: impl FnOnce() -> Vec<u8>, sram: &sm_game::save_file::SaveRam) {
    if stage == 6 {
        // All in-memory save data and both checksum pairs have been written.
        crate::verify_probe::hit("save.sram.committed");
    }
    CAPTURE.with(|capture| {
        if let Some((output, next)) = capture.borrow_mut().as_mut() {
            assert_eq!(stage, *next, "actual save observer stage order");
            case::state_image(output, stage, [0; 16], ram(), sram);
            *next += 1;
        }
    });
}
/// Called only by the selected-slot Yes branch of the Game Over menu.
/// Slot scans and failed checksums cannot qualify a successful Continue.
#[cfg(sm_verify_dll)]
pub fn selected_continue(load_result: u8) {
    if load_result == 0 {
        crate::verify_probe::hit("save.continue.restored");
    }
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
