//! The baby Metroid runs in `sm_game::actors::baby_metroid`; PLM spawns come from here.
#![forbid(unsafe_code)]
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game_state::Game;
use crate::types::{Rect16U, SpawnHardcodedPlmArgs, VoidP};
use sm_game::actors::baby_metroid;
pub(crate) use sm_game::actors::baby_metroid::Host;

/// Check for enemy collision with rectangle ($A9:EF06), from a `Rect16U`.
pub(crate) fn separated_from_rect(k: u16, rect: Rect16U, game: &mut Game) -> u8 {
    baby_metroid::separated_from_rect(k, (rect.x, rect.y, rect.w, rect.h), game)
}

// The named function is the one `sm_a9` dispatches on (`BabyMetroid_Func_5`).
const _: () = assert!((0xa9_0000 | baby_metroid::LET_SIDEHOPPER_LIVE as u32) == 11137067);

impl Host for Game {
    fn spawn_hardcoded_plm(&mut self, block_x: u8, block_y: u8, header: u16) {
        spawn_hardcoded(
            SpawnHardcodedPlmArgs::new(block_x, block_y, header as VoidP),
            self,
        );
    }
}
