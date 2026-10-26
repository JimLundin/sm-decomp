use crate::game::actors;
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
use crate::game::enemy_script::EnemyScript;
use crate::game::player_update::minimap;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;

use crate::game::music::{QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sprite_effect_runtime as sprite_effects;
use crate::game::visual_runtime::{initialize_draygon_layer, spawn_bg3_scroll};

pub use crate::types::Consts_A5;

crate::game::constants::typed_constants! { Consts_A5;
    addr_kDraygon_Ilist_99FC = 39420, addr_kDraygon_Ilist_99C0 = 39360,
    addr_kDraygon_Ilist_997A = 39290, addr_kDraygon_Ilist_9944 = 39236,
    addr_kDraygon_Ilist_98ED = 39149, addr_kDraygon_Ilist_9889 = 39049,
    addr_kDraygon_Ilist_9825 = 38949, addr_kDraygon_Ilist_9813 = 38931,
    addr_kDraygon_Ilist_97E7 = 38887, addr_kDraygon_Ilist_97D1 = 38865,
}

crate::type_aliases! {
    pub Enemy_Draygon = crate::game::enemy_layouts::EnemyLayout104;
}

crate::ram_map::ram_pointer_functions! {
    Get_Draygon(o: i32) -> Enemy_Draygon = [0xf78, o as isize];
}
crate::game::callback_dispatch! {
    pub unsafe fn CallDraygonFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10848691] Draygon_Func_10 [] {
        actors::draygon_flight::turret(game);
        printf(b"Wtf is varE26?\n\0" as *const u8 as *const c_char);
        actors::draygon_sweep::begin(false, game);
    };
    @external {
        10873627 => [actors::spore_spawn::descend(game)];
        10873755 => [actors::spore_spawn::begin_return(game)];
        10873838 => [actors::spore_spawn::finish_return(game)];
        10848027 => [actors::draygon_flight::opening(true, game)];
        10848139 => [actors::draygon_flight::opening(false, game)];
        10850693 => [actors::draygon_death::approach(game)];
        10850964 => [actors::draygon_death::ring(game)];
        10850987 => [actors::draygon_death::sink(game)];
        10849817 => [actors::draygon_grab::acquire(game)];
        10850064 => [actors::draygon_grab::cancel_grapple(game)];
        10850600 => [actors::draygon_grab::release(game)];
        10850644 => [actors::draygon_grab::depart(game)];
        10848244 => [actors::draygon_sweep::begin(true, game)];
        10848433 => [actors::draygon_sweep::advance(true, false, game)];
        10848546 => [actors::draygon_sweep::turn(true, game)];
        10848593 => [actors::draygon_sweep::advance(true, true, game)];
        10848768 => [actors::draygon_sweep::advance(false, false, game)];
        10848848 => [actors::draygon_sweep::turn(false, game)];
        10848912 => [actors::draygon_sweep::advance(false, true, game)];
        10849034 => [actors::draygon_flight::begin(true, game)];
        10849422 => [actors::draygon_flight::begin(false, game)];
        10849106 => [actors::draygon_flight::approach(true, game)];
        10849492 => [actors::draygon_flight::approach(false, game)];
        10849198 => [actors::draygon_flight::fire(true, game)];
        10849584 => [actors::draygon_flight::fire(false, game)];
        10846283 => [()];
        10849331 => [actors::draygon_flight::exit(true, game)];
        10849714 => [actors::draygon_flight::exit(false, game)];
        10850078 => [actors::draygon_grab::center(k, game)];
        10850262 => [actors::draygon_grab::orbit(k, game)]; 10850516 => [actors::draygon_grab::pause(k, game)];
        10850565 => [actors::draygon_grab::start_release(k, game)]; 10850596 => [actors::draygon_grab::attach(k, game)];
        10863757 => [actors::draygon_eye::advance(k, false, game)];
        10863891 => [actors::draygon_eye::advance(k, true, game)];
        10873626 => [()]; 10873682 => [actors::spore_spawn::orbit(k, game)];
    }
}
crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10847879] Draygon_Init [] {
        word_transfers!(game, descending, 25; le(target_palettes, 168) <- rom(0xa5a217, 24));
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
        (*Get_Draygon(word!(game.cur_enemy_index) as i32, game))
            .base
            .palette_index = 0xe00;
        *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 165;
        (*(ram_ptr!(game, LongPtr; 0x178c))).addr = (0xa59342i32) as u16;
        word!(game.flag_process_all_enemies = -1i32 as u16);
        (*Get_Draygon(0i32, game)).base.current_instruction = addr_kDraygon_Ilist_9889 as u16;
        (*Get_Draygon(0x40i32, game)).base.current_instruction = addr_kDraygon_Ilist_9944 as u16;
        (*Get_Draygon(0x80i32, game)).base.current_instruction = addr_kDraygon_Ilist_99FC as u16;
        (*Get_Draygon(0xc0i32, game)).base.current_instruction = addr_kDraygon_Ilist_9813 as u16;
        word!(game.enemy_bg2_tilemap_size = 1024);
        (*Get_Draygon(word!(game.cur_enemy_index) as i32, game)).var_A = (0xa5871bi32) as u16;
        initialize_draygon_layer(game);
        word!(game.room_loading_irq_handler = 12);
        spawn_bg3_scroll(game);
        (*Get_Draygon(0i32, game)).var_45 = 1;
        minimap::disable_for_boss(&mut crate::game::session_runtime::Runtime::new(game));
    };
    @external {
        10851661 => [actors::draygon_death::hurt(game)];
        10847996 => [actors::draygon_flight::update(true, |state, k, game| CallDraygonFunc(state, k, game), game)];
        10863750 => [actors::draygon_flight::update(false, |state, k, game| CallDraygonFunc(state, k, game), game)];
        10864025 => [actors::draygon_flight::initialize_part(false, game)];
        10864045 => [actors::draygon_flight::initialize_part(true, game)];
        10846223 => [actors::combat::select_frozen_ai(&mut Runtime::new(game))];
        10846243 => [actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize)];
        10846253 => [actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize)];
        10846273 => [actors::scheduling::update_frozen_enemy(&mut Runtime::new(game))];
        10873386 => [actors::spore_spawn::initialize(game)];
        10873619 => [actors::spore_spawn::update(game)];
        10874202 => [actors::spore_spawn::shot(game)];
        10874348 => [actors::spore_spawn::touch(game)];
        10851818 => [actors::draygon_death::touch(game)];
        10851847 => [actors::draygon_death::power_bomb(game)];
        10863723 => [actors::draygon_eye::initialize(game)]; }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10872671] Draygon_Instr_25 [jp] {
        let E = Get_Draygon(0, game);
        if (*E).var_0B as i32 - 40i32 & 0x8000 != 0 {
            (*E).var_0B = ((*E).var_0B).wrapping_add(8);
        }
    };
    [10872689] Draygon_Instr_24 [jp] { (*Get_Draygon(0, game)).var_2F = 0; };
    [10872877] Draygon_Instr_21 [jp.offset(2)] {
        let E = Get_Draygon(0, game);
        (*E).var_0B = jp.word(0);
        (*E).var_0C = jp.word(2);
    };
    [10872946] Draygon_Instr_22 [jp.offset(1)] { word!(game.kraid_unk9000 = jp.word(0)); };
    [10872956] Draygon_Instr_27 [jp] {
        let E = Get_Draygon(0, game);
        set_fields! {[(*E).base]; x_pos = 128, y_pos = 624,
            properties = (*E).base.properties & 0x5bff | 0xa000, };
    };
    [10872981] Draygon_Instr_23 [jp.offset(1)] { QueueSfx2_Max6(jp.word(0), game); };
    [10873018] Draygon_Instr_20 [jp.offset(1)] {
        (*Get_Draygon(word!(game.cur_enemy_index) as i32, game)).var_A = jp.word(0);
    };
    @external {
        10873034 => [actors::spore_spawn::fade(false, j, game)];
        10873116 => [actors::spore_spawn::fade(true, j, game)];
        10873198 => [actors::spore_spawn::emit_spore(j, game)];
        10873265 => [actors::spore_spawn::emit_hurt(j, game)];
        10873009 => [{ scatter(DropSource::SporeSpawn, k, game); j }];
        10851549 => [actors::draygon_flight::pose(j, game)];
        10852150 => [actors::draygon_eye::set_mode(k, j, game)];
        10852501 => [{ word!(game.room_loading_irq_handler = 12); j }];
        10852563 => [actors::draygon_flight::hide_parts(j, game)];
        10852591 => [actors::draygon_flight::tangible(j, game)];
        10853274 => [actors::draygon_grab::slam(j, game)];
        10853514 => [{ word!(game.room_loading_irq_handler = 12); j }];
        10853898 => [actors::draygon_flight::layer_offsets(j, game)];
        10854231 => [actors::draygon_eye::set_mode(0, j, game)];
        10854240 => [{ QueueSfx2_Max6(j.word(0), game); j.offset(1) }];
        10854254 => [{ QueueSfx3_Max6(j.word(0), game); j.offset(1) }];
        10863739 => [actors::draygon_eye::set_mode(0x40, j, game)];
        10852159 => [{ sprite_effects::draygon_random_sprite(21, game); j }];
        10852178 => [{ sprite_effects::draygon_random_sprite(3, game); j }];
        10852197 => [{ sprite_effects::draygon_random_sprite(29, game); j }];
        10852216 => [{ sprite_effects::draygon_random_sprite(24, game); j }];
        10854268 => [{ sprite_effects::draygon_spit(false, game); j }];
        10854318 => [{ sprite_effects::draygon_spit(true, game); j }];
    }
}
