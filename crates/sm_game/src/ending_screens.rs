//! Ending and post-credits screens: sprite pairs, gunship, Samus, clouds, time and zoom.
use crate::cinematic::*;
use crate::cinematic_objects::CinematicObjects;
use crate::cinematic_state::cinematic;
use crate::menus::prelude::words;
use crate::ram::RawRam as _;
use crate::ram::{word, words as ram_words};
use crate::visual_state::{CinematicTilemap, EndingGraphics};

/// Bank $8B cinematic functions the ending sets, with the bank log's names.
pub mod function {
    /// $8B:DB9D: an RTS; the cinematic waits on its objects.
    pub const IDLE: u16 = 0xdb9d;
    /// $8B:DD42: space view, gunship emergence, spinning slow.
    pub const GUNSHIP_SPINNING_SLOW: u16 = 0xdd42;
    /// $8B:DDCD: space view, gunship emergence, fly into camera.
    pub const GUNSHIP_FLY_INTO_CAMERA: u16 = 0xddcd;
    /// $8B:E265: post-credits idle Samus 1/2, cross-fading out Samus's suit.
    pub const IDLE_SAMUS_SUIT_CROSS_FADE: u16 = 0xe265;
    /// $8B:E293: post-credits idle Samus 1/2.
    pub const IDLE_SAMUS: u16 = 0xe293;
    /// $8B:E2DD: post-credits 1994 Nintendo.
    pub const NINTENDO_1994: u16 = 0xe2dd;
    /// $8B:E48A: post-credits fade to white.
    pub const FADE_TO_WHITE: u16 = 0xe48a;
    /// $8B:E504: post-credits fade from white.
    pub const FADE_FROM_WHITE: u16 = 0xe504;
}

/// Bank $8B cinematic sprite scripts, with the bank log's names.
mod sprite {
    /// $8B:93D9: an RTS pre-instruction.
    pub const NO_PREINSTRUCTION: u16 = 0x93d9;
    /// $8B:ED95: Samus's jump, falling.
    pub const JUMP_FALLING: u16 = 0xed95;
    /// $8B:ED9D: Samus's jump, landed.
    pub const JUMP_LANDED: u16 = 0xed9d;
    /// $8B:EE9B: delete.
    pub const DELETE: u16 = 0xee9b;
    /// $8B:F5DD: the pre-instruction of Samus's falling jump.
    pub const JUMP_FALLING_PREINSTRUCTION: u16 = 0xf5dd;
}

/// Bank $8B cinematic sprite definitions of the post-credits Samus, with the
/// bank log's names. Suitless Samus is the best ending's; the helmeted head
/// the worst's, the bare head the mediocre's, and the suited body both.
pub mod samus {
    /// $8B:EF27..EF3F: suitless Samus idle, its legs, letting her hair down,
    /// its legs, and her jump.
    pub const SUITLESS_IDLE: u16 = 0xef27;
    pub const SUITLESS_IDLE_LEGS: u16 = 0xef2d;
    pub const SUITLESS_LETTING_HAIR_DOWN: u16 = 0xef33;
    pub const SUITLESS_LETTING_HAIR_DOWN_LEGS: u16 = 0xef39;
    pub const SUITLESS_JUMP: u16 = 0xef3f;
    /// $8B:EF45..EF51: suited Samus idle, body and either head.
    pub const SUITED_IDLE_BODY: u16 = 0xef45;
    pub const SUITED_IDLE_HEAD_WITH_HELMET: u16 = 0xef4b;
    pub const SUITED_IDLE_HEAD_WITHOUT_HELMET: u16 = 0xef51;
    /// $8B:EF57..EF69: suited Samus thumbs up, body, arm and either head.
    pub const SUITED_THUMBS_UP_BODY: u16 = 0xef57;
    pub const SUITED_THUMBS_UP_ARM: u16 = 0xef5d;
    pub const SUITED_THUMBS_UP_HEAD_WITH_HELMET: u16 = 0xef63;
    pub const SUITED_THUMBS_UP_HEAD_WITHOUT_HELMET: u16 = 0xef69;
    /// $8B:EF6F..EF7B: suited Samus jump, body and either head.
    pub const SUITED_JUMP_BODY: u16 = 0xef6f;
    pub const SUITED_JUMP_HEAD_WITH_HELMET: u16 = 0xef75;
    pub const SUITED_JUMP_HEAD_WITHOUT_HELMET: u16 = 0xef7b;
}

