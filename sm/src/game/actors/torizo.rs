//! The Torizos run in `sm_game::actors::torizo`; movement, projectiles and the
//! shot reaction come from here.
#![forbid(unsafe_code)]
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::{self, api as actor_api};
use crate::game::constants::bank_86::addr_kEproj_BombTorizoLowHealthInitialDrool;
use crate::game::constants::bank_aa::{
    addr_kTorizo_Ilist_B1C8, addr_kTorizo_Ilist_CDE1, addr_kTorizo_Ilist_CE43,
    addr_kTorizo_Ilist_CEA5, addr_kTorizo_Ilist_CEFF, addr_kTorizo_Ilist_D1F1,
    addr_kTorizo_Ilist_D2AD,
};
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
pub(crate) use sm_game::actors::torizo::*;

const _: () = {
    use sm_game::actors::torizo::instruction_list as list;
    assert!(
        list::DEATH_SEQUENCE == addr_kTorizo_Ilist_B1C8 as u16
            && list::CAUGHT_SUPER_MISSILE_LEFT_LEFT_LEG == addr_kTorizo_Ilist_CDE1 as u16
            && list::CAUGHT_SUPER_MISSILE_LEFT_RIGHT_LEG == addr_kTorizo_Ilist_CE43 as u16
            && list::CAUGHT_SUPER_MISSILE_RIGHT_RIGHT_LEG == addr_kTorizo_Ilist_CEA5 as u16
            && list::CAUGHT_SUPER_MISSILE_RIGHT_LEFT_LEG == addr_kTorizo_Ilist_CEFF as u16
            && list::DODGE_TURNING_LEFT == addr_kTorizo_Ilist_D1F1 as u16
            && list::DODGE_TURNING_RIGHT == addr_kTorizo_Ilist_D2AD as u16
            && LOW_HEALTH_DROOL == addr_kEproj_BombTorizoLowHealthInitialDrool as u16
    );
};

impl Host for Game {
    fn move_horizontal_ignoring_slopes(&mut self, k: u16, amount: i32) -> bool {
        actor_api::move_horizontal(
            &mut Runtime::new(self),
            Key::from_legacy(k),
            amount,
            actor_api::SlopePolicy::Ignore,
        ) == actor_api::MovementResult::Blocked
    }

    fn snap_to_slopes(&mut self, k: u16) {
        actor_api::snap_to_slopes(&mut Runtime::new(self), Key::from_legacy(k));
    }

    fn spawn_projectile(&mut self, parameter: u16, k: u16, definition: u16) {
        SpawnEprojWithGfx(parameter, k, definition, self);
    }

    fn shot_reaction_only(&mut self) {
        actors::combat::shot_reaction(
            &mut Runtime::new(self),
            actors::api::ShotHandling::ReactionOnly,
        );
    }
}

impl Ai for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_aa_function(address, self);
    }

    fn wake_when_statue_crumbles(&mut self, k: u16) {
        crate::game::room_mechanisms::wait_statue(k, self);
    }

    fn boss_bit(&mut self, bit: u16) -> u16 {
        crate::game::gameplay_flag_runtime::CheckBossBitForCurArea(bit, self) as u16
    }

    fn set_boss_bit(&mut self, bit: u16) {
        crate::game::gameplay_flag_runtime::SetBossBitForCurArea(bit, self);
    }

    fn set_palette_targets(&mut self, palettes: Palettes) {
        use crate::game::palette_effects::torizo;
        let pair = match palettes {
            Palettes::BombStart => torizo::BOMB_START,
            Palettes::BombActive => torizo::BOMB_ACTIVE,
            Palettes::GoldenStart => torizo::GOLDEN_START,
            Palettes::GoldenActive => torizo::GOLDEN_ACTIVE,
        };
        torizo::set_targets(pair, self);
    }

    fn initialize_palette_targets(&mut self) {
        crate::game::palette_effects::torizo::initialize_targets(self);
    }

    fn flash_palette(&mut self) {
        crate::game::palette_effects::torizo::flash_current(self);
    }

    fn restore_palette(&mut self) {
        crate::game::palette_effects::torizo::restore_current(self);
    }

    fn health_palette(&mut self, health: u16) {
        crate::game::palette_effects::torizo::health_current(health, self);
    }

    fn spawn_palette_effect(&mut self, definition: u16) {
        crate::game::palette_effects::spawn(definition, self);
    }

    fn spawn_haze(&mut self) {
        crate::game::visual_runtime::initialize_bomb_torizo_haze(self);
    }
}

