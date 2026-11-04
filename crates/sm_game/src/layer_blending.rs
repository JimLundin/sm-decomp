//! Room, X-ray and Power Bomb layer/window composition.
use crate::constants::bank_8f::{ADDR_K_ROOM_A66A, ADDR_K_ROOM_CEFB};
use crate::display::preparation::{Field as DisplayField, Store as DisplayStore};

// W12SEL, W34SEL, WOBJSEL, TM, TS, TMW, TSW, CGWSEL and CGADSUB shadows.
fn room_key<H: crate::runtime_access::RuntimeAccess + ?Sized>(game: &mut H) -> u16 {
    use crate::session_access::SessionAccess;
    game.runtime().session().room_key()
}

fn apply<H: crate::display::preparation::Access + ?Sized>(settings: [u8; 9], game: &mut H) {
    for (address, value) in [
        DisplayField::WindowSelection0,
        DisplayField::WindowSelection1,
        DisplayField::WindowSelection2,
        DisplayField::MainLayers,
        DisplayField::SubLayers,
        DisplayField::MainWindow,
        DisplayField::SubWindow,
        DisplayField::NextColorSelection,
        DisplayField::NextColorMath,
    ]
    .into_iter()
    .zip(settings)
    {
        game.store_display_control_byte(address, value, DisplayStore::Named);
    }
}

// Ordered changes after the common room defaults, indexed by the configuration's low byte / 2.
const ROOM_SETTINGS: [&[(DisplayField, u8)]; 27] = [
    &[],
    &[],
    &[(DisplayField::MainLayers, 17), (DisplayField::SubLayers, 4)],
    &[
        (DisplayField::SubLayers, 20),
        (DisplayField::NextColorMath, 39),
    ],
    &[
        (DisplayField::SubLayers, 20),
        (DisplayField::NextColorMath, 34),
    ],
    &[(DisplayField::NextColorMath, 50)],
    &[
        (DisplayField::NextColorSelection, 0),
        (DisplayField::NextColorMath, 162),
    ],
    &[],
    &[
        (DisplayField::WindowSelection1, 2),
        (DisplayField::WindowSelection2, 32),
        (DisplayField::SubWindow, 4),
    ],
    &[
        (DisplayField::WindowSelection1, 2),
        (DisplayField::WindowSelection2, 32),
        (DisplayField::SubWindow, 4),
    ],
    &[(DisplayField::NextColorMath, 179)],
    &[
        (DisplayField::MainLayers, 17),
        (DisplayField::SubLayers, 6),
        (DisplayField::NextColorMath, 177),
    ],
    &[],
    &[
        (DisplayField::MainLayers, 21),
        (DisplayField::SubLayers, 2),
        (DisplayField::NextColorMath, 53),
    ],
    &[
        (DisplayField::MainLayers, 21),
        (DisplayField::SubLayers, 2),
        (DisplayField::NextColorMath, 85),
    ],
    &[],
    &[],
    &[(DisplayField::NextColorMath, 179)],
    &[
        (DisplayField::WindowSelection0, 0),
        (DisplayField::WindowSelection1, 2),
        (DisplayField::WindowSelection2, 32),
        (DisplayField::MainLayers, 19),
        (DisplayField::SubLayers, 4),
        (DisplayField::MainWindow, 19),
        (DisplayField::SubWindow, 4),
        (DisplayField::NextColorSelection, 16),
        (DisplayField::NextColorMath, 51),
    ],
    &[(DisplayField::NextColorMath, 119)],
    &[
        (DisplayField::NextColorSelection, 0),
        (DisplayField::NextColorMath, 179),
    ],
    &[
        (DisplayField::NextColorSelection, 0),
        (DisplayField::NextColorMath, 179),
    ],
    &[(DisplayField::NextColorSelection, 0)],
    &[(DisplayField::NextColorMath, 179)],
    &[],
    &[
        (DisplayField::SubLayers, 68),
        (DisplayField::NextColorMath, 178),
    ],
    &[],
];

pub fn update<
    H: crate::fx_domain::Controls
        + crate::display::preparation::Access
        + crate::ram::CompatibilityRam
        + crate::runtime_access::RuntimeAccess
        + ?Sized,