/// The ending sprites' palette index: palette 7, a multiple of $200.
const SPRITE_PALETTE: u16 = 7 * 0x200;
/// The Samus sprite's palette after the jump: palette 6.
const SAMUS_PALETTE: u16 = 6 * 0x200;

const _: () = assert!(SPRITE_PALETTE == 3584 && SAMUS_PALETTE == 3072);

pub fn initialize_ending_sprite_pair<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    leftward: bool,
    game: &mut H,
) {
    let position = if leftward { (530, 231) } else { (246, 519) };
    place_cinematic_sprite(object, position, SPRITE_PALETTE, game);
    cinematic!(game.SprGotoTimer[object >> 1] = 8);
}

pub fn advance_ending_sprite_pair<H: crate::menus::host::Host + ?Sized>(
    slot: usize,
    leftward: bool,
    game: &mut H,
) -> bool {
    let (direction, threshold, stop_x, stop_y) =
        [(1, 395, 394, 367), (-1, 382, 382, 383)][usize::from(!leftward)];
    let step = (i32::from(cinematic!(game.SprGotoTimer[slot])) * direction) as u16;
    cinematic!(game.BgArr8[slot] += step);
    let x = cinematic!(game.BgArr7[slot]).wrapping_sub(step);
    cinematic!(game.BgArr7[slot] = x);
    let arrived = ((i32::from(x) - threshold) & 0x8000 != 0) == leftward;
    if arrived {
        cinematic! {game; BgArr7[slot] = stop_x; BgArr8[slot] = stop_y;
        SprPreInstruction[slot] = sprite::NO_PREINSTRUCTION; }
    } else {
        cinematic!(game.SprGotoTimer[slot] += 2);
    }
    arrived
}

pub fn update_ending_critters<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    initialize: bool,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if initialize {
        cinematic! { game, 128; BgArr8[slot], BgArr7[slot] }
        cinematic! {game; BgArr9[slot] = SPRITE_PALETTE; SprGotoTimer[slot] = 260; }
        return;
    }
    advance_cinematic_coordinate(object, true, 128, game);
    cinematic!(game.SprArr6[slot] = cinematic!(game.SprArr6[slot]));
    if (cinematic!(game.BgArr7[slot] += 1, reread) as i32 - 272) & 0x8000 == 0 {
        resume_cinematic_sprite(slot, sprite::DELETE, game);
    }
}

pub fn fade_ending_gunship_backdrop<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let expired = cinematic!(game.FunctionTimer -= 1, reread) & 0x8000 != 0;
    if cinematic!(game.FunctionTimer) == 0 || expired {
        fade_cinematic_fixed_color(game);
        if game.ram().read_byte(crate::ram::mirror::COLDATA) == 32
            && game.ram().read_byte(crate::ram::bytes::REG_COLDATA + 1) == 64
            && game.ram().read_byte(crate::ram::bytes::REG_COLDATA + 2) == 0x80
        {
            game.ram().write_byte(crate::ram::mirror::CGWSEL, 0);
            game.ram().write_byte(crate::ram::mirror::CGADSUB, 0);
        }
        cinematic!(game.FunctionTimer = 8);
    }
}

