//! Power Bomb and Crystal Flash callbacks on the actual Session.
//! Direct-page scratch and full out-of-table object identities remain aliases.
#![forbid(unsafe_code)]
use super::context::Context;
use crate::constants::{Consts88, Consts89};
use crate::display::preparation::{Field as DisplayField, Store as DisplayStore};
use crate::display_refresh::mult8x8;

use crate::menus::prelude::do_until;
use crate::player_movement::forced;
use crate::scene::hdma_sources::LiveOwner;

#[derive(Clone, Default, PartialEq)]
struct Explosion {
    bomb: crate::session::PowerBomb,
    radius: u16,
}
fn explosion<H: Context>(game: &H) -> Explosion {
    Explosion {
        bomb: game.session().projectiles.power_bomb.clone(),
        radius: game.session().room.power_bomb_radius,
    }
}
fn native_explosion_record<H: Context>(
    address: u32,
    host: &mut H,
) -> sm_data::cartridge::RomRecord {
    crate::menus::menu_tilemap::native_record(address, host)
}
fn with_explosion<H: Context, R>(
    game: &mut H,
    action: impl FnOnce(&mut H, &mut Explosion) -> R,
) -> R {
    let before = explosion(game);
    let mut after = before.clone();
    let result = action(game, &mut after);
    // Match the original typed-copy's ordered changed-field publication. A
    // callback may import other fields, so unchanged values never overwrite it.
    let target = &mut game.session_mut().projectiles.power_bomb;
    for (old, new, target) in [
        (
            before.bomb.explosion_status,
            after.bomb.explosion_status,
            &mut target.explosion_status,
        ),
        (before.bomb.x, after.bomb.x, &mut target.x),
        (before.bomb.y, after.bomb.y, &mut target.y),
        (
            before.bomb.window_x,
            after.bomb.window_x,
            &mut target.window_x,
        ),
        (
            before.bomb.window_y,
            after.bomb.window_y,
            &mut target.window_y,
        ),
    ] {
        if old != new {
            *target = new;
        }
    }
    if before.radius != after.radius {
        game.session_mut().room.power_bomb_radius = after.radius;
    }
    let target = &mut game.session_mut().projectiles.power_bomb;
    for (old, new, target) in [
        (
            before.bomb.pre_flash_radius,
            after.bomb.pre_flash_radius,
            &mut target.pre_flash_radius,
        ),
        (before.bomb.flag, after.bomb.flag, &mut target.flag),
        (
            before.bomb.radius_speed,
            after.bomb.radius_speed,
            &mut target.radius_speed,
        ),
        (
            before.bomb.shape_definition,
            after.bomb.shape_definition,
            &mut target.shape_definition,
        ),
    ] {
        if old != new {
            *target = new;
        }
    }
    game.refresh_inputs();
    result
}
crate::typed_constants! { Consts88;
    ADDR_BYTE_889F06 = 40710, ADDR_POWER_BOMB_EXPLOSION_SHAPE_DEF0 = 37446,
}
crate::typed_constants! { Consts89;
    ADDR_INDIRECT_HDMA_TABLE_POWER_BOMB_EXPLODE_RIGHT = 41217, ADDR_INDIRECT_HDMA_TABLE_POWER_BOMB_EXPLODE_LEFT = 38912,
}

