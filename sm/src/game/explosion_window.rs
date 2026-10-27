//! Power Bomb and Crystal Flash window geometry, expansion and afterglow.
use crate::game::menu_tilemap::native_record;
use crate::game::music::{QueueSfx1_Max6, QueueSfx1_Max15};
use crate::game::player_movement::forced;
use crate::game::session_runtime::Runtime;
use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::visual_runtime::spawn_hdma;
use crate::game_state::Game;
use crate::ram_map::word;
use crate::rtl::Mult8x8;
use crate::types::{Consts_88, Consts_89, SpawnHdmaObject_Args, VoidP};
crate::game::constants::typed_constants! { Consts_88;
    addr_byte_889F06 = 40710, addr_kPowerBombExplosionShapeDef0 = 37446,
}
crate::game::constants::typed_constants! { Consts_89;
    addr_kIndirectHdmaTable_PowerBombExplodeRight = 41217, addr_kIndirectHdmaTable_PowerBombExplodeLeft = 38912,
}

pub(crate) fn spawn_power_bomb(game: &mut Game) {
    if word!(game.time_is_frozen_flag) != 0 {
        word!(game.power_bomb_explosion_status = 0x4000);
    } else {
        QueueSfx1_Max15(1, game);
        word!(game.power_bomb_explosion_status = 0x8000);
        static unk_888ABA: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x28, 0x8ace as VoidP);
        static unk_888AC2: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x40, 0x29, 0x8b80 as VoidP);
        spawn_hdma(0x88, &unk_888ABA, game);
        spawn_hdma(0x88, &unk_888AC2, game);
    };
}
static kPowerBombPreExplosionRadiusSpeed: u16 = 0x3000;
static kPowerBombPreExplosionRadiusAccel: u16 = 0x80;
static g_word_888B96: u16 = 3;
static kPowerBombExplosionRadiusSpeed: u16 = 0;
static kPowerBombExplosionRadiusAccel: u16 = 0x30;
#[forbid(unsafe_code)]
pub(crate) fn begin_pre_flash(game: &mut Game) {
    game.ram().write_byte(0xc606, 255);
    game.ram().write_byte(0xc607, 0);
    word!(game.power_bomb_pre_explosion_flash_radius = 1024);
    word!(game.power_bomb_pre_explosion_radius_speed = kPowerBombPreExplosionRadiusSpeed);
}
pub(crate) fn begin_prescaled_pre_flash(game: &mut Game) {
    {
        word!(game.pre_scaled_power_bomb_explosion_shape_def_ptr = addr_byte_889F06 as u16);
    }
}
pub(crate) fn begin_explosion(game: &mut Game) {
    {
        word!(game.power_bomb_explosion_radius = 1024);
        word!(game.power_bomb_pre_explosion_radius_speed = kPowerBombExplosionRadiusSpeed);
    }
}
pub(crate) fn begin_prescaled_explosion(game: &mut Game) {
    {
        word!(
            game.pre_scaled_power_bomb_explosion_shape_def_ptr =
                addr_kPowerBombExplosionShapeDef0 as u16
        );
    }
}
// Rejection and pose publication precede Session admission, including import failures.
pub(crate) fn try_crystal_flash(game: &mut Game) -> bool {
    use crate::ram_map::checked_word as word;
    if !forced::crystal_flash_allowed(
        word!(game.game_state),
        word!(game.joypad1_lastkeys),
        word!(game.button_config_shoot_x),
        word!(game.samus_y_speed) == 0 && word!(game.samus_y_subspeed) == 0,
        word!(game.samus_health),
        word!(game.samus_reserve_health),
        [
            word!(game.samus_missiles),
            word!(game.samus_super_missiles),
            word!(game.samus_power_bombs),
        ],
    ) {
        return false;
    }
    word!(
        game.samus_pose = if word!(game, 0xa1e;) as u8 == 4 {
            0xd4
        } else {
            0xd3
        }
    );
    forced::initialize_crystal_flash(&mut Runtime::new(game))
}
pub(crate) unsafe fn finish_power_bomb(v0: u16, game: &mut Game) {
    {
        if word!(game.samus_x_pos) != word!(game.power_bomb_explosion_x_pos)
            || word!(game.samus_y_pos) != word!(game.power_bomb_explosion_y_pos)
            || !try_crystal_flash(game)
        {
            word!(game.power_bomb_flag = 0);
        }
        word!(game.power_bomb_explosion_status = 0);
        let v1: i32 = v0 as i32 >> 1;
        word! { game, 0;
            hdma_object_channels_bitmask[v1], hdma_object_channels_bitmask[(v1 + 1)],
            power_bomb_pre_explosion_flash_radius, power_bomb_explosion_radius,
        }
        CallSomeSamusCode(0x1e, game);
    }
}
pub(crate) fn set_power_bomb_blending(k: u16, game: &mut Game) {
    {
        word!(game.fx_layer_blending_config_c |= (0x8000u16));
    }
}

