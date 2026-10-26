//! Actual candidate bridges and maintained bank-controller callers; shared frozen fixtures.
use super::sprite_effect_cases::{self, Operation};
use crate::{
    game::{actors::yapping_maw, sm_a3, sprite_effect_runtime as pool},
    game_state::Game,
};
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= super::sprite_alias_cases::FIRST {
        return super::sprite_alias_cases::capture_case(id, run);
    }
    sprite_effect_cases::capture_case(id, run)
}
fn run(operation: Operation, game: &mut Game) -> u16 {
    unsafe {
        match operation {
            Operation::PopulationClear => pool::clear_sprites(game),
            Operation::RetireAll => pool::retire_all(game),
            Operation::Spawn => return pool::create_sprite(0x1234, 0xabcd, 0, 0xbeef, game),
            Operation::SpawnTwice => {
                pool::create_sprite(0x1234, 0xabcd, 0, 0xbeef, game);
                return pool::create_sprite(0x4321, 0xdcba, 0, 0x0a55, game);
            }
            Operation::Update => pool::update_sprites(game),
            Operation::UpdateTwice => {
                pool::update_sprites(game);
                pool::update_sprites(game);
            }
            Operation::Draw => pool::draw_sprites(game),
            Operation::MetroidInit => sm_a3::Metroid_Init(game),
            Operation::MetroidHurt => sm_a3::Metroid_Hurt(game),
            Operation::MetroidMain => sm_a3::Metroid_Main(game),
            Operation::MetroidFrozen => sm_a3::Metroid_Frozen(game),
            Operation::MawAppearance => yapping_maw::appearance(game),
            Operation::DraygonMotion(parameter) => pool::draygon_motion(parameter, game),
            Operation::DraygonStart => pool::draygon_spawn_ring(game),
            Operation::DraygonFour => pool::draygon_spawn_trails(game),
        }
    }
    0
}
