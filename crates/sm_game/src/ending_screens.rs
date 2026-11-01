//! Ending and post-credits screens: sprite pairs, gunship, Samus, clouds, time and zoom.
use crate::cinematic::*;
use crate::cinematic_objects::CinematicObjects;
use crate::cinematic_state::{Cinematic, Word, cinematic};
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

pub fn initialize_ending_sprite_pair(object: u16, leftward: bool, cine: &mut Cinematic) {
    let position = if leftward { (530, 231) } else { (246, 519) };
    place_cinematic_sprite(object, position, SPRITE_PALETTE, cine);
    *cine.spr_goto_timer(object >> 1) = 8;
}

pub fn advance_ending_sprite_pair(slot: usize, leftward: bool, cine: &mut Cinematic) -> bool {
    let (direction, threshold, stop_x, stop_y) =
        [(1, 395, 394, 367), (-1, 382, 382, 383)][usize::from(!leftward)];
    let step = (i32::from(*cine.spr_goto_timer(slot)) * direction) as u16;
    *cine.bg_arr8(slot) = cine.bg_arr8(slot).wrapping_add(step);
    let x = (*cine.bg_arr7(slot)).wrapping_sub(step);
    *cine.bg_arr7(slot) = x;
    let arrived = ((i32::from(x) - threshold) & 0x8000 != 0) == leftward;
    if arrived {
        *cine.bg_arr7(slot) = stop_x;
        *cine.bg_arr8(slot) = stop_y;
        *cine.spr_pre_instruction(slot) = sprite::NO_PREINSTRUCTION;
    } else {
        *cine.spr_goto_timer(slot) = cine.spr_goto_timer(slot).wrapping_add(2);
    }
    arrived
}

pub fn update_ending_critters(object: u16, initialize: bool, cine: &mut Cinematic) {
    let slot = usize::from(object >> 1);
    if initialize {
        *cine.bg_arr8(slot) = 128;
        *cine.bg_arr7(slot) = 128;
        *cine.bg_arr9(slot) = SPRITE_PALETTE;
        *cine.spr_goto_timer(slot) = 260;
        return;
    }
    advance_cinematic_coordinate(object, true, 128, cine);
    *cine.spr_arr6(slot) = *cine.spr_arr6(slot);
    if ({
        *cine.bg_arr7(slot) = cine.bg_arr7(slot).wrapping_add(1);
        *cine.bg_arr7(slot)
    } as i32
        - 272)
        & 0x8000
        == 0
    {
        resume_cinematic_sprite(slot, sprite::DELETE, cine);
    }
}

pub fn fade_ending_gunship_backdrop<H: crate::menus::host::Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
) {
    let expired = {
        *cine.function_timer() = cine.function_timer().wrapping_sub(1);
        *cine.function_timer()
    } & 0x8000
        != 0;
    if *cine.function_timer() == 0 || expired {
        fade_cinematic_fixed_color(game);
        if game.ram().read_byte(crate::ram::mirror::COLDATA) == 32
            && game.ram().read_byte(crate::ram::bytes::REG_COLDATA + 1) == 64
            && game.ram().read_byte(crate::ram::bytes::REG_COLDATA + 2) == 0x80
        {
            game.ram().write_byte(crate::ram::mirror::CGWSEL, 0);
            game.ram().write_byte(crate::ram::mirror::CGADSUB, 0);
        }
        *cine.function_timer() = 8;
    }
}

