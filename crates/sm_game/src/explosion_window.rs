//! Power Bomb and Crystal Flash window geometry, expansion and afterglow.
use crate::constants::{Consts88, Consts89};
use crate::display::hdma_spawn::HdmaObjectDefinition;
use crate::display::preparation::{Field as DisplayField, Store as DisplayStore};
use crate::display_refresh::mult8x8;
use crate::menus::menu_tilemap::native_record;
use crate::menus::prelude::do_until;
use crate::player_movement::forced;
use crate::scene::hdma_sources::LiveOwner;

/// What the explosion windows need from the runtime besides RAM and sound.
pub trait Host:
    crate::fx_domain::Controls
    + crate::fx_domain::Sources
    + crate::display::preparation::Access
    + crate::ram::CompatibilityRam
    + crate::effects::SoundEffects
    + crate::cartridge::NativeDiagnostics
    + crate::menus::host::MenuRuntime
    + crate::runtime_access::RuntimeAccess
    + crate::runtime_access::SessionRam
    + crate::hdma_objects::HdmaTable
{
    /// Enter the Crystal Flash pose transition on the owned Session.
    fn initialize_crystal_flash(&mut self) -> bool;
    /// The scratch row byte one window pass leaves for the next.
    fn window_row(&self) -> u16;
    fn set_window_row(&mut self, row: u16);
}

/// The power bomb's explosion words: `projectiles.power_bomb` and the
/// explosion radius (`room.power_bomb_radius`, $0CEA), as a Session copy for
/// the explosion's HDMA objects.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Explosion {
    pub bomb: crate::session::PowerBomb,
    pub radius: u16,
}

impl Explosion {
    /// The explosion words of `words`.
    fn of(words: &crate::session_words::SessionWords) -> Self {
        Self {
            bomb: crate::session::PowerBomb {
                flag: words.power_bomb_flag,
                x: words.power_bomb_explosion_x_pos,
                y: words.power_bomb_explosion_y_pos,
                explosion_status: words.power_bomb_explosion_status,
                window_x: words.power_bomb_explosion_window_x,
                window_y: words.power_bomb_explosion_window_y,
                pre_flash_radius: words.power_bomb_pre_explosion_flash_radius,
                radius_speed: words.power_bomb_pre_explosion_radius_speed,
                shape_definition: words.pre_scaled_power_bomb_explosion_shape_def_ptr,
            },
            radius: words.power_bomb_explosion_radius,
        }
    }
    /// Store the explosion into `words`.
    fn store(&self, words: &mut crate::session_words::SessionWords) {
        let bomb = &self.bomb;
        words.power_bomb_explosion_status = bomb.explosion_status;
        words.power_bomb_explosion_x_pos = bomb.x;
        words.power_bomb_explosion_y_pos = bomb.y;
        words.power_bomb_explosion_window_x = bomb.window_x;
        words.power_bomb_explosion_window_y = bomb.window_y;
        words.power_bomb_explosion_radius = self.radius;
        words.power_bomb_pre_explosion_flash_radius = bomb.pre_flash_radius;
        words.power_bomb_flag = bomb.flag;
        words.power_bomb_pre_explosion_radius_speed = bomb.radius_speed;
        words.pre_scaled_power_bomb_explosion_shape_def_ptr = bomb.shape_definition;
    }
}

/// The explosion words as the sequence owner holds them.
pub fn explosion<H: crate::sequence_state::Access + ?Sized>(game: &H) -> Explosion {
    Explosion::of(&crate::session_words::current(game))
}

/// Run `action` on the explosion words of the sequence owner's words, then
/// store the ones it changed.
pub fn with_explosion<H, T>(game: &mut H, action: impl FnOnce(&mut H, &mut Explosion) -> T) -> T
where
    H: crate::sequence_state::Access + ?Sized,
{
    crate::sequence_state::with_words(game, |game, words| {
        let mut explosion = Explosion::of(words);
        let result = action(game, &mut explosion);
        explosion.store(words);
        result
    })
}

crate::typed_constants! { Consts88;
    ADDR_BYTE_889F06 = 40710, ADDR_POWER_BOMB_EXPLOSION_SHAPE_DEF0 = 37446,
}
crate::typed_constants! { Consts89;
    ADDR_INDIRECT_HDMA_TABLE_POWER_BOMB_EXPLODE_RIGHT = 41217, ADDR_INDIRECT_HDMA_TABLE_POWER_BOMB_EXPLODE_LEFT = 38912,
}