pub fn advance_ending_gunship_spin<H: crate::menus::host::Host + ?Sized>(
    slow: bool,
    game: &mut H,
) -> bool {
    if slow || cinematic!(game.Var13) == 0 {
        fade_ending_gunship_backdrop(game);
    } else {
        cinematic!(game.Var13 -= 1);
    }
    let (rotation, mask) = if slow { (1, 7) } else { (4, 15) };
    if !slow || cinematic!(game.Mode7Angle) != 224 {
        cinematic!(
            game.Mode7Angle = u16::from((cinematic!(game.Mode7Angle) as u8).wrapping_sub(rotation))
        );
    }
    let index = usize::from(cinematic!(game.Var14));
    let velocity = if slow {
        [1i32, 1, 1, -1, -1, 1, 1, -1][index] * 0x10000
    } else {
        [1i32, 1, 1, 1, -1, -1, 1, 1, 1, 1, -1, -1, 1, 1, -1, -1][index] * 0x8000
    };
    let fraction = ram_words::CINEMATIC_X_SUB;
    add_native_pair(fraction + 2, fraction, velocity as u32, game);
    cinematic!(game.Var14 = cinematic!(game.Var14).wrapping_add(1) & mask);
    cinematic!(game.Mode7Zoom -= if slow { 2 } else { 8 });
    if (cinematic!(game.Mode7Zoom) as i32 - (if slow { 1184 } else { 1456 })) & 0x8000 == 0 {
        return false;
    }
    if slow {
        crate::flow::set(
            game,
            crate::flow::Word::CinematicFunction,
            function::GUNSHIP_FLY_INTO_CAMERA,
        );
        cinematic!(game.Var14 = 0x8000);
        cinematic!(game.Var13 = 0);
    } else {
        cinematic!(game.Var13 = 1);
        cinematic!(game.Var14 = 0);
        crate::flow::set(
            game,
            crate::flow::Word::CinematicFunction,
            function::GUNSHIP_SPINNING_SLOW,
        );
    }
    true
}

pub fn advance_ending_samus_jump<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    phase: u8,
    game: &mut H,
) {
    if phase == 2 {
        crate::ending_graphics::advance_ending_graphics(EndingGraphics::Samus, game);
    }
    let low = ram_words::SAMUS_Y_SUBSPEED;
    add_native_pair(low + 2, low, 0x3800, game);
    let velocity = words!(game.SAMUS_Y_SPEED, SAMUS_Y_SUBSPEED);
    advance_cinematic_coordinate(object, true, velocity, game);
    let slot = usize::from(object >> 1);
    if phase == 2 {
        if (cinematic!(game.BgArr8[slot]) as i32 - 136) & 0x8000 == 0 {
            cinematic! {game; BgArr8[slot] = 136; SprInstructionTimer[slot] = 1;
            SprInstructionPointer[slot] = sprite::JUMP_LANDED; SprPreInstruction[slot] = sprite::NO_PREINSTRUCTION; }
        }
    } else if (cinematic!(game.BgArr8[slot]) as i32 + 80) & 0x8000 != 0 {
        if phase == 0 {
            game.ram().write_byte(crate::ram::mirror::OBSEL, 3);
        }
        cinematic! {game; SprInstructionTimer[slot] = 1;
        SprInstructionPointer[slot] = if phase == 0 { sprite::JUMP_FALLING } else { sprite::DELETE }; }
        if phase == 0 {
            cinematic! {game; SprPreInstruction[slot] = sprite::JUMP_FALLING_PREINSTRUCTION; BgArr9[slot] = SAMUS_PALETTE; }
        }
    }
}

pub fn position_ending_samus_head<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    jumping: bool,
    cursor: u16,
    game: &mut H,
) -> u16 {
    let slot = usize::from(object >> 1) + 1;
    let under_ten = (word!(game.GAME_TIME_HOURS) as i32 - 10) & 0x8000 != 0;
    let (x, y) = [[(118, 120), (120, 120)], [(120, 114), (121, 116)]][usize::from(jumping)]
        [usize::from(under_ten)];
    cinematic! {game; BgArr7[slot] = x; BgArr8[slot] = y; }
    cursor
}

