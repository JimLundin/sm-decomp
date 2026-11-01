//! Cinematic palettes, fades, tilemap queueing and sprite resumption over RAM.
use crate::cinematic_state::Cinematic;
use crate::ram::RawRam as _;
use crate::ram::word;

/// A bank-`BANK` cinematic record at `cursor`, after the low-half diagnostic.
pub fn cinematic_native_record<const BANK: u8>(
    cursor: u16,
    game: &mut (impl crate::menus::host::Host + ?Sized),
) -> sm_data::cartridge::RomRecord {
    crate::menus::menu_tilemap::native_record((u32::from(BANK) << 16) | u32::from(cursor), game)
}

pub fn advance_cinematic_fade<H: crate::menus::host::Host + ?Sized>(
    fade: crate::visual_state::CinematicFade,
    game: &mut H,
) -> bool {
    game.cinematic_screen_fade(fade)
}

/// The cinematic palette fades' working planes: the BG2 tilemap buffer
/// $7E:2000..$7E:2FFF (`actors.extended.blocks[4..6]`), by word. Word
/// `index` of plane `n` (0..8) is word `256 * n + index`.
#[derive(Clone, Debug, PartialEq)]
pub struct FadePlanes(pub Box<[u16; 2048]>);

impl Default for FadePlanes {
    fn default() -> Self {
        Self(Box::new([0; 2048]))
    }
}

impl crate::session_cohort::Cohort for FadePlanes {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        for (index, word) in self.0.iter_mut().enumerate() {
            visit(0x2000 + 2 * index, word);
        }
    }
    fn session_words(
        session: &mut crate::session::Session,
        mut visit: impl FnMut(usize, &mut u16),
    ) {
        let blocks = &mut session.actors.extended.blocks[4..6];
        for (index, word) in blocks.iter_mut().flatten().enumerate() {
            visit(0x2000 + 2 * index, word);
        }
    }
}

/// Samus's projectiles' X positions and kinds ($7E:0B64, $7E:0C18), which
/// the intro's Mother Brain watches.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct ShotWords {
    pub x: [u16; 10],
    pub kind: [u16; 10],
}

impl crate::session_cohort::Cohort for ShotWords {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        for (index, x) in self.x.iter_mut().enumerate() {
            visit(0xb64 + 2 * index, x);
        }
        for (index, kind) in self.kind.iter_mut().enumerate() {
            visit(0xc18 + 2 * index, kind);
        }
    }
    fn session_words(
        session: &mut crate::session::Session,
        mut visit: impl FnMut(usize, &mut u16),
    ) {
        let shots = &mut session.projectiles.shots;
        for (index, shot) in shots.iter_mut().enumerate() {
            visit(0xb64 + 2 * index, &mut shot.x);
        }
        for (index, shot) in shots.iter_mut().enumerate() {
            visit(0xc18 + 2 * index, &mut shot.kind);
        }
    }
}

/// The palette buffer $7E:C000..$7E:C1FF, the presentation's colours.
#[derive(Clone, Debug, PartialEq)]
pub struct PaletteBuffer(pub [u16; 256]);

impl Default for PaletteBuffer {
    fn default() -> Self {
        Self([0; 256])
    }
}

impl crate::session_cohort::Cohort for PaletteBuffer {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        for (index, word) in self.0.iter_mut().enumerate() {
            visit(0xc000 + 2 * index, word);
        }
    }
    fn session_words(
        session: &mut crate::session::Session,
        mut visit: impl FnMut(usize, &mut u16),
    ) {
        let colors = &mut session.presentation.visual_mut().colors;
        for (index, word) in colors.iter_mut().enumerate() {
            visit(0xc000 + 2 * index, word);
        }
    }
}

/// Store `value` to palette buffer colours `indices`.
pub fn set_cinematic_colors<H: crate::menus::host::Host + ?Sized>(
    indices: &[usize],
    value: u16,
    game: &mut H,
) {
    crate::session_cohort::with(game, |_, palette: &mut PaletteBuffer| {
        for &index in indices {
            palette.0[index] = value;
        }
    });
}

pub fn clear_cinematic_colors<H: crate::menus::host::Host + ?Sized>(
    offset: u16,
    count: u16,
    game: &mut H,
) {
    crate::session_cohort::with(game, |_, planes: &mut FadePlanes| {
        for index in cinematic_color_indices(offset, count) {
            for plane in [256, 512, 768, 1024] {
                planes.0[index + plane] = 0;
            }
        }
    });
}

