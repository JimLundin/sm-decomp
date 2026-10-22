pub use crate::types::{Input, Snes};
#[forbid(unsafe_code)]
pub fn input_init(snes: *mut Snes) -> *mut Input {
    Box::into_raw(Box::new(Input {
        snes,
        type_0: 1,
        currentState: 0,
    }))
}
pub unsafe fn input_reset(input: *mut Input) {}