pub fn fade_postcredits_samus_suit<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if cinematic!(game.FunctionTimer) & 3 == 0 {
        fade_cinematic_colors(0x40, 0x10, false, game);
        if (word!(game.GAME_TIME_HOURS) as i32 - 3) & 0x8000 == 0 {
            fade_cinematic_colors(0x1c0, 0x10, true, game);
        }
        fade_cinematic_colors(0x1a0, 0x10, true, game);
        compose_cinematic_palette(game);
    }
}

pub fn advance_postcredits_samus_reveal<H: crate::menus::host::Host + ?Sized>(
    stage: u8,
    game: &mut H,
) {
    if stage == 1 {
        fade_postcredits_samus_suit(game);
    }
    let initial = stage == 0;
    let timer = cinematic!(game.FunctionTimer -= 1, reread);
    if (initial && timer != 0) || (!initial && timer & 0x8000 == 0) {
        return;
    }
    if initial {
        cinematic!(game.FunctionTimer = 127);
    } else {
        set_cinematic_blend(16, 0, 0, 0, game);
        game.ram().write_byte(crate::ram::mirror::BGMODE, 7);
        game.ram().write_byte(crate::ram::mirror::M7SEL, 0x80);
        game.ram().write_byte(crate::ram::mirror::BG1SC, 0);
        game.ram().write_byte(crate::ram::mirror::BG12NBA, 0);
        crate::flow::set(game, crate::flow::Word::CinematicFunction, function::IDLE);
        cinematic!(game.Var14 = 0);
    }
    let under_three = (word!(game.GAME_TIME_HOURS) as i32 - 3) & 0x8000 != 0;
    let under_ten = under_three || (word!(game.GAME_TIME_HOURS) as i32 - 10) & 0x8000 != 0;
    if initial && under_ten {
        capture_cinematic_palette(game);
        prepare_cinematic_palette_fade(game);
    }
    let sprites: &[u16] = match (initial, under_three, under_ten) {
        (true, true, _) => &[samus::SUITLESS_IDLE, samus::SUITLESS_IDLE_LEGS],
        (true, false, true) => &[
            samus::SUITED_IDLE_HEAD_WITHOUT_HELMET,
            samus::SUITED_IDLE_BODY,
        ],
        (true, false, false) => &[samus::SUITED_IDLE_HEAD_WITH_HELMET, samus::SUITED_IDLE_BODY],
        (false, true, _) => &[
            samus::SUITLESS_LETTING_HAIR_DOWN,
            samus::SUITLESS_LETTING_HAIR_DOWN_LEGS,
        ],
        (false, false, true) => &[
            samus::SUITED_THUMBS_UP_HEAD_WITHOUT_HELMET,
            samus::SUITED_THUMBS_UP_ARM,
            samus::SUITED_THUMBS_UP_BODY,
        ],
        (false, false, false) => &[
            samus::SUITED_THUMBS_UP_HEAD_WITH_HELMET,
            samus::SUITED_THUMBS_UP_ARM,
            samus::SUITED_THUMBS_UP_BODY,
        ],
    };
    // $938A takes A as the spawn parameter, and nothing here sets it: the
    // first spawn gets the hours just compared, or after the palette
    // decomposition its last blue delta ($7E:2FFE); each later one the
    // palette index the previous initializer loaded.
    let mut parameter = if initial && under_ten {
        cinematic!(game.TilemapStuff[2047])
    } else {
        word!(game.GAME_TIME_HOURS)
    };
    for &definition in sprites {
        CinematicObjects::Sprites.spawn(definition, parameter, game);
        parameter = initializer_palette(definition);
    }
    if initial {
        if !under_ten {
            crate::flow::set(
                game,
                crate::flow::Word::CinematicFunction,
                function::IDLE_SAMUS,
            );
            set_cinematic_blend(16, 0, 0, 0, game);
        } else {
            if !under_three {
                clear_cinematic_colors(0x1c0, 0x10, game);
            }
            clear_cinematic_colors(0x1a0, 0x10, game);
            compose_cinematic_palette(game);
            set_cinematic_blend(2, 16, 2, 34, game);
            crate::flow::set(
                game,
                crate::flow::Word::CinematicFunction,
                function::IDLE_SAMUS_SUIT_CROSS_FADE,
            );
        }
    }
}