pub fn compose_cinematic_palette<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::session_cohort::with(
        game,
        |_, (planes, palette): &mut (FadePlanes, PaletteBuffer)| {
            for index in 0..256 {
                // Red is the high byte of plane 2's word.
                let red = u16::from(planes.0[index + 512].to_le_bytes()[1] & 31);
                let green = (planes.0[index + 768] >> 3) & 0x3e0;
                let blue = (planes.0[index + 1024] << 2) & 0x7c00;
                let color = red | green | blue;
                planes.0[index] = color;
                palette.0[index] = color;
            }
        },
    );
}

pub fn fade_cinematic_colors<H: crate::menus::host::Host + ?Sized>(
    offset: u16,
    count: u16,
    increasing: bool,
    game: &mut H,
) {
    crate::session_cohort::with(game, |_, planes: &mut FadePlanes| {
        for index in cinematic_color_indices(offset, count) {
            for plane in [512, 768, 1024] {
                let current = index + plane;
                let step = planes.0[current + 768];
                let word = &mut planes.0[current];
                *word = if increasing {
                    step.wrapping_add(*word) & 0x1fff
                } else {
                    word.wrapping_sub(step)
                };
            }
        }
    });
}

pub fn prepare_cinematic_palette_fade<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::session_cohort::with(game, |_, planes: &mut FadePlanes| {
        for index in 0..256 {
            let color = planes.0[index + 256];
            for channel in 0..3 {
                let component = (color >> (5 * channel)) & 31;
                let current = index + 512 + 256 * channel;
                planes.0[current] = component << 8;
                planes.0[current + 768] = component << 3;
            }
        }
    });
}

pub fn queue_cinematic_tilemap<H: crate::menus::host::Host + ?Sized>(
    tiles: crate::visual_state::CinematicTilemap,
    cine: &mut Cinematic,
    game: &mut H,
) {
    use crate::visual_state::CinematicTilemap::*;
    if !matches!(tiles, Japanese | JapaneseIntro) && game.ram().read_byte(0x55) == 7 {
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
        word!(game.VRAM_WRITE_QUEUE_TAIL),
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

pub fn set_cinematic_blend<H: crate::menus::host::Host + ?Sized>(
    main_screen: u8,
    sub_screen: u8,
    color_window: u8,
    color_math: u8,
    game: &mut H,
) {
    game.ram().write_byte(0x69, main_screen);
    game.ram().write_byte(0x6b, sub_screen);
    game.ram().write_byte(0x6f, color_window);
    game.ram().write_byte(0x72, color_math);
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

// Native split words retain low-before-high reads and writes.
pub fn add_native_pair<H: crate::menus::host::Host + ?Sized>(
    high: usize,
    low: usize,
    velocity: u32,
    game: &mut H,
) {
    let fraction = u16::from_ne_bytes([game.ram().read_byte(low), game.ram().read_byte(low + 1)]);
    let whole = u16::from_ne_bytes([game.ram().read_byte(high), game.ram().read_byte(high + 1)]);
    let position = (u32::from(fraction) | (u32::from(whole) << 16)).wrapping_add(velocity);
    crate::word_transfers!(game, ascending, 1;
        native(low) <- value(position as u16),
        native(high) <- value((position >> 16) as u16),
    );
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
    H: crate::display_refresh::DisplayOwner + ?Sized,
{
    for (channel, value) in channels.into_iter().enumerate() {
        game.write_owned_byte(0x74 + channel, value);
    }
}

pub fn set_cinematic_backdrop<H: crate::menus::host::Host + ?Sized>(lit: bool, game: &mut H) {
    game.ram().write_byte(0x6f, 0);
    game.ram().write_byte(0x72, if lit { 0x81 } else { 0 });
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

pub fn capture_cinematic_palette<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, ascending, 256; native(0x2200) <- le(PALETTE_BUFFER, 0));
}

pub fn fade_cinematic_fixed_color<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for (lane, black) in [0x20u8, 0x40, 0x80].into_iter().enumerate() {
        let at = 0x74 + lane;
        // Keep decrement and clamp as separate stores in channel order.
        game.ram()
            .write_byte(at, game.ram().read_byte(at).wrapping_sub(1));
        if game.ram().read_byte(at).wrapping_sub(black) & 0x80 != 0 {
            game.ram().write_byte(at, black);
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

pub fn clear_cinematic_background<H: crate::menus::host::Host + ?Sized>(
    fill: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    crate::word_transfers!(game, descending, 1024; native(0x37fe) <- value(fill));
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

/// Project `point` through the cinematic Mode 7 matrix and centre in RAM.
pub fn project_cinematic_point<H: crate::ram::WorkRamAccess + ?Sized>(
    point: crate::geometry::Point,
    game: &H,
) -> crate::geometry::Point {
    use crate::ram::RawRam as _;
    project_cinematic_point_reads(point, |at| {
        u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
    })
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
