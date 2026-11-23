//! Independent recorded weapons starts, upstream facts and semantic observations.
//! Ordered door membership is the bounded watch recorded in enemy-interactions.decode.

use super::{compare::Observation, data_failure, result::Outcome};
use data::assets::projectile as asset;
use enum_map::EnumMap;
use game::production::weapons::ActorSlotIdentity;
use game::production::weapons::{ActorOccurrence, DoorOpportunity, QuietBeamWorld, WeaponsOutputs};
use game::weapons_projectiles_bridge::{self as bridge, PowerContent};
use weapons_projectiles as beam;

const HEADINGS: [beam::Heading; 10] = [
    beam::Heading::UpFacingRight,
    beam::Heading::UpRight,
    beam::Heading::Right,
    beam::Heading::DownRight,
    beam::Heading::DownFacingRight,
    beam::Heading::DownFacingLeft,
    beam::Heading::DownLeft,
    beam::Heading::Left,
    beam::Heading::UpLeft,
    beam::Heading::UpFacingLeft,
];

fn word(wram: &[u8], at: usize) -> Result<u16, Outcome> {
    let bytes = wram
        .get(at..at.saturating_add(2))
        .ok_or_else(|| data_failure(None, "weapons record word"))?;
    Ok(u16::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| data_failure(None, "weapons record word"))?,
    ))
}

fn unsupported(dependency: &str) -> Outcome {
    Outcome::Unsupported {
        cut: None,
        dependency: dependency.to_owned(),
    }
}

pub(super) fn heading(code: u16) -> Result<beam::Heading, Outcome> {
    HEADINGS
        .get(usize::from(code))
        .copied()
        .ok_or_else(|| data_failure(None, "weapons heading observation"))
}

pub struct Content {
    pub weapons: PowerContent,
    places: import::projectile::Places,
}

impl Content {
    pub(super) fn flight_frame(
        &self,
        heading: beam::Heading,
        pointer: u16,
    ) -> Result<(beam::ClipId, beam::ClipFrame), Outcome> {
        let id = asset::ClipId::Flight(bridge::asset_heading(heading));
        let places = self
            .places
            .clips
            .iter()
            .find(|places| places.id == id)
            .ok_or_else(|| data_failure(None, "weapons clip observation"))?;
        let cursor = places
            .frames
            .iter()
            .position(|at| *at == pointer)
            .or_else(|| (places.end == pointer).then_some(places.frames.len()))
            .and_then(|index| beam::ClipFrame::new(index, places.frames.len()))
            .ok_or_else(|| data_failure(None, "weapons cursor observation"))?;
        Ok((bridge::clip_id(id), cursor))
    }

    pub(super) fn picture(&self, pointer: u16) -> Result<beam::PictureId, Outcome> {
        self.places
            .pictures
            .iter()
            .position(|at| *at == pointer)
            .and_then(|at| u16::try_from(at).ok())
            .map(bridge::picture)
            .ok_or_else(|| data_failure(None, "weapons picture observation"))
    }

    pub fn load(path: &std::path::Path) -> Result<Self, Outcome> {
        let image = std::fs::read(path).map_err(|_| data_failure(None, "player ROM file"))?;
        let rom = data::rom::Rom::new(image).map_err(|_| data_failure(None, "player ROM pin"))?;
        let imported = import::projectile::import(rom.lorom())
            .map_err(|_| data_failure(None, "projectile import"))?;
        let weapons = PowerContent::new(imported.content)
            .map_err(|_| data_failure(None, "projectile content relation"))?;
        Ok(Self {
            weapons,
            places: imported.places,
        })
    }
}

/// The admitted witness starts before firing; no retained appearance is invented.
pub fn seed(wram: &[u8], content: &Content) -> Result<beam::State, Outcome> {
    let state = decode(wram, content)?;
    if state.slots.values().any(|shot| shot.body().is_some()) {
        return Err(unsupported(
            "weapons occupied start needs independent retained appearance",
        ));
    }
    content
        .weapons
        .validate_state(&state)
        .map_err(|_| data_failure(None, "weapons initial state relation"))?;
    Ok(state)
}

pub struct Input {
    pub update: beam::UpdateInput,
    pub world: QuietBeamWorld,
    pub opportunities: Vec<DoorOpportunity>,
}

