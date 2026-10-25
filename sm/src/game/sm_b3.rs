use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
pub use crate::game::constants::bank_86::*;
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
use crate::game::enemy_script::EnemyScript;
use crate::game::gameplay_flag_runtime;
use crate::game::music::QueueMusic_Delayed8;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;

use crate::game::enemy_projectiles::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::gameplay_flag_runtime::{CheckEventHappened, SetBossBitForCurArea};
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game::sm_80::{NextRandom, QueueSfx2_Max6};
use crate::game::sm_a0::{CallEnemyPreInstr, ConvertAngleToXy, SignExtend8};
use crate::game::sprite_effect_runtime::create_sprite;

pub use crate::types::Consts_B3;

crate::game::constants::typed_constants! { Consts_B3;
    addr_kEscapeDachora_Ilist_E964 = 59748, addr_kEscapeEtecoon_Ilist_E5DA = 58842,
    addr_kEscapeEtecoon_Ilist_E5AE = 58798, addr_kEscapeEtecoon_Ilist_E582 = 58754,
    addr_kEscapeEtecoon_Ilist_E556 = 58710, addr_loc_B3E28C = 57996,
    addr_stru_B3E150 = 57680, addr_kBotwoon_Ilist_9389 = 37769,
    addr_kBrinstarYellowPipeBug_Ilist_8F38 = 36664, addr_kBrinstarYellowPipeBug_Ilist_8F24 = 36644,
    addr_kBrinstarYellowPipeBug_Ilist_8F10 = 36624, addr_kBrinstarYellowPipeBug_Ilist_8EFC = 36604,
    addr_kNorfairPipeBug_Ilist_8B45 = 35653, addr_kNorfairPipeBug_Ilist_8B21 = 35617,
    addr_kNorfairPipeBug_Ilist_8B05 = 35589, addr_kNorfairPipeBug_Ilist_8AE1 = 35553,
    addr_kBrinstarPipeBug_Ilist_8A1D = 35357, addr_kBrinstarPipeBug_Ilist_87AB = 34731,
    addr_kUnusedSpinningTurtleEye_Ilist_86A7 = 34471, addr_kSpritemap_Nothing_B3 = 32845,
}
pub use crate::types::Enemy_Botwoon;
crate::type_aliases! {
    pub Enemy_EscapeDachora, Enemy_EscapeEtecoon = crate::game::enemy_layouts::EnemyLayout8;
}

pub type Enemy_PipeBug = crate::game::enemy_layouts::EnemyLayout72;

const RomPtr_B3: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xb3>;
crate::game::constants::typed_constants! { i32;
    fnnullsub_171_B3 = 0xb3807b, fnBrinstarPipeBug_PreInstr_1 = 0xb38880,
    fnBrinstarPipeBug_PreInstr_2 = 0xb38890, fnBrinstarPipeBug_PreInstr_3 = 0xb388e3,
    fnBrinstarPipeBug_PreInstr_4 = 0xb3891c, fnBrinstarPipeBug_PreInstr_5 = 0xb3897e,
    fnNorfairPipeBug_Func_5 = 0xb38cff,
    fnBotwoon_Func_26 = 0xb39dc0, fnBotwoon_Func_27 = 0xb39e7d,
    fnBotwoon_Func_28 = 0xb39ee0, fnBotwoon_Func_29 = 0xb39f34,
    fnBotwoon_Func_30 = 0xb39f7a,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_Botwoon(o: i32) -> Enemy_Botwoon = [0xf78, o as isize];
    Get_EscapeDachora(o: i32) -> Enemy_EscapeDachora = [0xf78, o as isize];
    Get_EscapeEtecoon(o: i32) -> Enemy_EscapeEtecoon = [0xf78, o as isize];
    Get_PipeBug(o: i32) -> Enemy_PipeBug = [0xf78, o as isize];
}
static g_word_B39E77: [i16; 3] = [2, 3, 4];
pub fn UnusedSpinningTurtleEye_Main() {}
pub unsafe fn NorfairPipeBug_8BA8(game: &mut Game) {
    unsafe {
        if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) != 0 {
            let PipeBug = Get_PipeBug(word!(game.cur_enemy_index) as i32, game);
            (*PipeBug).base.properties =
                ((*PipeBug).base.properties) | ((kEnemyProps_Invisible) as u16);
            (*PipeBug).var_A = (0xb38bcdi32) as u16;
            (*PipeBug).base.x_pos = (*PipeBug).var_D;
            (*PipeBug).base.y_pos = (*PipeBug).var_E;
        }
    }
}
pub unsafe fn NorfairPipeBug_Func_3(k: u16, game: &mut Game) {
    unsafe {
        (*Get_PipeBug(k as i32, game)).var_A = (0xb38ca6i32) as u16;
        (*Get_PipeBug(k as i32 + 64i32, game)).var_A = (0xb38ca6i32) as u16;
        (*Get_PipeBug(k as i32 + 128i32, game)).var_A = (0xb38ca6i32) as u16;
        (*Get_PipeBug(k as i32 + 192i32, game)).var_A = (0xb38ca6i32) as u16;
        (*Get_PipeBug(k as i32 + 256i32, game)).var_A = (0xb38ca6i32) as u16;
        (*Get_PipeBug(k as i32, game)).var_02 = 104;
        (*Get_PipeBug(k as i32 + 64i32, game)).var_02 = 96;
        (*Get_PipeBug(k as i32 + 128i32, game)).var_02 = 88;
        (*Get_PipeBug(k as i32 + 192i32, game)).var_02 = 112;
        (*Get_PipeBug(k as i32 + 256i32, game)).var_02 = 120;
        (*Get_PipeBug(k as i32, game)).var_F = (0xb38cffi32) as u16;
        (*Get_PipeBug(k as i32 + 64i32, game)).var_F = (0xb38d0ci32) as u16;
        (*Get_PipeBug(k as i32 + 128i32, game)).var_F = (0xb38d4ei32) as u16;
        (*Get_PipeBug(k as i32 + 192i32, game)).var_F = (0xb38d90i32) as u16;
        (*Get_PipeBug(k as i32 + 256i32, game)).var_F = (0xb38dd2i32) as u16;
    }
}
macro_rules! move_pipe_bug_to_upper_limit {
    ($enemy:ident, $game:ident, $distance:literal) => {
        (*$enemy).var_C = (*$enemy).var_C.wrapping_add(1);
        words!(
            (*$enemy).base,
            y_pos,
            y_subpos += words!((
                crate::rtl::rom_table_word!($game, 0xa28187, 66),
                crate::rtl::rom_table_word!($game, 0xa28187, 67)
            ))
        );
        if (((*$enemy).base.y_pos as i32 + $distance - (*$enemy).var_01 as i32) as i16) < 0 {
            (*$enemy).base.y_pos = ((*$enemy).var_01).wrapping_sub($distance);
            (*$enemy).base.y_subpos = 0;
            (*$enemy).var_A = ((0xb38e5ai32) as u16);
        }
    };
}

