//! Defined-old setup observations; synthetic ROM tables only.
//! Child of expanded_core_checks, matching family_cases helper conventions.
fn CallEprojInit(ea: u32, k: u16, game: &mut Game) {
    super::super::with_execution(game, |ctx| super::super::setup::dispatch(ea, k, ctx));
}
use super::{bytes, put, slot};
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
use crate::types::{EnemyData, Point16U};

const SETUPS: &[u32] = &[
    8817209, // velocity high-byte table index, RNG and captured enemy
    8820772, // Phantoon modes 0/2/4/6
    8821923, 8822028, // Kraid spit/rising rocks
    8822450, // seahorse direction and three-entry velocity table
    8824275, 8824566, // Torizo drool and swipe
    8826389, 8826620, 8828143, // sonic boom, footsteps, lava seahorse
    8829070, 8829109, 8829176, // statue glow, particle, soul
    8829842, // Nuclear Waffle captured link offset
    8830364, 8830370, // Shaktool forwarding and shared body setup
    8832130, // Mother Brain bomb byte lane and active count
    8833377, // escape particle pairs
    8844177, // sparks random table pair
    8819839, 8820444, 8823534, 8823579, 8823715, // native spawn/Ceres
    8829744, // extra8000 origin plus loaded table
    8829160, 8837522, // slot-copy tail and flag-dependent velocity
    8840296, // dust cloud instruction table
    8819334, 8819533, 8821050, 8821976, 8823457,
    8824413, // drool's explicit unaligned random reads are defined
    8830543, 8830809, // room turret state table and byte-selected bullet
    8833327, 8833642, 8833667, 8833993, 8835642, 8838307, 8838356,
    8841777, // stalk, spawner, Botwoon link/activation
    8818723, 8827342, 8834960, 8834982, 8835208, 8835852, 8835886,
    8835905, // robot laser visibility/sound common helper
    8837912, 8838285, 8838870, 8839168, 8822252, 8824859,
    8824945, // mini-Kraid state, mutable spawn index, death coordinates
    8826704, 8826829, 8842338, // landing dust and Maw linked-body producer
];
pub(super) const CASES: u32 = SETUPS.len() as u32 * 8;
pub(super) const IMAGE_BYTES: usize = 0x200000;

pub(super) unsafe fn execute(id: u32, game: &mut Game, image: &RomImage) -> Option<usize> {
    assert!(id < CASES);
    let setup = SETUPS[id as usize / 8];
    let variant = id as usize % 8;
    let index = [0, 1, 2, 3, 32, 33, 34, 35][variant];
    let enemy = if variant & 1 == 0 { 0 } else { 64 };
    let mode = variant % 4;
    put(game, w::cur_enemy_index, enemy);
    put(game, w::random_number, [0, 1, 0x1234, 0xffff][mode]);
    put(game, w::samus_x_pos, [0, 0xffff, 128, 0x8000][mode]);
    put(game, w::samus_y_pos, [0xffff, 0, 160, 0x8000][mode]);
    put(game, w::layer1_x_pos, [0, 128, 0xff00, 0x8000][mode]);
    put(game, w::layer1_y_pos, [0, 128, 0xff00, 0x8000][mode]);
    put(game, w::eproj_init_param_2, [0, 63, 128, 255][mode]);
    game.eproj_spawn_pt = Point16U {
        x: [0, 0xffff, 1, 0x8000][mode],
        y: [0xffff, 0, 0x8000, 1][mode],
    };
    // Seed both enemy0 and the live enemy64: some routines intentionally use a fixed base.
    for handle in [0, 64] {
        for (field, value) in [
            (
                core::mem::offset_of!(EnemyData, x_pos),
                [1, 0xfff0, 128, 0x8000][mode],
            ),
            (
                core::mem::offset_of!(EnemyData, y_pos),
                [0, 0xffff, 128, 0x8000][mode],
            ),
            (core::mem::offset_of!(EnemyData, x_subpos), 0xffff),
            (core::mem::offset_of!(EnemyData, y_subpos), 1),
            (core::mem::offset_of!(EnemyData, x_width), 16),
            (core::mem::offset_of!(EnemyData, y_height), 16),
            (
                core::mem::offset_of!(EnemyData, parameter_1),
                [0, 0x8000, 0x4000, 0xc000][mode],
            ),
            (core::mem::offset_of!(EnemyData, parameter_2), mode as u16),
            (
                core::mem::offset_of!(EnemyData, ai_var_a),
                [0, 0x100, 0xff00, 0x8000][mode],
            ),
            (
                core::mem::offset_of!(EnemyData, ai_var_d),
                [0, 0x3f, 0x8080, 0xffff][mode],
            ),
        ] {
            put(game, 0xf78 + handle + field, value);
        }
        // Raw cast loads use aligned addresses and stay within real storage.
        put(game, 0x8000 + handle + 20, (mode as u16) * 2);
        put(game, 0x8000 + handle + 34, 0xfff0);
        put(game, 0x8000 + handle + 36, 0x1234);
        put(game, 0x8800 + handle + 8, mode as u16);
    }
    // Every source handle is a real projectile index. Half the cases alias destination.
    let source = if variant & 1 == 0 {
        index
    } else {
        34 - (index & !1)
    };
    slot(game, a::eproj_x_pos, source, 0xfff0);
    slot(game, a::eproj_y_pos, source, 0x1234);
    slot(game, a::eproj_y_subpos, source, mode as u16);
    let parameter = match setup {
        8820772 => [0x0000, 0x020f, 0x0408, 0x0607][mode],
        8822450 => [0, 2, 4, 0][mode],
        8829109 | 8829160 | 8830809 => source,
        8830543 | 8838356 | 8821050 | 8838307 => mode as u16,
        8824859 => [0, 2, 4, 6][mode], // post-increments keep index <=5
        _ => (mode as u16) * 2,
    };
    put(game, w::eproj_init_param_1, parameter);
    if setup == 8819533 {
        // Four-entry table uses parameter_1 directly.
        put(
            game,
            0xf78 + core::mem::offset_of!(EnemyData, parameter_1),
            mode as u16,
        );
    }
    if setup == 8841777 {
        // This is a linked enemy byte handle, not a velocity.
        put(
            game,
            0xf78 + core::mem::offset_of!(EnemyData, ai_var_a),
            (mode as u16) * 2,
        );
    }
    // All are checked loaded-image reads, including odd-address table starts.
    for address in [
        0x868a75, 0x86a64d, 0x86bb1e, 0x86d96a, 0x86e42c, 0x86a2e2, 0x86beb9, 0x86bec9, 0x86b209,
        0x86deb6, 0x86e9f1, 0x86f3d4,
    ] {
        for row in 0..32 {
            let value = (0x8100u16).wrapping_add((row * 17) as u16);
            bytes(
                image,
                crate::rtl::lorom_offset(address) as usize + row * 2,
                &value.to_le_bytes(),
            );
        }
    }
    // Turret state low byte selects one of eight velocity/direction entries.
    for row in 0..12 {
        bytes(
            image,
            crate::rtl::lorom_offset(0x86bee1) as usize + row * 2,
            &((row % 8) as u16).to_le_bytes(),
        );
    }
    {
        CallEprojInit(setup, index, game)
    };
    None
}