#[forbid(unsafe_code)]
fn fade_explosion_afterglow_color(game: &mut Game) {
    for (address, mask) in [(0x74, 0x20), (0x75, 0x40), (0x76, 0x80)] {
        if game.ram().read_byte(address) & 0x1f != 0 {
            let value = (game.ram().read_byte(address) & 0x1f).wrapping_sub(1) | mask;
            game.ram().write_byte(address, value);
        }
    }
}

#[forbid(unsafe_code)]
pub(crate) fn power_bomb_afterglow(k: u16, game: &mut Game) {
    if (word!(game.power_bomb_explosion_status) & 0x8000) != 0 {
        let v1: i32 = k as u8 as i32 >> 1;
        if (word!(game.hdma_object_timers[v1] -= 1, reread) & 0x8000) != 0 {
            let at = 0x1938 + usize::from(k as u8);
            let fresh127 = game.ram().read_byte(at);
            let value = game.ram().read_byte(at).wrapping_sub(1);
            game.ram().write_byte(at, value);
            if fresh127 == 1 {
                word!(game.hdma_object_instruction_timers[v1] = 1);
                word!(game.hdma_object_instruction_list_pointers[v1] += 2);
            } else {
                fade_explosion_afterglow_color(game);
                game.ram()
                    .write_byte(0x1908 + usize::from(k as u8), g_word_888B96 as u8);
            }
        }
    }
}
#[derive(Clone, Copy)]
enum ExplosionWindowSide {
    Left,
    OnScreen,
    Right,
}

#[derive(Clone, Copy)]
enum ExplosionWindowScale {
    PowerBomb(u8),
    CrystalFlash,
}

#[forbid(unsafe_code)]
fn explosion_window_side(byte_read: bool, game: &mut Game) -> ExplosionWindowSide {
    let mut high_byte = || {
        if byte_read {
            game.ram().read_byte(0xce7)
        } else {
            ((word!(game.power_bomb_explosion_x_pos_plus_0x100) as i32 & 0xff00) >> 8) as u8
        }
    };
    if high_byte() != 0 {
        if high_byte() == 1 {
            ExplosionWindowSide::OnScreen
        } else {
            ExplosionWindowSide::Right
        }
    } else {
        ExplosionWindowSide::Left
    }
}

macro_rules! write_explosion_window_row {
    ($k:expr, $left:expr, $right:expr, $game:ident) => {{
        #[forbid(unsafe_code)]
        {
            let left = $left;
            $game.ram().write_byte((0xc406isize + $k) as usize, left);
            let right = $right;
            $game.ram().write_byte((0xc506isize + $k) as usize, right);
        }
    }};
}

#[forbid(unsafe_code)]
fn calculate_power_bomb_hdma(
    mut k: u16,
    mut j: crate::rom::RomRecord,
    side: ExplosionWindowSide,
    game: &mut Game,
) {
    do_until! { k == 192;
        let w: u8 = j.byte(0);
        match side {
            ExplosionWindowSide::Left => {
                let c: u8 = word!(game.power_bomb_explosion_x_pos_plus_0x100) as u8;
                // This unscaled path writes right before left and uses 1, not 255, when empty.
                if u8::overflowing_add(w, c).1 {
                    game.ram().write_byte(0xc506 + usize::from(k), w.wrapping_add(c));
                    game.ram().write_byte(0xc406 + usize::from(k), 0);
                } else {
                    game.ram().write_byte(0xc506 + usize::from(k), 0);
                    game.ram().write_byte(0xc406 + usize::from(k), 1);
                }
            }
            ExplosionWindowSide::OnScreen => {
                if w == 0 {
                    break;
                }
                let c: u8 = word!(game.power_bomb_explosion_x_pos_plus_0x100) as u8;
                let right = c.saturating_add(w);
                let left = if c >= w { c.wrapping_sub(w) } else { 0 };
                write_explosion_window_row!(k as isize, left, right, game);
            }
            ExplosionWindowSide::Right => {
                if (word!(game.power_bomb_explosion_x_pos_plus_0x100) as u8) < w {
                    let left =
                        (word!(game.power_bomb_explosion_x_pos_plus_0x100) as u8).wrapping_sub(w);
                    write_explosion_window_row!(k as isize, left, -(1i32) as u8, game);
                } else {
                    write_explosion_window_row!(k as isize, -(1i32) as u8, -(2i32) as u8, game);
                }
            }
        }
        j.base = j.base.checked_add(1).expect("explosion shape offset overflow");
        k = k.wrapping_add(1);
    }
}

