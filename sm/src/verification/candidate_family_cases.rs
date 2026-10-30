//! Additional synthetic cases, separate from the frozen 354-case carrier.
//! Intended as a child of that carrier; no cartridge content is used.
use super::{bytes, put, slot};
fn CallEprojInit(ea: u32, k: u16, game: &mut Game) {
    super::super::with_execution(game, |ctx| super::super::setup::dispatch(ea, k, ctx));
}
fn CallEprojPreInstr(ea: u32, k: u16, game: &mut Game) {
    super::super::with_execution(game, |ctx| super::super::pre_actions::dispatch(ea, k, ctx));
}
fn move_along_angle(k: u16, game: &mut Game) {
    super::super::with_execution(game, |ctx| {
        super::super::pre_actions::move_along_angle(k, ctx)
    });
}
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
use crate::types::EnemyData;

pub(super) const CASES: u32 = 92;
pub(super) const IMAGE_BYTES: usize = 0x200000;

// Caller seeds the original carrier's Game and image, then invokes this inside
// its existing catch_unwind and captures the same complete output projection.
pub(super) unsafe fn execute(id: u32, game: &mut Game, image: &RomImage) -> Option<usize> {
    assert!(id < CASES);
    let index = [0, 1, 32, 34][id as usize % 4];
    let enemy = if id & 1 == 0 { 0 } else { 64 };
    put(game, w::cur_enemy_index, enemy);
    for (field, value) in [
        (core::mem::offset_of!(EnemyData, x_pos), 0xfff0),
        (core::mem::offset_of!(EnemyData, x_subpos), 0xffff),
        (core::mem::offset_of!(EnemyData, y_pos), 0x0080),
        (core::mem::offset_of!(EnemyData, y_subpos), 0x0001),
    ] {
        put(game, 0xf78 + usize::from(enemy) + field, value);
    }
    if id < 28 {
        let setups = [
            8826691, 8828061, 8829246, 8829265, 8829284, 8829915, 8822936,
        ];
        put(
            game,
            w::eproj_init_param_1,
            [0, 1, 0x8000, 0xffff][id as usize % 4],
        );
        put(
            game,
            w::eproj_unk1995,
            [0xffff, 0x8000, 1, 0][id as usize % 4],
        );
        {
            CallEprojInit(setups[id as usize / 4], index, game)
        };
    } else if id < 60 {
        let variant = (id - 28) as usize;
        // Samples remain below 0x8000, avoiding signed-min negation while
        // exercising nonzero integer/fraction products at maximum amplitude.
        let table = crate::rtl::lorom_offset(0xa0b7ee) as usize;
        for offset in 0..128 {
            let sample = ((offset * 251 + 137) & 0x7fff) as u16;
            bytes(image, table + offset * 2, &sample.to_le_bytes());
        }
        game.ram()
            .write_byte(0x60b, [0, 63, 64, 127, 128, 191, 192, 255][variant / 4]);
        put(
            game,
            w::eproj_init_param_1,
            [0, 1, 0xffff, 0x1234][variant % 4],
        );
        slot(game, a::eproj_x_subpos, index, 0xffff);
        slot(game, a::eproj_y_subpos, index, 1);
        {
            CallEprojInit(8842182, index, game);
            move_along_angle(index, game);
            move_along_angle(index, game);
        }
    } else {
        let variant = (id - 60) as usize % 8;
        let pre = [8817501, 8833356, 8833786, 8821183][(id - 60) as usize / 8];
        put(game, w::layer1_x_pos, 0x100);
        put(game, w::layer1_y_pos, 0x100);
        put(game, w::samus_x_pos, 0xfff0);
        put(game, w::samus_y_pos, 0x10);
        put(game, w::nmi_frame_counter_word, variant as u16 & 1);
        for (field, value) in [
            (a::eproj_x_pos, [0xff, 0x100, 0x1ff, 0x200][variant % 4]),
            (a::eproj_y_pos, [0x100, 0xff, 0x200, 0x1ff][variant % 4]),
            (a::eproj_x_subpos, 0xffff),
            (a::eproj_y_subpos, 1),
            (a::eproj_x_vel, [0, 1, 0xffff, 0x8000][variant % 4]),
            (a::eproj_y_vel, [0, 0x100, 0xff00, 0x7fff][variant % 4]),
            (
                a::eproj_E,
                [0, 1, 2, 0x7fff, 0x8000, 0x8001, 0xfffe, 0xffff][variant],
            ),
        ] {
            slot(game, field, index, value);
        }
        {
            CallEprojPreInstr(pre, index, game)
        };
    }
    None
}