/// The palette index a post-credits Samus sprite's initializer stores last
/// (and so leaves in A): palette 5 for suitless Samus and the head without
/// the helmet ($8B:F143, $8B:F17C), palette 6 for the rest ($8B:F156,
/// $8B:F169).
pub(crate) fn initializer_palette(definition: u16) -> u16 {
    use samus::*;
    match definition {
        SUITLESS_IDLE
        | SUITLESS_IDLE_LEGS
        | SUITLESS_LETTING_HAIR_DOWN
        | SUITLESS_LETTING_HAIR_DOWN_LEGS
        | SUITLESS_JUMP
        | SUITED_IDLE_HEAD_WITHOUT_HELMET
        | SUITED_THUMBS_UP_HEAD_WITHOUT_HELMET
        | SUITED_JUMP_HEAD_WITHOUT_HELMET => 5 * 0x200,
        _ => SAMUS_PALETTE,
    }
}

pub fn advance_postcredits_samus_message<H: crate::menus::host::Host + ?Sized>(
    fade_suit: bool,
    game: &mut H,
) {
    if fade_suit {
        fade_postcredits_samus_suit(game);
    }
    if (cinematic!(game.FunctionTimer -= 1, reread) as i32 - 65) & 0x8000 != 0 {
        crate::word_transfers!(game, descending, 288; native(0x347e) <- value(127));
        crate::word_transfers!(game, ascending, 64; pause_map(384) <- rom(0x8cdedb, 0));
        queue_cinematic_tilemap(CinematicTilemap::FullText, game);
        set_cinematic_blend(1, 0, 0, 0, game);
        cinematic!(game.Var13 = 180);
        crate::flow::set(
            game,
            crate::flow::Word::CinematicFunction,
            function::NINTENDO_1994,
        );
    }
}

// Cloud setup reads its parameter after the first position store, even for aliased slots.
pub fn initialize_ending_cloud_band<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    y: u16,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    cinematic!(game.BgArr8[slot] = y);
    let alternate = cinematic!(game.SpawnParameter) != 0;
    cinematic!(game.BgArr7[slot] = if alternate { 384 } else { 128 });
    cinematic!(game.BgArr9[slot] = 2560);
    if alternate {
        cinematic!(game.SprInstructionPointer[slot] += 4);
    }
}

pub fn initialize_ending_cloud_side<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    right: bool,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    cinematic!(game.BgArr7[slot] = if right { 320 } else { -64i32 as u16 });
    let alternate = cinematic!(game.SpawnParameter) != 0;
    let y: u16 = if right { 192 } else { 64 };
    let displacement = if right { 256 } else { -256i32 as u16 };
    cinematic!(game.BgArr8[slot] = y.wrapping_add(if alternate { displacement } else { 0 }));
    cinematic!(game.BgArr9[slot] = 2560);
}

pub fn wait_ending_cloud<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    band: bool,
    next: u16,
    game: &mut H,
) {
    let threshold = if band { 176 } else { 96 };
    let below = cinematic!(game.Mode7Zoom).wrapping_sub(threshold) & 0x8000 != 0;
    if below == band {
        cinematic!(game.SprPreInstruction[object >> 1] = next);
    }
}

pub fn move_ending_cloud<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    velocity: (i16, i16),
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    // Retain fractional self-stores before advancing each whole coordinate.
    cinematic!(game.SprArr7[slot] = cinematic!(game.SprArr7[slot]));
    cinematic!(game.BgArr8[slot] += velocity.1 as u16);
    if velocity.0 != 0 {
        cinematic!(game.SprArr6[slot] = cinematic!(game.SprArr6[slot]));
        cinematic!(game.BgArr7[slot] += velocity.0 as u16);
    }
}

