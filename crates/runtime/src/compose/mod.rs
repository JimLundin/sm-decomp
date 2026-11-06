//! The line compositor (ARCHITECTURE D17, under D1's closed list): a frame's picture from
//! the video memory image and the display frame, one line at a time on the CPU.
//!
//! **Contract.** For a frame whose register changes all fall between lines, the picture
//! equals the oracle's `video` stream picture dot for dot: each dot is the PPU-domain
//! 15-bit BGR after colour math and before the master brightness, and each line carries its
//! master brightness and forced blank (docs/oracle/HARNESS.md, "The picture compared").
//! The rules are the PPU's as `LakeSnes` (MIT, revision `9db90b8`) draws them: per dot, each
//! mode's layer order front to back, the first opaque plane wins; sprites are evaluated on
//! the line before; colour math, clip and the windows follow CGWSEL and CGADSUB.
//!
//! Not modelled yet: writes in the middle of a line, video memory writes while the frame
//! is shown, the sprite priority rotation, and the second dot of hires modes 5 and 6 and
//! pseudo-hires (the main screen's dot is kept).

mod background;
mod setup;
mod sprites;

use data::video::{DisplayFrame, LineWrite, MAX_LINES, NORMAL_LINES, VideoMemory};
use setup::{Bg, Depth, LineSetup, Masks, Mode, Plane, RegisterFile};

/// Dots per line.
pub const WIDTH: usize = 256;

/// A composed frame in the PPU's domain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    /// 224, or 239 on an overscan frame.
    pub lines: usize,
    /// `MAX_LINES` rows of `WIDTH` BGR555 dots; rows past `lines` stay black.
    pub dots: Box<[u16; MAX_LINES * WIDTH]>,
    /// Each line's master brightness, 0-15.
    pub brightness: [u8; MAX_LINES],
    /// Each line's forced blank.
    pub forced_blank: [bool; MAX_LINES],
}

impl Default for Picture {
    fn default() -> Self {
        Self {
            lines: NORMAL_LINES,
            dots: zeroed(),
            brightness: [0; MAX_LINES],
            forced_blank: [false; MAX_LINES],
        }
    }
}

/// A zeroed array built on the heap.
fn zeroed<const N: usize>() -> Box<[u16; N]> {
    let boxed = vec![0; N].into_boxed_slice();
    boxed.try_into().unwrap_or_else(|_| Box::new([0; N]))
}

/// A layer order entry: a plane and the priority it is drawn at.
type Entry = (Plane, u8);

const fn bg(layer: Bg, priority: u8) -> Entry {
    (Plane::Background(layer), priority)
}

const fn obj(priority: u8) -> Entry {
    (Plane::Sprites, priority)
}

use Bg::{Four, One, Three, Two};

const TWO_BACKGROUNDS: &[Entry] = &[
    obj(3),
    bg(One, 1),
    obj(2),
    bg(Two, 1),
    obj(1),
    bg(One, 0),
    obj(0),
    bg(Two, 0),
];

const MODE_0: &[Entry] = &[
    obj(3),
    bg(One, 1),
    bg(Two, 1),
    obj(2),
    bg(One, 0),
    bg(Two, 0),
    obj(1),
    bg(Three, 1),
    bg(Four, 1),
    obj(0),
    bg(Three, 0),
    bg(Four, 0),
];

const MODE_1: &[Entry] = &[
    obj(3),
    bg(One, 1),
    bg(Two, 1),
    obj(2),
    bg(One, 0),
    bg(Two, 0),
    obj(1),
    bg(Three, 1),
    obj(0),
    bg(Three, 0),
];

const MODE_1_BG3_FRONT: &[Entry] = &[
    bg(Three, 1),
    obj(3),
    bg(One, 1),
    bg(Two, 1),
    obj(2),
    bg(One, 0),
    bg(Two, 0),
    obj(1),
    obj(0),
    bg(Three, 0),
];

const MODE_6: &[Entry] = &[obj(3), bg(One, 1), obj(2), obj(1), bg(One, 0), obj(0)];

const MODE_7: &[Entry] = &[obj(3), obj(2), obj(1), bg(One, 0), obj(0)];

const MODE_7_EXTBG: &[Entry] = &[
    obj(3),
    obj(2),
    bg(Two, 1),
    obj(1),
    bg(One, 0),
    obj(0),
    bg(Two, 0),
];

/// Each mode's front-to-back plane order.
const fn order(mode: Mode) -> &'static [Entry] {
    match mode {
        Mode::Zero => MODE_0,
        Mode::One { bg3_front: false } => MODE_1,
        Mode::One { bg3_front: true } => MODE_1_BG3_FRONT,
        Mode::Two | Mode::Three | Mode::Four | Mode::Five => TWO_BACKGROUNDS,
        Mode::Six => MODE_6,
        Mode::Seven { extended: false } => MODE_7,
        Mode::Seven { extended: true } => MODE_7_EXTBG,
    }
}

