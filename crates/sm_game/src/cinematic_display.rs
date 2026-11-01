//! Cinematic display setup: PPU modes, layers and mirrors per scene, the Mode 7 reset and intro sprite order.
use crate::cinematic::set_fixed_color;
use crate::cinematic_objects::draw_cinematic_sprites;
use crate::cinematic_state::Cinematic;
use crate::display::control::{Brightness, Layers, MainWindowMask, SubScreen};
use crate::menu_state::OptionsMenu;
use crate::menus::prelude::{display_scroll, owned};
use crate::ram::RawRam as _;

pub fn configure_cinematic_display<H: crate::menus::host::Host + ?Sized>(
    display: crate::visual_state::CinematicDisplay,
    cine: &mut Cinematic,
    game: &mut H,
) {
    use crate::display::registers::{
        BG1SC, BG2SC, BG3SC, BG4SC, BG12NBA, BG34NBA, BGMODE, CGADSUB, CGWSEL, M7SEL, OBSEL,
        SETINI, TM, VMAIN,
    };
    use crate::visual_state::CinematicDisplay::*;
    let title = display == Title;
    let intro = display == Intro;
    let escape = display == Escape;
    let tiled = matches!(display, Landing | Credits);
    let sprites = match display {
        Escape => 0xa3,
        Credits => 0,
        _ => 3,
    };
    let mode = match display {
        Intro => 9,
        Landing | Credits => 1,
        _ => 7,
    };
    let selection = if matches!(display, Title | Station) {
        0x80
    } else {
        0
    };
    crate::display::sequence::register_writes! { game;
        show_mirror(Brightness::BLANK_DARK, 0x51, false); mirror(OBSEL as u16, sprites, 0x52, false);
        mirror(BGMODE as u16, mode, 0x55, false); mirror(M7SEL as u16, selection, 0x5f, false);
    }
    match display {
        Title | Station | Escape => {
            game.ram().write_byte(0x58, 0);
            crate::display::sequence::register_writes! { game;
                mirror(BG1SC as u16, 0, 0x59, false); mirror(BG2SC as u16, 0, 0x5a, false);
                mirror(BG3SC as u16, 0, 0x5c, false); mirror(BG4SC as u16, 0, 0x5d, false);
                mirror(BG12NBA as u16, 0, 0x5e, false); byte(BG34NBA as u16, 0);
            }
        }
        Intro => {
            crate::display::sequence::register_writes! { game;
                mirror(BG1SC as u16, 0x50, 0x58, true); mirror(BG2SC as u16, 0x48, 0x59, true);
                mirror(BG3SC as u16, 0x4c, 0x5a, true);
            }
            game.ram().write_byte(0x5c, 0);
            crate::display::sequence::register_writes! { game;
                mirror(BG4SC as u16, 0, 0x5d, false); byte(BG12NBA as u16, 0);
                mirror(BG34NBA as u16, 4, 0x5e, true);
            }
        }
        Landing => {
            crate::display::sequence::write_register_and_mirror(
                game,
                BG1SC as u16,
                0x5c,
                0x58,
                true,
            );
            game.ram().write_byte(0x59, 0);
            crate::display::sequence::register_writes! { game;
                mirror(BG2SC as u16, 0, 0x5a, false); mirror(BG3SC as u16, 0, 0x5c, false);
                byte(BG4SC as u16, 0); mirror(BG12NBA as u16, 6, 0x5d, true);
                mirror(BG34NBA as u16, 0, 0x5e, true);
            }
        }
        Credits => {
            crate::display::sequence::register_writes! { game;
                mirror(BG1SC as u16, 0x48, 0x58, true); mirror(BG2SC as u16, 0x4c, 0x59, true);
                mirror(BG3SC as u16, 0x24, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
                mirror(BG12NBA as u16, 0x54, 0x5d, true); mirror(BG34NBA as u16, 2, 0x5e, true);
            }
        }
    }
    let layers = match display {
        Title => 0x10,
        Intro => 4,
        Credits => 1,
        _ => 0x11,
    };
    crate::display::sequence::register_writes! { game;
        mirror(TM as u16, layers, 0x69, false); show_mirror(SubScreen(Layers::NONE), 0x6b, false);
    }
    if tiled {
        for at in [0x60, 0x61, 0x62, 0x6c, 0x6d, 0x6f, 0x72] {
            game.ram().write_byte(at, 0);
        }
        if display == Credits {
            set_fixed_color([32, 64, 0x80], game);
            crate::sprite_drawing::reset_cursor(game);
            reset_cinematic_transform(128, 128, game);
            *cine.mode7_angle() = 0;
            *cine.mode7_zoom() = 1;
            display_scroll!(game, 0; Bg1H, Bg1V, Bg2H, Bg2V);
            *cine.x_sub() = 0;
            *cine.x() = 0;
            *cine.y_sub() = 0;
            *cine.y() = 0;
        } else {
            display_scroll! { game, 0; Bg1H, Bg1V }
        }
        return;
    }
    crate::display::sequence::register_writes! { game;
        show_mirror(MainWindowMask(Layers::NONE), 0x6c, false); byte(VMAIN as u16, 0);
        mirror(CGWSEL as u16, 0, 0x6e, false); mirror(CGADSUB as u16, 0, 0x71, false);
    }
    if escape {
        game.ram().write_byte(0x6f, 0);
        game.ram().write_byte(0x72, 0);
    }
    // One COLDATA write, as the compat sink did.
    crate::display::sequence::register_writes! { game;
        show(crate::display::control::FixedColor::all(0));
    }
    if display != Station {
        crate::display::sequence::write_register_and_mirror(game, SETINI as u16, 0, 0x77, false);
    }
    if !title {
        if !escape {
            game.ram().write_byte(0x6f, 0);
            game.ram().write_byte(0x72, 0);
        }
        set_fixed_color([32, 64, 0x80], game);
        if !escape {
            game.ram().write_byte(0x57, 0);
        }
    }
    if title || intro {
        let fill = if title { 0 } else { 0x2f };
        crate::word_transfers!(game, ascending, 0x3ff; native(0x3000) <- value(fill));
    }
    match display {
        Title => {
            crate::sprite_drawing::reset_cursor(game);
            crate::session_cohort::with(game, |_, options: &mut OptionsMenu| {
                options.menu_option_index = 0
            });
            crate::screen_fade::store(game, 0, 0);
            crate::session_cohort::with(game, |_, menu: &mut crate::menu_state::MenuState| {
                menu.pausemenu_start_lr_pressed_highlight_timer = 0;
                menu.pausemenu_lr_animation_timer = 0;
                menu.pausemenu_item_selector_animation_timer = 0;
                menu.pausemenu_reserve_tank_animation_timer = 0;
                menu.unused_word_7e0731 = 0;
                menu.unused_word_7e073d = 0;
                menu.pausemenu_lr_animation_frame = 0;
                menu.pausemenu_item_selector_animation_frame = 0;
                menu.pausemenu_reserve_tank_animation_frame = 0;
                menu.unused_word_7e0745 = 0;
                menu.pausemenu_shoulder_button_highlight = 0;
                menu.pausemenu_button_label_mode = 0;
                menu.pausemenu_equipment_category_item = 0;
                menu.pausemenu_reserve_tank_delay_ctr = 0;
                menu.unused_word_7e0759 = 0;
            });
            crate::session_cohort::with(game, |_, inventory: &mut crate::inventory::Inventory| {
                inventory.reserve_health = 0;
                inventory.reserve_missiles = 0;
            });
            *cine.mode7_angle() = 0;
        }
        Intro => {
            crate::sprite_drawing::reset_cursor(game);
            *cine.mode7_angle() = 0;
        }
        Escape => {
            crate::sprite_drawing::reset_cursor(game);
            crate::screen_fade::store(game, 0, 0);
            *cine.mode7_angle() = 0;
        }
        _ => {}
    }
    if display != Station {
        *cine.mode7_zoom() = 256;
    }
    *cine.x_sub() = 0;
    *cine.x() = 0;
    *cine.y_sub() = 0;
    *cine.y() = 0;
    *cine.x_subspeed() = 0;
    *cine.x_speed() = 0;
    *cine.y_subspeed() = 0;
    *cine.y_speed() = 0;
    if matches!(display, Intro | Station) {
        crate::session_cohort::with(game, |_, words: &mut crate::session_words::SessionWords| {
            words.fx_layer_blending_config_a = 0
        });
    }
    if intro {
        display_scroll! { game, 0; Bg2H, Bg2V, Bg3H, Bg3V }
    }
}

pub fn reset_cinematic_transform<H: crate::menus::host::Host + ?Sized>(
    x: u16,
    y: u16,
    game: &mut H,
) {
    let transform = crate::display::control::Mode7Transform::identity(x, y);
    set_mode7_transform(transform, game);
}

/// Each register write is followed by its RAM mirror word before the next
/// register, as the original did.
fn set_mode7_transform<H: crate::menus::host::Host + ?Sized>(
    transform: crate::display::control::Mode7Transform,
    game: &mut H,
) {
    use crate::display::registers::{M7A, M7B, M7C, M7D, M7X, M7Y};
    use crate::ram::words;
    let registers = [M7A, M7B, M7C, M7D, M7X, M7Y];
    let mirrors = [
        words::REG_M7A,
        words::REG_M7B,
        words::REG_M7C,
        words::REG_M7D,
        words::REG_M7X,
        words::REG_M7Y,
    ];
    for ((register, mirror), value) in registers.into_iter().zip(mirrors).zip(transform.words()) {
        game.write_register_word(register as u16, value);
        crate::ram::checked_word!(game, mirror; = value);
    }
}

pub fn draw_intro_sprites<H: crate::menus::host::Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
) {
    let samus = *cine.intro_samus_display();
    if samus & 0x8000 != 0 {
        game.draw_player_frame();
        game.draw_projectiles();
    }
    draw_cinematic_sprites(false, cine, game);
    if samus != 0 && samus & 0x8000 == 0 {
        game.draw_player_frame();
        game.draw_projectiles();
    }
}