pub fn advance_ending_gunship_spin<H: crate::menus::host::Host + ?Sized>(
    slow: bool,
    cine: &mut Cinematic,
    game: &mut H,
) -> bool {
    if slow || *cine.var13() == 0 {
        fade_ending_gunship_backdrop(cine, game);
    } else {
        *cine.var13() = cine.var13().wrapping_sub(1);
    }
    let (rotation, mask) = if slow { (1, 7) } else { (4, 15) };
    if !slow || *cine.mode7_angle() != 224 {
        *cine.mode7_angle() = u16::from((*cine.mode7_angle() as u8).wrapping_sub(rotation));
    }
    let index = usize::from(*cine.var14());
    let velocity = if slow {
        [1i32, 1, 1, -1, -1, 1, 1, -1][index] * 0x10000
    } else {
        [1i32, 1, 1, 1, -1, -1, 1, 1, 1, 1, -1, -1, 1, 1, -1, -1][index] * 0x8000
    };
    cine.add_pair(Word::X, Word::XSub, velocity as u32);
    *cine.var14() = (*cine.var14()).wrapping_add(1) & mask;
    *cine.mode7_zoom() = cine.mode7_zoom().wrapping_sub(if slow { 2 } else { 8 });
    if (*cine.mode7_zoom() as i32 - (if slow { 1184 } else { 1456 })) & 0x8000 == 0 {
        return false;
    }
    if slow {
        crate::flow::set(
            game,
            crate::flow::Word::CinematicFunction,
            function::GUNSHIP_FLY_INTO_CAMERA,
        );
        *cine.var14() = 0x8000;
        *cine.var13() = 0;
    } else {
        *cine.var13() = 1;
        *cine.var14() = 0;
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
    cine: &mut Cinematic,
    game: &mut H,
) {
    if phase == 2 {
        crate::ending_graphics::advance_ending_graphics(EndingGraphics::Samus, cine, game);
    }
    let low = ram_words::SAMUS_Y_SUBSPEED;
    add_native_pair(low + 2, low, 0x3800, game);
    let velocity = words!(game.SAMUS_Y_SPEED, SAMUS_Y_SUBSPEED);
    advance_cinematic_coordinate(object, true, velocity, cine);
    let slot = usize::from(object >> 1);
    if phase == 2 {
        if (*cine.bg_arr8(slot) as i32 - 136) & 0x8000 == 0 {
            *cine.bg_arr8(slot) = 136;
            *cine.spr_instruction_timer(slot) = 1;
            *cine.spr_instruction_pointer(slot) = sprite::JUMP_LANDED;
            *cine.spr_pre_instruction(slot) = sprite::NO_PREINSTRUCTION;
        }
    } else if (*cine.bg_arr8(slot) as i32 + 80) & 0x8000 != 0 {
        if phase == 0 {
            game.ram().write_byte(crate::ram::mirror::OBSEL, 3);
        }
        *cine.spr_instruction_timer(slot) = 1;
        *cine.spr_instruction_pointer(slot) = if phase == 0 {
            sprite::JUMP_FALLING
        } else {
            sprite::DELETE
        };
        if phase == 0 {
            *cine.spr_pre_instruction(slot) = sprite::JUMP_FALLING_PREINSTRUCTION;
            *cine.bg_arr9(slot) = SAMUS_PALETTE;
        }
    }
}

pub fn position_ending_samus_head<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    jumping: bool,
    cursor: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> u16 {
    let slot = usize::from(object >> 1) + 1;
    let under_ten = (word!(game.GAME_TIME_HOURS) as i32 - 10) & 0x8000 != 0;
    let (x, y) = [[(118, 120), (120, 120)], [(120, 114), (121, 116)]][usize::from(jumping)]
        [usize::from(under_ten)];
    *cine.bg_arr7(slot) = x;
    *cine.bg_arr8(slot) = y;
    cursor
}

pub fn fade_postcredits_samus_suit<H: crate::menus::host::Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
) {
    if *cine.function_timer() & 3 == 0 {
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
    cine: &mut Cinematic,
    game: &mut H,
) {
    if stage == 1 {
        fade_postcredits_samus_suit(cine, game);
    }
    let initial = stage == 0;
    let timer = {
        *cine.function_timer() = cine.function_timer().wrapping_sub(1);
        *cine.function_timer()
    };
    if (initial && timer != 0) || (!initial && timer & 0x8000 == 0) {
        return;
    }
    if initial {
        *cine.function_timer() = 127;
    } else {
        set_cinematic_blend(16, 0, 0, 0, game);
        game.ram().write_byte(crate::ram::mirror::BGMODE, 7);
        game.ram().write_byte(crate::ram::mirror::M7SEL, 0x80);
        game.ram().write_byte(crate::ram::mirror::BG1SC, 0);
        game.ram().write_byte(crate::ram::mirror::BG12NBA, 0);
        crate::flow::set(game, crate::flow::Word::CinematicFunction, function::IDLE);
        *cine.var14() = 0;
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
        CinematicObjects::Sprites.spawn(definition, parameter, cine, game);
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
    cine: &mut Cinematic,
    game: &mut H,
) {
    if fade_suit {
        fade_postcredits_samus_suit(cine, game);
    }
    if ({
        *cine.function_timer() = cine.function_timer().wrapping_sub(1);
        *cine.function_timer()
    } as i32
        - 65)
        & 0x8000
        != 0
    {
        crate::word_transfers!(game, descending, 288; native(0x347e) <- value(127));
        crate::word_transfers!(game, ascending, 64; pause_map(384) <- rom(0x8cdedb, 0));
        queue_cinematic_tilemap(CinematicTilemap::FullText, cine, game);
        set_cinematic_blend(1, 0, 0, 0, game);
        *cine.var13() = 180;
        crate::flow::set(
            game,
            crate::flow::Word::CinematicFunction,
            function::NINTENDO_1994,
        );
    }
}

// Cloud setup reads its parameter after the first position store, even for aliased slots.
pub fn initialize_ending_cloud_band(object: u16, y: u16, cine: &mut Cinematic) {
    let slot = usize::from(object >> 1);
    *cine.bg_arr8(slot) = y;
    let alternate = *cine.spawn_parameter() != 0;
    *cine.bg_arr7(slot) = if alternate { 384 } else { 128 };
    *cine.bg_arr9(slot) = 2560;
    if alternate {
        *cine.spr_instruction_pointer(slot) = cine.spr_instruction_pointer(slot).wrapping_add(4);
    }
}

pub fn initialize_ending_cloud_side(object: u16, right: bool, cine: &mut Cinematic) {
    let slot = usize::from(object >> 1);
    *cine.bg_arr7(slot) = if right { 320 } else { -64i32 as u16 };
    let alternate = *cine.spawn_parameter() != 0;
    let y: u16 = if right { 192 } else { 64 };
    let displacement = if right { 256 } else { -256i32 as u16 };
    *cine.bg_arr8(slot) = y.wrapping_add(if alternate { displacement } else { 0 });
    *cine.bg_arr9(slot) = 2560;
}

pub fn wait_ending_cloud(object: u16, band: bool, next: u16, cine: &mut Cinematic) {
    let threshold = if band { 176 } else { 96 };
    let below = (*cine.mode7_zoom()).wrapping_sub(threshold) & 0x8000 != 0;
    if below == band {
        *cine.spr_pre_instruction(object >> 1) = next;
    }
}

pub fn move_ending_cloud(object: u16, velocity: (i16, i16), cine: &mut Cinematic) {
    let slot = usize::from(object >> 1);
    // Retain fractional self-stores before advancing each whole coordinate.
    *cine.spr_arr7(slot) = *cine.spr_arr7(slot);
    *cine.bg_arr8(slot) = cine.bg_arr8(slot).wrapping_add(velocity.1 as u16);
    if velocity.0 != 0 {
        *cine.spr_arr6(slot) = *cine.spr_arr6(slot);
        *cine.bg_arr7(slot) = cine.bg_arr7(slot).wrapping_add(velocity.0 as u16);
    }
}

pub fn prepare_ending_time<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let hours = word!(game.GAME_TIME_HOURS);
    cinematic! { game; SequenceStage = hours / 10; SequenceValue = hours % 10; }
    let minutes = word!(game.GAME_TIME_MINUTES);
    crate::word_transfers!(game, ascending, 1;
        native(0xdf0) <- value(minutes / 10), native(0xdf2) <- value(minutes % 10),
    );
}

pub fn initialize_ending_time_glyph<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    glyph: u8,
    cine: &mut Cinematic,
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
        *cine.spr_instruction_pointer(object >> 1) = cine
            .spr_instruction_pointer(object >> 1)
            .wrapping_add(digit.wrapping_mul(8));
    }
    place_cinematic_sprite(object, (156 + 8 * u16::from(glyph), 160), 0, cine);
}

