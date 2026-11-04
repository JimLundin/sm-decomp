//! Cinematic palettes, fades, live tilemap queueing and sprite resumption.
use crate::cinematic_state::Cinematic;
use crate::display::preparation::Field;
use crate::presentation_workspace::{Point, Role};
use crate::word_transfers::WordEndpoint;

/// A bank-`BANK` cinematic record at `cursor`, after the low-half diagnostic.
pub fn cinematic_native_record<const BANK: u8>(
    cursor: u16,
    game: &mut (impl crate::cartridge::NativeDiagnostics + ?Sized),
) -> sm_data::cartridge::RomRecord {
    crate::menus::menu_tilemap::native_record((u32::from(BANK) << 16) | u32::from(cursor), game)
}

pub fn advance_cinematic_fade<H: crate::cinematic_host::Scene + ?Sized>(
    fade: crate::visual_state::CinematicFade,
    game: &mut H,
) -> bool {
    game.cinematic_screen_fade(fade)
}

/// Run `change` on the lease's palette buffer colours ($7E:C000) and fade
/// planes ($7E:2000), then refresh its observations if they changed, as the
/// typed copies' stores did.
fn change_palette<H: crate::cinematic_host::Host>(
    game: &mut H,
    change: impl FnOnce(&mut [u16; 256], &mut [u16]),
) {
    let session = game.session_mut();
    let colors = &mut session.presentation.visual_mut().colors;
    let before_colors = *colors;
    let planes = session.actors.extended.blocks[4..6].as_flattened_mut();
    let before_planes = planes.to_vec();
    change(colors, planes);
    let session = game.session_mut();
    let changed = session.presentation.visual().colors != before_colors
        || session.actors.extended.blocks[4..6].as_flattened() != before_planes.as_slice();
    if changed {
        game.refresh_inputs();
    }
}

/// Store `value` to palette buffer colours `indices`.
pub fn set_cinematic_colors<H: crate::cinematic_host::Host>(
    indices: &[usize],
    value: u16,
    game: &mut H,
) {
    change_palette(game, |palette, _| {
        for &index in indices {
            palette[index] = value;
        }
    });
}

pub fn clear_cinematic_colors<H: crate::cinematic_host::Host>(
    offset: u16,
    count: u16,
    game: &mut H,
) {
    change_palette(game, |_, planes| {
        for index in cinematic_color_indices(offset, count) {
            for plane in [256, 512, 768, 1024] {
                planes[index + plane] = 0;
            }
        }
    });
}

pub fn compose_cinematic_palette<H: crate::cinematic_host::Host>(game: &mut H) {
    change_palette(game, |palette, planes| {
        for index in 0..256 {
            // Red is the high byte of plane 2's word.
            let red = u16::from(planes[index + 512].to_le_bytes()[1] & 31);
            let green = (planes[index + 768] >> 3) & 0x3e0;
            let blue = (planes[index + 1024] << 2) & 0x7c00;
            let color = red | green | blue;
            planes[index] = color;
            palette[index] = color;
        }
    });
}

pub fn fade_cinematic_colors<H: crate::cinematic_host::Host>(
    offset: u16,
    count: u16,
    increasing: bool,
    game: &mut H,
) {
    change_palette(game, |_, planes| {
        for index in cinematic_color_indices(offset, count) {
            for plane in [512, 768, 1024] {
                let current = index + plane;
                let step = planes[current + 768];
                let word = &mut planes[current];
                *word = if increasing {
                    step.wrapping_add(*word) & 0x1fff
                } else {
                    word.wrapping_sub(step)
                };
            }
        }
    });
}

pub fn prepare_cinematic_palette_fade<H: crate::cinematic_host::Host>(game: &mut H) {
    change_palette(game, |_, planes| {
        for index in 0..256 {
            let color = planes[index + 256];
            for channel in 0..3 {
                let component = (color >> (5 * channel)) & 31;
                let current = index + 512 + 256 * channel;
                planes[current] = component << 8;
                planes[current + 768] = component << 3;
            }
        }
    });
}

