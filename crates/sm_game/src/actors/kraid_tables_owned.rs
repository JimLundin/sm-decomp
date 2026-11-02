//! Actual lazy prepared table acquisitions; immutable loaded allocation is retained.
#![forbid(unsafe_code)]
use crate::actors::kraid_owned::Host;
use sm_data::kraid::*;
pub fn body_hitbox<H: Host>(game: &H, row: usize) -> BodyHitboxBand {
    BodyHitboxBand {
        bottom: game.loaded_table_word(
            BODY_HITBOX_ADDRESS,
            isize::try_from(2 * row).expect("table index"),
        ),
        left: game.loaded_table_word(
            BODY_HITBOX_ADDRESS,
            isize::try_from(2 * row + 1).expect("table index"),
        ),
    }
}
pub fn fingernail_boundary<H: Host>(game: &H, row: usize) -> FingernailBoundary {
    FingernailBoundary {
        left: game.loaded_table_word(
            FINGERNAIL_BOUNDARY_ADDRESS,
            isize::try_from(2 * row).expect("table index"),
        ),
        top: game.loaded_table_word(
            FINGERNAIL_BOUNDARY_ADDRESS,
            isize::try_from(2 * row + 1).expect("table index"),
        ),
    }
}
pub fn foot_target<H: Host>(game: &H, row: usize) -> FootTarget {
    FootTarget {
        x_position: game.loaded_table_word(
            FOOT_TARGET_ADDRESS,
            isize::try_from(2 * row).expect("table index"),
        ),
        movement: game.loaded_table_word(
            FOOT_TARGET_ADDRESS,
            isize::try_from(2 * row + 1).expect("table index"),
        ),
    }
}
pub fn bg_color<H: Host>(game: &H, index: usize) -> u16 {
    game.loaded_table_word(
        BG_PALETTE_ADDRESS,
        isize::try_from(index).expect("table index"),
    )
}
pub fn sprite_color<H: Host>(game: &H, index: usize) -> u16 {
    game.loaded_table_word(
        SPRITE_PALETTE_ADDRESS,
        isize::try_from(index).expect("table index"),
    )
}