/// No selector or pose eligibility policy: this tape covers the watched ordinary caller only.
pub fn input(wram: &[u8], content: &Content, row: &str, cut: u64) -> Result<Input, Outcome> {
    if row != "chain-any-0003-DFD7" || !(157..=160).contains(&cut) {
        return Err(unsupported("independent weapons/contact opportunity tape"));
    }
    ordinary_input(wram, content)
}

// The independently watched extended insertion supplies its own cut qualification.
pub(super) fn ordinary_input(wram: &[u8], content: &Content) -> Result<Input, Outcome> {
    // These checks admit the observed ordinary producer; they do not choose a producer.
    if word(wram, 0x0998)? != 8
        || word(wram, 0x09a6)? != 0
        || word(wram, 0x0a42)? != 0xe695
        || word(wram, 0x0d32)? != 0xc4f0
        || word(wram, 0x0a76)? != 0
        || word(wram, 0x09d2)? != 0
    {
        return Err(unsupported("watched ordinary weapons caller inputs"));
    }
    let pose = data::pose::PoseId(word(wram, 0x0a1c)?);
    let previous_pose = data::pose::PoseId(word(wram, 0x0a24)?);
    let binding = word(wram, 0x09b2)?;
    let movement = word(wram, 0x0a1f)?.to_le_bytes()[0];
    if !matches!(movement, 0 | 1 | 2 | 5 | 6 | 16 | 21) {
        return Err(unsupported("independently admitted cannon eligibility"));
    }
    let pending = word(wram, 0x0b5e)?;
    let distance = |at| -> Result<beam::MovementDistance, Outcome> {
        Ok(beam::MovementDistance {
            whole: word(wram, at)?,
            fraction: beam::PixelFraction(word(wram, at.saturating_add(2))?),
        })
    };
    let update = beam::UpdateInput {
        mode: beam::UpdateMode::CannonAndShots,
        time_stopped: word(wram, 0x0a78)? != 0,
        cannon: beam::CannonInput {
            shoot: word(wram, 0x008b)? & binding != 0,
            new_press: word(wram, 0x008f)? & binding != 0,
            previous_new_press: word(wram, 0x0e00)? & binding != 0,
            muzzle: content
                .weapons
                .muzzle(pose)
                .map_err(|_| data_failure(None, "current muzzle content"))?,
            previous_muzzle: content
                .weapons
                .muzzle(previous_pose)
                .map_err(|_| data_failure(None, "previous muzzle content"))?
                .aim,
            moving_muzzle: movement == 1,
            pending_aim: if pending == 0 {
                None
            } else {
                Some(heading(pending & 255)?)
            },
            centre: beam::SamusCentre(beam::Axes {
                x: word(wram, 0x0af6)?,
                y: word(wram, 0x0afa)?,
            }),
        },
        inheritance: beam::Inheritance {
            left: distance(0x0daa)?,
            right: distance(0x0dae)?,
            up: distance(0x0db2)?,
            down: distance(0x0db6)?,
            camera_vertical_fraction: beam::PixelFraction(word(wram, 0x0da8)?),
        },
        camera: beam::ScreenOrigin(beam::Axes {
            x: word(wram, 0x0911)?,
            y: word(wram, 0x0915)?,
        }),
        room: beam::RoomScreens(beam::Axes {
            x: word(wram, 0x07a9)?,
            y: word(wram, 0x07ab)?,
        }),
    };
    let world = terrain(wram)?;
    let opportunities = [0_usize, 1]
        .into_iter()
        .enumerate()
        .map(|(occurrence, slot)| opportunity(wram, occurrence, slot))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Input {
        update,
        world,
        opportunities,
    })
}

fn terrain(wram: &[u8]) -> Result<QuietBeamWorld, Outcome> {
    let width = word(wram, 0x07a5)?;
    let height = word(wram, 0x07a7)?;
    let count = usize::from(width)
        .checked_mul(usize::from(height))
        .ok_or_else(|| data_failure(None, "weapons terrain size"))?;
    let words = crate::source::LEVEL_DATA
        .read(wram)
        .map_err(|_| data_failure(None, "weapons terrain words"))?;
    let bts = crate::source::BTS
        .read(wram)
        .map_err(|_| data_failure(None, "weapons terrain BTS"))?;
    let words = words
        .get(
            ..count
                .checked_mul(2)
                .ok_or_else(|| data_failure(None, "weapons terrain size"))?,
        )
        .ok_or_else(|| data_failure(None, "weapons terrain words"))?;
    let bts = bts
        .get(..count)
        .ok_or_else(|| data_failure(None, "weapons terrain BTS"))?;
    let cells = words
        .as_chunks::<2>()
        .0
        .iter()
        .zip(bts)
        .map(|(bytes, bts)| data::level::Block::new(u16::from_le_bytes(*bytes), *bts).kind)
        .collect::<Vec<_>>();
    QuietBeamWorld::new(width, height, &cells)
        .map_err(|_| unsupported("weapons terrain production capability"))
}