#[forbid(unsafe_code)]
fn calculate_explosion_hdma_scaled(
    mut k: u16,
    mut j: u16,
    scale: ExplosionWindowScale,
    side: ExplosionWindowSide,
    game: &mut Game,
) -> u16 {
    let mut left: u8;
    let mut right: u8;
    do_until! { (j & 0x80) != 0;
        let r20: u8 = (Mult8x8(
            match scale {
                ExplosionWindowScale::PowerBomb(value) => value,
                ExplosionWindowScale::CrystalFlash => {
                    ((word!(game.power_bomb_explosion_radius) as i32 & 0xff00) >> 8) as u8
                }
            },
            native_record(0x88a206, game).byte(usize::from(j as u8) + 32),
        ) >> 8) as u8;
        // Power Bomb reads its center between the two shape bytes; Crystal Flash reads after both.
        let center_before_width = match scale {
            ExplosionWindowScale::PowerBomb(_) => {
                Some(word!(game.power_bomb_explosion_x_pos_plus_0x100) as u8)
            }
            ExplosionWindowScale::CrystalFlash => None,
        };
        let w: u8 = (Mult8x8(
            match scale {
                ExplosionWindowScale::PowerBomb(value) => value,
                ExplosionWindowScale::CrystalFlash => {
                    ((word!(game.power_bomb_explosion_radius) as i32 & 0xff00) >> 8) as u8
                }
            },
            native_record(0x88a206, game).byte(usize::from(j as u8)),
        ) >> 8) as u8;
        let c = match center_before_width {
            Some(c) => c,
            None => word!(game.power_bomb_explosion_x_pos_plus_0x100) as u8,
        };
        match side {
            ExplosionWindowSide::Left => {
                let visible = u8::overflowing_add(w, c).1;
                left = if visible { 0 } else { 255 };
                right = if visible { w.wrapping_add(c) } else { 0 };
            }
            ExplosionWindowSide::OnScreen => {
                right = w.saturating_add(c);
                let c = match scale {
                    ExplosionWindowScale::PowerBomb(_) => {
                        word!(game.power_bomb_explosion_x_pos_plus_0x100) as u8
                    }
                    ExplosionWindowScale::CrystalFlash => c,
                };
                left = if c < w { 0 } else { c.wrapping_sub(w) };
            }
            ExplosionWindowSide::Right => {
                left = if c < w { c.wrapping_sub(w) } else { 255 };
                right = if c < w { 255 } else { 0 };
            }
        }
        loop {
            write_explosion_window_row!(k as u8 as isize, left, right, game);
            if (k as u8) == r20 {
                break;
            }
            match scale {
                ExplosionWindowScale::PowerBomb(_) => {
                    k = u16::from_ne_bytes([(k as u8).wrapping_sub(1), k.to_ne_bytes()[1]])
                }
                ExplosionWindowScale::CrystalFlash => k = k.wrapping_sub(1),
            }
        }
        match scale {
            ExplosionWindowScale::PowerBomb(_) => {
                j = u16::from_ne_bytes([(j as u8).wrapping_add(1), j.to_ne_bytes()[1]])
            }
            ExplosionWindowScale::CrystalFlash => j = j.wrapping_add(1),
        }
    }
    game.k_out = k.to_ne_bytes()[0] as u16;
    ((right as i32) << 8 | left as i32) as u16
}

