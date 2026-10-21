use crate::c_library::malloc;
pub use crate::types::{size_t, Input, Snes};
use ::core::mem::size_of;
pub unsafe fn input_init(snes: *mut Snes) -> *mut Input {
    let input = malloc(size_of::<Input>() as size_t) as *mut Input;
    (*input).snes = snes;
    (*input).type_0 = 1;
    (*input).currentState = 0;
    input
}
pub unsafe fn input_reset(input: *mut Input) {}