pub fn prepare_ending_time<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let hours = word!(game.GAME_TIME_HOURS);
    word! { game; SUBSTATE = hours / 10; SUIT_PICKUP_LIGHT_BEAM_POS = hours % 10; }
    let minutes = word!(game.GAME_TIME_MINUTES);
    crate::word_transfers!(game, ascending, 1;
        native(0xdf0) <- value(minutes / 10), native(0xdf2) <- value(minutes % 10),
    );
}

pub fn initialize_ending_time_glyph<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    glyph: u8,
    game: &mut H,
) {
    if glyph != 2 {
        let at = 0xdec + 2 * usize::from(glyph - u8::from(glyph > 2));
        let bytes = [game.ram().read_byte(at), game.ram().read_byte(at + 1)];
        // Hours are LE; minute digits retain native word order.
        let digit = if glyph < 2 {
            u16::from_le_bytes(bytes)
        } else {
            u16::from_ne_bytes(bytes)
        };
        cinematic!(game.SprInstructionPointer[object >> 1] += digit.wrapping_mul(8));
    }
    place_cinematic_sprite(object, (156 + 8 * u16::from(glyph), 160), 0, game);
}

pub fn advance_postcredits_zoom<H: crate::menus::host::Host + ?Sized>(zooming: bool, game: &mut H) {
    if zooming && cinematic!(game.Var14) != 0 {
        cinematic!(game.Var14 -= 1);
        fade_cinematic_colors(0x180, 0x10, false, game);
        if cinematic!(game.FunctionTimer) != 0 {
            compose_cinematic_palette(game);
        }
    }
    if !zooming || cinematic!(game.FunctionTimer) == 0 {
        if cinematic!(game.Var17) != 0 {
            cinematic!(game.Var17 -= 1);
            fade_cinematic_colors(0x1e0, 0x10, false, game);
            compose_cinematic_palette(game);
        }
    } else {
        cinematic!(game.FunctionTimer -= 1);
    }
    if zooming {
        cinematic!(
            game.Mode7Angle = u16::from((cinematic!(game.Mode7Angle) as u8).wrapping_sub(8))
        );
        cinematic!(game.Mode7Zoom -= 64);
        if (cinematic!(game.Mode7Zoom) as i32 - 24) & 0x8000 != 0 {
            cinematic!(game.Mode7Zoom = 24);
            crate::flow::set(
                game,
                crate::flow::Word::CinematicFunction,
                function::FADE_TO_WHITE,
            );
            cinematic!(game.FunctionTimer = 180);
            cinematic!(game.Var14 = 0);
        }
        return;
    }
    crate::ending_graphics::advance_ending_graphics(EndingGraphics::Background, game);
    if cinematic!(game.FunctionTimer -= 1, reread) == 0 {
        game.ram().write_byte(crate::ram::mirror::BGMODE, 1);
        set_cinematic_blend(17, 0, 0, 49, game);
        set_fixed_color([63, 95, 159], game);
        game.ram().write_byte(crate::ram::mirror::M7SEL, 0);
        game.ram().write_byte(crate::ram::mirror::BG1SC, 78);
        game.ram().write_byte(crate::ram::mirror::BG12NBA, 84);
        crate::word_transfers!(game, ascending, 16; le(PALETTE_BUFFER, 192) <- rom(0x8ce7e9, 192));
        cinematic! { game, 0; X, Y }
        crate::flow::set(
            game,
            crate::flow::Word::CinematicFunction,
            function::FADE_FROM_WHITE,
        );
        cinematic!(game.FunctionTimer = 32);
        enable_cinematic_background(game);
        set_cinematic_tilemap_updates(true, game);
        cinematic!(game.BgVramAddress = 19456);
        crate::word_transfers!(game, descending, 64; native(0x337e) <- value(127));
        queue_cinematic_tilemap(CinematicTilemap::FullText, game);
    }
}
