//! Read discovery's finite animation, signed particle paths, sprite maps and sound offers.

use core::num::NonZeroU16;
use data::{
    assets::discovery::{Clip, Coord, Flight, Frame, Offset, Pair, RoomPoint, Scene},
    rom::{AddressError, Lorom, RomAddress},
    spritemap::Spritemap,
};

/// Incomplete or invalid discovery content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A required table or picture lies outside the image.
    Address(AddressError),
    /// A named scene relation or value is invalid.
    Invalid(&'static str),
}

impl From<AddressError> for Error {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

struct Reader<'a> {
    rom: Lorom<'a>,
    drawings: Vec<(u16, Spritemap)>,
}

impl Reader<'_> {
    fn word(&self, at: u16) -> Result<u16, Error> {
        let bytes = self.rom.read(RomAddress::new(0x8b, at), 2)?;
        Ok(u16::from_le_bytes(
            bytes.try_into().map_err(|_| Error::Invalid("word"))?,
        ))
    }

    fn picture(&mut self, at: u16) -> Result<u16, Error> {
        let address = self.word(at)?;
        let index = if let Some(index) = self.drawings.iter().position(|(old, _)| *old == address) {
            index
        } else {
            let map = Spritemap::load(self.rom, RomAddress::new(0x8c, address))?;
            if !(1..=128).contains(&map.entries.len()) {
                return Err(Error::Invalid("sprite map"));
            }
            let index = self.drawings.len();
            self.drawings.push((address, map));
            index
        };
        u16::try_from(index).map_err(|_| Error::Invalid("drawing count"))
    }

    fn frame(&mut self, at: u16) -> Result<Frame, Error> {
        let duration = self.word(at)?;
        let hold = NonZeroU16::new(duration)
            .filter(|n| n.get() < 0x8000)
            .ok_or(Error::Invalid("picture hold"))?;
        Ok(Frame {
            picture: self.picture(at.wrapping_add(2))?,
            hold,
        })
    }

    fn clip(&mut self, at: u16, count: u16) -> Result<Clip, Error> {
        let first = self.frame(at)?;
        let rest = (1..count)
            .map(|i| self.frame(at.wrapping_add(i.wrapping_mul(4))))
            .collect::<Result<_, _>>()?;
        let clip = Clip { first, rest };
        duration(&clip)?;
        Ok(clip)
    }

    // The data between the closed and spent loops is finite. Expand only the
    // authored rocking group, retaining its actual repeat count and the scatter cue.
    fn hatch(&mut self) -> Result<(Clip, u16), Error> {
        let first = self.frame(0xcb3b)?;
        if self.word(0xcb3f)? != 0x94d6
            || self.word(0xcb53)? != 0x94c3
            || self.word(0xcb55)? != 0xcb43
            || self.word(0xcb73)? != 0xa918
            || self.word(0xcb91)? != 0xb33e
        {
            return Err(Error::Invalid("hatch markers"));
        }
        let repeats = self.word(0xcb41)?;
        if !(1..=128).contains(&repeats) {
            return Err(Error::Invalid("rocking repeats"));
        }
        let mut rest = Vec::new();
        let rocking = (0xcb43..0xcb53)
            .step_by(4)
            .map(|at| self.frame(at))
            .collect::<Result<Vec<_>, _>>()?;
        for _ in 0..repeats {
            rest.extend_from_slice(&rocking);
        }
        for at in (0xcb57..0xcb73).step_by(4) {
            rest.push(self.frame(at)?);
        }
        let mut hatch = Clip { first, rest };
        let release = duration(&hatch)?;
        for at in (0xcb75..0xcb91).step_by(4) {
            hatch.rest.push(self.frame(at)?);
        }
        if release >= duration(&hatch)? {
            return Err(Error::Invalid("release outside hatch"));
        }
        Ok((hatch, release))
    }

    fn offset(&self, at: u16) -> Result<Offset, Error> {
        let whole = self.word(at)?;
        let fraction = self.word(at.wrapping_add(2))?;
        Ok(Offset(
            (i32::from(whole.cast_signed()) << 16) | i32::from(fraction),
        ))
    }

    fn path(&self, at: u16, steps: u16) -> Result<Vec<Offset>, Error> {
        (0..steps)
            .map(|i| self.offset(at.wrapping_add(i.wrapping_mul(4))))
            .collect()
    }

    fn point(&self, x: u16, y: u16) -> Result<RoomPoint, Error> {
        Ok(Pair {
            x: Coord(i32::from(self.word(x)?.cast_signed()) << 16),
            y: Coord(i32::from(self.word(y)?.cast_signed()) << 16),
        })
    }
}

