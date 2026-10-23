//! Projectile launch records and the shared animation/trail playback kernels.
#![forbid(unsafe_code)]
use super::{Host, content};
use crate::game::projectile_initialization::Initialization;
use crate::game::projectile_playback::{
    self as playback, Command, Definitions, Frame, Owner, Write,
};
use crate::game::projectile_trail_selection::{self as trail, Source, TrailDefinitions};
use crate::rom::RomRecord;

pub fn table(kind: Initialization, flags: u16) -> (u32, isize) {
    match kind {
        Initialization::SuperMissile => (0x93842b, ((flags >> 8) & 15) as isize),
        Initialization::Bomb => (0x9383f1, ((flags >> 8) & 15) as isize),
        Initialization::SpecialAttack => (0x938413, (flags & 15) as isize),
        Initialization::Echo => (0x938403, isize::from(flags.to_le_bytes()[0]) - 34),
        Initialization::Projectile if flags & 0xf00 != 0 => {
            (0x9383f1, isize::from(flags.to_ne_bytes()[1] & 15))
        }
        Initialization::Projectile => (
            if flags & 0x10 != 0 {
                0x9383d9
            } else {
                0x9383c1
            },
            (flags & 15) as isize,
        ),
    }
}
pub fn initialize(h: &mut impl Host, slot: u16, kind: Initialization) {
    let direction = matches!(kind, Initialization::Projectile | Initialization::Echo)
        .then(|| usize::from(shot!(h, slot, direction) & 15));
    let (address, index) = table(kind, shot!(h, slot, kind));
    let pointer = content::table(h, address, index);
    let launch = h.record(0x93, pointer);
    let damage = launch.word(0);
    if kind != Initialization::Projectile {
        shot!(h, slot, damage = damage);
    }
    let checked = if kind == Initialization::Echo {
        launch.word(0)
    } else {
        damage
    };
    if checked & 0x8000 != 0 {
        h.invalid_damage();
    }
    if kind == Initialization::Projectile {
        shot!(h, slot, damage = launch.word(0));
    }
    let field = direction.map_or(2, |direction| {
        [2, 4, 6, 8, 10, 12, 14, 16, 18, 20][direction]
    });
    let cursor = launch.word(field);
    shot!(h, slot, cursor = cursor);
    if kind == Initialization::Projectile {
        let frame = h.record(0x93, cursor);
        shot!(h, slot, x_radius = u16::from(frame.byte(4)));
        shot!(h, slot, y_radius = u16::from(frame.byte(5)));
    }
    shot!(h, slot, remaining = 1);
}

impl<H: Host> Owner for &mut H {
    type Cursor = u16;
    type Sprite = u16;
    type Slot = u16;
    fn remaining(&self, slot: u16) -> u16 {
        shot!(self, slot, remaining)
    }
    fn cursor(&self, slot: u16) -> u16 {
        shot!(self, slot, cursor)
    }
    fn write(&mut self, slot: u16, value: Write<u16, u16>) {
        match value {
            Write::Timer(v) => shot!(self, slot, remaining = v),
            Write::Cursor(v) => {
                shot!(self, slot, cursor = v);
                self.probe("player_projectiles.playback.frame");
            }
            Write::Sprite(v) => shot!(self, slot, sprite = v),
            Write::XRadius(v) => shot!(self, slot, x_radius = v),
            Write::YRadius(v) => shot!(self, slot, y_radius = v),
        }
    }
}
pub struct View {
    data: RomRecord,
    cursor: u16,
}
impl Frame<u16, u16> for View {
    fn command(&self) -> Command {
        match self.data.word(0) {
            0x822f => Command::Release,
            0x8239 => Command::Jump,
            duration if duration & 0x8000 == 0 => Command::Show { duration },
            _ => Command::Unknown,
        }
    }
    fn sprite(&self) -> u16 {
        self.data.word(2)
    }
    fn x_radius(&self) -> u16 {
        u16::from(self.data.byte(4))
    }
    fn y_radius(&self) -> u16 {
        u16::from(self.data.byte(5))
    }
    fn next(&self) -> u16 {
        self.cursor.wrapping_add(8)
    }
}
struct Playback;
impl<'a, H: Host> Definitions<&'a mut H> for Playback {
    type Frame = View;
    fn open(&mut self, h: &'a mut H, cursor: u16) -> (&'a mut H, View) {
        let data = h.record(0x93, cursor);
        (h, View { data, cursor })
    }
    fn follow_jump(&mut self, h: &'a mut H, cursor: u16) -> (&'a mut H, Option<u16>) {
        let data = h.record(0x93, cursor.wrapping_add(2));
        let target = data.word(0);
        (h, (target != 0).then_some(target))
    }
    fn unknown(&mut self, h: &'a mut H) -> (&'a mut H, Option<u16>) {
        let result = h.unknown();
        (h, result.then_some(1))
    }
}
impl<'a, H: Host> TrailDefinitions<&'a mut H> for Playback {
    type Marker = u16;
    fn sample_trail(&mut self, h: &'a mut H, cursor: u16, source: Source) -> (&'a mut H, u16) {
        let cursor = if source == Source::Upcoming {
            cursor
        } else {
            cursor.wrapping_sub(8)
        };
        let data = h.record(0x93, cursor);
        (h, data.word(6))
    }
}
pub fn advance(h: &mut impl Host) {
    let selected = h.session().selected;
    let (h, outcome) = playback::advance(h, selected, &mut Playback);
    if outcome == playback::Outcome::Release {
        super::motion::clear(h, selected);
    }
}
pub fn trail_frame(h: &mut impl Host, slot: u16) -> u16 {
    trail::sample(h, slot, &mut Playback).1
}