// The named functions, lists and projectiles are the decompilation's.
const _: () = {
    use crate::game::constants::bank_86 as eproj;
    use crate::game::constants::bank_aa as aa;
    use sm_game::actors::torizo::{function as f, list as l, movement as m, projectile as p};
    const fn at(word: u16) -> u32 {
        0xaa_0000 | word as u32
    }
    assert!(at(f::NOTHING) == 11191979 && at(f::SIMPLE_MOVEMENT) == 11191999);
    assert!(at(f::WAKE_WHEN_STATUE_CRUMBLES) == 11192006 && at(f::NORMAL_MOVEMENT) == 11192063);
    assert!(
        at(f::GOLDEN_WAKE_WHEN_SAMUS_BELOW_RIGHT) == 11195842
            && at(f::GOLDEN_SIMPLE_MOVEMENT) == 11195871
    );
    assert!(at(f::GOLDEN_NORMAL_MOVEMENT) == 11195878);
    assert!(at(m::WALKING) == 11192146 && at(m::ATTACKING) == 11192360);
    assert!(at(m::JUMPING_OR_FALLING) == 11192364 && m::NOTHING == aa::addr_locret_AAC95E as u16);
    assert!(at(m::GOLDEN_ATTACKING) == 11195885 && at(m::GOLDEN_WALKING) == 11195889);
    assert!(
        l::BLOW_UP_GUT == aa::addr_kTorizo_Ilist_B0E5 as u16
            && l::BLOW_UP_FACE == aa::addr_kTorizo_Ilist_B155 as u16
    );
    assert!(
        l::TURNING_LEFT == aa::addr_kTorizo_Ilist_B962 as u16
            && l::FALLING_FACING_LEFT == aa::addr_off_AABC78 as u16
    );
    assert!(
        l::FACELESS_TURNING_LEFT == aa::addr_kTorizo_Ilist_BD0E as u16
            && l::TURNING_RIGHT == aa::addr_kTorizo_Ilist_BDD8 as u16
    );
    assert!(
        l::FALLING_FACING_RIGHT == aa::addr_off_AAC0F2 as u16
            && l::FACELESS_TURNING_RIGHT == aa::addr_kTorizo_Ilist_C188 as u16
    );
    assert!(
        l::GOLDEN_TURNING_LEFT == aa::addr_kTorizo_Ilist_D203 as u16
            && l::GOLDEN_TURNING_RIGHT == aa::addr_kTorizo_Ilist_D2BF as u16
    );
    assert!(l::EXTENDED_SPRITEMAP == aa::addr_kTorizo_ExtSprmap_87D0 as u16);
    assert!(
        p::CONTINUOUS_DROOL == eproj::addr_stru_86A95B as u16
            && p::EXPLOSIVE_SWIPE == eproj::addr_kEproj_BombTorizoExplosiveSwipe as u16
    );
    assert!(
        p::LOW_HEALTH_EXPLOSION == eproj::addr_kEproj_BombTorizoLowHealthExplode as u16
            && p::DEATH_EXPLOSION == eproj::addr_kEproj_BombTorizoDeathExplosion as u16
    );
    assert!(
        p::BOMB_CHOZO_ORBS == eproj::addr_kEproj_BombTorizosChozoOrbs as u16
            && p::GOLDEN_CHOZO_ORBS == eproj::addr_kEproj_GoldenTorizosChozoOrbs as u16
    );
    assert!(
        p::BOMB_SONIC_BOOM == eproj::addr_kEproj_BombTorizoSonicBoom as u16
            && p::GOLDEN_SONIC_BOOM == eproj::addr_kEproj_GoldenTorizoSonicBoom as u16
    );
    assert!(
        p::LANDING_DUST_RIGHT_FOOT == eproj::addr_kEproj_TourianLandingDustCloudsRightFoot as u16
            && p::LANDING_DUST_LEFT_FOOT == eproj::addr_stru_86AFF3 as u16
    );
    assert!(
        p::GOLDEN_EGG == eproj::addr_stru_86B1C0 as u16
            && p::GOLDEN_SUPER_MISSILE == eproj::addr_kEproj_GoldenTorizoSuperMissile as u16
    );
    assert!(p::GOLDEN_EYE_BEAM == eproj::addr_kEproj_GoldenTorizoEyeBeam as u16);
    assert!(
        sm_game::actors::torizo::BOMB_BELLY_GLOW
            == crate::game::constants::bank_8d::addr_stru_8DF759 as u16
    );
    assert!(
        sm_game::actors::torizo::GOLDEN_BELLY_GLOW
            == crate::game::constants::bank_8d::addr_stru_8DF75D as u16
    );
};
