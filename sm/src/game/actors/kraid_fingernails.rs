//! Runs in `sm_game::actors::kraid_fingernails`.
#![forbid(unsafe_code)]
pub(crate) use sm_game::actors::kraid_fingernails::*;

// The named bank $A7 functions are the ones `sm_a7` dispatches on.
const _: () = {
    use sm_game::actors::kraid_fingernails::function as f;
    const fn at(word: u16) -> u32 {
        0xa7_0000 | word as u32
    }
    assert!(at(f::HANDLE_FUNCTION_TIMER) == 10991917 && at(f::INITIALISE_FINGERNAIL) == 10992992);
};
