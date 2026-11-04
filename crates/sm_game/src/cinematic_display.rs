//! Cinematic display setup: PPU modes, layers and mirrors per scene, the Mode 7 reset and intro sprite order.
use crate::cinematic::CinematicDestination;
use crate::cinematic::set_fixed_color;
use crate::cinematic_objects::draw_cinematic_sprites;
use crate::cinematic_state::Cinematic;
use crate::display::control::{Brightness, FixedColor, Layers};
use crate::display::preparation::Field;
use crate::display::video::{
    Background, Command, Control, Mirror, Mode7Parameter, apply_controls, apply_mirrored,
};
use crate::menu_state::OptionsMenu;
use crate::menus::prelude::display_scroll;
use crate::presentation_workspace::{Point, Role};
use crate::word_transfers::WordEndpoint;

/// The producer name of the cinematic display setup's commands.
const CINEMATIC: &str = "cinematic display";

pub fn configure_cinematic_display<H: crate::cinematic_host::Host>(
    display: crate::visual_state::CinematicDisplay,
    cine: &mut Cinematic,
    game: &mut H,
) {
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
    apply_controls(
        game,
        CINEMATIC,
        &[
            (
                Control::Brightness(Brightness::BLANK_DARK),
                Mirror::After(Field::Brightness),
            ),
            (
                Control::ObjectLayout(sprites),
                Mirror::After(Field::ObjectLayout),
            ),
            (Control::BackgroundMode(mode), Mirror::After(Field::Mode)),
            (
                Control::Mode7Selection(selection),
                Mirror::After(Field::AffineSelection),
            ),
        ],
    );
    match display {
        Title | Station | Escape => {
            crate::frame_display_byte!(game, Field::Screens0 => 0);
            // Each mirror is one field on from its control's, as the original stores them.
            apply_controls(
                game,
                CINEMATIC,
                &[
                    (
                        Control::Tilemap(Background::Bg1, 0),
                        Mirror::After(Field::Screens1),
                    ),
                    (
                        Control::Tilemap(Background::Bg2, 0),
                        Mirror::After(Field::Screens2),
                    ),
                    (
                        Control::Tilemap(Background::Bg3, 0),
                        Mirror::After(Field::Screens3),
                    ),
                    (
                        Control::Tilemap(Background::Bg4, 0),
                        Mirror::After(Field::Characters0),
                    ),
                    (
                        Control::CharactersBg1Bg2(0),
                        Mirror::After(Field::Characters1),
                    ),
                    (Control::CharactersBg3Bg4(0), Mirror::None),
                ],
            );
        }
        Intro => {
            apply_controls(
                game,
                CINEMATIC,
                &[
                    (
                        Control::Tilemap(Background::Bg1, 0x50),
                        Mirror::Before(Field::Screens0),
                    ),
                    (
                        Control::Tilemap(Background::Bg2, 0x48),
                        Mirror::Before(Field::Screens1),
                    ),
                    (
                        Control::Tilemap(Background::Bg3, 0x4c),
                        Mirror::Before(Field::Screens2),
                    ),
                ],
            );
            crate::frame_display_byte!(game, Field::Screens3 => 0);
            apply_controls(
                game,
                CINEMATIC,
                &[
                    (
                        Control::Tilemap(Background::Bg4, 0),
                        Mirror::After(Field::Characters0),
                    ),
                    (Control::CharactersBg1Bg2(0), Mirror::None),
                    (
                        Control::CharactersBg3Bg4(4),
                        Mirror::Before(Field::Characters1),
                    ),
                ],
            );
        }
        Landing => {
            apply_mirrored(
                game,
                CINEMATIC,
                Control::Tilemap(Background::Bg1, 0x5c),
                Field::Screens0,
                true,
            );
            crate::frame_display_byte!(game, Field::Screens1 => 0);
            apply_controls(
                game,
                CINEMATIC,
                &[
                    (
                        Control::Tilemap(Background::Bg2, 0),
                        Mirror::After(Field::Screens2),
                    ),
                    (
                        Control::Tilemap(Background::Bg3, 0),
                        Mirror::After(Field::Screens3),
                    ),
                    (Control::Tilemap(Background::Bg4, 0), Mirror::None),
                    (
                        Control::CharactersBg1Bg2(6),
                        Mirror::Before(Field::Characters0),
                    ),
                    (
                        Control::CharactersBg3Bg4(0),
                        Mirror::Before(Field::Characters1),
                    ),
                ],
            );
        }
        Credits => {
            apply_controls(
                game,
                CINEMATIC,
                &[
                    (
                        Control::Tilemap(Background::Bg1, 0x48),
                        Mirror::Before(Field::Screens0),
                    ),
                    (
                        Control::Tilemap(Background::Bg2, 0x4c),
                        Mirror::Before(Field::Screens1),
                    ),
                    (
                        Control::Tilemap(Background::Bg3, 0x24),
                        Mirror::Before(Field::Screens2),
                    ),
                    (
                        Control::Tilemap(Background::Bg4, 0),
                        Mirror::Before(Field::Screens3),
                    ),
                    (
                        Control::CharactersBg1Bg2(0x54),
                        Mirror::Before(Field::Characters0),
                    ),
                    (
                        Control::CharactersBg3Bg4(2),
                        Mirror::Before(Field::Characters1),
                    ),
                ],
            );
        }
    }
    let layers = match display {
        Title => Layers::SPRITES,
        Intro => Layers::BG3,
        Credits => Layers::BG1,
        _ => Layers::BG1_SPRITES,
    };
    apply_controls(
        game,
        CINEMATIC,
        &[
            (
                Control::MainScreen(layers),
                Mirror::After(Field::MainLayers),
            ),
            (
                Control::SubScreen(Layers::NONE),
                Mirror::After(Field::SubLayers),
            ),
        ],
    );
    if tiled {
        for field in [
            Field::WindowSelection0,
            Field::WindowSelection1,
            Field::WindowSelection2,
            Field::MainWindow,
            Field::SubWindow,
            Field::ColorSelection,
            Field::ColorMath,
        ] {
            crate::frame_display_byte!(game, field => 0);
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
    apply_controls(
        game,
        CINEMATIC,
        &[
            (
                Control::MainWindowMask(Layers::NONE),
                Mirror::After(Field::MainWindow),
            ),
            (Control::VramStep(0), Mirror::None),
            (
                Control::ColorSelection(0),
                Mirror::After(Field::NextColorSelection),
            ),
            (Control::ColorMath(0), Mirror::After(Field::NextColorMath)),
        ],
    );
    if escape {
        crate::frame_display_byte!(game, Field::ColorSelection => 0);
        crate::frame_display_byte!(game, Field::ColorMath => 0);
    }
    // One fixed-colour write for all three channels, as the original did.
    game.apply(CINEMATIC, Command::FixedColor(FixedColor::all(0)));
    if display != Station {
        apply_mirrored(
            game,
            CINEMATIC,
            Control::ScreenSettings(0),
            Field::Geometry,
            false,
        );
    }
    if !title {
        if !escape {
            crate::frame_display_byte!(game, Field::ColorSelection => 0);
            crate::frame_display_byte!(game, Field::ColorMath => 0);
        }
        set_fixed_color([32, 64, 0x80], game);
        if !escape {
            crate::frame_display_byte!(game, Field::Mosaic => 0);
        }
    }
    if title || intro {
        let fill = if title { 0 } else { 0x2f };
        crate::cinematic::transfer_cinematic_words(
            0x3ff,
            2,
            &[(
                CinematicDestination::Screen(Point::new(Role::Pause, 0x0)),
                WordEndpoint::Value(fill),
            )],
            game,
        );
    }
    match display {
        Title => {
            crate::sprite_drawing::reset_cursor(game);
            crate::menus::state::with(game, |_, options: &mut OptionsMenu| {
                options.menu_option_index = 0
            });
            crate::screen_fade::store(game, 0, 0);
            crate::menus::state::with(game, |_, menu: &mut crate::menu_state::MenuState| {
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
            crate::gameplay_context::with_inventory(
                game,
                |_, inventory: &mut crate::inventory::Inventory| {
                    inventory.reserve_health = 0;
                    inventory.reserve_missiles = 0;
                },
            );
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
        crate::sequence_state::with_words(
            game,
            |_, words: &mut crate::session_words::SessionWords| {
                words.fx_layer_blending_config_a = 0
            },
        );
    }
    if intro {
        display_scroll! { game, 0; Bg2H, Bg2V, Bg3H, Bg3V }
    }
}

pub fn reset_cinematic_transform<H: crate::cinematic_host::Host>(x: u16, y: u16, game: &mut H) {
    let transform = crate::display::control::Mode7Transform::identity(x, y);
    set_mode7_transform(transform, game);
}

/// Each parameter's 16-bit store is followed by its RAM mirror word before the
/// next parameter, as the original did. The store's high byte also reaches the
/// next register up, as a word store does.
fn set_mode7_transform<H: crate::cinematic_host::Host>(
    transform: crate::display::control::Mode7Transform,
    game: &mut H,
) {
    use crate::display::preparation::Place;
    let mirrors = [
        (Field::AffineWords0Low, Field::AffineWords0High),
        (Field::AffineWords1Low, Field::AffineWords1High),
        (Field::AffineWords2Low, Field::AffineWords2High),
        (Field::AffineWords3Low, Field::AffineWords3High),
        (Field::AffineWords4Low, Field::AffineWords4High),
        (Field::AffineWords5Low, Field::AffineWords5High),
    ];
    for ((parameter, (low, high)), value) in Mode7Parameter::ALL
        .into_iter()
        .zip(mirrors)
        .zip(transform.words())
    {
        game.apply(CINEMATIC, Command::Mode7Word(parameter, value));
        Place::Controls(low, high).write(game, value);
    }
}

pub fn draw_intro_sprites<H: crate::cinematic_host::Host>(cine: &mut Cinematic, game: &mut H) {
    let samus = *cine.intro_samus_display();
    if samus & 0x8000 != 0 {
        game.owned_player_call(crate::player_drawing::draw_player_frame);
        crate::player_drawing::Host::draw_projectiles(game);
    }
    draw_cinematic_sprites(false, cine, game);
    if samus != 0 && samus & 0x8000 == 0 {
        game.owned_player_call(crate::player_drawing::draw_player_frame);
        crate::player_drawing::Host::draw_projectiles(game);
    }
}
