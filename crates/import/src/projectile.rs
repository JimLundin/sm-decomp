//! The power-beam asset importer. Original pointers and control words stay at this edge.

use data::assets::projectile::{
    Clip, ClipEnd, ClipId, Content, Damage, Frame, Heading, Headings, Hold, Muzzle, MuzzleAim,
    Offset, PowerBeam, Radius, TrailAppearance, TrailAppearanceId, TrailOffsets,
};
use data::assets::samus::{FrameNumber, SpriteId, SpriteSheet, map};
use data::pose::{POSE_COUNT, POSE_DEFINITIONS};
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::Spritemap;

/// Why power-beam content cannot be imported. Errors carry no content bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// A table or picture is outside the image.
    Address(AddressError),
    /// Damage is negative in the original's signed range.
    Damage,
    /// A clip contains an unknown control word.
    Control,
    /// A clip has no picture frames.
    EmptyClip,
    /// A repeat does not name one of the clip's existing frames.
    RepeatTarget,
    /// The common impact clip repeats instead of ending.
    RepeatingImpact,
    /// A picture, appearance, frame or pool identifier is invalid.
    Identifier,
    /// A pose has an invalid heading descriptor.
    Aim,
    /// Dress variants of one named pose disagree about its cannon origin.
    PoseConflict,
}

/// Original places of one clip, for record decoding and reconstruction only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipPlaces {
    /// Its named identity.
    pub id: ClipId,
    /// Original addresses of its frames.
    pub frames: Vec<u16>,
    /// The place of its ending.
    pub end: u16,
}

/// Verification-only mapping; no engine record holds these pointers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Places {
    /// Original picture pointers, indexed by our sprite identifier.
    pub pictures: Vec<u16>,
    /// The original places of each named clip.
    pub clips: Vec<ClipPlaces>,
}

/// Content beside the original-place map used by the verifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imported {
    /// The engine's content records.
    pub content: Content,
    /// Mapping used only to decode and check original records.
    pub places: Places,
}

/// Import all ten flight headings, the common impact and every named pose's muzzle.
///
/// # Errors
///
/// Returns [`ImportError`] for malformed content or inconsistent pose identities.
pub fn import(rom: Lorom<'_>) -> Result<Imported, ImportError> {
    let descriptor = word(rom, 0x93, 0x83c1)?;
    let damage = Damage::try_from(word(rom, 0x93, descriptor)?).map_err(|_| ImportError::Damage)?;
    let mut pools = Pools::default();
    let flight = headings(|heading| {
        let offset = direction_code(heading).wrapping_mul(2).wrapping_add(2);
        let pointer = word(rom, 0x93, advance(descriptor, offset)?)?;
        pools.clip(rom, pointer, ClipId::Flight(heading))
    })?;
    let impact = pools.clip(rom, word(rom, 0x93, 0x867b)?, ClipId::Impact)?;
    if impact.end != ClipEnd::Remove {
        return Err(ImportError::RepeatingImpact);
    }
    let stationary = offsets(rom, 0xc204, 0xc218)?;
    let moving = offsets(rom, 0xc22c, 0xc240)?;
    // Read complete records even though muzzle content uses only aim and unsigned lift.
    let definitions = rom
        .read(POSE_DEFINITIONS, POSE_COUNT.saturating_mul(8))
        .map_err(ImportError::Address)?;
    let mut muzzles: Vec<Muzzle> = Vec::new();
    for entry in &map::MAP {
        let start = usize::from(entry.id.0).saturating_mul(8);
        let aim = definitions
            .get(start.saturating_add(3))
            .copied()
            .ok_or(ImportError::Identifier)?;
        let lift = definitions
            .get(start.saturating_add(4))
            .copied()
            .ok_or(ImportError::Identifier)?;
        let muzzle = Muzzle {
            pose: entry.key,
            aim: muzzle_aim(aim)?,
            lift,
            stationary: stationary.clone(),
            moving: moving.clone(),
        };
        if let Some(existing) = muzzles.iter().find(|existing| existing.pose == entry.key) {
            if existing != &muzzle {
                return Err(ImportError::PoseConflict);
            }
        } else {
            muzzles.push(muzzle);
        }
    }
    let imported = Imported {
        content: Content {
            power: PowerBeam {
                sheet: pools.sheet,
                flight,
                impact,
                damage,
                trails: pools.trails,
            },
            muzzles,
        },
        places: pools.places,
    };
    validate(&imported.content)?;
    Ok(imported)
}

