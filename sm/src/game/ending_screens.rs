//! Moved to `sm_game::ending_screens`.

// The ending's named cinematic functions are the ids bank $8B dispatches on.
const _: () = {
    use sm_game::ending_screens::function::*;
    let bank = 0x8b_0000u32;
    assert!(bank | IDLE as u32 == 9165725);
    assert!(bank | GUNSHIP_SPINNING_SLOW as u32 == 9166146);
    assert!(bank | GUNSHIP_FLY_INTO_CAMERA as u32 == 9166285);
    assert!(bank | IDLE_SAMUS_SUIT_CROSS_FADE as u32 == 9167461);
    assert!(bank | IDLE_SAMUS as u32 == 9167507);
    assert!(bank | NINTENDO_1994 as u32 == 9167581);
    assert!(bank | FADE_TO_WHITE as u32 == 9168010);
    assert!(bank | FADE_FROM_WHITE as u32 == 9168132);
};
