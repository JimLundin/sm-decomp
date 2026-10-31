//! The compatibility side of the enemy species tables in `sm_game::actors::species`:
//! the routines that still run here, by name.
#![forbid(unsafe_code)]
use crate::game::actors::api;
use crate::game::session_runtime::Runtime;
use crate::game::sm_a9;
use crate::game_state::Game;
use crate::ram_map::word;
use sm_game::actors::access::ActorKey;

impl sm_game::actors::host::EnemyHost for Game {
    fn current_enemy(&self) -> u16 {
        word!(self.CUR_ENEMY_INDEX)
    }
    fn set_current_enemy(&mut self, k: u16) {
        word!(self.CUR_ENEMY_INDEX = k);
    }
    fn random_number(&self) -> u16 {
        word!(self.RANDOM_NUMBER)
    }
    fn area_boss_flags(&mut self) -> u8 {
        crate::game::gameplay_flag_runtime::boss_byte_at(word!(self.AREA_INDEX), self)
    }
    fn angle_to_player(&mut self, k: u16) -> u16 {
        api::angle_to_player(&Runtime::new(self), ActorKey::from_legacy(k))
    }
    fn collided_projectile_direction(&self) -> u16 {
        word!(self.PROJECTILE_DIR[word!(self.COLLISION_DETECTION_INDEX)])
    }
}
impl sm_game::actors::species::SpeciesHost for Game {
    fn unknown_routine(&mut self) {
        crate::rtl::report_unreachable(self);
    }
}

impl sm_game::actors::species::TouchHost for Game {
    fn native_touch(&mut self, touch: sm_game::actors::species::NativeTouch) {
        use sm_game::actors::species::NativeTouch::*;
        match touch {
            MotherBrainBrain => sm_a9::brain_touch(self),
        }
    }
}

impl sm_game::actors::species::ShotHost for Game {
    fn native_shot(&mut self, shot: sm_game::actors::species::NativeShot) {
        use sm_game::actors::species::NativeShot::*;
        match shot {
            Dud => api::create_dud(&mut Runtime::new(self)),
            Ridley => sm_game::actors::ridley::shot(self),
            MotherBrainBody => sm_a9::body_shot(self),
            MotherBrainBrain => sm_a9::brain_shot(self),
        }
    }
}

impl sm_game::actors::species::DrawHookHost for Game {
    fn native_draw_hook(&mut self, hook: sm_game::actors::species::NativeDrawHook) {
        use sm_game::actors::species::NativeDrawHook::*;
        match hook {
            CeresRidleyBabyMetroidAndWings => {
                sm_game::actors::ridley::draw_baby_metroid_and_door(self)
            }
            BlueFaceBlockPalette => sm_game::actors::blue_brinstar_face_block::cycle_palette(self),
            MotherBrainBrainAndNeck => sm_a9::draw_brain_and_neck_hook(self),
            MotherBrainBrain => sm_a9::draw_brain_hook(self),
            DeadTorizo => sm_game::actors::corpses::torizo_corpse_graphics_drawn_hook(self),
        }
    }
}