/// One plane's dots on a line: a colour index (0 is transparent) and its priority.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Dot {
    index: u16,
    priority: u8,
}

/// The line's planes, each drawn once before the dots are composed.
struct Planes {
    backgrounds: [Option<[Dot; WIDTH]>; 4],
    sprites: [Dot; WIDTH],
}

/// The layer slot colour math reads for the backdrop.
const BACKDROP: usize = 5;

/// The plane that won a dot: its layer slot and its colour.
#[derive(Clone, Copy, Debug)]
struct Shown {
    slot: usize,
    colour: [i16; 3],
}

/// The plane that wins a dot of one screen, before its colour is looked up.
#[derive(Clone, Copy, Debug)]
struct Winner {
    plane: Plane,
    index: u16,
}

/// The mask of a plane its screen doesn't window.
const OPEN: [bool; WIDTH] = [false; WIDTH];

const BACKDROP_WINNER: Winner = Winner {
    plane: Plane::Colour,
    index: 0,
};

impl Planes {
    /// One screen's winning plane per dot: the opaque, unmasked plane dot that comes first in
    /// the mode's order. Each plane is read once; its priority gives its place in the order.
    fn screen(&self, setup: &LineSetup, masks: &Masks, screen: setup::Screen) -> [Winner; WIDTH] {
        let order = order(setup.mode);
        let mut winners = [BACKDROP_WINNER; WIDTH];
        let mut places = [Place::BEHIND; WIDTH];
        let layers = self
            .backgrounds
            .iter()
            .zip(Bg::ALL)
            .filter_map(|(dots, layer)| Some((Plane::Background(layer), dots.as_ref()?)))
            .chain([(Plane::Sprites, &self.sprites)]);
        for (plane, dots) in layers {
            if !screen.enables(plane) {
                continue;
            }
            let ranks = Place::of(order, plane);
            let mask = masks
                .get(plane.index())
                .filter(|_| screen.windows(plane))
                .unwrap_or(&OPEN);
            let lanes = winners
                .iter_mut()
                .zip(places.iter_mut())
                .zip(dots)
                .zip(mask);
            for (((winner, best), dot), &hidden) in lanes {
                let place = ranks
                    .get(usize::from(dot.priority))
                    .copied()
                    .unwrap_or(Place::BEHIND);
                if dot.index != 0 && !hidden && place < *best {
                    *best = place;
                    *winner = Winner {
                        plane,
                        index: dot.index,
                    };
                }
            }
        }
        winners
    }
}

/// A plane dot's place in its mode's front-to-back order: lower is nearer the front.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Place(u8);

impl Place {
    /// Behind every entry of the order: what a plane at a priority the mode doesn't draw takes.
    const BEHIND: Self = Self(u8::MAX);

    /// The place of `plane` at each priority 0-3.
    fn of(order: &[Entry], plane: Plane) -> [Self; 4] {
        let mut places = [Self::BEHIND; 4];
        for (place, &(entry, priority)) in (0_u8..).zip(order) {
            if entry == plane
                && let Some(slot) = places.get_mut(usize::from(priority))
            {
                *slot = Self(place);
            }
        }
        places
    }
}

impl Winner {
    /// The winner's layer slot for colour math, and its colour.
    fn shown(self, setup: &LineSetup, palette: &Palette) -> Shown {
        let slot = self.plane.index();
        let direct = matches!(self.plane, Plane::Background(_))
            && setup.math.direct_colour
            && setup.mode.depths().get(slot).copied().flatten() == Some(Depth::Eight);
        let colour = if direct {
            direct_colour(self.index)
        } else {
            palette_colour(palette, self.index)
        };
        // Sprites from palettes 0-3 never take colour math.
        let slot = if self.plane == Plane::Sprites && self.index < 0xc0 {
            6
        } else {
            slot
        };
        Shown { slot, colour }
    }
}

/// CGRAM's 256 colours split into red, green and blue once a frame: CGRAM doesn't change
/// while the frame is shown.
type Palette = [[i16; 3]; 256];

fn split(cgram: &[u16; 256]) -> Palette {
    cgram.map(components)
}

/// A CGRAM colour's red, green and blue.
fn palette_colour(palette: &Palette, index: u16) -> [i16; 3] {
    palette
        .get(usize::from(index & 0xff))
        .copied()
        .unwrap_or_default()
}

fn components(word: u16) -> [i16; 3] {
    [0, 5, 10].map(|shift| ((word >> shift) & 0x1f).cast_signed())
}

/// An 8-bit background's dot as its own colour: BBGGGRRR plus the palette's three bits.
fn direct_colour(index: u16) -> [i16; 3] {
    let red = ((index & 7) << 2) | ((index & 0x100) >> 7);
    let green = ((index & 0x38) >> 1) | ((index & 0x200) >> 8);
    let blue = ((index & 0xc0) >> 3) | ((index & 0x400) >> 8);
    [red, green, blue].map(u16::cast_signed)
}