pub fn queue_cinematic_tilemap<H: crate::cinematic_host::Host>(
    tiles: crate::visual_state::CinematicTilemap,
    cine: &mut Cinematic,
    game: &mut H,
) {
    use crate::visual_state::CinematicTilemap::*;
    if !matches!(tiles, Japanese | JapaneseIntro) && game.display_control_byte(Field::Mode) == 7 {
        return;
    }
    let (size, source) = match tiles {
        Text => (1920, 0x3000),
        FullText => (2048, 0x3000),
        Eyes => (128, 0x3b40),
        Japanese => (1536, 0x4000),
        JapaneseIntro => (256, 0x3000),
    };
    crate::display_refresh::queue_native_transfer(
        game.queue_tail(),
        size,
        |_| source,
        126,
        // Earlier queue writes can overlap this destination field.
        |_| match tiles {
            Eyes => 18848,
            Japanese => 16768,
            JapaneseIntro => 20192,
            _ => *cine.bg_vram_address(),
        },
        game,
    );
}

pub fn set_cinematic_blend<H: crate::cinematic_host::Host>(
    main_screen: u8,
    sub_screen: u8,
    color_window: u8,
    color_math: u8,
    game: &mut H,
) {
    crate::frame_display_byte!(game, Field::MainLayers => main_screen);
    crate::frame_display_byte!(game, Field::SubLayers => sub_screen);
    crate::frame_display_byte!(game, Field::ColorSelection => color_window);
    crate::frame_display_byte!(game, Field::ColorMath => color_math);
}

pub fn set_cinematic_tilemap_updates(enabled: bool, cine: &mut Cinematic) {
    let updates = cine.enable_bg_tilemap();
    *updates = (*updates & 0x7fff) | (u16::from(enabled) << 15);
}

pub fn resume_cinematic_sprite(slot: usize, instruction: u16, cine: &mut Cinematic) {
    *cine.spr_instruction_timer(slot) = 1;
    *cine.spr_instruction_pointer(slot) = instruction;
}

pub fn cinematic_color_indices(mut offset: u16, count: u16) -> impl Iterator<Item = usize> {
    // The do-while count admits zero (65,536 entries) and wrapping odd byte offsets.
    (0..=count.wrapping_sub(1)).map(move |_| {
        let index = usize::from(offset >> 1);
        offset = offset.wrapping_add(2);
        index
    })
}

/// Advance the ending jump's actual player velocity, fraction before whole.
pub fn advance_cinematic_jump_velocity<H: crate::sequence_state::Access + ?Sized>(
    game: &mut H,
) -> u32 {
    use crate::session_words::Word;
    let speed = game.sequence_words();
    let pair = (u32::from(speed.samus_y_subspeed.to_le())
        | (u32::from(speed.samus_y_speed.to_le()) << 16))
        .wrapping_add(0x3800);
    let [low_0, low_1, high_0, high_1] = pair.to_le_bytes();
    game.set_sequence_word(
        Word::SamusYSubspeed,
        u16::from_le_bytes([low_0, low_1]).to_le(),
    );
    game.set_sequence_word(
        Word::SamusYSpeed,
        u16::from_le_bytes([high_0, high_1]).to_le(),
    );
    let speed = game.sequence_words();
    (u32::from(speed.samus_y_speed) << 16) | u32::from(speed.samus_y_subspeed)
}

/// Add the 16.16 `velocity` to sprite object `object`'s X or Y position, its
/// whole and fraction words, fraction first.
pub fn advance_cinematic_coordinate(
    object: u16,
    vertical: bool,
    velocity: u32,
    cine: &mut Cinematic,
) {
    use crate::cinematic_state::Array;
    let (whole, fraction) = if vertical {
        (Array::BgArr8, Array::SprArr7)
    } else {
        (Array::BgArr7, Array::SprArr6)
    };
    add_cinematic_pair(whole, fraction, object >> 1, velocity, cine);
}

