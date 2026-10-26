//! Appended actual-controller alias cases; frozen0..116 fixtures are unchanged.
#![forbid(unsafe_code)]
use super::sprite_effect_cases::{Operation, put};
use crate::{
    game::enemy_layouts::{EnemyLayout40 as Metroid, EnemyLayout104 as Maw},
    game_state::Game,
    ram_map::words as w,
    types::EnemyBase,
};
use core::mem::{offset_of, size_of};
pub(crate) const FIRST: u32 = 117;
pub(crate) const END: u32 = 121;
fn read(game: &Game, address: usize) -> u16 {
    u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]])
}
pub(crate) fn capture_case(id: u32, run: fn(Operation, &mut Game) -> u16) -> Option<Vec<u8>> {
    if !(FIRST..END).contains(&id) {
        return None;
    }
    let mut game = Game::new();
    game.g_ram.fill(0);
    game.g_ram[0x1ffff] = 1;
    let operation = if id < 119 {
        let enemy = 0xe0dausize;
        let base = 0xf78 + enemy;
        assert!(base + size_of::<Maw>() <= game.g_ram.len());
        assert_eq!(base + offset_of!(EnemyBase, frozen_timer), 0xf078);
        assert_eq!(base + offset_of!(Maw, var_45), 0x168e4);
        assert_eq!(base + offset_of!(Maw, var_46), 0x168e6);
        put(&mut game, w::cur_enemy_index, enemy as u16);
        put(&mut game, base + offset_of!(Maw, var_45), 0);
        put(&mut game, base + offset_of!(Maw, var_46), 0); // sprite byte handle0
        put(&mut game, 0xf078, if id == 117 { 0xa00 } else { 1 });
        Operation::MawAppearance
    } else {
        let enemy = 0xe17ausize;
        let base = 0xf78 + enemy;
        assert!(base + size_of::<Metroid>() <= game.g_ram.len());
        assert_eq!(base + offset_of!(EnemyBase, x_pos), 0xf0f4);
        assert_eq!(base + offset_of!(EnemyBase, y_pos), 0xf0f8);
        assert_eq!(base + offset_of!(Metroid, var_00), 0x1597a);
        assert_eq!(base + offset_of!(Metroid, var_01), 0x1597c);
        put(&mut game, w::cur_enemy_index, enemy as u16);
        put(&mut game, w::samus_x_pos, 0x112);
        put(&mut game, w::samus_y_pos, 0x220);
        put(&mut game, base + offset_of!(Metroid, var_F), 2); // actual Func3, then both attachments
        put(
            &mut game,
            base + offset_of!(Metroid, var_00),
            if id == 119 { 0 } else { 2 },
        );
        put(
            &mut game,
            base + offset_of!(Metroid, var_01),
            if id == 119 { 2 } else { 0 },
        );
        put(
            &mut game,
            base + offset_of!(EnemyBase, vram_tiles_index),
            0x155,
        );
        put(
            &mut game,
            base + offset_of!(EnemyBase, palette_index),
            0xa00,
        );
        for address in [0xf1f8, 0xf1fa, 0xf2f8, 0xf2fa] {
            put(&mut game, address, 0x7abc);
        }
        Operation::MetroidMain
    };
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(operation, &mut game)));
    assert!(result.is_ok(), "sprite.actor.alias.{id}.defined-no-panic");
    assert!(!game.g_fail, "sprite.actor.alias.{id}.no-diagnostic");
    match id {
        117 => assert_eq!(read(&game, 0xf078), 0, "sprite.actor.maw_live_first_timer"),
        118 => assert_eq!(
            read(&game, 0xf078),
            0xc01,
            "sprite.actor.maw_live_second_timer"
        ),
        119 | 120 => {
            assert_eq!(
                (read(&game, 0xf0f8), read(&game, 0xf1f8)),
                (0x112, 0x112),
                "sprite.actor.attach_live_y_alias"
            );
            assert_eq!(
                (read(&game, 0xf0fa), read(&game, 0xf1fa)),
                (0x112, if id == 119 { 0x112 } else { 0x218 }),
                "sprite.actor.attach_paired_order"
            );
            assert_eq!(
                (read(&game, 0xf078), read(&game, 0xf07a)),
                (0xb55, 0xb55),
                "sprite.actor.attach_paired_appearance"
            );
            assert_eq!(
                (read(&game, 0xf2f8), read(&game, 0xf2fa)),
                (0, 0),
                "sprite.actor.attach_paired_enabled"
            );
        }
        _ => unreachable!(),
    }
    let mut bytes = game.g_ram.to_vec();
    bytes.extend_from_slice(&result.unwrap().to_le_bytes());
    bytes.extend_from_slice(&[0, 0]);
    Some(bytes)
}