static POWER_BOMB_PRE_EXPLOSION_RADIUS_SPEED: u16 = 0x3000;
static POWER_BOMB_PRE_EXPLOSION_RADIUS_ACCEL: u16 = 0x80;
static G_WORD_888B96: u16 = 3;
static POWER_BOMB_EXPLOSION_RADIUS_SPEED: u16 = 0;
static POWER_BOMB_EXPLOSION_RADIUS_ACCEL: u16 = 0x30;
pub fn begin_pre_flash<H: Context>(game: &mut H) {
    game.store_source_byte(LiveOwner::ExplosionOffscreen, 0, 255);
    game.store_source_byte(LiveOwner::ExplosionOffscreen, 1, 0);
    with_explosion(game, |_, e: &mut Explosion| {
        e.bomb.pre_flash_radius = 1024;
        e.bomb.radius_speed = POWER_BOMB_PRE_EXPLOSION_RADIUS_SPEED;
    });
}
pub fn begin_prescaled_pre_flash<H: Context>(game: &mut H) {
    with_explosion(game, |_, e: &mut Explosion| {
        e.bomb.shape_definition = ADDR_BYTE_889F06 as u16;
    });
}
pub fn begin_explosion<H: Context>(game: &mut H) {
    with_explosion(game, |_, e: &mut Explosion| {
        e.radius = 1024;
        e.bomb.radius_speed = POWER_BOMB_EXPLOSION_RADIUS_SPEED;
    });
}
pub fn begin_prescaled_explosion<H: Context>(game: &mut H) {
    with_explosion(game, |_, e: &mut Explosion| {
        e.bomb.shape_definition = ADDR_POWER_BOMB_EXPLOSION_SHAPE_DEF0 as u16;
    });
}
// Rejection and pose publication precede Session admission, including import failures.
pub fn try_crystal_flash<H: Context>(game: &mut H) -> bool {
    let inventory = &game.session().inventory;
    if !forced::crystal_flash_allowed(
        game.inputs().transitions.game_state,
        game.inputs().held,
        game.inputs().shoot_button,
        game.session().player.contact.y_speed.whole == 0
            && game.session().player.contact.y_speed.fraction == 0,
        inventory.health.current,
        inventory.reserve_health,
        inventory.ammunition.each_ref().map(|supply| supply.current),
    ) {
        return false;
    }
    game.session_mut().player.pose =
        if game.session().player.movement_and_facing.to_le_bytes()[0] == 4 {
            0xd4
        } else {
            0xd3
        };
    forced::initialize_crystal_flash(game)
}
pub fn finish_power_bomb<H: Context>(hdma_index: usize, v0: u16, game: &mut H) {
    let bomb = explosion(game).bomb;
    // A crystal flash starts on its own lease, after the position check.
    let crystal_flash = game.session().player.x == bomb.x
        && game.session().player.y == bomb.y
        && try_crystal_flash(game);
    with_explosion(game, |_, e: &mut Explosion| {
        if !crystal_flash {
            e.bomb.flag = 0;
        }
        e.bomb.explosion_status = 0;
        e.bomb.pre_flash_radius = 0;
        e.radius = 0;
    });
    let v1: i32 = v0 as i32 >> 1;
    // The right half is the next hdma_index, which stays on the host.
    crate::hdma_word!(game, hdma_index, Channels => 0);
    let next = usize::from(v0 >> 1) + 1;
    if next < crate::hdma::OBJECTS {
        game.store_hdma_word(
            crate::hdma::Word::Object {
                index: next,
                field: crate::hdma::ObjectWord::Channels,
            },
            0,
            crate::fx_domain::native_order(),
        );
    } else {
        game.hdma_slice_native(
            0x18b4 + 2 * usize::try_from(v1 + 1).expect("right object alias"),
            0,
        );
    }
    game.hdma_player_command(0x1e);
}
pub fn set_power_bomb_blending<H: Context>(_k: u16, game: &mut H) {
    {
        let fx_value = game.fx_controls().blending | (0x8000u16);
        let mut fx = game.fx_controls();
        fx.blending = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
    };
}

fn fade_explosion_afterglow_color<H: Context>(game: &mut H) {
    for (address, mask) in [
        (DisplayField::FixedColor0, 0x20),
        (DisplayField::FixedColor1, 0x40),
        (DisplayField::FixedColor2, 0x80),
    ] {
        if game.display_control_byte(address) & 0x1f != 0 {
            let value = (game.display_control_byte(address) & 0x1f).wrapping_sub(1) | mask;
            game.store_display_control_byte(address, value, DisplayStore::Named);
        }
    }
}

pub fn power_bomb_afterglow<H: Context>(hdma_index: usize, game: &mut H) {
    if (explosion(game).bomb.explosion_status & 0x8000) != 0 {
        crate::hdma_word!(game, hdma_index, Timer => crate::hdma_word!(game, hdma_index, Timer).wrapping_sub(1));
        if (crate::hdma_word!(game, hdma_index, Timer) & 0x8000) != 0 {
            // D's low byte counts the afterglow's fades.
            let fresh127 = crate::hdma_word!(game, hdma_index, D) as u8;
            crate::hdma_byte!(game, hdma_index, D, false => fresh127.wrapping_sub(1));
            if fresh127 == 1 {
                advance_program(hdma_index, game);
            } else {
                fade_explosion_afterglow_color(game);
                crate::hdma_byte!(game, hdma_index, Timer, false => G_WORD_888B96 as u8);
            }
        }
    }
}