/// Check relations between owned records, including identifiers and repeat targets.
///
/// # Errors
///
/// Returns [`ImportError`] when clips or pose descriptors cannot name valid content.
pub fn validate(content: &Content) -> Result<(), ImportError> {
    let power = &content.power;
    headings(|heading| validate_clip(power, power.flight.get(heading)))?;
    validate_clip(power, &power.impact)?;
    if power.impact.end != ClipEnd::Remove {
        return Err(ImportError::RepeatingImpact);
    }
    for (index, muzzle) in content.muzzles.iter().enumerate() {
        if content
            .muzzles
            .iter()
            .take(index)
            .any(|previous| previous.pose == muzzle.pose)
        {
            return Err(ImportError::PoseConflict);
        }
        if !map::MAP.iter().any(|entry| entry.key == muzzle.pose) {
            return Err(ImportError::Identifier);
        }
    }
    if map::MAP
        .iter()
        .any(|entry| content.muzzle(entry.key).is_none())
    {
        return Err(ImportError::Identifier);
    }
    Ok(())
}

fn validate_clip(power: &PowerBeam, clip: &Clip) -> Result<(), ImportError> {
    if clip.frames.is_empty() {
        return Err(ImportError::EmptyClip);
    }
    if let ClipEnd::RepeatFrom(from) = clip.end
        && usize::from(from.0) >= clip.frames.len()
    {
        return Err(ImportError::RepeatTarget);
    }
    for frame in &clip.frames {
        if power.sheet.sprite(frame.picture).is_none()
            || power
                .trails
                .get(usize::from(frame.trail_appearance.0))
                .is_none()
        {
            return Err(ImportError::Identifier);
        }
    }
    Ok(())
}

fn word(rom: Lorom<'_>, bank: u8, pointer: u16) -> Result<u16, ImportError> {
    let bytes = rom
        .read(RomAddress::new(bank, pointer), 2)
        .map_err(ImportError::Address)?;
    let pair = bytes.first_chunk::<2>().ok_or(ImportError::Identifier)?;
    Ok(u16::from_le_bytes(*pair))
}

fn advance(pointer: u16, bytes: u16) -> Result<u16, ImportError> {
    pointer.checked_add(bytes).ok_or(ImportError::Identifier)
}

fn headings<T>(
    mut make: impl FnMut(Heading) -> Result<T, ImportError>,
) -> Result<Headings<T>, ImportError> {
    Ok(Headings {
        up_facing_right: make(Heading::UpFacingRight)?,
        up_right: make(Heading::UpRight)?,
        right: make(Heading::Right)?,
        down_right: make(Heading::DownRight)?,
        down_facing_right: make(Heading::DownFacingRight)?,
        down_facing_left: make(Heading::DownFacingLeft)?,
        down_left: make(Heading::DownLeft)?,
        left: make(Heading::Left)?,
        up_left: make(Heading::UpLeft)?,
        up_facing_left: make(Heading::UpFacingLeft)?,
    })
}

const fn direction_code(heading: Heading) -> u16 {
    match heading {
        Heading::UpFacingRight => 0,
        Heading::UpRight => 1,
        Heading::Right => 2,
        Heading::DownRight => 3,
        Heading::DownFacingRight => 4,
        Heading::DownFacingLeft => 5,
        Heading::DownLeft => 6,
        Heading::Left => 7,
        Heading::UpLeft => 8,
        Heading::UpFacingLeft => 9,
    }
}

const fn muzzle_aim(byte: u8) -> Result<MuzzleAim, ImportError> {
    let heading = match byte {
        0 => Heading::UpFacingRight,
        1 => Heading::UpRight,
        2 => Heading::Right,
        3 => Heading::DownRight,
        4 => Heading::DownFacingRight,
        5 => Heading::DownFacingLeft,
        6 => Heading::DownLeft,
        7 => Heading::Left,
        8 => Heading::UpLeft,
        9 => Heading::UpFacingLeft,
        0x10 => return Ok(MuzzleAim::PreviousAim),
        0x11..=0xff => return Ok(MuzzleAim::Forbidden),
        0x0a..=0x0f => return Err(ImportError::Aim),
    };
    Ok(MuzzleAim::Aimed(heading))
}

fn offsets(rom: Lorom<'_>, x: u16, y: u16) -> Result<Headings<Offset>, ImportError> {
    headings(|heading| {
        let bytes = direction_code(heading).wrapping_mul(2);
        Ok(Offset {
            x: i16::from_le_bytes(word(rom, 0x90, advance(x, bytes)?)?.to_le_bytes()),
            y: i16::from_le_bytes(word(rom, 0x90, advance(y, bytes)?)?.to_le_bytes()),
        })
    })
}