/// Add the 16.16 `value` to the pair word `slot` of `whole` and `fraction`,
/// storing the fraction first.
pub fn add_cinematic_pair(
    whole: crate::cinematic_state::Array,
    fraction: crate::cinematic_state::Array,
    slot: u16,
    value: u32,
    cine: &mut Cinematic,
) {
    let pair = u32::from(*cine.array(fraction, slot)) | (u32::from(*cine.array(whole, slot)) << 16);
    let [low_0, low_1, high_0, high_1] = pair.wrapping_add(value).to_le_bytes();
    *cine.array(fraction, slot) = u16::from_le_bytes([low_0, low_1]);
    *cine.array(whole, slot) = u16::from_le_bytes([high_0, high_1]);
}

pub fn set_fixed_color<H>(channels: [u8; 3], game: &mut H)
where
    H: crate::display::preparation::Access + ?Sized,
{
    for (channel, value) in channels.into_iter().enumerate() {
        crate::display::preparation::Access::store_display_control_byte(
            &mut *game,
            [
                crate::display::preparation::Field::FixedColor0,
                crate::display::preparation::Field::FixedColor1,
                crate::display::preparation::Field::FixedColor2,
            ][channel],
            value,
            crate::display::preparation::Store::Owned,
        );
    }
}

pub fn set_cinematic_backdrop<H: crate::cinematic_host::Host>(lit: bool, game: &mut H) {
    crate::frame_display_byte!(game, Field::ColorSelection => 0);
    crate::frame_display_byte!(game, Field::ColorMath => if lit { 0x81 } else { 0 });
    let color = if lit { [56, 88, 0x80] } else { [32, 64, 0x80] };
    set_fixed_color(color, game);
}

pub fn place_cinematic_sprite(
    object: u16,
    position: (u16, u16),
    palette: u16,
    cine: &mut Cinematic,
) {
    let slot = usize::from(object >> 1);
    *cine.bg_arr7(slot) = position.0;
    *cine.bg_arr8(slot) = position.1;
    *cine.bg_arr9(slot) = palette;
}

pub fn drift_cinematic_sprite(
    object: u16,
    horizontal: u32,
    vertical: Option<u32>,
    wrap_horizontal: bool,
    cine: &mut Cinematic,
) {
    if let Some(velocity) = vertical {
        advance_cinematic_coordinate(object, true, velocity, cine);
    }
    advance_cinematic_coordinate(object, false, horizontal, cine);
    if wrap_horizontal {
        *cine.bg_arr7(object >> 1) &= 0x1ff;
    }
}

/// Copy the palette buffer's colours into the fade planes' plane 1
/// ($7E:2200..$7E:23FF), colour by colour, each stored in native byte order.
pub fn capture_cinematic_palette<H: crate::cinematic_host::Host>(game: &mut H) {
    use crate::cinematic_state::Array;
    use crate::projection::Field as View;
    for (color, plane) in (0..256).zip(256..) {
        let value = game.view_word(View::PaletteColor(color));
        let plane = View::CinematicElement(Array::TilemapStuff, plane);
        game.set_view_word(plane, crate::word_transfers::stored_native(value));
    }
}

pub fn fade_cinematic_fixed_color<H: crate::cinematic_host::Host>(game: &mut H) {
    for (field, black) in [
        (Field::FixedColor0, 0x20u8),
        (Field::FixedColor1, 0x40),
        (Field::FixedColor2, 0x80),
    ] {
        // Keep decrement and clamp as separate stores in channel order.
        crate::frame_display_byte!(game, field => game.display_control_byte(field).wrapping_sub(1));
        if game.display_control_byte(field).wrapping_sub(black) & 0x80 != 0 {
            crate::frame_display_byte!(game, field => black);
        }
    }
}

pub fn enable_cinematic_background(cine: &mut Cinematic) {
    *cine.enable_objects() |= 0x8000;
}

/// Clear every cinematic sprite object's instruction pointer and drawing.
pub fn clear_cinematic_sprites(cine: &mut Cinematic) {
    for slot in 0..16 {
        *cine.spr_instruction_pointer(slot) = 0;
        *cine.spr_what_to_draw(slot) = 0;
    }
}