pub fn advance_postcredits_zoom<H: crate::menus::host::Host + ?Sized>(
    zooming: bool,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if zooming && *cine.var14() != 0 {
        *cine.var14() = cine.var14().wrapping_sub(1);
        fade_cinematic_colors(0x180, 0x10, false, game);
        if *cine.function_timer() != 0 {
            compose_cinematic_palette(game);
        }
    }
    if !zooming || *cine.function_timer() == 0 {
        if *cine.var17() != 0 {
            *cine.var17() = cine.var17().wrapping_sub(1);
            fade_cinematic_colors(0x1e0, 0x10, false, game);
            compose_cinematic_palette(game);
        }
    } else {
        *cine.function_timer() = cine.function_timer().wrapping_sub(1);
    }
    if zooming {
        *cine.mode7_angle() = u16::from((*cine.mode7_angle() as u8).wrapping_sub(8));
        *cine.mode7_zoom() = cine.mode7_zoom().wrapping_sub(64);
        if (*cine.mode7_zoom() as i32 - 24) & 0x8000 != 0 {
            *cine.mode7_zoom() = 24;
            crate::flow::set(
                game,
                crate::flow::Word::CinematicFunction,
                function::FADE_TO_WHITE,
            );
            *cine.function_timer() = 180;
            *cine.var14() = 0;
        }
        return;
    }
    crate::ending_graphics::advance_ending_graphics(EndingGraphics::Background, cine, game);
    if {
        *cine.function_timer() = cine.function_timer().wrapping_sub(1);
        *cine.function_timer()
    } == 0
    {
        game.ram().write_byte(crate::ram::mirror::BGMODE, 1);
        set_cinematic_blend(17, 0, 0, 49, game);
        set_fixed_color([63, 95, 159], game);
        game.ram().write_byte(crate::ram::mirror::M7SEL, 0);
        game.ram().write_byte(crate::ram::mirror::BG1SC, 78);
        game.ram().write_byte(crate::ram::mirror::BG12NBA, 84);
        crate::word_transfers!(game, ascending, 16; le(PALETTE_BUFFER, 192) <- rom(0x8ce7e9, 192));
        *cine.x() = 0;
        *cine.y() = 0;
        crate::flow::set(
            game,
            crate::flow::Word::CinematicFunction,
            function::FADE_FROM_WHITE,
        );
        *cine.function_timer() = 32;
        enable_cinematic_background(cine);
        set_cinematic_tilemap_updates(true, cine);
        *cine.bg_vram_address() = 19456;
        crate::word_transfers!(game, descending, 64; native(0x337e) <- value(127));
        queue_cinematic_tilemap(CinematicTilemap::FullText, cine, game);
    }
}