fn advance_program<H: Context>(hdma_index: usize, game: &mut H) {
    crate::hdma_word!(game, hdma_index, InstructionTimer => 1);
    crate::hdma_word!(game, hdma_index, InstructionList => crate::hdma_word!(game, hdma_index, InstructionList).wrapping_add(2));
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

/// The explosion window's side of the screen, by the window X's high byte.
fn explosion_window_side(e: &Explosion) -> ExplosionWindowSide {
    let high_byte = e.bomb.window_x.to_le_bytes()[1];
    if high_byte != 0 {
        if high_byte == 1 {
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
        let left = $left;
        $game.store_source_signed_byte(LiveOwner::ExplosionLeft, $k, left);
        let right = $right;
        $game.store_source_signed_byte(LiveOwner::ExplosionRight, $k, right);
    }};
}

fn calculate_power_bomb_hdma<H: Context>(
    mut k: u16,
    mut j: sm_data::cartridge::RomRecord,
    side: ExplosionWindowSide,
    e: &Explosion,
    game: &mut H,
) {
    do_until! { k == 192;
        let w: u8 = j.byte(0);
        match side {
            ExplosionWindowSide::Left => {
                let c: u8 = e.bomb.window_x as u8;
                // This unscaled path writes right before left and uses 1, not 255, when empty.
                if u8::overflowing_add(w, c).1 {
                    game.store_source_byte(LiveOwner::ExplosionRight, usize::from(k), w.wrapping_add(c));
                    game.store_source_byte(LiveOwner::ExplosionLeft, usize::from(k), 0);
                } else {
                    game.store_source_byte(LiveOwner::ExplosionRight, usize::from(k), 0);
                    game.store_source_byte(LiveOwner::ExplosionLeft, usize::from(k), 1);
                }
            }
            ExplosionWindowSide::OnScreen => {
                if w == 0 {
                    break;
                }
                let c: u8 = e.bomb.window_x as u8;
                let right = c.saturating_add(w);
                let left = if c >= w { c.wrapping_sub(w) } else { 0 };
                write_explosion_window_row!(k as isize, left, right, game);
            }
            ExplosionWindowSide::Right => {
                if (e.bomb.window_x as u8) < w {
                    let left =
                        (e.bomb.window_x as u8).wrapping_sub(w);
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

fn calculate_explosion_hdma_scaled<H: Context>(
    mut k: u16,
    mut j: u16,
    scale: ExplosionWindowScale,
    side: ExplosionWindowSide,
    e: &Explosion,
    game: &mut H,
) -> u16 {
    let mut left: u8;
    let mut right: u8;
    do_until! { (j & 0x80) != 0;
        let r20: u8 = (mult8x8(
            match scale {
                ExplosionWindowScale::PowerBomb(value) => value,
                ExplosionWindowScale::CrystalFlash => {
                    ((e.radius as i32 & 0xff00) >> 8) as u8
                }
            },
            native_explosion_record(0x88a206, game).byte(usize::from(j as u8) + 32),
        ) >> 8) as u8;
        // Power Bomb reads its center between the two shape bytes; Crystal Flash reads after both.
        let center_before_width = match scale {
            ExplosionWindowScale::PowerBomb(_) => {
                Some(e.bomb.window_x as u8)
            }
            ExplosionWindowScale::CrystalFlash => None,
        };
        let w: u8 = (mult8x8(
            match scale {
                ExplosionWindowScale::PowerBomb(value) => value,
                ExplosionWindowScale::CrystalFlash => {
                    ((e.radius as i32 & 0xff00) >> 8) as u8
                }
            },
            native_explosion_record(0x88a206, game).byte(usize::from(j as u8)),
        ) >> 8) as u8;
        let c = match center_before_width {
            Some(c) => c,
            None => e.bomb.window_x as u8,
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
                        e.bomb.window_x as u8
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
    game.hdma_slice_native(0x12, u16::from(k.to_ne_bytes()[0]));
    ((right as i32) << 8 | left as i32) as u16
}

fn calculate_explosion_hdma_table_pointers<H: Context>(
    hdma_index: usize,
    k: u16,
    pre_explosion: bool,
    e: &mut Explosion,
    game: &mut H,
) {
    let bomb = &mut e.bomb;
    if pre_explosion && (bomb.explosion_status & 0x8000) == 0 {
        return;
    }
    let layer1 = game.session().room.camera.layer1;
    let x = (i32::from(bomb.x) - i32::from(layer1[0]) + 256) as u16;
    let y_offset = if x >= 0x300 {
        0
    } else {
        bomb.window_x = x;
        let y = (i32::from(bomb.y) - i32::from(layer1[1]) + 256) as u16;
        if y >= 0x300 { 0 } else { y }
    };
    bomb.window_y = ((y_offset as i32 ^ 0x3ff) - 256i32) as u16;
    let radius = if pre_explosion {
        bomb.pre_flash_radius
    } else {
        e.radius
    };
    if (radius & 0xff00) == 0 {
        e.bomb.window_y = 0;
    }
    // The table offset passes through the direct-page scratch word $16 ($88:8CB4).
    let offset = e.bomb.window_y.wrapping_mul(3);
    game.write_current_slice(0x16, &offset.to_le_bytes());
    crate::hdma_word!(game, hdma_index, Table => offset.wrapping_add(ADDR_INDIRECT_HDMA_TABLE_POWER_BOMB_EXPLODE_LEFT as u16));
    let next = usize::from(k >> 1) + 1;
    if next < crate::hdma::OBJECTS {
        game.store_hdma_word(
            crate::hdma::Word::Object {
                index: next,
                field: crate::hdma::ObjectWord::Table,
            },
            offset.wrapping_add(ADDR_INDIRECT_HDMA_TABLE_POWER_BOMB_EXPLODE_RIGHT as u16),
            crate::fx_domain::native_order(),
        );
    } else {
        game.hdma_slice_native(
            0x18d8 + 2 * (usize::from(k >> 1) + 1),
            offset.wrapping_add(ADDR_INDIRECT_HDMA_TABLE_POWER_BOMB_EXPLODE_RIGHT as u16),
        );
    }
}

pub fn begin_crystal_flash<H: Context>(game: &mut H) {
    begin_pre_flash(game);
    crate::effects::SoundEffects::queue_sound(game, 0, 1_u8, 6);
}
pub fn finish_crystal_flash<H: Context>(hdma_index: usize, k: u16, game: &mut H) {
    with_explosion(game, |_, e: &mut Explosion| {
        e.bomb.flag = 0;
        e.bomb.explosion_status = 0;
        e.bomb.pre_flash_radius = 0;
        e.radius = 0;
    });
    // The right half is the next hdma_index, which stays on the host.
    crate::hdma_word!(game, hdma_index, Channels => 0);
    let next = usize::from(k >> 1) + 1;
    if next < crate::hdma::OBJECTS {
        game.store_hdma_word(
            crate::hdma::Word::Object {
                index: next,
                field: crate::hdma::ObjectWord::Channels,
            },
            0,
            crate::fx_domain::native_order(),
        );
    } else {
        game.hdma_slice_native(0x18b4 + 2 * (usize::from(k >> 1) + 1), 0);
    }
}
pub fn set_crystal_flash_blending<H: Context>(_k: u16, game: &mut H) {
    game.store_display_control_byte(DisplayField::WindowSelection0, 0, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::WindowSelection1, 8, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::WindowSelection2, 0x80, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::NextColorSelection, 2, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::NextColorMath, 51, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::MainWindow, 0, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::SubWindow, 4, DisplayStore::Named);
}
pub fn crystal_flash_afterglow<H: Context>(hdma_index: usize, game: &mut H) {
    if (explosion(game).bomb.explosion_status & 0x8000) != 0 {
        crate::hdma_word!(game, hdma_index, Timer => crate::hdma_word!(game, hdma_index, Timer).wrapping_sub(1));
        if (crate::hdma_word!(game, hdma_index, Timer) & 0x8000) != 0 {
            if (game.display_control_byte(DisplayField::FixedColor2)
                | game.display_control_byte(DisplayField::FixedColor1)
                | game.display_control_byte(DisplayField::FixedColor0))
                & 0x1f
                != 0
            {
                fade_explosion_afterglow_color(game);
                crate::hdma_byte!(game, hdma_index, Timer, false => G_WORD_888B96 as u8);
            } else {
                advance_program(hdma_index, game);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    PreFlash,
    PowerBomb,
    CrystalFlash,
}
impl Phase {
    /// The phase's radius word.
    fn radius(self, e: &mut Explosion) -> &mut u16 {
        if self == Self::PreFlash {
            &mut e.bomb.pre_flash_radius
        } else {
            &mut e.radius
        }
    }
    fn expand(self, e: &mut Explosion) {
        let speed = e.bomb.radius_speed;
        let radius = self.radius(e);
        *radius = radius.wrapping_add(speed);
    }
    fn set_color<H: Context>(self, e: &mut Explosion, game: &mut H) {
        let high = (*self.radius(e) >> 8) as i32;
        let (table, index) = if self == Self::PreFlash {
            (0x889079, (high >> 3) & 0xf)
        } else {
            (0x888d85, high >> 3)
        };
        let pos = index * 3;
        for (channel, (field, mask)) in [
            (DisplayField::FixedColor0, 0x20),
            (DisplayField::FixedColor1, 0x40),
            (DisplayField::FixedColor2, 0x80),
        ]
        .into_iter()
        .enumerate()
        {
            let value = native_explosion_record(table, game)
                .byte(usize::try_from(pos).unwrap() + channel)
                | mask;
            game.store_display_control_byte(field, value, DisplayStore::Named);
        }
    }
    fn accelerate(self, e: &mut Explosion) {
        let speed = &mut e.bomb.radius_speed;
        *speed = if self == Self::PreFlash {
            speed.wrapping_sub(POWER_BOMB_PRE_EXPLOSION_RADIUS_ACCEL)
        } else {
            speed.wrapping_add(POWER_BOMB_EXPLOSION_RADIUS_ACCEL)
        };
    }
}

fn advance_instruction<H: Context>(hdma_index: usize, game: &mut H) {
    advance_program(hdma_index, game);
    crate::hdma_word!(game, hdma_index, Timer => 0);
}

pub fn update_scaled<H: Context>(hdma_index: usize, k: u16, phase: Phase, game: &mut H) {
    if explosion(game).bomb.explosion_status & 0x8000 == 0 {
        return;
    }
    with_explosion(game, |game, e: &mut Explosion| {
        calculate_explosion_hdma_table_pointers(hdma_index, k, phase == Phase::PreFlash, e, game);
        let bottom = (mult8x8(
            (*phase.radius(e) >> 8) as u8,
            native_explosion_record(0x88a286, game).byte(0),
        ) >> 8) as i32;
        let side = explosion_window_side(e);
        let scale = if phase == Phase::CrystalFlash {
            ExplosionWindowScale::CrystalFlash
        } else {
            ExplosionWindowScale::PowerBomb((*phase.radius(e) >> 8) as u8)
        };
        let edges = calculate_explosion_hdma_scaled(bottom as u16, 96, scale, side, e, game);
        // k_out is a byte. All three original downward counters stop before a
        // negative index; the pre-flash byte counter wraps only on that last step.
        let right = if phase == Phase::PreFlash {
            edges.to_ne_bytes()[1]
        } else {
            (edges >> 8) as u8
        };
        let mut row = game.current_native_word(0x12) as u8;
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
        phase.set_color(e, game);
        phase.expand(e);
        let threshold = match phase {
            Phase::PreFlash => 0x9200,
            Phase::PowerBomb => 0x8600,
            Phase::CrystalFlash => 0x2000,
        };
        if *phase.radius(e) < threshold {
            phase.accelerate(e);
        } else {
            advance_instruction(hdma_index, game);
        }
    });
}

pub fn update_prescaled<H: Context>(hdma_index: usize, k: u16, phase: Phase, game: &mut H) {
    if explosion(game).bomb.explosion_status & 0x8000 == 0 {
        return;
    }
    with_explosion(game, |game, e: &mut Explosion| {
        calculate_explosion_hdma_table_pointers(hdma_index, k, phase == Phase::PreFlash, e, game);
        let shape = native_explosion_record(0x880000 | u32::from(e.bomb.shape_definition), game);
        let side = explosion_window_side(e);
        calculate_power_bomb_hdma(0, shape, side, e, game);
        phase.set_color(e, game);
        e.bomb.shape_definition = e.bomb.shape_definition.wrapping_add(192);
        let end = if phase == Phase::PreFlash {
            0xa206
        } else {
            ADDR_BYTE_889F06 as u16
        };
        if e.bomb.shape_definition == end {
            advance_instruction(hdma_index, game);
            if phase == Phase::PowerBomb {
                crate::hdma_word!(game, hdma_index, D => 32);
            }
        }
        if i32::from(e.bomb.radius_speed) + i32::from(*phase.radius(e)) < 0x10000 {
            phase.expand(e);
            phase.accelerate(e);
        }
    });
}