pub fn clear_cinematic_background<H: crate::cinematic_host::Host>(
    fill: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    crate::cinematic::transfer_cinematic_words(
        1024,
        -2,
        &[(
            CinematicDestination::Screen(Point::new(Role::Pause, 0x7fe)),
            WordEndpoint::Value(fill),
        )],
        game,
    );
    // The original's four-word clear runs one word past CINEMATICBG_ARR1.
    for slot in 0..4 {
        *cine.bg_arr1(slot) = 0;
        *cine.bg_instruction_pointer(slot) = 0;
    }
    *cine.intro_text_click_flag() = 0;
}

pub fn scaled_cinematic_value(value: u16, scale: u16) -> u16 {
    ((i32::from(value as i16) * i32::from(scale as i16)) >> 8) as u16
}

/// Project a point through the live cinematic matrix and centre, preserving
/// every original query and each word's low-before-high reads.
pub fn project_cinematic_point<H: crate::display::preparation::Access + ?Sized>(
    point: crate::geometry::Point,
    game: &H,
) -> crate::geometry::Point {
    use crate::display::preparation::Place;
    project_cinematic_point_reads(point, |address| {
        let (low, high) = match address {
            0x78 => (Field::AffineWords0Low, Field::AffineWords0High),
            0x7a => (Field::AffineWords1Low, Field::AffineWords1High),
            0x7c => (Field::AffineWords2Low, Field::AffineWords2High),
            0x80 => (Field::AffineWords4Low, Field::AffineWords4High),
            0x82 => (Field::AffineWords5Low, Field::AffineWords5High),
            _ => unreachable!(),
        };
        Place::Controls(low, high).read(game)
    })
}

/// A destination in the existing presentation owners. Displacements stay full
/// width; only the source algorithms decide when their cursors wrap.
#[derive(Clone, Copy)]
pub enum CinematicDestination {
    Screen(crate::presentation_workspace::Point),
    Palette { offset: usize, native: bool },
}

/// The bank $7F buffers the intro and the credits decompress their tilemaps
/// into, by the owners of the level data they overlay.
const LEVEL_SIZE: usize = 0x10000;
const LEVEL_WORDS: usize = 0x10002;
const LEVEL_BEHAVIOR: usize = 0x16402;
const BACKGROUND_BLOCKS: usize = 0x19602;
const BACKGROUND_END: usize = 0x1fa02;

/// The byte at `at` of the decompressed tilemaps in bank $7F: the level data
/// size word ($7F:0000), the room terrain's level words ($7F:0002..$7F:6401)
/// and behaviour bytes ($7F:6402..$7F:9601), then the custom background
/// blocks ($7F:9602..$7F:FA01). The credits rows come from $7F:0000..$7F:1FFF,
/// the intro's Japanese text tiles from $7F:8290 and its background from
/// $7F:9000..$7F:97FF, across the behaviour bytes and the background blocks.
pub(crate) fn decompressed_byte<H: crate::cinematic_host::Scene + ?Sized>(
    game: &H,
    at: usize,
) -> u8 {
    let lane = |word: u16, offset: usize| word.to_le_bytes()[offset % 2];
    match at {
        LEVEL_SIZE..LEVEL_WORDS => lane(game.level_size_word(), at - LEVEL_SIZE),
        LEVEL_WORDS..LEVEL_BEHAVIOR => {
            let offset = at - LEVEL_WORDS;
            lane(game.cell_level(offset / 2), offset)
        }
        LEVEL_BEHAVIOR..BACKGROUND_BLOCKS => game.cell_behavior(at - LEVEL_BEHAVIOR),
        BACKGROUND_BLOCKS..BACKGROUND_END => {
            let offset = at - BACKGROUND_BLOCKS;
            lane(game.background_block(offset / 2), offset)
        }
        // LEGACY(original-overflow): a source outside these buffers (a credits
        // row offset from the ROM past the background blocks) reads whatever
        // RAM lies there.
        _ => crate::legacy_overflow::ram_byte(game, at),
    }
}