macro_rules! move_pipe_bug_to_lower_limit {
    ($enemy:ident, $game:ident, $signed_distance:literal, $distance:literal) => {
        (*$enemy).var_C = (*$enemy).var_C.wrapping_add(1);
        words!(
            (*$enemy).base,
            y_pos,
            y_subpos += words!((
                crate::rtl::rom_table_word!($game, 0xa28187, 64),
                crate::rtl::rom_table_word!($game, 0xa28187, 65)
            ))
        );
        if (((*$enemy).base.y_pos as i32 - $signed_distance - (*$enemy).var_01 as i32) as i16) >= 0
        {
            (*$enemy).base.y_pos = ((*$enemy).var_01).wrapping_add($distance);
            (*$enemy).base.y_subpos = 0;
            (*$enemy).var_A = ((0xb38e5ai32) as u16);
        }
    };
}

crate::game::callback_dispatch! {
    pub unsafe fn CallBugInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11766733] NorfairPipeBug_Func_1 [] {
        if byte!((*(Get_PipeBug as unsafe fn(i32, &mut Game) -> *mut Enemy_PipeBug)(
        word!(game.cur_enemy_index) as i32,
        game,
    ))
    .parameter_2; 0)
            != 0
            && (*Get_PipeBug(word!(game.cur_enemy_index) as i32 + 64i32, game)).var_A as i32
                == 0xb38bcd & 0xffff
            && (*Get_PipeBug(word!(game.cur_enemy_index) as i32 + 128i32, game)).var_A as i32
                == 0xb38bcd & 0xffff
            && (*Get_PipeBug(word!(game.cur_enemy_index) as i32 + 192i32, game)).var_A as i32
                == 0xb38bcd & 0xffff
            && (*Get_PipeBug(word!(game.cur_enemy_index) as i32 + 256i32, game)).var_A as i32
                == 0xb38bcd & 0xffff
        {
            (*Get_PipeBug(word!(game.cur_enemy_index) as i32, game)).var_A = (0xb38bffi32) as u16;
        }
    };
    [11766783] NorfairPipeBug_Func_2 [] {
        let k: i32 = word!(game.cur_enemy_index) as i32;
        let PipeBug = Get_PipeBug(word!(game.cur_enemy_index) as i32, game);
        if ({ let (actor, distance) = (word!(game.cur_enemy_index), byte!((*PipeBug).parameter_2; 0) as u16); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
            && (((*PipeBug).base.y_pos) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16) >= 0
        {
            (*PipeBug).var_C = (*PipeBug).var_C.wrapping_add(1);
            (*PipeBug).base.instruction_timer = 1;
            (*PipeBug).base.timer = 0;
            if (({ let actor = k as u16; actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
                (*Get_PipeBug(k, game)).base.current_instruction =
                    addr_kNorfairPipeBug_Ilist_8B21 as u16;
                (*Get_PipeBug(k + 64i32, game)).base.current_instruction =
                    addr_kNorfairPipeBug_Ilist_8B21 as u16;
                (*Get_PipeBug(k + 128i32, game)).base.current_instruction =
                    addr_kNorfairPipeBug_Ilist_8B21 as u16;
                (*Get_PipeBug(k + 192i32, game)).base.current_instruction =
                    addr_kNorfairPipeBug_Ilist_8B21 as u16;
                (*Get_PipeBug(k + 256i32, game)).base.current_instruction =
                    addr_kNorfairPipeBug_Ilist_8B21 as u16;
            } else {
                (*PipeBug).base.current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as u16;
                (*Get_PipeBug(k + 64i32, game)).base.current_instruction =
                    addr_kNorfairPipeBug_Ilist_8AE1 as u16;
                (*Get_PipeBug(k + 128i32, game)).base.current_instruction =
                    addr_kNorfairPipeBug_Ilist_8AE1 as u16;
                (*Get_PipeBug(k + 192i32, game)).base.current_instruction =
                    addr_kNorfairPipeBug_Ilist_8AE1 as u16;
                (*Get_PipeBug(k + 256i32, game)).base.current_instruction =
                    addr_kNorfairPipeBug_Ilist_8AE1 as u16;
            }
            NorfairPipeBug_Func_3(k as u16, game);
        }
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_PipeBug(word!(game.cur_enemy_index) as i32, game); };
    [11766950] NorfairPipeBug_Func_4 [] {
        (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
        words!(
            (*E).base,
            y_pos,
            y_subpos += words!((
                crate::rtl::rom_table_word!(game, 0xa28187, 66),
                crate::rtl::rom_table_word!(game, 0xa28187, 67)
            ))
        );
        if (((*E).base.y_pos) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16) < 0 {
            (*E).var_A = (*E).var_F;
            (*E).var_01 = (*E).base.y_pos;
            (*E).base.instruction_timer = 1;
            (*E).base.timer = 0;
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
                (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8B21 as u16;
            } else {
                (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as u16;
            }
        }
    };
    [11767052] NorfairPipeBug_Func_6 [] {
        move_pipe_bug_to_upper_limit!(E, game, 16);
    };
    [11767118] NorfairPipeBug_Func_7 [] {
        move_pipe_bug_to_upper_limit!(E, game, 32);
    };
    [11767184] NorfairPipeBug_Func_8 [] {
        move_pipe_bug_to_lower_limit!(E, game, 16i32, 16);
    };
    [11767250] NorfairPipeBug_Func_9 [] {
        move_pipe_bug_to_lower_limit!(E, game, 32i32, 32);
    };
    [11767316] NorfairPipeBug_Func_10 [] {
        let v2: i32 = (*E).var_B as i32 >> 1;
        words!(
            (*E).base,
            x_pos,
            x_subpos += words!((
                crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 2) as isize),
                crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 3) as isize)
            ))
        );
    };
    [11767349] NorfairPipeBug_Func_11 [] {
        let v2: i32 = (*E).var_B as i32 >> 1;
        words!(
            (*E).base,
            x_pos,
            x_subpos += words!((
                crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize),
                crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1) as isize)
            ))
        );
    };
    [11767386] NorfairPipeBug_Func_12 [] {
        (*E).var_C = (*E).var_C.wrapping_add(1);
        if (((*E).var_C) as i16).wrapping_sub(((*E).var_02) as i16) >= 0 {
            (*E).var_C = 0;
            (*E).base.instruction_timer = 1;
            (*E).base.timer = 0;
            (*E).var_A = (0xb38e14i32) as u16;
            (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8B05 as u16;
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) == 0 {
                (*E).var_A = (0xb38e35i32) as u16;
                (*E).base.current_instruction = addr_kNorfairPipeBug_Ilist_8B45 as u16;
            }
        }
    };
    BrinstarYellowPipeBug_Init [] {
        (*E).var_07 = (*E).base.x_pos;
        (*E).var_08 = (*E).base.y_pos;
        set_fields! {[(*E).base], 0; x_subpos, y_subpos}
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as u16;
        if (*E).parameter_1 == 0 {
            (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as u16;
        }
        let v1: i32 = 8 * (*E).parameter_2 as i32 >> 1;
        set_fields! {[(*E)]; var_01 = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize),
            var_00 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1) as isize),
            var_03 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2) as isize),
            var_02 = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3) as isize),
            var_A = (0xb38fb5i32) as u16, var_06 = 0, };
    };
    BrinstarYellowPipeBug_Main [] {
        CallBugInstr(
            (((*E).var_A) as u32) | (0xb30000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11767733] BrinstarYellowPipeBug_Func_1 [] {

        let samus_in_x_range: bool = if (*E).parameter_1 != 0 {
            let mut SamusEnemyDelta_X: i16 =
                ({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) as i16;
            SamusEnemyDelta_X < 0 && SamusEnemyDelta_X as i32 + 192 & 0x8000 == 0
        } else {
            let mut v2: i16 = ({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) as i16;
            v2 >= 0 && v2 as i32 - 192i32 & 0x8000 != 0
        };
        if samus_in_x_range && ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x30); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0 {
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
            (*E).var_20 = 24;
            (*E).var_A = (0xb38ff5i32) as u16;
        };
    };
    [11767848] BrinstarYellowPipeBug_Func_3 [] {
        BrinstarYellowPipeBug_Func_4(word!(game.cur_enemy_index), game);
        if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) != 0 {
            (*E).base.x_pos = (*E).var_07;
            (*E).base.y_pos = (*E).var_08;
            set_fields! {[(*E).base], 0; x_subpos, y_subpos}
            (*E).var_A = (0xb38fb5i32) as u16;
            (*E).var_06 = 0;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        } else if (*E).var_06 == 0
            && (*E).base.x_pos as i32 - word!(game.samus_x_pos) as i32 - 48i32 & 0x8000 != 0
        {
            (*E).var_A = (0xb3915ai32) as u16;
            set_fields! {[(*E)], 0; var_C, var_D}
            (*E).var_E = 40;
            (*E).var_F = 1;
            (*E).var_04 = (*E).base.x_pos;
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F10 as u16, };
        }
    };
    [11767997] BrinstarYellowPipeBug_Func_5 [] {
        BrinstarYellowPipeBug_Func_6(word!(game.cur_enemy_index), game);
        if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) != 0 {
            (*E).base.x_pos = (*E).var_07;
            (*E).base.y_pos = (*E).var_08;
            set_fields! {[(*E).base], 0; x_subpos, y_subpos}
            (*E).var_A = (0xb38fb5i32) as u16;
            (*E).var_06 = 0;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        } else if (*E).var_06 == 0
            && word!(game.samus_x_pos) as i32 - (*E).base.x_pos as i32 - 48i32 & 0x8000 != 0
        {
            (*E).var_A = (0xb391d8i32) as u16;
            set_fields! {[(*E)], 0; var_C, var_D}
            (*E).var_E = 40;
            (*E).var_F = 1;
            (*E).var_04 = (*E).base.x_pos;
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F38 as u16, };
        }
    };
    [11768154] BrinstarYellowPipeBug_Func_7 [] {
        if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) != 0 {
            (*E).base.x_pos = (*E).var_07;
            (*E).base.y_pos = (*E).var_08;
            set_fields! {[(*E).base], 0; x_subpos, y_subpos}
            (*E).var_A = (0xb38fb5i32) as u16;
            (*E).var_06 = 0;
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as u16,
                properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16), };
        } else {
            BrinstarYellowPipeBug_Func_4(word!(game.cur_enemy_index), game);
            if (*E).var_F != 0 {
                BrinstarYellowPipeBug_Func_10(word!(game.cur_enemy_index), game);
            } else {
                BrinstarYellowPipeBug_Func_9(word!(game.cur_enemy_index), game);
                if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_08) as i16) < 0 {
                    set_fields! {[(*E)], 1; var_06, var_F}
                    (*E).var_A = (0xb39028i32) as u16;
                    set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                        current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as u16, };
                }
            }
        };
    };
    [11768280] BrinstarYellowPipeBug_Func_8 [] {
        if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) != 0 {
            (*E).base.x_pos = (*E).var_07;
            (*E).base.y_pos = (*E).var_08;
            set_fields! {[(*E).base], 0; x_subpos, y_subpos}
            (*E).var_A = (0xb38fb5i32) as u16;
            (*E).var_06 = 0;
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as u16,
                properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16), };
        } else {
            BrinstarYellowPipeBug_Func_6(word!(game.cur_enemy_index), game);
            if (*E).var_F != 0 {
                BrinstarYellowPipeBug_Func_10(word!(game.cur_enemy_index), game);
            } else {
                BrinstarYellowPipeBug_Func_9(word!(game.cur_enemy_index), game);
                if (((*E).var_08) as i16).wrapping_sub(((*E).base.y_pos) as i16) >= 0 {
                    set_fields! {[(*E)], 1; var_06, var_F}
                    (*E).base.instruction_timer = 1;
                    (*E).base.timer = 0;
                    (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as u16;
                    (*E).var_A = (0xb390bdi32) as u16;
                }
            }
        };
    };
    @external {
        11767039 => [NorfairPipeBug_Func_5(k, game)];
        11767797 => [{
                    BrinstarYellowPipeBug_Func_2(word!(game.cur_enemy_index), game);
                }];
    }
}
pub unsafe fn Botwoon_QueueExplosionSfx(game: &mut Game) {
    QueueSfx2_Max6(0x24, game);
}
crate::game::callback_dispatch! {
    pub unsafe fn CallBotwoonFuncVarD(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_Botwoon(word!(game.cur_enemy_index) as i32, game); };
    Botwoon_Init [] {
        if (gameplay_flag_runtime::boss_byte_at(4, game) & 2) != 0 {
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x4, 0xb797 as VoidP), game);
            crate::game::room_mechanisms::open_passage(game);
            (*E).base.current_instruction = addr_kBotwoon_Ilist_9389 as u16;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        } else {
            let v3: u16 = 24;
            (*E).var_A = 24;
            do_until! { v5;
    SpawnEprojWithGfx(
        v3,
        word!(game.cur_enemy_index),
        addr_kEproj_BotwoonsBody as u16,
        game,
    );
    (*E).var_A = (*E).var_A.wrapping_sub(1);
    let mut v5: bool = (((*E).var_A) as i16).wrapping_sub(1) < 0;
    (*E).var_A = (*E).var_A.wrapping_sub(1);
}
            (*E).base.current_instruction = addr_kBotwoon_Ilist_9389 as u16;
            set_fields! {[(*E)]; var_3B = addr_kBotwoon_Ilist_9389 as u16,
                var_D = (0xb39878i32) as u16, var_E = (0xb39bb7i32) as u16,
                var_F = (0xb39dc0i32) as u16, var_20 = 256,
                var_38 = crate::rtl::rom_table_word!(game, 0xb394bb, 0),
                var_C = crate::rtl::rom_table_word!(game, 0xb394bb, 1), };
            set_fields! {[(*E)], 1; var_33, var_34, var_59}
            (*E).var_36 = -1i32 as u16;
            (*E).var_37 = 0;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            let mut health: u16 = (*E).base.health;
            (*E).var_5B = health;
            health = health >> 1;
            (*E).var_5C = health;
            (*E).var_5D = health >> 1;
            let x_pos: u16 = (*E).base.x_pos;
            (*E).var_50 = x_pos;
            (*E).var_52 = x_pos;
            (*E).var_54 = x_pos;
            (*E).var_56 = x_pos;
            let y_pos: u16 = (*E).base.y_pos;
            set_fields! {[(*E)]; var_51 = y_pos, var_53 = y_pos, var_55 = y_pos, var_57 = y_pos,
                var_4E = (((*E).base.palette_index as i32 >> 4) + 256) as u16, };
        };
    };
    Botwoon_Func_3 [] {
        if (*E).var_2F != 0 && (*E).var_3E != 0 {
            (*E).var_30 = 1;
            (*E).var_D = (0xb39a46i32) as u16;
            (*E).var_22 = 240;
            set_fields! {[(*E)], 0; var_2F, var_3E}
        }
    };
    Botwoon_HealthBasedPalHandling [] {
        if (*E).var_4F != 16
            && (((*E).base.health) as i16).wrapping_sub(
                (crate::rtl::rom_table_word!(game, 0xb3981b, ((*E).var_4F >> 1) as isize)) as i16,
            ) < 0
        {
            let mut r18: u16 = (16u16).wrapping_mul((*E).var_4F);
            let mut r20: u16 = (*E).var_4E;
            do_until! { r20 == 512 ;
    crate::update_place! { previous fresh5 = r18 => r18.wrapping_add(1); }
    word!(
        game.palette_buffer[(r20 >> 1)] =
            crate::rtl::rom_table_word!(game, 0xb3971b, (fresh5 >> 1) as isize)
    );
    r18 = r18.wrapping_add(1);
    r20 = (r20).wrapping_add(2);
}
            (*E).var_4F = ((*E).var_4F).wrapping_add(2);
        }
    };
    [11769976] Botwoon_Func_6 [] {
        let v1: u16 = ((*E).var_20).wrapping_sub(1);
        (*E).var_20 = v1;
        if v1 == 0 {
            (*E).var_D = (0xb3989di32) as u16;
        }
    };
    [11770013] Botwoon_Func_7 [] {
        if (*E).var_3C != 0 {
            (*E).var_3C = 0;
            let mut r18: u8 = 0;
            if (*E).var_33 == 0 && (*E).var_59 == 0 && (*E).var_3F == 0 {
                r18 = ((NextRandom(game)) as u8) & (0xeu8);
            }
            (*E).var_59 = 0;
            if r18 == 0 || r18 == 2 || r18 == 4 {
                Botwoon_Func_8(game);
            } else {
                if r18 != 6 && r18 != 8 && r18 != 10 && r18 != 12 && r18 != 14 {
                    Unreachable(game);
                }
                Botwoon_Func_9(game);
            }
        } else {
            CallBotwoonFuncVarE((((*E).var_E) as u32) | (0xb30000u32), game);
            crate::game::enemy_projectiles::record_botwoon_head(game);
            crate::game::enemy_projectiles::follow_botwoon_history(game);
            CallEnemyPreInstr((((*E).var_F) as u32) | (0xb30000u32), game);
            crate::game::enemy_projectiles::orient_botwoon_body(game);
            crate::game::enemy_projectiles::advance_botwoon_history(game);
            Botwoon_Func_31(word!(game.cur_enemy_index), game);
        };
    };
    Botwoon_Func_8 [] {
        set_fields! {[(*E)]; var_D = (0xb399a4i32) as u16, var_E = (0xb3e250i32) as u16,
            var_21 = 0, var_F = (0xb39dc0i32) as u16, var_4B = 0, };
        Botwoon_Func_10(word!(game.cur_enemy_index), game);
    };
    Botwoon_Func_9 [] {
        (*E).var_D = (0xb399e4i32) as u16;
        (*E).var_F = (0xb39e7di32) as u16;
        (*E).var_21 = 48;
        (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
    };
    [11770276] Botwoon_Func_12 [] {
        if (*E).var_3C != 0 {
            (*E).var_3C = 0;
            (*E).var_D = (0xb3989di32) as u16;
            (*E).var_E = (0xb39bb7i32) as u16;
            if (*E).var_33 != 0 {
                (*E).var_35 = 0;
            }
        } else {
            CallBotwoonFuncVarE((((*E).var_E) as u32) | (0xb30000u32), game);
            crate::game::enemy_projectiles::record_botwoon_head(game);
            crate::game::enemy_projectiles::follow_botwoon_history(game);
            CallEnemyPreInstr((((*E).var_F) as u32) | (0xb30000u32), game);
            crate::game::enemy_projectiles::orient_botwoon_body(game);
            crate::game::enemy_projectiles::advance_botwoon_history(game);
        };
    };
    [11770340] Botwoon_Func_13 [] {
        if (*E).var_21 != 0 {
            (*E).var_2E = 0;
            CallEnemyPreInstr((((*E).var_F) as u32) | (0xb30000u32), game);
        } else {
            (*E).var_3C = 0;
            (*E).var_D = (0xb399a4i32) as u16;
            (*E).var_E = (0xb3e250i32) as u16;
            (*E).var_F = (0xb39dc0i32) as u16;
            if (*E).var_2F != 0 || {
                let mut v3: u16 = (NextRandom(game)) & (1u16);
                (*E).var_33 = v3;
                (*E).var_34 = v3;
                (*E).var_34 == 0
            } {
                (*E).var_4B = 0;
            } else {
                (*E).var_35 = 0;
                (*E).var_36 = -1i32 as u16;
            }
            Botwoon_Func_10(word!(game.cur_enemy_index), game);
        };
    };
    [11770438] Botwoon_Func_14 [] {
        let v1: u16 = ((*E).var_22).wrapping_add(1);
        (*E).var_22 = v1;
        if v1 as i32 - 256i32 & 0x8000 == 0 {
            (*E).var_D = (0xb39a5ei32) as u16;
        }
    };
    [11770462] Botwoon_Func_15 [] {
        let v2: i32 = 8 * byte!((*E).var_28; 1) as i32 >> 1;
        words!(
            (*E).base,
            y_pos,
            y_subpos += words!((
                crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 1) as isize),
                crate::rtl::rom_table_word!(game, 0xa2838f, v2 as isize)
            ))
        );
        if (*E).base.y_pos as i32 - 200i32 & 0x8000 != 0 {
            (*E).var_28 = ((*E).var_28).wrapping_add(192);
        } else {
            (*E).base.y_pos = 200;
            (*E).var_D = (0xb39acai32) as u16;
            game.eproj_spawn_pt = Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x1d, game);
            Botwoon_QueueExplosionSfx(game);
            (*E).base.properties = ((*E).base.properties) | (0x500u16);
        };
    };
    [11770570] Botwoon_Func_16 [] {
        if (*E).var_5F != 0 {
            (*E).var_D = (0xb39af9i32) as u16;
            Botwoon_Func_17(word!(game.cur_enemy_index), game);
        }
    };
    Botwoon_Func_21 [] {
        let pt: Point32 = ConvertAngleToXy((*E).var_3A, (*E).var_38, game);
        Runtime::new(game).session_mut().compatibility_enemy_scratch = (*E).var_3A;
        ({ let (phase, point) = ((*E).var_3A, pt); actor_api::apply_vector(&mut Runtime::new(game), phase, crate::game::session::Vector32 { x: point.x, y: point.y }) });
    };
    Botwoon_Shot [] {
        (*E).var_4C = (*E).base.health;
        Enemy_NormalShotAI_SkipSomeParts_B3(game);
        if (*E).base.health == 0 {
            (*E).var_2F = 1;
            Botwoon_Func_4(game);
        }
    };
    Botwoon_Func_32 [] {
        (*E).var_E = addr_loc_B3E28C as u16;
        (*E).var_3C = 0;
        let v1 = RomPtr_B3(((*E).var_40 as i32 + addr_stru_B3E150 as i32) as u16, game);
        (*E).var_42 = *(v1 as *mut u16);
        (*E).var_37 = *(v1.offset(4) as *mut u16);
        let v2: i16 = *(v1.offset(2) as *mut u16) as i16;
        (*E).var_44 = v2 as u16;
        if v2 < 0 {
            (*E).var_42 = ((*E).var_42).wrapping_sub(4);
        }
        Botwoon_Func_33(game);
    };
    @external {
        11770617 => [Botwoon_Func_18(k, game)];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallBotwoonFuncVarE(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11770807] Botwoon_Func_19 [] {
        let pt: Point16U = Botwoon_Func_20(word!(game.cur_enemy_index), game);
        let v1: u16 = actor_api::angle_from_delta(pt.x, pt.y);
        let E = Get_Botwoon(word!(game.cur_enemy_index) as i32, game);
        (*E).var_39 = v1;
        (*E).var_3A = ((64u8).wrapping_sub((v1) as u8)) as u16;
        if (*E).var_33 == (*E).var_34 {
            Botwoon_Func_21(game);
        } else {
            (*E).var_34 = (*E).var_33;
            (*E).var_3C = 1;
        };
    };
    [11788940] Botwoon_Func_33 [] {
        let mut x: u16 = 0;
        let mut y: u16 = 0;
        let E = Get_Botwoon(word!(game.cur_enemy_index) as i32, game);
        let mut n: i32 = (*E).var_38 as i32;
        let step: i32 = if ((*E).var_44 & 0x8000) == 0 {
            2
        } else {
            -2i32
        };
        do_until! { n == 0 ;
    let v6 = RomPtr_B3((*E).var_42, game);
    let v7: u16 = SignExtend8(*v6 as u16);
    let mut v8: u16 = 0;
    if v7 == 0xff80 || {
        x = (x).wrapping_add(v7);
        v8 = SignExtend8(*v6.offset(1) as u16);
        v8 == 0xff80
    } {
        (*E).var_41 = 0;
        (*E).var_3C = 1;
        return;
    }
    y = (y).wrapping_add(v8);
    (*E).var_42 = ((*E).var_42 as i32 + step) as u16;
    n -= 1;
}
        if ((*E).var_44 & 0x8000) != 0 {
            x = -(x as i32) as u16;
            y = -(y as i32) as u16;
        }
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(x);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(y);
    };
    @external {
        11788880 => Botwoon_Func_32;
    }
}
pub unsafe fn Botwoon_Func_4(game: &mut Game) {
    unsafe {
        let E = Get_Botwoon(0, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    }
}
pub unsafe fn Botwoon_Func_10(k: u16, game: &mut Game) {
    unsafe {
        Botwoon_Func_11(k, game);
        let E = Get_Botwoon(k as i32, game);
        let r18: u16 = (if (*E).var_33 != 0 { 128 } else { 0 }) as u16;
        (*E).var_40 =
            ((NextRandom(game) as i32 & 0x18) + r18 as i32 + 4 * (*E).var_37 as i32) as u16;
    }
}
pub unsafe fn Botwoon_Func_11(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Botwoon(k as i32, game);
        if (*E).var_33 == 0 {
            (*E).var_3F = 0;
            let mut health: i16 = (*E).base.health as i16;
            if health > 0 && (health).wrapping_sub(((*E).var_5C) as i16) < 0 {
                if (health).wrapping_sub(((*E).var_5D) as i16) < 0 {
                    (*E).var_3F = 2;
                } else {
                    (*E).var_3F = 1;
                }
                let v3: i32 = ((4u16).wrapping_mul((*E).var_3F)) as i32 >> 1;
                (*E).var_38 = crate::rtl::rom_table_word!(game, 0xb394bb, v3 as isize);
                (*E).var_C = crate::rtl::rom_table_word!(game, 0xb394bb, (v3 + 1) as isize);
            }
        }
    }
}
pub unsafe fn Botwoon_Func_17(k: u16, game: &mut Game) {
    unsafe {
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x4, 0xb79b as VoidP), game);
        scatter(DropSource::Botwoon, k, game);
        let E = Get_Botwoon(k as i32, game);
        set_fields! {[(*E)], 0; var_23, var_24, var_25}
    }
}
pub unsafe fn Botwoon_Func_18(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Botwoon(k as i32, game);
        if (*E).var_23 as i32 - 192i32 & 0x8000 != 0 {
            if (*E).var_23 as i32 - 64i32 & 0x8000 == 0 {
                let v2: i16 = (((*E).var_24) as i16).wrapping_sub(1);
                (*E).var_24 = v2 as u16;
                if v2 < 0 {
                    (*E).var_24 = 12;
                    let x: u16 = ((NextRandom(game) as i32 & 0x1f) + 232) as u16;
                    let y: u16 =
                        ((*E).var_23 as i32 + (NextRandom(game) as i32 & 0x1f) - 8i32) as u16;
                    create_sprite(x, y, 29, 2560, game);
                    Botwoon_QueueExplosionSfx(game);
                }
                let v3: i16 = (((*E).var_25) as i16).wrapping_sub(1);
                (*E).var_25 = v3 as u16;
                if v3 < 0 {
                    (*E).var_25 = 4;
                    for _ in 0..2 {
                        let x_0: u16 = ((NextRandom(game) as i32 & 0x3f) + 224) as u16;
                        let y_0: u16 = ((*Get_Botwoon(k as i32, game)).var_23 as i32
                            + ((NextRandom(game) as i32 & 0x1f) - 8i32))
                            as u16;
                        create_sprite(x_0, y_0, 9, 2560, game);
                    }
                }
            }
            (*E).var_23 = (*E).var_23.wrapping_add(1);
        } else {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
            SetBossBitForCurArea(2, game);
            QueueMusic_Delayed8(3, game);
        };
    }
}
pub unsafe fn Botwoon_Func_20(k: u16, game: &mut Game) -> Point16U {
    unsafe {
        let E = Get_Botwoon(k as i32, game);
        let v2: i32 = (*E).var_37 as i32 >> 1;
        let v3: u16 = (crate::rtl::rom_table_word!(game, 0xb3949b, v2 as isize) as i32 + 4
            - (*E).base.x_pos as i32) as u16;
        let mut r18: u16 = v3;
        if v3 as i32 - 256i32 & 0x8000 != 0 {
            if v3 as i32 + 256 & 0x8000 != 0 {
                r18 = -255i32 as u16;
            }
        } else {
            r18 = 255;
        }
        let v4: u16 = (crate::rtl::rom_table_word!(game, 0xb3949b, (v2 + 2) as isize) as i32 + 4
            - (*E).base.y_pos as i32) as u16;
        let mut r20: u16 = v4;
        if v4 as i32 - 256i32 & 0x8000 != 0 {
            if v4 as i32 + 256 & 0x8000 != 0 {
                r20 = -255i32 as u16;
            }
        } else {
            r20 = 255;
        }
        Point16U { x: r18, y: r20 }
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11765499] UnusedSpinningTurtleEye_Init [] {
        let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        (*v0).properties = ((*v0).properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*v0).current_instruction = addr_kUnusedSpinningTurtleEye_Ilist_86A7 as u16;
    };
    [11769448] Botwoon_Main [] {
        Botwoon_Func_3(game);
        let E = Get_Botwoon(word!(game.cur_enemy_index) as i32, game);
        CallBotwoonFuncVarD(
            (((*E).var_D) as u32) | (0xb30000u32),
            word!(game.cur_enemy_index),
            game,
        );
        Botwoon_HealthBasedPalHandling(game);
    };
    [11771903] Botwoon_Touch [] {
        Enemy_NormalTouchAI_SkipDeathAnim_B3(game);
        let E = Get_Botwoon(word!(game.cur_enemy_index) as i32, game);
        if (*E).base.health == 0 {
            (*E).var_2F = 1;
            Botwoon_Func_4(game);
        }
    };
    [11771969] Botwoon_Powerbomb [] {
        Enemy_NormalPowerBombAI_SkipDeathAnim_B3(game);
        let E = Get_Botwoon(word!(game.cur_enemy_index) as i32, game);
        if (*E).base.health == 0 {
            (*E).var_2F = 1;
            Botwoon_Func_4(game);
        }
    };
    [11789909] EscapeEtecoon_Main [] {
        let E = Get_EscapeEtecoon(word!(game.cur_enemy_index) as i32, game);
        CallEnemyPreInstr((((*E).var_F) as u32) | (0xb30000u32), game);
    };
    [11790027] EscapeEtecoon_Init [] {
        let E = Get_EscapeEtecoon(word!(game.cur_enemy_index) as i32, game);
        if CheckEventHappened(0xf, game) != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        } else {
            (*E).base.properties = ((*E).base.properties as i32
                | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Tangible as i32 | 0x8000))
                as u16;
            (*E).base.instruction_timer = 1;
            set_fields! {[(*E).base], 0; timer, palette_index}
            let v3: i32 = (*E).parameter_1 as i32 >> 1;
            (*E).base.x_pos = crate::rtl::rom_table_word!(game, 0xb3e718, v3 as isize);
            (*E).base.y_pos = crate::rtl::rom_table_word!(game, 0xb3e71e, v3 as isize);
            (*E).var_F = crate::rtl::rom_table_word!(game, 0xb3e724, v3 as isize);
            (*E).base.current_instruction =
                crate::rtl::rom_table_word!(game, 0xb3e72a, v3 as isize);
            (*E).var_A = crate::rtl::rom_table_word!(game, 0xb3e730, v3 as isize);
        };
    };
    [11791077] EscapeDachora_Init [] {
        let E = Get_EscapeDachora(word!(game.cur_enemy_index) as i32, game);
        if CheckEventHappened(0xf, game) != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        } else {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
            (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_B3 as u16;
            (*E).base.instruction_timer = 1;
            (*E).base.timer = 0;
            (*E).base.current_instruction = addr_kEscapeDachora_Ilist_E964 as u16;
        };
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let PipeBug = Get_PipeBug(word!(game.cur_enemy_index) as i32, game); };
    [11765819] BrinstarPipeBug_Init [] {
        (*PipeBug).var_B = (*PipeBug).base.x_pos;
        let y_pos: u16 = (*PipeBug).base.y_pos;
        set_fields! {[(*PipeBug)]; var_C = y_pos, var_00 = (y_pos).wrapping_sub(16),
            var_F = (0xb38880i32) as u16, var_D = 48, };
        set_fields! {[(*PipeBug)], 0; var_E, var_01}
        (*PipeBug).base.current_instruction = addr_kBrinstarPipeBug_Ilist_87AB as u16;
        if (*PipeBug).parameter_1 != 0 {
            (*PipeBug).base.current_instruction = addr_kBrinstarPipeBug_Ilist_8A1D as u16;
        }
    };
    [11765882] BrinstarPipeBug_Main [] {
        CallEnemyPreInstr((((*PipeBug).var_F) as u32) | (0xb30000u32), game);
    };
    BrinstarPipeBug_Func_1 [] {
        let pbg_var_E: u16 = (*PipeBug).var_E;
        if pbg_var_E != (*PipeBug).var_01 {
            (*PipeBug).var_01 = pbg_var_E;
            enemy_instruction! {[(*PipeBug).base]; instruction = (if (*PipeBug).parameter_1 != 0 {
                crate::rtl::rom_table_word!(game, 0xb38833, pbg_var_E as isize) as i32
            } else {
                crate::rtl::rom_table_word!(game, 0xb3882b, pbg_var_E as isize) as i32
            }) as u16, timer = 1}
            (*PipeBug).base.timer = 0;
        }
    };
    [11766625] NorfairPipeBug_Init [] {
        (*PipeBug).var_D = (*PipeBug).base.x_pos;
        (*PipeBug).var_E = (*PipeBug).base.y_pos;
        (*PipeBug).base.properties = ((*PipeBug).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*PipeBug).var_B = (8u16).wrapping_mul((byte!((*PipeBug).parameter_2; 1)) as u16);
        (*PipeBug).var_C = 0;
        set_fields! {[(*PipeBug).base]; instruction_timer = 1, timer = 0,
            current_instruction = addr_kNorfairPipeBug_Ilist_8AE1 as u16, };
        (*PipeBug).var_A = (0xb38bcdi32) as u16;
    };
    [11766686] NorfairPipeBug_Main [] {
        CallBugInstr(
            (((*PipeBug).var_A) as u32) | (0xb30000u32),
            word!(game.cur_enemy_index),
            game,
        );
        NorfairPipeBug_8BA8(game);
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [11763722] Enemy_GrappleReact_KillEnemy_B3 => actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Grapple);
    [11763727] Enemy_GrappleReact_CancelBeam_B3 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11763747] Enemy_NormalTouchAI_B3 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    Enemy_NormalTouchAI_SkipDeathAnim_B3 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    [11763757] Enemy_NormalShotAI_B3 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    Enemy_NormalShotAI_SkipSomeParts_B3 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
    Enemy_NormalPowerBombAI_SkipDeathAnim_B3 => actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    [11763777] Enemy_NormalFrozenAI_B3 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11763819] Enemy_SetAiPreInstr_B3 [jp.offset(1)] {
        (*gEnemyData(k as i32, game)).ai_preinstr = jp.word(0);
    };
    [11763828] Enemy_ClearAiPreInstr_B3 [jp] {
        (*gEnemyData(k as i32, game)).ai_preinstr = (0xb3807bi32) as u16;
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @prefix { let E = Get_Botwoon(word!(game.cur_enemy_index) as i32, game); };
    [11769031] Botwoon_Instr_1 [jp] {  (*E).base.x_width = 8; (*E).base.y_height = 16; };
    [11769047] Botwoon_Instr_2 [jp] {  set_fields! {[(*E).base], 12; x_width, y_height} };
    [11769063] Botwoon_Instr_3 [jp] {  (*E).base.x_width = 16; (*E).base.y_height = 8; };
    [11769079] Botwoon_Instr_4 [jp] {  set_fields! {[(*E).base], 12; x_width, y_height} };
    [11769095] Botwoon_Instr_5 [jp] {  (*E).base.x_width = 8; (*E).base.y_height = 16; };
    [11769111] Botwoon_Instr_6 [jp] {  (*E).base.x_width = 8; (*E).base.y_height = 16; };
    [11769127] Botwoon_Instr_7 [jp] {  set_fields! {[(*E).base], 12; x_width, y_height} };
    [11769143] Botwoon_Instr_8 [jp] {  (*E).base.x_width = 16; (*E).base.y_height = 8; };
    [11769159] Botwoon_Instr_9 [jp] {  set_fields! {[(*E).base], 12; x_width, y_height} };
    [11769175] Botwoon_Instr_10 [jp] {  (*E).base.x_width = 8; (*E).base.y_height = 16; };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11769191] Botwoon_Instr_SetSpitting [jp] {
        (*Get_Botwoon(word!(game.cur_enemy_index) as i32, game)).var_5A = 1;
    };
    [11769202] Botwoon_Instr_QueueSpitSfx [jp] { QueueSfx2_Max6(0x7c, game); };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11789637] EscapeEtecoon_Instr_1 [] {
        if word!(game.lava_acid_y_pos) >= 0xce {
            jp.offset(1)
        } else {
            EnemyScript::jump(jp.word(0))
        }
    };
    [11789840] EscapeEtecoon_Instr_2 [jp.offset(1)] {
        let E = Get_EscapeEtecoon(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(jp.word(0));
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11791016] EscapeDachora_Instr_2 [] {
        if word!(game.lava_acid_y_pos) >= 0xce {
            jp.offset(1)
        } else {
            EnemyScript::jump(jp.word(0))
        }
    };
    [11791032] EscapeDachora_Instr_3 [] {
        if CheckEventHappened(0xf, game) != 0 {
            EnemyScript::jump(jp.word(0))
        } else {
            jp.offset(1)
        }
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @prefix { let E = Get_EscapeDachora(word!(game.cur_enemy_index) as i32, game); };
    [11791049] EscapeDachora_Instr_1 [jp] {  (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(6); };
    [11791063] EscapeDachora_Instr_4 [jp] {  (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(6); };
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [11765888] BrinstarPipeBug_PreInstr_1 [] {
        if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) == 0 {
            (*Get_PipeBug(word!(game.cur_enemy_index) as i32, game)).var_F = (0xb38890i32) as u16;
        }
    };
    [11765904] BrinstarPipeBug_PreInstr_2 [] {
        let mut SamusEnemyDelta_Y: i16 = ({ let actor = k; actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y) }) as i16;
        if SamusEnemyDelta_Y < 0 && SamusEnemyDelta_Y as i32 + 96 & 0x8000 == 0 {
            let PipeBug = Get_PipeBug(k as i32, game);
            let a: u16 = { let actor = k; actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) };
            (*PipeBug).var_A = (*PipeBug).var_A & 0x7fff | a & 0x8000;
            if abs16(((*PipeBug).base.x_pos).wrapping_sub(word!(game.samus_x_pos))) < 0x40 {
                (*PipeBug).var_F = (0xb388e3i32) as u16;
                (*PipeBug).base.properties =
                    ((*PipeBug).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
                (*PipeBug).base.timer = 0;
                let mut v4: u16 = if ((*PipeBug).var_A & 0x8000) != 0 {
                    0
                } else {
                    2
                };
                (*PipeBug).var_E = v4;
                BrinstarPipeBug_Func_1(game);
            }
        }
    };
    [11765987] BrinstarPipeBug_PreInstr_3 [] {
        let PipeBug = Get_PipeBug(k as i32, game);
        crate::update_place! { previous fresh7 = (*PipeBug).base.y_subpos => (*PipeBug).base.y_subpos.wrapping_sub(1); }
        let v2: bool = fresh7 != 0;
        (*PipeBug).base.y_pos = ((*PipeBug).base.y_pos as i32 + (v2 as i32 - 2i32)) as u16;
        if (((*PipeBug).var_00) as i16).wrapping_sub(((*PipeBug).base.y_pos) as i16) >= 0
            && (*PipeBug).base.y_pos < word!(game.samus_y_pos)
        {
            (*PipeBug).var_E = ((*PipeBug).var_E) | (1u16);
            BrinstarPipeBug_Func_1(game);
            (*PipeBug).var_F = (0xb3891ci32) as u16;
        }
    };
    [11766044] BrinstarPipeBug_PreInstr_4 [] {
        let PipeBug = Get_PipeBug(k as i32, game);
        if ((*PipeBug).var_A & 0x8000) == 0 {
            (*PipeBug).base.x_subpos = (*PipeBug).base.x_subpos;
            (*PipeBug).base.x_pos = ((*PipeBug).base.x_pos).wrapping_add(2);
        } else {
            (*PipeBug).base.x_subpos = (*PipeBug).base.x_subpos;
            (*PipeBug).base.x_pos = ((*PipeBug).base.x_pos).wrapping_sub(2);
        }
        if ({ let actor = k; actor_api::leaving_screen(&Runtime::new(game), Key::from_legacy(actor)) as u8 }) & 1 != 0 {
            (*PipeBug).base.x_pos = (*PipeBug).var_B;
            (*PipeBug).base.x_subpos = 0;
            let pbg_var_C: u16 = (*PipeBug).var_C;
            (*PipeBug).base.y_pos = pbg_var_C;
            (*PipeBug).base.y_subpos = pbg_var_C;
            (*PipeBug).var_E = 0;
            BrinstarPipeBug_Func_1(game);
            (*PipeBug).base.properties =
                ((*PipeBug).base.properties) | ((kEnemyProps_Invisible) as u16);
            (*PipeBug).var_D = 48;
            (*PipeBug).var_F = (0xb3897ei32) as u16;
        }
    };
    [11767039] NorfairPipeBug_Func_5 [] {
        let E = Get_PipeBug(word!(game.cur_enemy_index) as i32, game);
        (*E).var_C = (*E).var_C.wrapping_add(1);
        (*E).var_A = (0xb38e5ai32) as u16;
    };
    [11771770] Botwoon_Func_30 [] {
        let E = Get_Botwoon(k as i32, game);
        let v2: i16 = (((*E).var_21) as i16).wrapping_sub(1);
        (*E).var_21 = v2 as u16;
        if v2 < 0 {
            (*E).var_21 = 0;
            (*E).var_F = (0xb39dc0i32) as u16;
        }
    };
    [11789916] EscapeEtecoon_E65C [] {
        let E = Get_EscapeEtecoon(k as i32, game);
        words!((*E).base, x_pos, x_subpos += 0x38000);
    };
    [11789936] EscapeEtecoon_E670 [] {
        if CheckEventHappened(0xf, game) != 0 {
            (*Get_EscapeEtecoon(k as i32, game))
                .base
                .current_instruction = addr_kEscapeEtecoon_Ilist_E5DA as u16;
        }
    };
    [11789952] EscapeEtecoon_E680 [] {
        let E = Get_EscapeEtecoon(k as i32, game);
        if ({ let (actor, amount) = (k, ((*E).var_A as i16 as i32) << 8); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            (*E).base.instruction_timer = 1;
            let v3: bool = (-((*E).var_A as i32) & 0x8000) != 0;
            (*E).var_A = -((*E).var_A as i32) as u16;
            (*E).base.current_instruction = (if v3 {
                addr_kEscapeEtecoon_Ilist_E556 as i32
            } else {
                addr_kEscapeEtecoon_Ilist_E582 as i32
            }) as u16;
            if CheckEventHappened(0xf, game) != 0 {
                (*E).base.current_instruction = addr_kEscapeEtecoon_Ilist_E5AE as u16;
            }
        }
        ({ let (actor, amount) = (k, (1i16 as i32) << 16); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_PipeBug(k as i32, game); };
    [11766142] BrinstarPipeBug_PreInstr_5 [] {
        crate::update_place! { previous fresh8 = (*E).var_D => (*E).var_D.wrapping_sub(1); }
        if fresh8 == 1 {
            (*E).var_F = (0xb38890i32) as u16;
        }
    };
    BrinstarYellowPipeBug_Func_2 [] {
        let v2: u16 = ((*E).var_20).wrapping_sub(1);
        (*E).var_20 = v2;
        if v2 == 0 {
            set_fields! {[(*E).base]; instruction_timer = 1, timer = 0,
                current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8EFC as u16, };
            (*E).var_A = (0xb39028i32) as u16;
            if (*E).parameter_1 == 0 {
                (*E).base.current_instruction = addr_kBrinstarYellowPipeBug_Ilist_8F24 as u16;
                (*E).var_A = (0xb390bdi32) as u16;
            }
        }
    };
    BrinstarYellowPipeBug_Func_4 [] {
        words!((*E).base, x_pos, x_subpos += words!((*E), var_03, var_02));
    };
    BrinstarYellowPipeBug_Func_6 [] {
        words!((*E).base, x_pos, x_subpos += words!((*E), var_01, var_00));
    };
    BrinstarYellowPipeBug_Func_9 [] {
        (*E).var_E = (*E).var_E.wrapping_add(1);
        let v3: i32 = 8 * (*E).var_E as i32 >> 1;
        words!(
            (*E).base,
            y_pos,
            y_subpos += words!((
                crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 3) as isize),
                crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 2) as isize)
            ))
        );
    };
    BrinstarYellowPipeBug_Func_10 [] {
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        if ((*E).var_E & 0x8000) != 0 {
            set_fields! {[(*E)], 0; var_E, var_F}
        } else {
            let v4: i32 = 8 * (*E).var_E as i32 >> 1;
            words!(
                (*E).base,
                y_pos,
                y_subpos += words!((
                    crate::rtl::rom_table_word!(game, 0xa2838f, (v4 + 1) as isize),
                    crate::rtl::rom_table_word!(game, 0xa2838f, v4 as isize)
                ))
            );
        };
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_Botwoon(word!(game.cur_enemy_index) as i32, game); };
    [11771328] Botwoon_Func_26 [] {
        let x: u16 = ((*E).base.x_pos).wrapping_sub((*E).var_56);
        let y: u16 = ((*E).base.y_pos).wrapping_sub((*E).var_57);
        let r22: u16 = actor_api::angle_from_delta(x, y);
        if x != 0 || y != 0 {
            let mut v1: u16 = if (*E).var_33 != 0 {
                (*E).base.layer = 7;
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
                addr_kBotwoon_Ilist_9389 as u16
            } else {
                (*E).base.layer = 2;
                (*E).base.properties =
                    ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
                (*E).var_45 = ((*E).var_45 as i32
                    + (((r22) as i8).wrapping_sub(((*E).var_45) as i8)) as i32 * 3 / 4)
                    as u8 as u16;
                crate::rtl::rom_table_word!(game, 0xb3946b, ((*E).var_45 >> 5) as isize)
            };
            if v1 != (*E).var_3B {
                (*E).base.current_instruction = v1;
                (*E).var_3B = v1;
                (*E).base.instruction_timer = 1;
                (*E).base.timer = 0;
            }
        }
        set_fields! {[(*E)]; var_56 = (*E).var_54, var_57 = (*E).var_55, var_54 = (*E).var_52,
            var_55 = (*E).var_53, var_52 = (*E).var_50, var_53 = (*E).var_51,
            var_50 = (*E).base.x_pos, var_51 = (*E).base.y_pos, };
    };
    [11771517] Botwoon_Func_27 [] {
        (*E).base.layer = 2;
        let v2: u16 = { let actor = word!(game.cur_enemy_index); actor_api::angle_to_player(&Runtime::new(game), Key::from_legacy(actor)) };
        (*E).var_3D = v2;
        let v3: u16 = crate::rtl::rom_table_word!(
            game,
            0xb3948b,
            ((((v2) as u8).wrapping_add(16)) >> 5) as isize
        );
        (*E).base.current_instruction = v3;
        (*E).var_3B = v3;
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).var_3D = ((64u8).wrapping_sub(((*E).var_3D) as u8)) as u16;
        (*E).var_F = (0xb39ee0i32) as u16;
        if (*E).var_D != 0x99e4 {
            (*E).var_F = (0xb39f34i32) as u16;
        }
        CallEnemyPreInstr((((*E).var_F) as u32) | (0xb30000u32), game);
    };
    [11771616] Botwoon_Func_28 [] {
        if (*E).var_5A != 0 {
            *ram_ptr!(game; 0x60b) = (((*E).var_3D) as u8).wrapping_sub(32);
            let varE32: u16 = g_word_B39E77[(*E).var_3F as usize] as u16;
            for _ in 0..5 {
                SpawnEprojWithGfx(
                    varE32,
                    word!(game.cur_enemy_index),
                    addr_kEproj_BotwoonsSpit as u16,
                    game,
                );
                crate::update_place! { reference fresh2 = *ram_ptr!(game; 0x60b) => (*fresh2).wrapping_add(16); }
            }
            (*E).var_5A = 0;
            (*E).var_F = (0xb39f7ai32) as u16;
        }
    };
    [11771700] Botwoon_Func_29 [] {
        *ram_ptr!(game; 0x60b) = (((*E).var_3D) as u8).wrapping_sub(16);
        let varE32: u16 = g_word_B39E77[(*E).var_3F as usize] as u16;
        for _ in 0..3 {
            SpawnEprojWithGfx(
                varE32,
                word!(game.cur_enemy_index),
                addr_kEproj_BotwoonsSpit as u16,
                game,
            );
            crate::update_place! { reference fresh3 = *ram_ptr!(game; 0x60b) => (*fresh3).wrapping_add(16); }
        }
        (*Get_Botwoon(word!(game.cur_enemy_index) as i32, game)).var_F = (0xb39f7ai32) as u16;
    };
    Botwoon_Func_31 [] {
        if (*Get_Botwoon(k as i32, game)).var_35 == 0 {
            let mut n: i32 = 24;
            loop {
                let v2: i32 = n >> 1;
                if (((*E).base.x_pos) as i16)
                    .wrapping_sub((crate::rtl::rom_table_word!(game, 0xb3949b, v2 as isize)) as i16)
                    >= 0
                    && (((*E).base.x_pos) as i16).wrapping_sub(
                        (crate::rtl::rom_table_word!(game, 0xb3949b, (v2 + 1) as isize)) as i16,
                    ) < 0
                    && (((*E).base.y_pos) as i16).wrapping_sub(
                        (crate::rtl::rom_table_word!(game, 0xb3949b, (v2 + 2) as isize)) as i16,
                    ) >= 0
                    && (((*E).base.y_pos) as i16).wrapping_sub(
                        (crate::rtl::rom_table_word!(game, 0xb3949b, (v2 + 3) as isize)) as i16,
                    ) < 0
                {
                    (*E).var_35 = 1;
                    (*E).var_33 = ((*E).var_33) ^ (1u16);
                    (*E).var_36 = (*E).var_B;
                    (*E).var_26 = (*E).var_26.wrapping_sub(1);
                    break;
                } else {
                    (*E).var_35 = 0;
                    n -= 8;
                    if n < 0 {
                        break;
                    }
                }
            }
        }
    };
}
