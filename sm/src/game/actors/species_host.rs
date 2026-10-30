//! The compatibility side of the enemy species tables in `sm_game::actors::species`:
//! the routines that still run here, by name.
#![forbid(unsafe_code)]
use crate::game::actors::reflec;
use crate::game::actors::{self, api};
use crate::game::session_runtime::Runtime;
use crate::game::{sm_a6, sm_a7, sm_a9, sm_aa};
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
            LargeSnail { damage } => crate::game::large_snail::touch(damage, self),
            SporeSpawn => actors::spore_spawn::touch(self),
            CeresSteam => crate::game::ceres_doors::steam_touch(self),
            KraidArmAndFoot => sm_a7::Kraid_Touch_ArmFoot(self),
            KraidArm => sm_a7::KraidsArm_Touch(self),
            Phantoon => sm_a7::Phantoon_Touch(self),
            MotherBrainBrain => sm_a9::MotherBrainsBrain_Touch(self),
            GoldenTorizo => sm_aa::GoldTorizo_Touch(self),
        }
    }
}

impl sm_game::actors::species::ShotHost for Game {
    fn native_shot(&mut self, shot: sm_game::actors::species::NativeShot) {
        use sm_game::actors::species::NativeShot::*;
        match shot {
            Dud => api::create_dud(&mut Runtime::new(self)),
            LargeSnail => crate::game::large_snail::shot(self),
            SporeSpawn => actors::spore_spawn::shot(self),
            Ridley => sm_a6::Ridley_Shot(self),
            KraidArm { projectile } => sm_a7::Kraid_Arm_Shot(projectile, self),
            Phantoon => sm_a7::Phantoon_Shot(self),
            MotherBrainBody => sm_a9::MotherBrainsBody_Shot(self),
            MotherBrainBrain => sm_a9::MotherBrainsBrain_Shot(self),
            Torizo => sm_aa::Torizo_Shot(self),
            TorizoOutsideCrateria => sm_aa::Torizo_Func_8(self),
        }
    }
}

impl sm_game::actors::species::DrawHookHost for Game {
    fn native_draw_hook(&mut self, hook: sm_game::actors::species::NativeDrawHook) {
        use sm_game::actors::species::NativeDrawHook::*;
        match hook {
            ReflecPalette => reflec::palette(self),
            CeresRidleyBabyMetroidAndWings => sm_a6::Ridley_A2F2(self),
            WreckedShipRobotPalette => crate::game::actors::wrecked_ship_robots::palette(self),
            BlueFaceBlockPalette => sm_game::actors::blue_brinstar_face_block::cycle_palette(self),
            MotherBrainBrainAndNeck => sm_a9::MotherBrain_DrawBrainNeck_EnemyGfxDrawHook(self),
            MotherBrainBrain => sm_a9::MotherBrainsBrain_GfxDrawHook(self),
            DeadTorizo => sm_game::actors::corpses::DeadTorizo_MainGfxHook(self),
        }
    }
}