fn calculate_explosion_hdma_table_pointers(k: u16, pre_explosion: bool, game: &mut Game) {
    {
        if pre_explosion && (word!(game.power_bomb_explosion_status) & 0x8000) == 0 {
            return;
        }
        let mut v1: u16 = 0;
        if ((word!(game.power_bomb_explosion_x_pos) as i32 - word!(game.layer1_x_pos) as i32 + 256)
            as u16)
            >= 0x300
            || {
                word!(
                    game.power_bomb_explosion_x_pos_plus_0x100 =
                        (word!(game.power_bomb_explosion_x_pos) as i32
                            - word!(game.layer1_x_pos) as i32
                            + 256) as u16
                );
                v1 = (word!(game.power_bomb_explosion_y_pos) as i32
                    - word!(game.layer1_y_pos) as i32
                    + 256) as u16;
                v1 >= 0x300
            }
        {
            v1 = 0;
        }
        word!(game.power_bomb_explosion_y_pos_rsub_0x1ff = ((v1 as i32 ^ 0x3ff) - 256i32) as u16);
        let radius = if pre_explosion {
            word!(game.power_bomb_pre_explosion_flash_radius)
        } else {
            word!(game.power_bomb_explosion_radius)
        };
        if (radius & 0xff00) == 0 {
            word!(game.power_bomb_explosion_y_pos_rsub_0x1ff = 0);
        }
        word! { game;
            hdma_object_table_pointers[(k >> 1)] =
                (3 * word!(game.power_bomb_explosion_y_pos_rsub_0x1ff) as i32
                    + addr_kIndirectHdmaTable_PowerBombExplodeLeft as i32) as u16;
            hdma_object_table_pointers[((k as i32 >> 1) + 1)] =
                (3 * word!(game.power_bomb_explosion_y_pos_rsub_0x1ff) as i32
                    + addr_kIndirectHdmaTable_PowerBombExplodeRight as i32) as u16;
        }
    }
}

