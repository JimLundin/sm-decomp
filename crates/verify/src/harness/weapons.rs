//! W1's uncharged power beam. All inputs and actor boxes come from the preceding cut.
//!
//! Checked `OrdinaryPower` production owns the persisted inputs and per-actor C1 events.
//! The bounded complete modeled State/OAM prefix leaves broader callers and sound unsupported.

use std::fmt;

use data::assets::projectile as asset;
use data::assets::samus::map;
use enum_map::EnumMap;
use oracle_record::FrameRecord;
use weapons_projectiles as beam;

use super::{Domain, Entry, Field, FieldSet, Interval, Mask, Uncompared};

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
pub struct Defect(pub(super) &'static str);
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

fn inheritance(wram: &[u8]) -> Result<beam::Inheritance, Defect> {
    let distance = |whole, fraction| -> Result<beam::MovementDistance, Defect> {
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

impl From<super::gameplay_caller::Error> for Defect {
    fn from(error: super::gameplay_caller::Error) -> Self {
        Self(error.message())
    }
}

/// Imported production content and verification-only original-place relations.
pub struct Weapons {
    caller: super::gameplay_caller::Caller,
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
        let content = game::weapons_projectiles_bridge::PowerContent::new(imported.content.clone())
            .map_err(|_| Defect("AF3 model content conversion"))?
            .model()
            .clone();
        Ok(Self {
            caller: super::gameplay_caller::Caller::load(image)?,
            imported,
            content,
            rules: beam::Rules::REFERENCE,
        })
    }
    /// Independently decode absent neighbouring producers from the same seed cut.
    /// # Errors
    /// Missing words, unsupported beam equipment or occupied competing pools.
    pub fn ordinary_seed_facts(
        from: &FrameRecord,
    ) -> Result<game::state::projectile::PowerSeedFacts, Defect> {
        use game::state::projectile::{
            ChargeEquipment, GrappleActivity, HyperActivity, PowerSeedFacts, PowerTime,
            ProducerPool,
        };
        let w = |at| word(&from.wram, at);
        if w(0x0e12)? != 0
            || w(0x185e)? != 0
            || w(0x0e16)? != 0
            || w(0x09d2)? != 0
            || w(0x0a04)? != 0
            || !matches!(w(0x0a42)?, 0xe695 | 0xe6c9 | 0xe713)
        {
            return Err(Defect(
                "ordinary seed has unsupported caller interaction or selection",
            ));
        }
        let equipment = w(0x09a6)?;
        if equipment & !0x1000 != 0 {
            return Err(Defect("ordinary seed has unsupported beam equipment"));
        }
        let pool = |bases: &[usize], count: usize| -> Result<ProducerPool, Defect> {
            for &base in bases {
                for offset in (0..count.saturating_mul(2)).step_by(2) {
                    if w(base.saturating_add(offset))? != 0 {
                        return Ok(ProducerPool::Occupied);
                    }
                }
            }
            Ok(ProducerPool::Empty)
        };
        let mut other_player_slots = ProducerPool::Empty;
        for offset in (10..20).step_by(2) {
            if w(0x0c40_usize.saturating_add(offset))? != 0 {
                other_player_slots = ProducerPool::Occupied;
            }
        }
        Ok(PowerSeedFacts {
            beams: game::state::slots::FlagSet::default(),
            charge: if equipment & 0x1000 == 0 {
                ChargeEquipment::Unequipped
            } else {
                ChargeEquipment::Equipped
            },
            charge_history: (
                game::state::slots::Frames(w(0x0cd0)?),
                game::state::slots::Frames(w(0x0dc2)?),
            ),
            hyper: if w(0x0a76)? == 0 {
                HyperActivity::Inactive
            } else {
                HyperActivity::Active
            },
            grapple: if w(0x0d32)? == 0xc4f0 {
                GrappleActivity::Inactive
            } else {
                GrappleActivity::Unsupported
            },
            time: if w(0x0a78)? == 0 {
                PowerTime::Running
            } else {
                PowerTime::Stopped
            },
            sprite_objects: pool(&[0xef78], 32)?,
            enemy_projectiles: pool(&[0x1997], 18)?,
            // Both sides independently remain occupied while their timer word is nonzero.
            trail_effects: pool(&[0xd658, 0xd67c], 18)?,
            other_player_slots,
        })
    }
    /// Decode the complete ordinary owner from this cut alone.
    ///
    /// # Errors
    /// Unsupported caller/binding, ambiguous Impact seed or invalid imported references.
    pub fn ordinary_owner(
        &self,
        from: &FrameRecord,
        facts: game::state::projectile::PowerSeedFacts,
        content: &game::weapons_projectiles_bridge::PowerContent,
    ) -> Result<game::state::projectile::OrdinaryPower, Defect> {
        if word(&from.wram, 0x0998)? != 8 {
            return Err(Defect(
                "ordinary ownership requires Gameplay; R1 is unsupported",
            ));
        }
        self.ordinary_owner_fields(from, facts, content)
    }

    /// Restore only the exact reference cut that passed the completed-entry certificate.
    #[cfg(test)]
    pub(crate) fn completed_entry_owner(
        &self,
        certificate: &crate::decode::CompletedEntry<'_>,
        facts: game::state::projectile::PowerSeedFacts,
        content: &game::weapons_projectiles_bridge::PowerContent,
    ) -> Result<game::state::projectile::OrdinaryPower, Defect> {
        self.ordinary_owner_fields(certificate.record(), facts, content)
    }

    fn ordinary_owner_fields(
        &self,
        from: &FrameRecord,
        facts: game::state::projectile::PowerSeedFacts,
        content: &game::weapons_projectiles_bridge::PowerContent,
    ) -> Result<game::state::projectile::OrdinaryPower, Defect> {
        if word(&from.wram, 0x09b2)? != 0x0040 {
            return Err(Defect("ordinary engine shoot binding is unsupported"));
        }
        let mut decoded = self.state(&from.wram)?;
        for shot in decoded.slots.values_mut() {
            match shot {
                beam::ShotState::Free { .. } => {}
                beam::ShotState::Impact { .. } => {
                    return Err(Defect(
                        "retained Impact trail appearance seed is unsupported",
                    ));
                }
                beam::ShotState::Flying { body, .. } => {
                    let clip = self
                        .content
                        .clips
                        .get(&body.clip)
                        .ok_or(Defect("ordinary seed clip"))?;
                    let displayed = body
                        .next_frame
                        .index()
                        .checked_sub(1)
                        .and_then(|index| clip.frames.get(index))
                        .ok_or(Defect(
                            "ordinary seed has no preceding displayed flight frame",
                        ))?;
                    body.trail_appearance
                        .clone_from(&displayed.trail_appearance);
                }
            }
        }
        game::state::projectile::OrdinaryPower::from_seed(decoded, facts, content)
            .map_err(|_| Defect("ordinary seed producer/content capability"))
    }
    /// Predict post-W1 bodies exclusively from preceding inputs and imported content.
    pub(super) fn predict(
        &self,
        from: &FrameRecord,
        interval: Interval,
    ) -> Result<beam::State, Defect> {
        let (mut state, input) = self.decode(from, interval)?;
        let mut world = self.preceding_world(from)?;
        if let Some(input) = input {
            world.borrowed.clone()?;
            let _ = beam::update(&mut state, &input, &self.content, self.rules, &mut world);
        }
        self.checked_step(&world)?;
        Ok(state)
    }
    pub(super) fn marker_memory_inputs(&self, from: &FrameRecord) -> Result<(), Defect> {
        self.caller.marker_memory_inputs(from).map_err(Defect::from)
    }
    pub(super) fn contact_start(&self, from: &FrameRecord) -> Result<(), Defect> {
        self.caller
            .start(from)
            .and_then(|()| self.caller.weapon_inputs(from))
            .and_then(|()| {
                self.caller.contact_inputs(
                    from,
                    &super::contact::processing_receipt(&from.wram)
                        .map_err(|_| super::gameplay_caller::Error::BorrowedActorInputs)?,
                )
            })
            .map_err(Defect::from)
    }
    fn preceding_world(&self, from: &FrameRecord) -> Result<World, Defect> {
        Ok(World {
            wram: from.wram.clone(),
            active: super::contact::processing_receipt(&from.wram)?,
            caller: self.contact_start(from),
            borrowed: self.caller.weapon_inputs(from).map_err(Defect::from),
            completion: Ok(()),
            defect: None,
            events: Vec::new(),
            contacts: 0,
            actor_hits: 0,
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

/// Before-cut geometry/content and source processing receipt, never after-shot values.
#[derive(Clone)]
pub struct World {
    wram: Box<[u8]>,
    active: Vec<usize>,
    caller: Result<(), Defect>,
    borrowed: Result<(), Defect>,
    completion: Result<(), Defect>,
    defect: Option<Defect>,
    events: Vec<beam::Event>,
    contacts: u64,
    actor_hits: u64,
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
    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<World, Defect> {
        // The raw preceding list is cleared at the cut. Reconstruct its source-backed
        // processing receipt; this is separate from Samus's solid interaction list.
        let mut world = self.preceding_world(from)?;
        world.completion = super::gameplay_caller::completed(from, next).map_err(Defect::from);
        Ok(world)
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
        if let Err(error) = &world.borrowed {
            world.defect = Some(error.clone());
            return;
        }
        world.events = beam::update(state, input, &self.content, self.rules, world);
        if super::contact::has_live_power(state)
            && let Err(error) = world.caller.clone().and_then(|()| world.completion.clone())
        {
            world.defect = Some(error);
            return;
        }
        match super::contact::apply(&world.wram, &world.active, state) {
            Ok(hits) => world.actor_hits = world.actor_hits.saturating_add(hits),
            Err(error) => world.defect = Some(error),
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

    #[test]
    fn quiet_prediction_requires_valid_borrowed_inputs_before_update() {
        use crate::harness::{Domain, Interval};
        let Some((cache, rom)) = crate::harness::tests::environment() else {
            return;
        };
        let domain = super::Weapons::load(&std::fs::read(rom).unwrap()).unwrap();
        let lock = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scenarios/locks/lsnes.toml");
        let cached = crate::harness::open_cached(&lock, &cache, "chain-any-0003-DFD7").unwrap();
        let records = cached
            .records_without_pictures()
            .map(|record| record.unwrap().1)
            .collect::<Vec<_>>();
        let pair = records
            .windows(2)
            .find(|pair| {
                domain
                    .predict(&pair[0], Interval::ending_at(&pair[1]))
                    .is_ok_and(|state| !super::super::contact::has_live_power(&state))
            })
            .unwrap();
        let mut from = pair[0].clone();
        let put = |record: &mut oracle_record::FrameRecord, at: usize, value: u16| {
            record.wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
        };
        for slot in (0..12).step_by(2) {
            put(&mut from, 0x18b4 + slot, 0);
        }
        put(&mut from, 0x18b0, 0x8000);
        put(&mut from, 0x18b4, 1);
        put(&mut from, 0x18f0, 0x86ef);
        from.wram[0x18fc] = 0x88;
        put(&mut from, 0x18e4, 0);
        put(&mut from, 0x0a7a, 5);
        put(&mut from, 0x0a78, 1);
        let interval = Interval::ending_at(&pair[1]);
        assert!(domain.predict(&from, interval).is_err());
        let (mut state, input) = domain.decode(&from, interval).unwrap();
        assert!(input.is_some());
        let before = state.clone();
        let mut world = domain.world(&from, &pair[1]).unwrap();
        domain.step(&mut state, &input, &mut world);
        assert!(domain.checked_step(&world).is_err());
        assert_eq!(state, before);
    }

    fn world(width: u16) -> World {
        let mut wram = vec![0_u8; 0x20000].into_boxed_slice();
        wram[0x07a5..0x07a7].copy_from_slice(&width.to_le_bytes());
        World {
            wram,
            active: Vec::new(),
            caller: Ok(()),
            borrowed: Ok(()),
            completion: Ok(()),
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
