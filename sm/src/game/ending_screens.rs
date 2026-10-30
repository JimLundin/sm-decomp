//! Moved to `sm_game::ending_screens`.
pub(crate) use sm_game::ending_screens::*;

// The ending's named cinematic functions are the ids bank $8B dispatches on.
const _: () = {
    use sm_game::ending_screens::function::*;
    let bank = 0x8b_0000u32;
    assert!(bank | IDLE as u32 == 9165725);
    assert!(bank | SLOW_GUNSHIP_SPIN as u32 == 9166146);
    assert!(bank | GUNSHIP_BACKDROP_FADE as u32 == 9166285);
    assert!(bank | POSTCREDITS_MESSAGE_FADING_SUIT as u32 == 9167461);
    assert!(bank | POSTCREDITS_MESSAGE as u32 == 9167507);
    assert!(bank | POSTCREDITS_TEXT_HOLD as u32 == 9167581);
    assert!(bank | POSTCREDITS_ZOOMED as u32 == 9168010);
    assert!(bank | POSTCREDITS_FIXED_COLOR_FADE as u32 == 9168132);
};
