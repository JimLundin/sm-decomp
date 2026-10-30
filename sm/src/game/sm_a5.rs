use crate::game::actors;
pub use crate::game::prelude::*;
use sm_game::enemy_script::EnemyScript;
use {
    crate::game::enemy_projectiles::scatter_drop as scatter,
    sm_game::enemy_projectiles::drops::DropSource,
};

use crate::game::music::{QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sprite_effect_runtime as sprite_effects;
use crate::game::visual_runtime::initialize_draygon_layer;

crate::game::callback_dispatch! {
    pub fn CallDraygonFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10848691] Draygon_Func_10 [] {
        actors::draygon_flight::fire_wall_turret(game);
        unsafe { printf(b"Wtf is varE26?\n\0" as *const u8 as *const c_char) };
        actors::draygon_sweep::swoop_setup(false, game);
    };
    @external {
        10873627 => [actors::spore_spawn::descend(game)];
        10873755 => [actors::spore_spawn::begin_return(game)];
        10873838 => [actors::spore_spawn::finish_return(game)];
        10848027 => [actors::draygon_flight::fight_intro(true, game)];
        10848139 => [actors::draygon_flight::fight_intro(false, game)];
        10850693 => [actors::draygon_death::drift_to_death_spot(game)];
        10850964 => [actors::draygon_death::wait_for_evirs(game)];
        10850987 => [actors::draygon_death::buried_by_evirs(game)];
        10849817 => [actors::draygon_grab::chase_samus(game)];
        10850064 => [actors::draygon_grab::repelled_by_grapple(game)];
        10850600 => [actors::draygon_grab::flail_tail_and_fly_straight_up(game)];
        10850644 => [actors::draygon_grab::fly_straight_up(game)];
        10848244 => [actors::draygon_sweep::swoop_setup(true, game)];
        10848433 => [actors::draygon_sweep::swoop(true, false, game)];
        10848546 => [actors::draygon_sweep::swoop_apex(true, game)];
        10848593 => [actors::draygon_sweep::swoop(true, true, game)];
        10848768 => [actors::draygon_sweep::swoop(false, false, game)];
        10848848 => [actors::draygon_sweep::swoop_apex(false, game)];
        10848912 => [actors::draygon_sweep::swoop(false, true, game)];
        10849034 => [actors::draygon_flight::goop_setup(true, game)];
        10849422 => [actors::draygon_flight::goop_setup(false, game)];
        10849106 => [actors::draygon_flight::goop_move_until_in_range(true, game)];
        10849492 => [actors::draygon_flight::goop_move_until_in_range(false, game)];
        10849198 => [actors::draygon_flight::goop_firing(true, game)];
        10849584 => [actors::draygon_flight::goop_firing(false, game)];
        10846283 => [()];
        10849331 => [actors::draygon_flight::goop_move_until_off_screen(true, game)];
        10849714 => [actors::draygon_flight::goop_move_until_off_screen(false, game)];
        10850078 => [actors::draygon_grab::move_to_target(k, game)];
        10850262 => [actors::draygon_grab::rising_spiral(k, game)]; 10850516 => [actors::draygon_grab::tail_whip(k, game)];
        10850565 => [actors::draygon_grab::final_tail_whips(k, game)]; 10850596 => [actors::draygon_grab::move_samus_with_draygon(k, game)];
        10863757 => [actors::draygon_eye::look_at_samus(k, false, game)];
        10863891 => [actors::draygon_eye::look_at_samus(k, true, game)];
        10873626 => [()]; 10873682 => [actors::spore_spawn::orbit(k, game)];
    }
}
crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    @external {
        10847879 => [actors::draygon_flight::initialize(|game| initialize_draygon_layer(game), game)];
        10847996 => [actors::draygon_flight::main_ai(true, |state, k, game| CallDraygonFunc(state, k, game), game)];
        10863750 => [actors::draygon_flight::main_ai(false, |state, k, game| CallDraygonFunc(state, k, game), game)];



        10873386 => [actors::spore_spawn::initialize(game)];
        10873619 => [actors::spore_spawn::update(game)];
        10874202 => [actors::spore_spawn::shot(game)];
        10874348 => [actors::spore_spawn::touch(game)];

        }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @body checked;
    @external {
        10872671 => [{ actors::spore_spawn::increase_max_x_radius(game); j }];
        10872689 => [{ actors::spore_spawn::clear_damaged_flag(game); j }];
        10872877 => [actors::spore_spawn::set_max_x_radius_and_angle_delta(j, game)];
        10872946 => [{ word!(game.kraid_unk9000 = j.word(0)); j.offset(1) }];
        10872956 => [{ actors::spore_spawn::harden(game); j }];
        10872981 => [{ QueueSfx2_Max6(j.word(0), game); j.offset(1) }];
        10873018 => [{ actors::spore_spawn::set_function(j.word(0), game); j.offset(1) }];
        10873034 => [actors::spore_spawn::fade(false, j, game)];
        10873116 => [actors::spore_spawn::fade(true, j, game)];
        10873198 => [actors::spore_spawn::emit_spore(j, game)];
        10873265 => [actors::spore_spawn::emit_hurt(j, game)];
        10873009 => [{ scatter(DropSource::SporeSpawn, k, game); j }];

        10852501 => [{ word!(game.room_loading_irq_handler = 12); j }];

        10853514 => [{ word!(game.room_loading_irq_handler = 12); j }];

        10854240 => [{ QueueSfx2_Max6(j.word(0), game); j.offset(1) }];
        10854254 => [{ QueueSfx3_Max6(j.word(0), game); j.offset(1) }];
        10852159 => [{ sprite_effects::draygon_random_sprite(21, game); j }];
        10852178 => [{ sprite_effects::draygon_random_sprite(3, game); j }];
        10852197 => [{ sprite_effects::draygon_random_sprite(29, game); j }];
        10852216 => [{ sprite_effects::draygon_random_sprite(24, game); j }];
        10854268 => [{ sprite_effects::draygon_spit(false, game); j }];
        10854318 => [{ sprite_effects::draygon_spit(true, game); j }];
    }
}