/// Copy captured lanes in their original traversal and publication order.
/// Native room-payload sources remain a disclosed compatibility boundary.
pub fn transfer_cinematic_words<H: crate::cinematic_host::Scene + ?Sized>(
    count: usize,
    step: isize,
    lanes: &[(
        CinematicDestination,
        crate::word_transfers::WordEndpoint<'_>,
    )],
    game: &mut H,
) {
    use crate::word_transfers::{WordEndpoint, address};
    for index in 0..count {
        let displacement = isize::try_from(index).unwrap().checked_mul(step).unwrap();
        for (destination, source) in lanes {
            let value = match source {
                WordEndpoint::Ram(base, native) => {
                    let at = address(*base, displacement);
                    let bytes = [decompressed_byte(game, at), decompressed_byte(game, at + 1)];
                    if *native {
                        u16::from_ne_bytes(bytes)
                    } else {
                        u16::from_le_bytes(bytes)
                    }
                }
                source => source.read(displacement),
            };
            match *destination {
                CinematicDestination::Screen(point) => {
                    let offset = address(isize::try_from(point.offset).unwrap(), displacement);
                    game.screen_write_native_word(
                        crate::presentation_workspace::Point::new(point.role, offset),
                        value,
                    );
                }
                CinematicDestination::Palette { offset, native } => {
                    let offset = address(isize::try_from(offset).unwrap(), displacement);
                    if native {
                        game.palette_write_native_word(false, offset, value);
                    } else {
                        game.palette_write_word(false, offset, value);
                    }
                }
            }
        }
    }
}

/// Capture the current ROM allocation at the transfer endpoint's original phase.
pub fn cinematic_rom_words<H: crate::word_transfers::LoadedRom + ?Sized>(
    address: u32,
    index: isize,
    game: &H,
) -> crate::word_transfers::WordEndpoint<'static> {
    let image = game.loaded_rom_image();
    let base = crate::word_transfers::lorom_offset(address);
    crate::word_transfers::WordEndpoint::Rom(
        image,
        base.checked_add(index.checked_mul(2).unwrap()).unwrap(),
    )
}

/// Project `point` with the matrix and centre words `read` returns by RAM address.
pub fn project_cinematic_point_reads(
    point: crate::geometry::Point,
    mut read: impl FnMut(usize) -> u16,
) -> crate::geometry::Point {
    let x = point.x.wrapping_sub(read(0x80));
    let y = read(0x82).wrapping_sub(point.y);
    let projected_x =
        scaled_cinematic_value(x, read(0x78)).wrapping_add(scaled_cinematic_value(read(0x7a), y));
    let projected_y =
        scaled_cinematic_value(read(0x7c), x).wrapping_add(scaled_cinematic_value(read(0x78), y));
    crate::geometry::Point {
        x: projected_x.wrapping_add(read(0x80)),
        y: read(0x82).wrapping_sub(projected_y),
    }
}

/// Clear the cinematic entrance's exit through the supplied world identity map.
pub fn clear_cinematic_exit<H: crate::session_access::SessionAccess + ?Sized>(game: &mut H) {
    game.session_mut().set_exit_key(0);
    game.refresh_inputs();
}

/// Set the intro drawing dispatch on its existing player transition owner.
pub fn set_cinematic_draw_mode<H: crate::session_access::SessionAccess + ?Sized>(
    mode: crate::player_transitions::state::DrawMode,
    game: &mut H,
) {
    game.session_mut().player.transitions.draw_handler = mode;
    game.refresh_inputs();
}

/// Set scene two's player frame dispatch in alpha-before-beta order.
pub fn set_cinematic_frame_modes<H: crate::session_access::SessionAccess + ?Sized>(game: &mut H) {
    use crate::player_transitions::state::{FrameAlphaMode, FrameBetaMode};
    let transitions = &mut game.session_mut().player.transitions;
    transitions.frame_alpha = FrameAlphaMode::from_raw(0xe6c9);
    transitions.frame_beta = FrameBetaMode::from_raw(0xe833);
    game.refresh_inputs();
}
