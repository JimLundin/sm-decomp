//! The compatibility side of the enemy species tables in `sm_game::actors::species`:
//! the routines that still run here, by name.
#![forbid(unsafe_code)]
use crate::game::actors::reflec;
use crate::game::actors::{self, api};
use crate::game::session_runtime::Runtime;
use crate::game::sm_a9;
use crate::game_state::Game;

impl sm_game::actors::species::SpeciesHost for Game {
    fn unknown_routine(&mut self) {
        crate::rtl::Unreachable(self);
    }
}

impl sm_game::actors::species::TouchHost for Game {
    fn native_touch(&mut self, touch: sm_game::actors::species::NativeTouch) {
        use sm_game::actors::species::NativeTouch::*;
        match touch {
            Oum { damage } => crate::game::large_snail::touch(damage, self),
            SporeSpawn => actors::spore_spawn::touch(self),
            CeresSteam => crate::game::ceres_doors::steam_touch(self),
            MotherBrainBrain => sm_a9::touch_brain(self),
        }
    }
}

impl sm_game::actors::species::ShotHost for Game {
    fn native_shot(&mut self, shot: sm_game::actors::species::NativeShot) {
        use sm_game::actors::species::NativeShot::*;
        match shot {
            Dud => api::create_dud(&mut Runtime::new(self)),
            Oum => crate::game::large_snail::shot(self),
            SporeSpawn => actors::spore_spawn::shot(self),
            Ridley => sm_game::actors::ridley::shot(self),
            MotherBrainBody => sm_a9::shot_body(self),
            MotherBrainBrain => sm_a9::brain_shot(self),
        }
    }
}

impl sm_game::actors::species::DrawHookHost for Game {
    fn native_draw_hook(&mut self, hook: sm_game::actors::species::NativeDrawHook) {
        use sm_game::actors::species::NativeDrawHook::*;
        match hook {
            ReflecPalette => reflec::cycle_palette(self),
            CeresRidleyBabyMetroidAndWings => {
                sm_game::actors::ridley::draw_baby_metroid_and_door(self)
            }
            WorkRobotPalette => crate::game::actors::wrecked_ship_robots::palette(self),
            BlueFaceBlockPalette => sm_game::actors::blue_brinstar_face_block::cycle_palette(self),
            MotherBrainBrainAndNeck => sm_a9::draw_brain_neck_enemy_graphics_draw_hook(self),
            MotherBrainBrain => sm_a9::brain_graphics_draw_hook(self),
            DeadTorizo => sm_game::actors::corpses::torizo_main_graphics_hook(self),
        }
    }
}