fn duration(clip: &Clip) -> Result<u16, Error> {
    clip.rest
        .iter()
        .try_fold(clip.first.hold.get(), |sum, frame| {
            sum.checked_add(frame.hold.get())
                .ok_or(Error::Invalid("clip duration"))
        })
}

fn shifted(point: RoomPoint, offset: RoomPoint) -> Result<RoomPoint, Error> {
    let axis = |at: Coord, by: Coord| {
        at.0.checked_add(by.0)
            .map(Coord)
            .ok_or(Error::Invalid("anchor"))
    };
    Ok(Pair {
        x: axis(point.x, offset.x)?,
        y: axis(point.y, offset.y)?,
    })
}

fn check_flight(start: RoomPoint, flight: &Flight) -> Result<(), Error> {
    let mut at = start;
    for step in &flight.path {
        at = shifted(
            at,
            Pair {
                x: Coord(flight.drift.0),
                y: Coord(step.0),
            },
        )?;
    }
    Ok(())
}

/// Import the scene's checked drawing and motion content.
///
/// # Errors
/// Refuses invalid picture data, durations, markers or motion outside the room coordinate range.
pub fn load(rom: Lorom<'_>) -> Result<Scene, Error> {
    let mut read = Reader {
        rom,
        drawings: Vec::new(),
    };
    let home = read.point(0xa8d6, 0xa8dc)?;
    if home != read.point(0xba4c, 0xba52)? {
        return Err(Error::Invalid("egg and baby home"));
    }
    let (hatch, release_at) = read.hatch()?;
    let closed = read.picture(0xcb35)?;
    let spent = read.picture(0xcb99)?;
    let pulse = read.clip(0xcc2b, 4)?;
    let splash = read.clip(0xcd71, 4)?;
    let scatter_origin = read.point(0xa966, 0xa970)?;
    let mut fragments = Vec::new();
    for i in 0..6_u16 {
        let entry = i.wrapping_mul(4);
        let anchor = shifted(
            read.point(
                0xa97c_u16.wrapping_add(entry),
                0xa97e_u16.wrapping_add(entry),
            )?,
            scatter_origin,
        )?;
        let flight = Flight {
            picture: read.picture(0xcd3b_u16.wrapping_add(i.wrapping_mul(8)))?,
            drift: read.offset(0xa9ea_u16.wrapping_add(entry))?,
            path: read.path(0xaa02, 38)?,
        };
        check_flight(anchor, &flight)?;
        fragments.push((anchor, flight));
    }
    let mut drops = Vec::new();
    for (i, (at, steps)) in (0..4_u16).zip([(0xac41, 69), (0xab49, 62), (0xac41, 69), (0xab49, 62)])
    {
        let flight = Flight {
            picture: read.picture(0xcd6b)?,
            drift: read.offset(0xab35_u16.wrapping_add(i.wrapping_mul(4)))?,
            path: read.path(at, steps)?,
        };
        if flight.path.last().is_none_or(|step| step.0 <= 0) {
            return Err(Error::Invalid("droplet descent"));
        }
        check_flight(home, &flight)?;
        drops.push(flight);
    }
    let palette = u8::try_from(read.word(0xa976)? >> 9).map_err(|_| Error::Invalid("palette"))?;
    if palette > 7 {
        return Err(Error::Invalid("palette"));
    }
    let sound = |at| u8::try_from(read.word(at)?).map_err(|_| Error::Invalid("sound"));
    let hatch_sound = sound(0xa950)?;
    let cry = sound(0xbaa0)?;
    Ok(Scene {
        home,
        closed,
        hatch,
        release_at,
        spent,
        pulse,
        fragments,
        drops,
        splash,
        palette,
        hatch_sound,
        cry,
        drawings: read.drawings.into_iter().map(|(_, map)| map).collect(),
    })
}