fn opportunity(wram: &[u8], occurrence: usize, slot: usize) -> Result<DoorOpportunity, Outcome> {
    let offset = slot.saturating_mul(64);
    let w = |at: usize| word(wram, at.saturating_add(offset));
    if w(0x0f78)? != 0xe23f || !matches!(w(0x0fb4)?, 0 | 1 | 4) || w(0x0f88)? & 4 != 0 {
        return Err(unsupported("watched single-box Ceres door actor"));
    }
    let door = if w(0x0f86)? & 0x0400 != 0 {
        enemy_interactions::DoorContact::Passable
    } else if matches!(w(0x0f8e)?, 0 | 0x804d) {
        enemy_interactions::DoorContact::WithoutFigure
    } else {
        enemy_interactions::DoorContact::Box(enemy_interactions::ContactBox {
            centre: enemy_interactions::Centre {
                x: w(0x0f7a)?,
                y: w(0x0f7e)?,
            },
            radii: enemy_interactions::Radii {
                x: w(0x0f82)?,
                y: w(0x0f84)?,
            },
        })
    };
    Ok(DoorOpportunity {
        occurrence: ActorOccurrence(occurrence),
        actor: ActorSlotIdentity(u8::try_from(slot).map_err(|_| data_failure(None, "actor slot"))?),
        door,
        conditions: enemy_interactions::ContactConditions {
            actor_updates_enabled: word(wram, 0x0e4c)? != 0,
            interactions_enabled: word(wram, 0x0e12)? == 0,
            samus_time_stopped: word(wram, 0x0a78)? != 0,
            enemies_stopped_in_time: word(wram, 0x185e)? != 0,
            processing_membership: true,
            protection_remaining: enemy_interactions::ProtectionFrames(w(0x0fa0)?),
        },
    })
}

/// Decode only observed fields. Retained trail appearance has no independent raw field here.
fn decode(wram: &[u8], content: &Content) -> Result<beam::State, Outcome> {
    let mut state = beam::State::reset(EnumMap::from_fn(|_| beam::Axes {
        x: beam::PixelFraction(0),
        y: beam::PixelFraction(0),
    }));
    state.cooldown = beam::FrameWord(word(wram, 0x0ccc)?);
    state.count = beam::ShotCount(word(wram, 0x0cce)?);
    for (index, shot) in state.slots.values_mut().enumerate() {
        *shot = observed_slot(wram, index.saturating_mul(2), content)?;
    }
    if state.count.0 == 0
        && state
            .slots
            .values()
            .any(|shot| matches!(shot, beam::ShotState::Flying { .. }))
    {
        return Err(unsupported("zero-count flying observation"));
    }
    Ok(state)
}