/// The colour math of one dot: the main screen's winner against the sub screen or the fixed
/// colour, clamped to 5 bits.
fn shade(
    setup: &LineSetup,
    palette: &Palette,
    main: Winner,
    sub: Option<&Winner>,
    window: bool,
) -> u16 {
    let math = &setup.math;
    let main = main.shown(setup, palette);
    let mut colour = if math.clip.contains(window) {
        [0; 3]
    } else {
        main.colour
    };
    let enabled =
        math.enabled.get(main.slot).copied().unwrap_or(false) && !math.prevent.contains(window);
    if enabled {
        let subscreen = math.operand == setup::Operand::Subscreen;
        let sub = sub
            .filter(|_| subscreen)
            .map(|sub| sub.shown(setup, palette))
            .filter(|sub| sub.slot != BACKDROP);
        let operand = sub.map_or_else(|| math.fixed.map(i16::from), |sub| sub.colour);
        let half = math.half && (sub.is_some() || !subscreen);
        for (value, operand) in colour.iter_mut().zip(operand) {
            let sum = if math.subtract {
                value.wrapping_sub(operand)
            } else {
                value.wrapping_add(operand)
            };
            let sum = if half { sum >> 1 } else { sum };
            *value = sum.clamp(0, 31);
        }
    }
    let [red, green, blue] = colour.map(i16::cast_unsigned);
    red | (green << 5) | (blue << 10)
}

/// Compose a frame from the video memory and the display frame.
#[must_use]
pub fn compose(memory: &VideoMemory, frame: &DisplayFrame) -> Picture {
    let mut picture = Picture {
        lines: if frame.overscan {
            MAX_LINES
        } else {
            NORMAL_LINES
        },
        ..Picture::default()
    };
    compose_into(memory, frame, &mut picture);
    picture
}

/// Compose into an existing picture, so a caller can reuse its buffer.
pub fn compose_into(memory: &VideoMemory, frame: &DisplayFrame, picture: &mut Picture) {
    picture.lines = if frame.overscan {
        MAX_LINES
    } else {
        NORMAL_LINES
    };
    let mut file = RegisterFile::new(frame.start);
    let mut setup = LineSetup::from(&file);
    let mut masks = setup.windows.masks();
    // The PPU restarts mosaic blocks at line 1 each frame, and at a MOSAIC write's line.
    let mut mosaic_start: u16 = 1;
    let mut writes = frame.writes.iter().peekable();
    let table = sprites::Table::new(memory);
    let palette = split(&memory.cgram);
    let mut planes = Planes {
        backgrounds: [None; 4],
        sprites: [Dot::default(); WIDTH],
    };
    let rows = picture
        .dots
        .as_chunks_mut::<WIDTH>()
        .0
        .iter_mut()
        .take(picture.lines);
    let flags = picture
        .brightness
        .iter_mut()
        .zip(picture.forced_blank.iter_mut());
    for (line, (row, (brightness, blank))) in (1_u16..).zip(rows.zip(flags)) {
        let mut changed = false;
        while let Some(write) = writes.next_if(|write| write.line.0 <= line) {
            let LineWrite {
                beam_line,
                port,
                value,
                ..
            } = *write;
            file.write(port, value);
            if port == 0x06 {
                mosaic_start = beam_line;
            }
            changed = true;
        }
        if changed {
            setup = LineSetup::from(&file);
            masks = setup.windows.masks();
        }
        *brightness = setup.brightness;
        *blank = setup.forced_blank;
        if setup.forced_blank {
            row.fill(0);
            continue;
        }
        let context = background::LineContext {
            setup: &setup,
            vram: &memory.vram,
            line,
            mosaic_start,
            even_frame: frame.even_frame,
        };
        sprites::evaluate(
            setup.sprites,
            &table,
            &memory.vram,
            frame.even_frame,
            line,
            &mut planes.sprites,
        );
        for (layer, slot) in Bg::ALL.iter().zip(planes.backgrounds.iter_mut()) {
            *slot = background::draw(&context, *layer);
        }
        let main = planes.screen(&setup, &masks, setup.main);
        let subscreen = (setup.math.operand == setup::Operand::Subscreen
            && setup.math.enabled.contains(&true))
        .then(|| planes.screen(&setup, &masks, setup.sub));
        let colour_window = masks.get(Plane::Colour.index()).unwrap_or(&OPEN);
        for (x, ((dot, main), &window)) in row.iter_mut().zip(main).zip(colour_window).enumerate() {
            let sub = subscreen.as_ref().and_then(|line| line.get(x));
            *dot = shade(&setup, &palette, main, sub, window);
        }
    }
}

#[cfg(test)]
mod tests;
