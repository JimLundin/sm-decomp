//! Boss contacts run on the admitted Runtime's typed actor/player owners.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
pub(crate) use sm_game::actors::mother_brain_fight_owned::Contact;
pub(crate) fn samus_contact(boxes: u16, x: u16, y: u16, game: &mut Game) -> Contact {
    Runtime::new(game).owned_call(|host| {
        sm_game::actors::boss_part_hitboxes_owned::samus_contact(boxes, x, y, host)
    })
}
pub(crate) fn dead_torizo_contact(game: &mut Game) -> bool {
    Runtime::new(game).owned_call(sm_game::actors::boss_part_hitboxes_owned::dead_torizo_contact)
}