fn observed_slot(
    wram: &[u8],
    offset: usize,
    content: &Content,
) -> Result<beam::ShotState, Outcome> {
    let w = |at: usize| word(wram, at.saturating_add(offset));
    let trail = beam::FrameWord(w(0x0c90)?);
    let fraction = beam::Axes {
        x: beam::PixelFraction(w(0x0b8c)?),
        y: beam::PixelFraction(w(0x0ba0)?),
    };
    let pointer = w(0x0c40)?;
    if pointer == 0 {
        return Ok(beam::ShotState::Free {
            trail,
            retained_fraction: fraction,
        });
    }
    let direction = w(0x0c04)?;
    if direction & !0x001f != 0 {
        return Err(data_failure(None, "weapons direction flags"));
    }
    let heading = heading(direction & 15)?;
    let id = match w(0x0c18)? {
        0x8000 => asset::ClipId::Flight(bridge::asset_heading(heading)),
        0x8700 => asset::ClipId::Impact,
        _ => return Err(unsupported("weapons phase observation")),
    };
    let places = content
        .places
        .clips
        .iter()
        .find(|places| places.id == id)
        .ok_or_else(|| data_failure(None, "weapons clip observation"))?;
    let cursor = places
        .frames
        .iter()
        .position(|at| *at == pointer)
        .or_else(|| (pointer == places.end).then_some(places.frames.len()))
        .ok_or_else(|| data_failure(None, "weapons cursor observation"))?;
    let body = beam::ShotBody {
        position: beam::Axes {
            x: beam::FixedPosition::from_parts(w(0x0b64)?, fraction.x),
            y: beam::FixedPosition::from_parts(w(0x0b78)?, fraction.y),
        },
        heading,
        speed: beam::Axes {
            x: beam::BeamSpeed(w(0x0bdc)?.cast_signed()),
            y: beam::BeamSpeed(w(0x0bf0)?.cast_signed()),
        },
        radius: beam::Axes {
            x: beam::Radius(
                u8::try_from(w(0x0bb4)?).map_err(|_| data_failure(None, "weapons radius"))?,
            ),
            y: beam::Radius(
                u8::try_from(w(0x0bc8)?).map_err(|_| data_failure(None, "weapons radius"))?,
            ),
        },
        damage: beam::Damage::new(w(0x0c2c)?)
            .ok_or_else(|| data_failure(None, "weapons damage"))?,
        trail,
        clip: bridge::clip_id(id),
        next_frame: beam::ClipFrame::new(cursor, places.frames.len())
            .ok_or_else(|| data_failure(None, "weapons cursor relation"))?,
        remaining_hold: beam::FrameWord(w(0x0c54)?),
        picture: Some(content.picture(w(0x0cb8)?)?),
        trail_appearance: None,
    };
    Ok(match id {
        asset::ClipId::Flight(_) => beam::ShotState::Flying {
            body,
            consumed: direction & 16 != 0,
        },
        asset::ClipId::Impact => {
            if direction & 16 != 0 {
                return Err(unsupported(
                    "occupied impact has unrepresented consumed marker",
                ));
            }
            beam::ShotState::Impact { body }
        }
    })
}

pub fn expected(wram: &[u8], content: &Content) -> Result<Vec<Observation>, Outcome> {
    Ok(project(&decode(wram, content)?))
}

pub fn actual(state: &beam::State, _outputs: &WeaponsOutputs) -> Vec<Observation> {
    project(state)
}
pub fn initial(state: &beam::State) -> Vec<Observation> {
    project(state)
}

fn project(state: &beam::State) -> Vec<Observation> {
    let mut bytes = Vec::new();
    bytes.extend(state.cooldown.0.to_le_bytes());
    bytes.extend(state.count.0.to_le_bytes());
    let mut markers = Vec::new();
    for shot in state.slots.values() {
        let (phase, marker) = match shot {
            beam::ShotState::Free { .. } => (0_u8, false),
            beam::ShotState::Flying { consumed, .. } => (1, *consumed),
            beam::ShotState::Impact { .. } => (2, false),
        };
        bytes.push(phase);
        markers.push(u8::from(marker));
        match shot {
            beam::ShotState::Free {
                trail,
                retained_fraction,
            } => {
                bytes.extend(trail.0.to_le_bytes());
                bytes.extend(retained_fraction.x.0.to_le_bytes());
                bytes.extend(retained_fraction.y.0.to_le_bytes());
            }
            beam::ShotState::Flying { body, .. } | beam::ShotState::Impact { body } => {
                bytes.extend(body.position.x.0.to_le_bytes());
                bytes.extend(body.position.y.0.to_le_bytes());
                bytes.extend(body.speed.x.0.to_le_bytes());
                bytes.extend(body.speed.y.0.to_le_bytes());
                bytes.extend([body.radius.x.0, body.radius.y.0]);
                bytes.extend(body.damage.value().to_le_bytes());
                bytes.extend(body.trail.0.to_le_bytes());
                bytes.extend(body.remaining_hold.0.to_le_bytes());
                bytes.extend(body.next_frame.index().to_string().as_bytes());
                bytes.push(0);
                bytes.extend(format!("{:?}", body.heading).as_bytes());
                bytes.push(0);
                bytes.extend(body.clip.0.as_bytes());
                bytes.push(0);
                if let Some(picture) = &body.picture {
                    bytes.extend(picture.0.as_bytes());
                }
                bytes.push(0);
            }
        }
    }
    vec![
        Observation {
            field: "weapons.state".to_owned(),
            bytes,
        },
        Observation {
            field: "contact.consumed-markers".to_owned(),
            bytes: markers,
        },
    ]
}
