//! Enemy animation interpreter, shared controls and captured-slot updates.
use crate::c_library::__assert_fail;
use crate::game::constants::enemy_properties::kEnemyProps_Deleted;
use crate::game::enemy_script::{bank_base, EnemyScript};
use crate::game::{sm_a2, sm_a3, sm_a4, sm_a5, sm_a6, sm_a7, sm_a8, sm_a9, sm_aa, sm_b2, sm_b3};
use crate::game_state::Game;
use crate::rtl::Unreachable;
use core::ffi::c_char;

fn jump(_k: u16, jp: EnemyScript) -> EnemyScript {
    EnemyScript::jump(jp.word(0))
}
crate::game::callback_dispatch! {
    pub(crate) unsafe fn dispatch_instruction(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default {
            Unreachable(game);
            EnemyScript::Stop
        };
    @functions [fn (k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10846332 | 11174012] stop_script [] {
        let properties = game.enemy_instructions(k).properties() | kEnemyProps_Deleted as u16;
        game.enemy_instructions(k).set_properties(properties);
        EnemyScript::Stop
    };
    @functions [fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10649872 | 10846480 | 10977552 | 11043088 | 11174160 | 11698448 | 11763984]
    countdown_jump [] {
        let timer = game.enemy_instructions(k).timer();
        let decremented = game.enemy_instructions(k).timer().wrapping_sub(1);
        game.enemy_instructions(k).set_timer(decremented);
        if timer == 1 {
            jp.offset(1)
        } else {
            jump(k, jp)
        }
    };
    [10649891 | 10846499 | 10977571 | 11043107 | 11174179 | 11698467 | 11764003]
    set_timer [jp.offset(1)] { game.enemy_instructions(k).set_timer(jp.word(0)); };
    [10649903 | 10715439 | 10780975 | 10846511 | 10912047 | 10977583 | 11043119 | 11108655 | 11174191 | 11698479 | 11764015]
    sleep [EnemyScript::Stop] {
        let base = bank_base(game.enemy_instructions(k).bank());
        let instruction = jp.advance(-2).offset_from(base) as u16;
        game.enemy_instructions(k)
            .set_current_instruction(instruction);
    };
    [10846522 | 11174202 | 11698490] wait_frames [EnemyScript::Stop] {
        let base = bank_base(game.enemy_instructions(k).bank());
        game.enemy_instructions(k).set_instruction_timer(jp.word(0));
        let instruction = jp.advance(2).offset_from(base) as u16;
        game.enemy_instructions(k)
            .set_current_instruction(instruction);
    };
    @functions [fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11174219] copy_to_vram [] {
        let tail = game.vram_write_queue_tail();
        let cursor = game.vram_write_queue_tail();
        let word = jp.word(0);
        game.write_vram_queue_word(cursor, 0, word);
        let word = jp.word(2);
        game.write_vram_queue_word(cursor, 2, word);
        let word = jp.word(3);
        game.write_vram_queue_word(cursor, 3, word);
        let word = jp.word(5);
        game.write_vram_queue_word(cursor, 5, word);
        game.set_vram_write_queue_tail(tail.wrapping_add(7));
        jp.advance(7)
    };
    @functions [fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10649971 | 10715507 | 11043187] enable_offscreen [] {
        let properties = game.enemy_instructions(k).properties() | 0x800;
        game.enemy_instructions(k).set_properties(properties);
        jp
    };
    @functions [fn (
    k: u16,
    jp: EnemyScript,
    game: &mut Game,
) -> EnemyScript];
    [10649981 | 10715517 | 11043197] disable_offscreen [] {
        let properties = game.enemy_instructions(k).properties() & !0x800;
        game.enemy_instructions(k).set_properties(properties);
        jp
    };
    @external {
        10649837 => [jump(k, j)];
        10651845 => [sm_a2::BouncingGoofball_Instr_88C5(k, j)];
        10715373 => [jump(k, j)];
        10716899 => sm_a3::Waver_Instr_1;
        10780909 => [jump(k, j)];
        10846445 => [jump(k, j)];
        10911981 => [jump(k, j)];
        10977517 => [jump(k, j)];
        10991155 => [sm_a7::Kraid_Instr_1(k, j)];
        11043053 => [jump(k, j)];
        11056990 => [sm_a8::Beetom_Instr_1(k, j)];
        11062987 => [sm_a8::WreckedShipRobot_Instr_7(k, j)];
        11063490 => [sm_a8::WreckedShipRobot_Instr_10(k, j)];
        11128756 => [sm_a9::Shitroid_Instr_1(k, j)];
        11128778 => [sm_a9::Shitroid_Instr_2(k, j)];
        11139360 => [sm_a9::Shitroid_Instr_3(k, j)]; 11139382 => [sm_a9::Shitroid_Instr_4(k, j)];
        11139472 => [sm_a9::Shitroid_Instr_6(k, j)];
        11174125 => [jump(k, j)];
        11190984 => [sm_aa::Torizo_Instr_7(k, j)];
        11196831 => [sm_aa::Shaktool_Instr_6(k, j)];
        11698413 => [jump(k, j)];
        11763949 => [jump(k, j)];
        0xa20000..=0xa2ffff => [sm_a2::DispatchEnemyInstr(ea, k, j, game)];
        0xa30000..=0xa3ffff => [sm_a3::DispatchEnemyInstr(ea, k, j, game)];
        0xa40000..=0xa4ffff => [sm_a4::DispatchEnemyInstr(ea, k, j, game)];
        0xa50000..=0xa5ffff => [sm_a5::DispatchEnemyInstr(ea, k, j, game)];
        0xa60000..=0xa6ffff => [sm_a6::DispatchEnemyInstr(ea, k, j, game)];
        0xa70000..=0xa7ffff => [sm_a7::DispatchEnemyInstr(ea, k, j, game)];
        0xa80000..=0xa8ffff => [sm_a8::DispatchEnemyInstr(ea, k, j, game)];
        0xa90000..=0xa9ffff => [sm_a9::DispatchEnemyInstr(ea, k, j, game)];
        0xaa0000..=0xaaffff => [sm_aa::DispatchEnemyInstr(ea, k, j, game)];
        0xb20000..=0xb2ffff => [sm_b2::DispatchEnemyInstr(ea, k, j, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyInstr(ea, k, j, game)];
    }
}

pub(crate) unsafe fn advance_animation(game: &mut Game) {
    // Callbacks may change the current index; animation writes retain this slot.
    let index = game.cur_enemy_index();
    if game.enemy_instructions(index).ai_handler_bits() & 4 != 0 {
        return;
    }
    let timer = game.enemy_instructions(index).instruction_timer();
    let decremented = game
        .enemy_instructions(index)
        .instruction_timer()
        .wrapping_sub(1);
    game.enemy_instructions(index)
        .set_instruction_timer(decremented);
    if timer != 1 {
        let properties = game.enemy_instructions(index).extra_properties() & !0x8000;
        game.enemy_instructions(index)
            .set_extra_properties(properties);
        return;
    }
    if game.enemy_instructions(index).current_instruction() & 0x8000 == 0 {
        crate::native_assertion! {
            @failure
            b"E->current_instruction & 0x8000\0", b"src/sm_a0.c\0", 3399,
            b"void ProcessEnemyInstructions(void)\0"
        };
    }
    let base = bank_base(game.enemy_instructions(index).bank());
    let image = game
        .loaded_rom
        .as_ref()
        .expect("ROM must be loaded")
        .clone();
    let mut pc = EnemyScript::at(
        image.clone(),
        usize::try_from(base + game.enemy_instructions(index).current_instruction() as isize)
            .expect("enemy script outside ROM"),
    );
    while pc.word(0) & 0x8000 != 0 {
        let address =
            (u32::from(game.enemy_instructions(index).bank()) << 16) | u32::from(pc.word(0));
        let next = dispatch_instruction(address, game.cur_enemy_index(), pc.offset(1), game);
        let Some(next) = next.resolve(&image, base) else {
            return;
        };
        pc = next;
    }
    game.enemy_instructions(index)
        .set_instruction_timer(pc.word(0));
    game.enemy_instructions(index)
        .set_spritemap_pointer(pc.word(2));
    let instruction = pc.advance(4).offset_from(base) as u16;
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
    let properties = game.enemy_instructions(index).extra_properties() | 0x8000;
    game.enemy_instructions(index)
        .set_extra_properties(properties);
}