>(
    game: &mut H,
) {
    let configuration = game.fx_controls().blending;
    let mut bomb_mode = 0;
    if configuration as u8 != 0 {
        apply([0, 0, 0, 19, 4, 0, 0, 2, 51], game);
        let mode = (configuration as u8 >> 1) as usize;
        // Index after initialization: an invalid mode leaves the original defaults written.
        for &(address, value) in ROOM_SETTINGS[mode] {
            game.store_display_control_byte(address, value, DisplayStore::Named);
        }
        bomb_mode = match mode {
            11 | 13 => 4,
            12 | 15 | 24 => {
                if game.display_control_byte(DisplayField::NmiEnable) & 0x30 == 0x30 {
                    game.store_display_control_byte(
                        DisplayField::NextColorMath,
                        36,
                        DisplayStore::Named,
                    );
                    game.store_display_control_byte(
                        DisplayField::SubLayers,
                        19,
                        DisplayStore::Named,
                    );
                    game.store_display_control_byte(
                        DisplayField::MainLayers,
                        4,
                        DisplayStore::Named,
                    );
                }
                2
            }
            20 | 21 if configuration & 0x8000 == 0 => {
                game.store_display_control_byte(
                    DisplayField::FixedColor0,
                    if mode == 20 { 37 } else { 38 },
                    DisplayStore::Named,
                );
                game.store_display_control_byte(
                    DisplayField::FixedColor1,
                    if mode == 20 { 64 } else { 66 },
                    DisplayStore::Named,
                );
                game.store_display_control_byte(
                    DisplayField::FixedColor2,
                    0x80,
                    DisplayStore::Named,
                );
                0
            }
            26 => 6,
            _ => 0,
        };
    }
    if configuration & 0x8000 != 0 {
        if room_key(game) == ADDR_K_ROOM_A66A as u16 {
            bomb_mode = 6;
        }
        let special = bomb_mode == 4;
        // Power Bombs write the color-math settings before the screen/window masks.
        game.store_display_control_byte(
            DisplayField::WindowSelection0,
            if special { 0x80 } else { 0 },
            DisplayStore::Named,
        );
        game.store_display_control_byte(DisplayField::WindowSelection1, 8, DisplayStore::Named);
        game.store_display_control_byte(DisplayField::WindowSelection2, 0x80, DisplayStore::Named);
        game.store_display_control_byte(DisplayField::NextColorSelection, 2, DisplayStore::Named);
        game.store_display_control_byte(
            DisplayField::NextColorMath,
            if bomb_mode == 6 { 49 } else { 55 },
            DisplayStore::Named,
        );
        game.store_display_control_byte(DisplayField::MainWindow, 0, DisplayStore::Named);
        game.store_display_control_byte(
            DisplayField::SubWindow,
            if special { 6 } else { 4 },
            DisplayStore::Named,
        );
        game.store_display_control_byte(
            DisplayField::MainLayers,
            if special { 17 } else { 19 },
            DisplayStore::Named,
        );
        game.store_display_control_byte(
            DisplayField::SubLayers,
            if special { 6 } else { 4 },
            DisplayStore::Named,
        );
    } else if configuration & 0x4000 != 0 {
        apply(
            [
                200,
                8,
                128,
                19,
                4,
                3,
                4,
                34,
                game.display_control_byte(DisplayField::NextColorMath) & 0x80 | 0x73,
            ],
            game,
        );
    } else if configuration & 0x2000 != 0 {
        apply(
            [
                0,
                8,
                128,
                19,
                4,
                3,
                4,
                34,
                game.display_control_byte(DisplayField::NextColorMath) & 0x80 | 0x61,
            ],
            game,
        );
        if room_key(game) == ADDR_K_ROOM_CEFB as u16 {
            game.store_display_control_byte(DisplayField::MainLayers, 17, DisplayStore::Named);
        }
    } else if configuration & 0x1000 != 0 {
        apply([0, 8, 128, 19, 0, 3, 4, 32, 179], game);
    }
}