pub fn spawn_power_bomb<H: Host + ?Sized>(game: &mut H) {
    if crate::session_words::current(game).time_is_frozen_flag != 0 {
        with_explosion(game, |_, e: &mut Explosion| {
            e.bomb.explosion_status = 0x4000
        });
    } else {
        game.queue_sound(0, 1_u8, 15);
        with_explosion(game, |_, e: &mut Explosion| {
            e.bomb.explosion_status = 0x8000
        });
        static UNK_888ABA: HdmaObjectDefinition = HdmaObjectDefinition::new(0x40, 0x28, 0x8ace_u16);
        static UNK_888AC2: HdmaObjectDefinition = HdmaObjectDefinition::new(0x40, 0x29, 0x8b80_u16);
        game.spawn_hdma(0x88, &UNK_888ABA);
        game.spawn_hdma(0x88, &UNK_888AC2);
    };
}
static POWER_BOMB_PRE_EXPLOSION_RADIUS_SPEED: u16 = 0x3000;
static POWER_BOMB_PRE_EXPLOSION_RADIUS_ACCEL: u16 = 0x80;
static G_WORD_888B96: u16 = 3;
static POWER_BOMB_EXPLOSION_RADIUS_SPEED: u16 = 0;
static POWER_BOMB_EXPLOSION_RADIUS_ACCEL: u16 = 0x30;
pub fn begin_pre_flash<H: Host + ?Sized>(game: &mut H) {
    game.store_source_byte(LiveOwner::ExplosionOffscreen, 0, 255);
    game.store_source_byte(LiveOwner::ExplosionOffscreen, 1, 0);
    with_explosion(game, |_, e: &mut Explosion| {
        e.bomb.pre_flash_radius = 1024;
        e.bomb.radius_speed = POWER_BOMB_PRE_EXPLOSION_RADIUS_SPEED;
    });
}
pub fn begin_prescaled_pre_flash<H: Host + ?Sized>(game: &mut H) {
    with_explosion(game, |_, e: &mut Explosion| {
        e.bomb.shape_definition = ADDR_BYTE_889F06 as u16;
    });
}
pub fn begin_explosion<H: Host + ?Sized>(game: &mut H) {
    with_explosion(game, |_, e: &mut Explosion| {
        e.radius = 1024;
        e.bomb.radius_speed = POWER_BOMB_EXPLOSION_RADIUS_SPEED;
    });
}
pub fn begin_prescaled_explosion<H: Host + ?Sized>(game: &mut H) {
    with_explosion(game, |_, e: &mut Explosion| {
        e.bomb.shape_definition = ADDR_POWER_BOMB_EXPLOSION_SHAPE_DEF0 as u16;
    });
}
// Rejection and pose publication precede Session admission, including import failures.
pub fn try_crystal_flash<H: Host + ?Sized>(game: &mut H) -> bool {
    let inventory = crate::gameplay_context::inventory(game);
    if !forced::crystal_flash_allowed(
        crate::gameplay_word!(game.GAME_STATE),
        crate::gameplay_word!(game.JOYPAD1_LASTKEYS),
        crate::gameplay_word!(game.BUTTON_CONFIG_SHOOT_X),
        {
            let words = crate::session_words::current(game);
            words.samus_y_speed == 0 && words.samus_y_subspeed == 0
        },
        inventory.health.current,
        inventory.reserve_health,
        inventory.ammunition.map(|supply| supply.current),
    ) {
        return false;
    }
    let pose = if game.view_word(crate::projection::Field::MovementAndFacing) as u8 == 4 {
        0xd4
    } else {
        0xd3
    };
    crate::sequence_state::with_words(game, |_, words: &mut crate::session_words::SessionWords| {
        words.samus_pose = pose;
    });
    game.initialize_crystal_flash()
}
pub fn finish_power_bomb<H: Host + ?Sized>(hdma_index: usize, v0: u16, game: &mut H) {
    let bomb = explosion(game).bomb;
    // A crystal flash starts on its own lease, after the position check.
    let crystal_flash =
        game.player_x() == bomb.x && game.player_y() == bomb.y && try_crystal_flash(game);
    with_explosion(game, |_, e: &mut Explosion| {
        if !crystal_flash {
            e.bomb.flag = 0;
        }
        e.bomb.explosion_status = 0;
        e.bomb.pre_flash_radius = 0;
        e.radius = 0;
    });
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
        // LEGACY(original-overflow): the last object's right half is past the table.
        crate::legacy_overflow::ram_write_word_le(
            game,
            crate::ram::word_arrays::HDMA_OBJECT_CHANNELS_BITMASK + 2 * next,
            0,
        );
    }
    game.player_command(0x1e);
}
pub fn set_power_bomb_blending<H: crate::fx_domain::Controls + ?Sized>(game: &mut H) {
    {
        let fx_value = game.fx_controls().blending | (0x8000u16);
        let mut fx = game.fx_controls();
        fx.blending = fx_value;
        game.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
    };
}

