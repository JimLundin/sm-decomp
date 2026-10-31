//! Spore Spawn runs in `sm_game::actors::spore_spawn`; its stem and boss
//! bits come from here.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub(crate) use sm_game::actors::spore_spawn::*;

impl Host for Game {
    fn update_spore_stem(&mut self) {
        crate::game::enemy_projectiles::update_spore_stem(self);
    }
    fn boss_bits(&mut self, area: u16) -> u8 {
        crate::game::gameplay_flag_runtime::boss_byte_at(area, self)
    }
    fn scatter_spore_spawn_drop(&mut self, k: u16) {
        use sm_game::enemy_projectiles::drops::DropSource::SporeSpawn;
        crate::game::enemy_projectiles::scatter_drop(SporeSpawn, k, self);
    }
}

// The named bank $A5 functions are the ones Draygon's function dispatch names.
const _: () = {
    use sm_game::actors::spore_spawn::function as f;
    assert!(
        0xa5_0000 | f::NOTHING as u32 == 10873626 && 0xa5_0000 | f::BEGIN_RETURN as u32 == 10873755
    );
    assert!(0xa5_0000 | f::DESCEND as u32 == 10873627 && 0xa5_0000 | f::ORBIT as u32 == 10873682);
    assert!(0xa5_0000 | f::FINISH_RETURN as u32 == 10873838);
};
