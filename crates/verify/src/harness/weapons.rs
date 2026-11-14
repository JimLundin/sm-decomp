//! W1's uncharged power beam. All inputs and actor boxes come from the preceding cut.
//! Production integration waits for persisted movement inheritance and C1 events; see LEGACY.

use std::collections::BTreeMap;
use std::fmt;

use data::assets::projectile as asset;
use data::assets::samus::map;
use enum_map::EnumMap;
use oracle_record::{Cut, FrameRecord};
use weapons_projectiles as beam;

use super::{Domain, Entry, Field, FieldSet, Interval, Mask, Uncompared};
use crate::lock::{Frame, Sha256};

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

/// A decode or neighboring capability that this first part cannot answer.
#[derive(Clone, Debug)]
pub struct Defect(&'static str);
impl fmt::Display for Defect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

fn word(wram: &[u8], at: usize) -> Result<u16, Defect> {
    let bytes = wram
        .get(at..at.saturating_add(2))
        .ok_or(Defect("short WRAM"))?;
    Ok(u16::from_le_bytes(
        bytes.try_into().map_err(|_| Defect("short word"))?,
    ))
}
fn heading(code: u16) -> Result<beam::Heading, Defect> {
    HEADINGS
        .get(usize::from(code))
        .copied()
        .ok_or(Defect("unknown heading"))
}
const fn asset_heading(value: beam::Heading) -> asset::Heading {
    match value {
        beam::Heading::UpFacingRight => asset::Heading::UpFacingRight,
        beam::Heading::UpRight => asset::Heading::UpRight,
        beam::Heading::Right => asset::Heading::Right,
        beam::Heading::DownRight => asset::Heading::DownRight,
        beam::Heading::DownFacingRight => asset::Heading::DownFacingRight,
        beam::Heading::DownFacingLeft => asset::Heading::DownFacingLeft,
        beam::Heading::DownLeft => asset::Heading::DownLeft,
        beam::Heading::Left => asset::Heading::Left,
        beam::Heading::UpLeft => asset::Heading::UpLeft,
        beam::Heading::UpFacingLeft => asset::Heading::UpFacingLeft,
    }
}
fn clip_id(id: asset::ClipId) -> beam::ClipId {
    beam::ClipId(format!("{id:?}"))
}
fn picture(id: u16) -> beam::PictureId {
    beam::PictureId(id.to_string())
}
fn appearance(id: u16) -> beam::TrailAppearanceId {
    beam::TrailAppearanceId(id.to_string())
}

fn inheritance(wram: &[u8]) -> Result<beam::Inheritance, Defect> {
    let distance = |whole, fraction| {
        Ok(beam::MovementDistance {
            whole: word(wram, whole)?,
            fraction: beam::PixelFraction(word(wram, fraction)?),
        })
    };
    Ok(beam::Inheritance {
        left: distance(0x0daa, 0x0dac)?,
        right: distance(0x0dae, 0x0db0)?,
        up: distance(0x0db2, 0x0db4)?,
        down: distance(0x0db6, 0x0db8)?,
        camera_vertical_fraction: beam::PixelFraction(word(wram, 0x0da8)?),
    })
}

/// Imported production content and verification-only original-place relations.
pub struct Weapons {
    imported: import::projectile::Imported,
    content: beam::PowerBeam,
    /// A deliberate fault can change a parameter without editing the clean crate.
    pub rules: beam::Rules,
}
impl Weapons {
    /// Load through AF3's production importer, including all headings and impact.
    ///
    /// # Errors
    /// A ROM or projectile-content import failure, with no content bytes.
    pub fn load(image: &[u8]) -> Result<Self, Defect> {
        let rom = data::rom::Rom::new(image.to_vec()).map_err(|_| Defect("ROM import"))?;
        let imported = import::projectile::import(rom.lorom()).map_err(|_| Defect("AF3 import"))?;
        let source = &imported.content.power;
        let mut clips = BTreeMap::new();
        for places in &imported.places.clips {
            let clip = source.clip(places.id);
            let frames = clip
                .frames
                .iter()
                .map(|frame| {
                    Ok::<_, Defect>(beam::Frame {
                        hold: beam::FrameHold::new(frame.hold.updates())
                            .ok_or(Defect("AF3 hold conversion"))?,
                        picture: picture(frame.picture.0),
                        radius: beam::Axes {
                            x: beam::Radius(frame.radius.x),
                            y: beam::Radius(frame.radius.y),
                        },
                        trail_appearance: match source
                            .trails
                            .get(usize::from(frame.trail_appearance.0))
                        {
                            Some(asset::TrailAppearance::PowerBeam(_)) => {
                                Some(appearance(frame.trail_appearance.0))
                            }
                            Some(asset::TrailAppearance::Inactive) => None,
                            None => return Err(Defect("missing AF3 trail appearance")),
                        },
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let ending = match clip.end {
                asset::ClipEnd::Remove => beam::ClipEnding::Remove,
                asset::ClipEnd::RepeatFrom(frame) => beam::ClipEnding::RepeatFrom(
                    beam::ClipFrame::new(usize::from(frame.0), clip.frames.len())
                        .ok_or(Defect("AF3 repeat conversion"))?,
                ),
            };
            clips.insert(clip_id(places.id), beam::Clip { frames, ending });
        }
        let content = beam::PowerBeam {
            damage: beam::Damage::new(source.damage.points())
                .ok_or(Defect("AF3 damage conversion"))?,
            flying_clips: EnumMap::from_fn(|h| clip_id(asset::ClipId::Flight(asset_heading(h)))),
            impact_clip: clip_id(asset::ClipId::Impact),
            clips,
        };
        Ok(Self {
            imported,
            content,
            rules: beam::Rules::REFERENCE,
        })
    }
    fn muzzle(&self, pose: u16) -> Result<beam::Muzzle, Defect> {
        let key = map::MAP
            .iter()
            .find(|entry| entry.id.0 == pose)
            .ok_or(Defect("unknown muzzle pose"))?
            .key;
        let muzzle = self
            .imported
            .content
            .muzzle(key)
            .ok_or(Defect("missing muzzle"))?;
        let aim = match muzzle.aim {
            asset::MuzzleAim::Forbidden => beam::MuzzleAim::Forbidden,
            asset::MuzzleAim::PreviousAim => beam::MuzzleAim::PreviousAim,
            asset::MuzzleAim::Aimed(h) => beam::MuzzleAim::Aimed(
                HEADINGS
                    .into_iter()
                    .find(|value| asset_heading(*value) == h)
                    .ok_or(Defect("heading map"))?,
            ),
        };
        let offsets = |map: &asset::Headings<asset::Offset>| {
            EnumMap::from_fn(|h| {
                let offset = map.get(asset_heading(h));
                beam::Offset {
                    x: i64::from(offset.x),
                    y: i64::from(offset.y),
                }
            })
        };
        Ok(beam::Muzzle {
            aim,
            lift: u64::from(muzzle.lift),
            stationary: offsets(&muzzle.stationary),
            moving: offsets(&muzzle.moving),
        })
    }
    fn state(&self, wram: &[u8]) -> Result<beam::State, Defect> {
        let mut state = beam::State::reset(EnumMap::from_fn(|_| beam::Axes {
            x: beam::PixelFraction(0),
            y: beam::PixelFraction(0),
        }));
        state.cooldown = beam::FrameWord(word(wram, 0x0ccc)?);
        state.count = beam::ShotCount(word(wram, 0x0cce)?);
        for ((_, shot), index) in state.slots.iter_mut().zip(0_usize..) {
            let offset = index.saturating_mul(2);
            let trail = beam::FrameWord(word(wram, 0x0c90_usize.saturating_add(offset))?);
            let fraction = beam::Axes {
                x: beam::PixelFraction(word(wram, 0x0b8c_usize.saturating_add(offset))?),
                y: beam::PixelFraction(word(wram, 0x0ba0_usize.saturating_add(offset))?),
            };
            let pointer = word(wram, 0x0c40_usize.saturating_add(offset))?;
            if pointer == 0 {
                *shot = beam::ShotState::Free {
                    trail,
                    retained_fraction: fraction,
                };
                continue;
            }
            let direction = word(wram, 0x0c04_usize.saturating_add(offset))?;
            if direction & !0x001f != 0 {
                return Err(Defect("unknown direction flags"));
            }
            let heading = heading(direction & 15)?;
            let kind = word(wram, 0x0c18_usize.saturating_add(offset))?;
            let id = match kind {
                0x8000 => asset::ClipId::Flight(asset_heading(heading)),
                0x8700 => asset::ClipId::Impact,
                _ => return Err(Defect("outside uncharged power phase")),
            };
            let places = self
                .imported
                .places
                .clips
                .iter()
                .find(|places| places.id == id)
                .ok_or(Defect("missing clip places"))?;
            let cursor = places
                .frames
                .iter()
                .position(|&at| at == pointer)
                .or_else(|| (places.end == pointer).then_some(places.frames.len()))
                .ok_or(Defect("unknown clip cursor"))?;
            let picture_id = self.picture_id(wram, offset)?;
            let body = beam::ShotBody {
                position: beam::Axes {
                    x: beam::FixedPosition::from_parts(
                        word(wram, 0x0b64_usize.saturating_add(offset))?,
                        fraction.x,
                    ),
                    y: beam::FixedPosition::from_parts(
                        word(wram, 0x0b78_usize.saturating_add(offset))?,
                        fraction.y,
                    ),
                },
                heading,
                speed: beam::Axes {
                    x: beam::BeamSpeed(
                        word(wram, 0x0bdc_usize.saturating_add(offset))?.cast_signed(),
                    ),
                    y: beam::BeamSpeed(
                        word(wram, 0x0bf0_usize.saturating_add(offset))?.cast_signed(),
                    ),
                },
                radius: beam::Axes {
                    x: beam::Radius(
                        u8::try_from(word(wram, 0x0bb4_usize.saturating_add(offset))?)
                            .map_err(|_| Defect("radius"))?,
                    ),
                    y: beam::Radius(
                        u8::try_from(word(wram, 0x0bc8_usize.saturating_add(offset))?)
                            .map_err(|_| Defect("radius"))?,
                    ),
                },
                damage: beam::Damage::new(word(wram, 0x0c2c_usize.saturating_add(offset))?)
                    .ok_or(Defect("negative damage"))?,
                trail,
                clip: clip_id(id),
                next_frame: beam::ClipFrame::new(cursor, places.frames.len())
                    .ok_or(Defect("cursor"))?,
                remaining_hold: beam::FrameWord(word(wram, 0x0c54_usize.saturating_add(offset))?),
                picture: Some(picture(
                    u16::try_from(picture_id).map_err(|_| Defect("picture index"))?,
                )),
                trail_appearance: None,
            };
            *shot = match id {
                asset::ClipId::Flight(_) => beam::ShotState::Flying {
                    body,
                    consumed: direction & 16 != 0,
                },
                asset::ClipId::Impact => beam::ShotState::Impact { body },
            };
        }
        Ok(state)
    }

    fn picture_id(&self, wram: &[u8], offset: usize) -> Result<usize, Defect> {
        let pointer = word(wram, 0x0cb8_usize.saturating_add(offset))?;
        self.imported
            .places
            .pictures
            .iter()
            .position(|&at| at == pointer)
            .ok_or(Defect("unknown shot picture"))
    }
}

/// Before-cut world geometry/content and C1's interactive boxes, never after-shot values.
#[derive(Clone)]
pub struct World {
    wram: Box<[u8]>,
    active: Vec<usize>,
    observed_c1_receipt: bool,
    defect: Option<Defect>,
    events: Vec<beam::Event>,
    contacts: u64,
    actor_hits: u64,
}

/// Full preceding WRAM identities bind the independently watched pre-AI receipts. A room,
/// species or selected-field projection cannot prove the earlier actors' cross-effects.
const OBSERVED_C1_RECEIPTS: [(Frame, &str); 5] = [
    (
        Frame(156),
        "aa0021a536771517434717a5291c1158c77c8c0e529ef8d2dd28bb5c195052be",
    ),
    (
        Frame(157),
        "9dc898d6660bd089aa31434cfede7cbf72c0fc542aef54c4c1936034a0394297",
    ),
    (
        Frame(158),
        "3434e9a31081bb09138197aae88821a5fe1ac6b4859400f179d7aa02d4503add",
    ),
    (
        Frame(159),
        "fe8ceaea4ad75caf9c0c7bf0651cf74b6f3b909a5dbd7ff52de37164cabf72c0",
    ),
    (
        Frame(160),
        "df0e3d3ac8dcfe59a22c642b188616920fbc93bd187f42ee7090713c69bc265f",
    ),
];

fn has_observed_c1_receipt(from: &FrameRecord) -> bool {
    if !matches!(from.cut, Cut::NmiReturn(_)) {
        return false;
    }
    let Some((_, expected)) = OBSERVED_C1_RECEIPTS
        .iter()
        .find(|(frame, _)| *frame == Frame(from.index))
    else {
        return false;
    };
    Sha256::of(&from.wram).to_string() == *expected
}
impl World {
    /// Current selector receipt, in execution order, expressed as stable enemy slots.
    #[must_use]
    pub fn active_enemy_slots(&self) -> Vec<usize> {
        self.active.iter().map(|offset| offset / 64).collect()
    }
}
impl beam::World for World {
    fn contact_cell(&mut self, contact: &beam::Contact, cell: u16) -> beam::ContactResponse {
        self.contacts = self.contacts.saturating_add(1);
        let answer = (|| {
            let width = word(&self.wram, 0x07a5)?;
            let (row, col) = match contact.axis {
                beam::Axis::Horizontal => (
                    contact.perpendicular_start >> 4,
                    contact.leading_boundary >> 4,
                ),
                beam::Axis::Vertical => (
                    contact.leading_boundary >> 4,
                    contact.perpendicular_start >> 4,
                ),
            };
            let mut offset = ((row & 255).wrapping_mul(width & 255))
                .wrapping_add(col)
                .wrapping_mul(2);
            for _ in 0..cell {
                offset = match contact.axis {
                    beam::Axis::Horizontal => {
                        let (first, carry) = offset.overflowing_add(width);
                        first.wrapping_add(width).wrapping_add(u16::from(carry))
                    }
                    beam::Axis::Vertical => offset.wrapping_add(2),
                };
            }
            // W2 continuation is read directly, including odd overlapping words. Only the
            // unconditional Air reaction is established by this first harness part.
            if word(&self.wram, 0x10002 + usize::from(offset))? >> 12 != 0 {
                return Err(Defect("W2 shot reaction not yet decoded"));
            }
            Ok(beam::ContactResponse::Passes)
        })();
        match answer {
            Ok(answer) => answer,
            Err(error) => {
                self.defect = Some(error);
                beam::ContactResponse::Passes
            }
        }
    }
}

/// Compared State, with derived appearance omitted and free trails isolated for R1's mask.
#[derive(Clone)]
pub struct Fields {
    state: beam::State,
    free_trails: Vec<beam::FrameWord>,
}
fn fields(state: &beam::State) -> Fields {
    let mut state = state.clone();
    let mut free_trails = Vec::new();
    for (_, shot) in &mut state.slots {
        match shot {
            beam::ShotState::Free { trail, .. } => {
                free_trails.push(*trail);
                *trail = beam::FrameWord(0);
            }
            beam::ShotState::Flying { body, .. } | beam::ShotState::Impact { body } => {
                body.trail_appearance = None;
            }
        }
    }
    Fields { state, free_trails }
}
macro_rules! body_field {
    ($name:literal, $field:ident) => {
        Field {
            name: $name,
            same: |a, b| {
                a.state
                    .slots
                    .values()
                    .zip(b.state.slots.values())
                    .all(|(a, b)| {
                        a.body().map(|body| &body.$field) == b.body().map(|body| &body.$field)
                    })
            },
            show: |_| "quantity differs".into(),
        }
    };
}
impl Domain for Weapons {
    const NAME: &'static str = "weapons-projectiles";
    type State = beam::State;
    type Inputs = Option<beam::UpdateInput>;
    type World = World;
    type Fields = Fields;
    type Skip = Defect;
    const FIELDS: &'static [Field<Fields>] = &[
        Field {
            name: "phase",
            same: |a, b| {
                a.state
                    .slots
                    .values()
                    .zip(b.state.slots.values())
                    .all(|(a, b)| std::mem::discriminant(a) == std::mem::discriminant(b))
            },
            show: |_| "phase differs".into(),
        },
        Field {
            name: "free-trails",
            same: |a, b| a.free_trails == b.free_trails,
            show: |_| "trails differ".into(),
        },
        Field {
            name: "cooldown",
            same: |a, b| a.state.cooldown == b.state.cooldown,
            show: |_| "cooldown differs".into(),
        },
        Field {
            name: "count",
            same: |a, b| a.state.count == b.state.count,
            show: |_| "count differs".into(),
        },
        Field {
            name: "consumed",
            same: |a, b| {
                a.state
                    .slots
                    .values()
                    .zip(b.state.slots.values())
                    .all(|(a, b)| {
                        matches!(a, beam::ShotState::Flying { consumed: true, .. })
                            == matches!(b, beam::ShotState::Flying { consumed: true, .. })
                    })
            },
            show: |_| "consumed differs".into(),
        },
        Field {
            name: "retained-fractions",
            same: |a, b| {
                a.state
                    .slots
                    .values()
                    .zip(b.state.slots.values())
                    .all(|(a, b)| match (a, b) {
                        (
                            beam::ShotState::Free {
                                retained_fraction: a,
                                ..
                            },
                            beam::ShotState::Free {
                                retained_fraction: b,
                                ..
                            },
                        ) => a == b,
                        (beam::ShotState::Free { .. }, _) | (_, beam::ShotState::Free { .. }) => {
                            false
                        }
                        _ => true,
                    })
            },
            show: |_| "fraction differs".into(),
        },
        body_field!("position", position),
        body_field!("heading", heading),
        body_field!("speed", speed),
        body_field!("radius", radius),
        body_field!("damage", damage),
        body_field!("live-trail", trail),
        body_field!("clip", clip),
        body_field!("cursor", next_frame),
        body_field!("hold", remaining_hold),
        body_field!("picture", picture),
    ];
    fn behaviours(&self) -> Vec<Entry> {
        beam::catalogue()
            .values()
            .map(|behaviour| {
                let indices: &[usize] = match behaviour.id {
                    beam::BehaviourId::ShotSpacing => &[2],
                    beam::BehaviourId::HoldingShoot
                    | beam::BehaviourId::CannonOrigin
                    | beam::BehaviourId::MuzzleContact => {
                        &[0, 2, 3, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
                    }
                    beam::BehaviourId::InheritedMotion => &[8],
                    beam::BehaviourId::FlyingBeam => &[6, 8],
                    beam::BehaviourId::ShotContact => &[0, 6, 10, 12, 13, 14],
                    beam::BehaviourId::BeamTrail => &[11],
                    beam::BehaviourId::LeavingView => &[0, 3, 5],
                    beam::BehaviourId::EndingShot => &[0, 3, 4, 5],
                    beam::BehaviourId::BeamPictures => &[0, 3, 9, 13, 14, 15],
                    beam::BehaviourId::BeamVisibility => &[],
                };
                Entry {
                    name: format!("{:?}", behaviour.id),
                    fields: indices.iter().fold(FieldSet::EMPTY, |fields, &index| {
                        fields.union(FieldSet::single(index))
                    }),
                }
            })
            .collect()
    }
    fn exercised(
        &self,
        state: &beam::State,
        input: &Option<beam::UpdateInput>,
        world: &World,
    ) -> Vec<usize> {
        let Some(input) = input else {
            return Vec::new();
        };
        let active = input.mode == beam::UpdateMode::ShotsOnly || !input.time_stopped;
        let flying = state.slots.values().any(|shot| {
            matches!(
                shot,
                beam::ShotState::Flying {
                    consumed: false,
                    ..
                }
            )
        });
        let live = state.slots.values().any(|shot| shot.body().is_some());
        let spawned = world
            .events
            .iter()
            .any(|event| matches!(event, beam::Event::FireSound(_)));
        beam::catalogue()
            .values()
            .enumerate()
            .filter_map(|(index, behaviour)| {
                let exercised = match behaviour.id {
                    beam::BehaviourId::ShotSpacing => {
                        input.mode == beam::UpdateMode::CannonAndShots
                    }
                    beam::BehaviourId::HoldingShoot => {
                        active
                            && input.mode == beam::UpdateMode::CannonAndShots
                            && input.cannon.shoot
                    }
                    beam::BehaviourId::CannonOrigin
                    | beam::BehaviourId::InheritedMotion
                    | beam::BehaviourId::MuzzleContact => spawned,
                    beam::BehaviourId::FlyingBeam | beam::BehaviourId::LeavingView => {
                        active && (flying || spawned)
                    }
                    beam::BehaviourId::ShotContact => world.contacts != 0,
                    beam::BehaviourId::BeamTrail => active && (flying || spawned),
                    beam::BehaviourId::EndingShot => active && (live || world.actor_hits != 0),
                    beam::BehaviourId::BeamPictures => active && (live || spawned),
                    beam::BehaviourId::BeamVisibility => false,
                };
                exercised.then_some(index)
            })
            .collect()
    }
    fn decode(
        &self,
        from: &FrameRecord,
        _: Interval,
    ) -> Result<(beam::State, Option<beam::UpdateInput>), Defect> {
        let w = |at| word(&from.wram, at);
        if w(0x0998)? == 9 {
            if w(0x099c)? != 0xe17d || w(0x0e16)? != 0 {
                return Err(Defect("Quieting caller path not yet validated"));
            }
            let state = self.state(&from.wram)?;
            if state.count.0 != 0 || state.slots.values().any(|shot| shot.body().is_some()) {
                return Err(Defect(
                    "NoWeapons caller with live C1 effects not yet decoded",
                ));
            }
            // R1's Quieting path runs neither cannon nor shots and does not reset them.
            // C1 has no owned shot to consume under the independently checked guard.
            return Ok((state, None));
        }
        let pose = w(0x0a1c)?;
        if !matches!(w(0x0998)?, 8 | 11) || !matches!(w(0x0a42)?, 0xe695 | 0xe6c9 | 0xe713) {
            return Err(Defect("caller game mode or PRE handler not yet validated"));
        }
        if (w(0x008b)? | w(0x008f)?) & (w(0x09b8)? | w(0x09ba)?) != 0 {
            return Err(Defect(
                "current-frame item selection producer not yet modelled",
            ));
        }
        let movement = w(0x0a1f)? & 255;
        let binding = w(0x09b2)?;
        let eligible = match movement {
            0 | 1 | 2 | 5 | 6 | 16 | 21 => true,
            3 => false,
            _ if w(0x008b)? & binding == 0 => false,
            _ => return Err(Defect("caller eligibility not yet decoded")),
        };
        let pending = w(0x0b5e)?;
        let input = beam::UpdateInput {
            mode: if matches!(pose, 0 | 0x009b) || w(0x0a42)? == 0xe713 {
                beam::UpdateMode::ShotsOnly
            } else {
                beam::UpdateMode::CannonAndShots
            },
            time_stopped: w(0x0a78)? != 0,
            cannon: beam::CannonInput {
                shoot: eligible && w(0x008b)? & binding != 0 && w(0x09a6)? == 0,
                new_press: w(0x008f)? & binding != 0,
                previous_new_press: w(0x0e00)? & binding != 0,
                muzzle: self.muzzle(pose)?,
                previous_muzzle: self.muzzle(w(0x0a24)?)?.aim,
                moving_muzzle: movement == 1 || matches!(pose, 0x75 | 0x76),
                pending_aim: if pending == 0 {
                    None
                } else {
                    Some(heading(pending & 255)?)
                },
                centre: beam::SamusCentre(beam::Axes {
                    x: w(0x0af6)?,
                    y: w(0x0afa)?,
                }),
            },
            // Inheritance is unread unless a shot spawns. Available firing attempts
            // remain defects until the firing-boundary input supplier is validated.
            inheritance: beam::Inheritance::stationary(),
            camera: beam::ScreenOrigin(beam::Axes {
                x: w(0x0911)?,
                y: w(0x0915)?,
            }),
            room: beam::RoomScreens(beam::Axes {
                x: w(0x07a9)?,
                y: w(0x07ab)?,
            }),
        };
        let state = self.state(&from.wram)?;
        if w(0x0998)? == 11
            && (state.count.0 != 0
                || state.cooldown.0 != 0
                || state.slots.values().any(|shot| shot.body().is_some()))
        {
            return Err(Defect(
                "live W1 passage/reset caller contract not yet decoded",
            ));
        }
        let mut cooldown = state.cooldown;
        beam::update_spacing(&mut cooldown, input.mode, input.time_stopped);
        if input.mode == beam::UpdateMode::CannonAndShots
            && !input.time_stopped
            && input.cannon.shoot
            && cooldown.0.trailing_zeros() >= 8
            && state.count.0.wrapping_sub(5).cast_signed() < 0
        {
            let ordinary = matches!(w(0x0a42)?, 0xe695 | 0xe6c9)
                && matches!(movement, 0 | 1 | 2 | 5 | 6 | 16 | 21)
                && w(0x0d32)? == 0xc4f0
                && w(0x09a6)? & 0x100f == 0
                && w(0x0a76)? == 0
                && w(0x09d2)? == 0;
            if !ordinary {
                return Err(Defect(
                    "firing caller outside validated ordinary power-beam contract",
                ));
            }
        }
        let mut input = input;
        input.inheritance = inheritance(&from.wram)?;
        Ok((state, Some(input)))
    }
    fn world(&self, from: &FrameRecord, _: &FrameRecord) -> Result<World, Defect> {
        let camera = (word(&from.wram, 0x0911)?, word(&from.wram, 0x0915)?);
        let commons = crate::enemy::EnemySources::ORIGINAL
            .commons(&from.wram)
            .map_err(|_| Defect("enemy selector input decode"))?;
        // Selection rebuilds its receipt each frame. The original empties the raw lists
        // before the cut, so those stored lists cannot answer this frame's contacts.
        let active = commons
            .iter()
            .enumerate()
            .filter_map(|(index, common)| {
                common
                    .as_ref()
                    .filter(|common| game::enemy::touchable(common, camera.0, camera.1))
                    .map(|_| index.saturating_mul(64))
            })
            .collect();
        Ok(World {
            wram: from.wram.clone(),
            active,
            observed_c1_receipt: has_observed_c1_receipt(from),
            defect: None,
            events: Vec::new(),
            contacts: 0,
            actor_hits: 0,
        })
    }
    fn masks(
        &self,
        from: &FrameRecord,
        _: &FrameRecord,
        _: &Option<beam::UpdateInput>,
    ) -> Result<Vec<Mask>, Defect> {
        Ok(if word(&from.wram, 0x0998)? == 11 {
            vec![Mask::another(
                "R1 room reset clears only Free trail storage",
                FieldSet::single(1),
            )]
        } else {
            Vec::new()
        })
    }
    fn step(&self, state: &mut beam::State, input: &Option<beam::UpdateInput>, world: &mut World) {
        let Some(input) = input else {
            return;
        };
        world.events = beam::update(state, input, &self.content, self.rules, world);
        // C1's strict overlap and first shot per interactive enemy, before enemy AI.
        let event = (|| {
            if word(&world.wram, 0x0998)? != 8 {
                return Ok(());
            }
            if state.count.0 == 0
                || !state
                    .slots
                    .values()
                    .any(|shot| matches!(shot, beam::ShotState::Flying { .. }))
            {
                return Ok(());
            }
            if !world.observed_c1_receipt || world.active != [0, 64] {
                return Err(Defect(
                    "C1 pre-AI receipt outside independently watched inputs",
                ));
            }
            if word(&world.wram, 0x0e4c)? == 0
                || word(&world.wram, 0x0e12)? != 0
                || word(&world.wram, 0x0a78)? != 0
                || word(&world.wram, 0x185e)? != 0
            {
                return Err(Defect(
                    "C1 interaction guards outside validated simple-box contract",
                ));
            }
            for &off in &world.active {
                if word(&world.wram, 0x0f78_usize.saturating_add(off))? == 0xdaff {
                    continue;
                }
                if word(&world.wram, 0x0f88_usize.saturating_add(off))? & 4 != 0 {
                    return Err(Defect("C1 multibox contact not yet decoded"));
                }
                if word(&world.wram, 0x0f78_usize.saturating_add(off))? == 0
                    || matches!(
                        word(&world.wram, 0x0f8e_usize.saturating_add(off))?,
                        0 | 0x804d
                    )
                    || word(&world.wram, 0x0f86_usize.saturating_add(off))? & 0x0400 != 0
                    || word(&world.wram, 0x0fa0_usize.saturating_add(off))? != 0
                {
                    continue;
                }
                let mut consumed = None;
                for (slot, shot) in &state.slots {
                    let beam::ShotState::Flying { body, .. } = shot else {
                        continue;
                    };
                    let overlap = |position: u16, radius: u8, at: usize, extent: usize| {
                        let distance =
                            position.wrapping_sub(word(&world.wram, at.saturating_add(off))?);
                        let distance = if distance.cast_signed() < 0 {
                            distance.wrapping_neg()
                        } else {
                            distance
                        };
                        Ok::<_, Defect>(
                            distance < u16::from(radius)
                                || distance.wrapping_sub(u16::from(radius))
                                    < word(&world.wram, extent.saturating_add(off))?,
                        )
                    };
                    if overlap(body.position.x.integer(), body.radius.x.0, 0x0f7a, 0x0f82)?
                        && overlap(body.position.y.integer(), body.radius.y.0, 0x0f7e, 0x0f84)?
                    {
                        consumed = Some(slot);
                        break;
                    }
                }
                if let Some(slot) = consumed {
                    state.consume(slot);
                    world.actor_hits = world.actor_hits.saturating_add(1);
                }
            }
            Ok::<(), Defect>(())
        })();
        if let Err(error) = event {
            world.defect = Some(error);
        }
    }
    fn fields(&self, state: &beam::State) -> Fields {
        fields(state)
    }
    fn checked_step(&self, world: &World) -> Result<(), Defect> {
        world.defect.clone().map_or(Ok(()), Err)
    }
    fn expected(&self, next: &FrameRecord) -> Result<Fields, Defect> {
        self.state(&next.wram).map(|state| fields(&state))
    }
    fn uncompared(&self, skip: &Defect) -> Uncompared {
        Uncompared::Defect(skip.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{World, beam};

    fn world(width: u16) -> World {
        let mut wram = vec![0_u8; 0x20000].into_boxed_slice();
        wram[0x07a5..0x07a7].copy_from_slice(&width.to_le_bytes());
        World {
            wram,
            active: Vec::new(),
            observed_c1_receipt: false,
            defect: None,
            events: Vec::new(),
            contacts: 0,
            actor_hits: 0,
        }
    }

    fn contact() -> beam::Contact {
        beam::Contact {
            slot: beam::ShotSlot::Slot0,
            heading: beam::Heading::Right,
            axis: beam::Axis::Horizontal,
            centre: beam::Axes { x: 0, y: 16 },
            radius: beam::Axes {
                x: beam::Radius(1),
                y: beam::Radius(1),
            },
            leading_boundary: 0,
            perpendicular_start: 16,
            cell_count: 2,
        }
    }

    #[test]
    fn continuation_keeps_carry_and_overlapping_words_instead_of_rounding_to_cells() {
        use beam::World as _;
        let mut world = world(u16::MAX);
        // Invented first cell is Air. Carry between horizontal width additions makes
        // the second visit's word overlap it, exposing an unsupported solid response.
        world.wram[0x10002 + 510] = 0x10;
        assert_eq!(
            world.contact_cell(&contact(), 0),
            beam::ContactResponse::Passes
        );
        assert!(world.defect.is_none());
        let _ = world.contact_cell(&contact(), 1);
        assert!(world.defect.is_some());
        assert_eq!(world.contacts, 2);
    }

    #[test]
    fn continuation_past_owned_level_is_read_and_unreadable_memory_is_a_defect() {
        use beam::World as _;
        let mut world = world(3);
        assert_eq!(
            world.contact_cell(&contact(), 0),
            beam::ContactResponse::Passes
        );
        assert!(world.defect.is_none());
        world.wram = Box::default();
        let _ = world.contact_cell(&contact(), 0);
        assert!(world.defect.is_some());
    }
}