fn fade_explosion_afterglow_color<H: Host + ?Sized>(game: &mut H) {
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

pub fn power_bomb_afterglow<H: Host + ?Sized>(hdma_index: usize, game: &mut H) {
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

fn advance_program<H: crate::hdma_objects::HdmaTable + ?Sized>(hdma_index: usize, game: &mut H) {
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

fn calculate_power_bomb_hdma<H: Host + ?Sized>(
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

fn calculate_explosion_hdma_scaled<H: Host + ?Sized>(
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
            native_record(0x88a206, game).byte(usize::from(j as u8) + 32),
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
            native_record(0x88a206, game).byte(usize::from(j as u8)),
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
    game.set_window_row(k.to_ne_bytes()[0] as u16);
    ((right as i32) << 8 | left as i32) as u16
}

fn calculate_explosion_hdma_table_pointers<H: Host + ?Sized>(
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
    let layer1 = [0, 1].map(|axis| game.view_word(crate::projection::Field::Layer1(axis)));
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
    game.set_view_word(crate::projection::Field::SpawnArgument, offset);
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
        // LEGACY(original-overflow): the last object's right half is past the table.
        crate::legacy_overflow::ram_write_word_le(
            game,
            crate::ram::word_arrays::HDMA_OBJECT_TABLE_POINTERS + 2 * next,
            offset.wrapping_add(ADDR_INDIRECT_HDMA_TABLE_POWER_BOMB_EXPLODE_RIGHT as u16),
        );
    }
}

pub fn spawn_crystal_flash<H: Host + ?Sized>(game: &mut H) {
    with_explosion(game, |_, e: &mut Explosion| {
        e.bomb.explosion_status = (0x888000i32) as u16;
    });
    static UNK_88A2B0: HdmaObjectDefinition = HdmaObjectDefinition::new(0x40, 0x28, 0xa2bd_u16);
    static UNK_88A2B8: HdmaObjectDefinition = HdmaObjectDefinition::new(0x40, 0x29, 0xa32a_u16);
    game.spawn_hdma(0x88, &UNK_88A2B0);
    game.spawn_hdma(0x88, &UNK_88A2B8);
}
pub fn begin_crystal_flash<H: Host + ?Sized>(game: &mut H) {
    begin_pre_flash(game);
    game.queue_sound(0, 1_u8, 6);
}
pub fn finish_crystal_flash<H: Host + ?Sized>(hdma_index: usize, k: u16, game: &mut H) {
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
        // LEGACY(original-overflow): the last object's right half is past the table.
        crate::legacy_overflow::ram_write_word_le(
            game,
            crate::ram::word_arrays::HDMA_OBJECT_CHANNELS_BITMASK + 2 * next,
            0,
        );
    }
}
pub fn set_crystal_flash_blending<H: crate::display::preparation::Access + ?Sized>(game: &mut H) {
    game.store_display_control_byte(DisplayField::WindowSelection0, 0, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::WindowSelection1, 8, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::WindowSelection2, 0x80, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::NextColorSelection, 2, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::NextColorMath, 51, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::MainWindow, 0, DisplayStore::Named);
    game.store_display_control_byte(DisplayField::SubWindow, 4, DisplayStore::Named);
}
pub fn crystal_flash_afterglow<H: Host + ?Sized>(hdma_index: usize, game: &mut H) {
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
    fn set_color<H: Host + ?Sized>(self, e: &mut Explosion, game: &mut H) {
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
            let value =
                native_record(table, game).byte(usize::try_from(pos).unwrap() + channel) | mask;
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

fn advance_instruction<H: crate::hdma_objects::HdmaTable + ?Sized>(
    hdma_index: usize,
    game: &mut H,
) {
    advance_program(hdma_index, game);
    crate::hdma_word!(game, hdma_index, Timer => 0);
}

pub fn update_scaled<H: Host + ?Sized>(hdma_index: usize, k: u16, phase: Phase, game: &mut H) {
    if explosion(game).bomb.explosion_status & 0x8000 == 0 {
        return;
    }
    with_explosion(game, |game, e: &mut Explosion| {
        calculate_explosion_hdma_table_pointers(hdma_index, k, phase == Phase::PreFlash, e, game);
        let bottom = (mult8x8(
            (*phase.radius(e) >> 8) as u8,
            native_record(0x88a286, game).byte(0),
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
        let mut row = game.window_row() as u8;
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

pub fn update_prescaled<H: Host + ?Sized>(hdma_index: usize, k: u16, phase: Phase, game: &mut H) {
    if explosion(game).bomb.explosion_status & 0x8000 == 0 {
        return;
    }
    with_explosion(game, |game, e: &mut Explosion| {
        calculate_explosion_hdma_table_pointers(hdma_index, k, phase == Phase::PreFlash, e, game);
        let shape = native_record(0x880000 | u32::from(e.bomb.shape_definition), game);
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