pub(crate) fn spawn_crystal_flash(game: &mut Game) {
    word!(game.power_bomb_explosion_status = (0x888000i32) as u16);
    static unk_88A2B0: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40, 0x28, 0xa2bd as VoidP);
    static unk_88A2B8: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x40, 0x29, 0xa32a as VoidP);
    spawn_hdma(0x88, &unk_88A2B0, game);
    spawn_hdma(0x88, &unk_88A2B8, game);
}
#[forbid(unsafe_code)]
pub(crate) fn begin_crystal_flash(game: &mut Game) {
    begin_pre_flash(game);
    QueueSfx1_Max6(1, game);
}
pub(crate) fn finish_crystal_flash(k: u16, game: &mut Game) {
    {
        let v0: u16 = k;
        word! { game, 0; power_bomb_flag, power_bomb_explosion_status }
        let v1: i32 = v0 as i32 >> 1;
        word! { game, 0;
            hdma_object_channels_bitmask[v1], hdma_object_channels_bitmask[(v1 + 1)],
            power_bomb_pre_explosion_flash_radius, power_bomb_explosion_radius,
        }
    }
}
#[forbid(unsafe_code)]
pub(crate) fn set_crystal_flash_blending(k: u16, game: &mut Game) {
    game.ram().write_byte(0x60, 0);
    game.ram().write_byte(0x61, 8);
    game.ram().write_byte(0x62, 0x80);
    game.ram().write_byte(0x6e, 2);
    game.ram().write_byte(0x71, 51);
    game.ram().write_byte(0x6c, 0);
    game.ram().write_byte(0x6d, 4);
}
#[forbid(unsafe_code)]
pub(crate) fn crystal_flash_afterglow(k: u16, game: &mut Game) {
    if (word!(game.power_bomb_explosion_status) & 0x8000) != 0 {
        let v1: i32 = k as i32 >> 1;
        if (word!(game.hdma_object_timers[v1] -= 1, reread) & 0x8000) != 0 {
            if (game.ram().read_byte(0x76)
                | game.ram().read_byte(0x75)
                | game.ram().read_byte(0x74))
                & 0x1f
                != 0
            {
                fade_explosion_afterglow_color(game);
                game.ram()
                    .write_byte(0x1908 + usize::from(k), g_word_888B96 as u8);
            } else {
                word!(game.hdma_object_instruction_timers[v1] = 1);
                word!(game.hdma_object_instruction_list_pointers[v1] += 2);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Phase {
    PreFlash,
    PowerBomb,
    CrystalFlash,
}
impl Phase {
    fn radius(self, game: &mut Game) -> u16 {
        {
            if self == Self::PreFlash {
                word!(game.power_bomb_pre_explosion_flash_radius)
            } else {
                word!(game.power_bomb_explosion_radius)
            }
        }
    }
    fn expand(self, game: &mut Game) {
        {
            if self == Self::PreFlash {
                word!(
                    game.power_bomb_pre_explosion_flash_radius +=
                        word!(game.power_bomb_pre_explosion_radius_speed)
                );
            } else {
                word!(
                    game.power_bomb_explosion_radius +=
                        word!(game.power_bomb_pre_explosion_radius_speed)
                );
            }
        }
    }
    #[forbid(unsafe_code)]
    fn set_color(self, game: &mut Game) {
        let high = (self.radius(game) >> 8) as i32;
        let (table, index) = if self == Self::PreFlash {
            (0x889079, (high >> 3) & 0xf)
        } else {
            (0x888d85, high >> 3)
        };
        let pos = index * 3;
        for (channel, mask) in [(0, 0x20), (1, 0x40), (2, 0x80)] {
            let value = native_record(table, game).byte((pos + channel) as usize) | mask;
            game.ram().write_byte(0x74 + channel as usize, value);
        }
    }
    fn accelerate(self, game: &mut Game) {
        {
            if self == Self::PreFlash {
                word!(
                    game.power_bomb_pre_explosion_radius_speed -= kPowerBombPreExplosionRadiusAccel
                );
            } else {
                word!(game.power_bomb_pre_explosion_radius_speed += kPowerBombExplosionRadiusAccel);
            }
        }
    }
}

fn advance_instruction(k: u16, game: &mut Game) {
    {
        word!(game.hdma_object_instruction_timers[(k >> 1)] = 1);
        word!(game.hdma_object_instruction_list_pointers[(k >> 1)] += 2);
        word!(game.hdma_object_timers[(k >> 1)] = 0);
    }
}

#[forbid(unsafe_code)]
pub(crate) fn update_scaled(k: u16, phase: Phase, game: &mut Game) {
    if word!(game.power_bomb_explosion_status) & 0x8000 == 0 {
        return;
    }
    calculate_explosion_hdma_table_pointers(k, phase == Phase::PreFlash, game);
    let bottom = (Mult8x8(
        (phase.radius(game) >> 8) as u8,
        native_record(0x88a286, game).byte(0),
    ) >> 8) as i32;
    let side = explosion_window_side(phase == Phase::PreFlash, game);
    let scale = if phase == Phase::CrystalFlash {
        ExplosionWindowScale::CrystalFlash
    } else {
        ExplosionWindowScale::PowerBomb((phase.radius(game) >> 8) as u8)
    };
    let edges = calculate_explosion_hdma_scaled(bottom as u16, 96, scale, side, game);
    // k_out is a byte. All three original downward counters stop before a
    // negative index; the pre-flash byte counter wraps only on that last step.
    let right = if phase == Phase::PreFlash {
        edges.to_ne_bytes()[1]
    } else {
        (edges >> 8) as u8
    };
    let mut row = game.k_out as u8;
    do_until! { row & 0x80 != 0;
        write_explosion_window_row!(row as isize, edges as u8, right, game);
        row = row.wrapping_sub(1);
    }
    let mut row = bottom + 1;
    while row != 0xc0 {
        write_explosion_window_row!(row as isize, 255, 0, game);
        row += 1;
        if phase != Phase::PowerBomb {
            row = row as u8 as i32;
        }
    }
    phase.set_color(game);
    phase.expand(game);
    let threshold = match phase {
        Phase::PreFlash => 0x9200,
        Phase::PowerBomb => 0x8600,
        Phase::CrystalFlash => 0x2000,
    };
    if phase.radius(game) < threshold {
        phase.accelerate(game);
    } else {
        advance_instruction(k, game);
    }
}

pub(crate) unsafe fn update_prescaled(k: u16, phase: Phase, game: &mut Game) {
    if word!(game.power_bomb_explosion_status) & 0x8000 == 0 {
        return;
    }
    calculate_explosion_hdma_table_pointers(k, phase == Phase::PreFlash, game);
    let shape = native_record(
        0x880000 | u32::from(word!(game.pre_scaled_power_bomb_explosion_shape_def_ptr)),
        game,
    );
    let side = explosion_window_side(false, game);
    calculate_power_bomb_hdma(0, shape, side, game);
    phase.set_color(game);
    word!(game.pre_scaled_power_bomb_explosion_shape_def_ptr += 192);
    let end = if phase == Phase::PreFlash {
        0xa206
    } else {
        addr_byte_889F06 as u16
    };
    if word!(game.pre_scaled_power_bomb_explosion_shape_def_ptr) == end {
        advance_instruction(k, game);
        if phase == Phase::PowerBomb {
            word!(game.hdma_object_D[(k >> 1)] = 32);
        }
    }
    if word!(game.power_bomb_pre_explosion_radius_speed) as i32 + (phase.radius(game) as i32)
        < 0x10000
    {
        phase.expand(game);
        phase.accelerate(game);
    }
}