#[derive(Default)]
struct Pools {
    sheet: SpriteSheet,
    places: Places,
    trails: Vec<TrailAppearance>,
}

impl Pools {
    fn trail(
        &mut self,
        rom: Lorom<'_>,
        id: ClipId,
        selector: u16,
    ) -> Result<TrailAppearanceId, ImportError> {
        // The registered power family has one appearance per heading. A different selector
        // is not a registered appearance and must not alias neighbouring content.
        let appearance = match id {
            ClipId::Impact => TrailAppearance::Inactive,
            ClipId::Flight(heading) => {
                if selector != 0 {
                    return Err(ImportError::Identifier);
                }
                let table = word(rom, 0x9b, 0xa4b3)?;
                let pointer = word(
                    rom,
                    0x9b,
                    advance(table, direction_code(heading).wrapping_mul(2))?,
                )?;
                let bytes = rom
                    .read(RomAddress::new(0x9b, pointer), 4)
                    .map_err(ImportError::Address)?;
                let [left_x, left_y, right_x, right_y] =
                    *bytes.first_chunk::<4>().ok_or(ImportError::Identifier)?;
                let offset = |value| i16::from(i8::from_le_bytes([value])).wrapping_sub(4);
                TrailAppearance::PowerBeam(TrailOffsets {
                    left: Offset {
                        x: offset(left_x),
                        y: offset(left_y),
                    },
                    right: Offset {
                        x: offset(right_x),
                        y: offset(right_y),
                    },
                })
            }
        };
        if let Some(index) = self
            .trails
            .iter()
            .position(|&existing| existing == appearance)
        {
            return u16::try_from(index)
                .map(TrailAppearanceId)
                .map_err(|_| ImportError::Identifier);
        }
        let id = TrailAppearanceId(
            u16::try_from(self.trails.len()).map_err(|_| ImportError::Identifier)?,
        );
        self.trails.push(appearance);
        Ok(id)
    }

    fn picture(&mut self, rom: Lorom<'_>, pointer: u16) -> Result<SpriteId, ImportError> {
        if let Some(index) = self
            .places
            .pictures
            .iter()
            .position(|&place| place == pointer)
        {
            return u16::try_from(index)
                .map(SpriteId)
                .map_err(|_| ImportError::Identifier);
        }
        let picture =
            Spritemap::load(rom, RomAddress::new(0x93, pointer)).map_err(ImportError::Address)?;
        let id =
            SpriteId(u16::try_from(self.sheet.sprites.len()).map_err(|_| ImportError::Identifier)?);
        self.sheet.sprites.push(picture);
        self.places.pictures.push(pointer);
        Ok(id)
    }

    fn clip(&mut self, rom: Lorom<'_>, start: u16, id: ClipId) -> Result<Clip, ImportError> {
        let mut pointer = start;
        let mut frames = Vec::new();
        let mut starts = Vec::new();
        let end = loop {
            let hold = word(rom, 0x93, pointer)?;
            match hold {
                0x822f => break ClipEnd::Remove,
                0x8239 => {
                    let target = word(rom, 0x93, advance(pointer, 2)?)?;
                    let index = starts
                        .iter()
                        .position(|&place| place == target)
                        .ok_or(ImportError::RepeatTarget)?;
                    break ClipEnd::RepeatFrom(FrameNumber(
                        u16::try_from(index).map_err(|_| ImportError::Identifier)?,
                    ));
                }
                0x8000..=0xffff => return Err(ImportError::Control),
                0..=0x7fff => {}
            }
            let bytes = rom
                .read(RomAddress::new(0x93, pointer), 8)
                .map_err(ImportError::Address)?;
            let [_, _, low, high, x, y, trail_low, trail_high] =
                *bytes.first_chunk::<8>().ok_or(ImportError::Identifier)?;
            let trail_appearance =
                self.trail(rom, id, u16::from_le_bytes([trail_low, trail_high]))?;
            let picture = self.picture(rom, u16::from_le_bytes([low, high]))?;
            starts.push(pointer);
            frames.push(Frame {
                hold: Hold::try_from(hold).map_err(|_| ImportError::Control)?,
                picture,
                radius: Radius { x, y },
                trail_appearance,
            });
            pointer = advance(pointer, 8)?;
        };
        if frames.is_empty() {
            return Err(ImportError::EmptyClip);
        }
        self.places.clips.push(ClipPlaces {
            id,
            frames: starts,
            end: pointer,
        });
        Ok(Clip { frames, end })
    }
}

#[cfg(test)]
mod tests;
