//! Ordinary power ownership, content conversion and ordered presentation adapters.
//! Original content-place associations remain in the verifier's importer edge.

use data::assets::projectile as asset;
use data::assets::samus::map;
use data::pose::PoseId;
use enum_map::EnumMap;
use std::collections::BTreeMap;
use weapons_projectiles as beam;

/// A unsupported content or production capability, without original bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Invalid imported or restored content relationship.
    Content,
    /// Unavailable named pose muzzle.
    Muzzle,
    /// Unrecognized picture identity.
    Picture,
    /// Unsupported trail appearance or undelivered request.
    Trail,
    /// Unavailable or unsupported typed collision cell.
    World,
    /// A neighbouring producer or caller lies outside ordinary ownership.
    Applicability,
}

/// Immutable production content; construction establishes the model conversion invariant.
#[derive(Clone, Debug)]
pub struct PowerContent {
    assets: asset::Content,
    model: beam::PowerBeam,
}
impl PartialEq for PowerContent {
    fn eq(&self, other: &Self) -> bool {
        self.assets == other.assets
    }
}
impl Eq for PowerContent {}

/// Convert a typed asset heading into the clean model heading.
#[must_use]
pub const fn heading(value: asset::Heading) -> beam::Heading {
    match value {
        asset::Heading::UpFacingRight => beam::Heading::UpFacingRight,
        asset::Heading::UpRight => beam::Heading::UpRight,
        asset::Heading::Right => beam::Heading::Right,
        asset::Heading::DownRight => beam::Heading::DownRight,
        asset::Heading::DownFacingRight => beam::Heading::DownFacingRight,
        asset::Heading::DownFacingLeft => beam::Heading::DownFacingLeft,
        asset::Heading::DownLeft => beam::Heading::DownLeft,
        asset::Heading::Left => beam::Heading::Left,
        asset::Heading::UpLeft => beam::Heading::UpLeft,
        asset::Heading::UpFacingLeft => beam::Heading::UpFacingLeft,
    }
}

/// Convert a clean model heading into the typed asset heading.
#[must_use]
pub const fn asset_heading(value: beam::Heading) -> asset::Heading {
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

/// The stable content identity of a named asset clip.
#[must_use]
pub fn clip_id(id: asset::ClipId) -> beam::ClipId {
    beam::ClipId(format!("{id:?}"))
}
/// The stable content identity of an imported sprite.
#[must_use]
pub fn picture(id: u16) -> beam::PictureId {
    beam::PictureId(id.to_string())
}
/// The stable content identity of an imported trail appearance.
#[must_use]
pub fn appearance(id: u16) -> beam::TrailAppearanceId {
    beam::TrailAppearanceId(id.to_string())
}

impl PowerContent {
    /// Convert checked AF3 assets into the clean model's content.
    /// # Errors
    /// Invalid frame, picture, trail or repetition references.
    pub fn new(assets: asset::Content) -> Result<Self, Fault> {
        let source = &assets.power;
        let mut clips = BTreeMap::new();
        let kinds = asset::Heading::ALL
            .into_iter()
            .map(asset::ClipId::Flight)
            .chain([asset::ClipId::Impact]);
        for kind in kinds {
            let source_clip = source.clip(kind);
            if source_clip.frames.is_empty() {
                return Err(Fault::Content);
            }
            let frames = source_clip
                .frames
                .iter()
                .map(|frame| {
                    source
                        .sheet
                        .sprites
                        .get(usize::from(frame.picture.0))
                        .ok_or(Fault::Picture)?;
                    let trail = source
                        .trails
                        .get(usize::from(frame.trail_appearance.0))
                        .ok_or(Fault::Trail)?;
                    Ok(beam::Frame {
                        hold: beam::FrameHold::new(frame.hold.updates()).ok_or(Fault::Content)?,
                        picture: picture(frame.picture.0),
                        radius: beam::Axes {
                            x: beam::Radius(frame.radius.x),
                            y: beam::Radius(frame.radius.y),
                        },
                        trail_appearance: match trail {
                            asset::TrailAppearance::PowerBeam(_) => {
                                Some(appearance(frame.trail_appearance.0))
                            }
                            asset::TrailAppearance::Inactive => None,
                        },
                    })
                })
                .collect::<Result<Vec<_>, Fault>>()?;
            let ending = match source_clip.end {
                asset::ClipEnd::Remove => beam::ClipEnding::Remove,
                asset::ClipEnd::RepeatFrom(at) => beam::ClipEnding::RepeatFrom(
                    beam::ClipFrame::new(usize::from(at.0), frames.len()).ok_or(Fault::Content)?,
                ),
            };
            clips.insert(clip_id(kind), beam::Clip { frames, ending });
        }
        let model = beam::PowerBeam {
            damage: beam::Damage::new(source.damage.points()).ok_or(Fault::Content)?,
            flying_clips: EnumMap::from_fn(|h| clip_id(asset::ClipId::Flight(asset_heading(h)))),
            impact_clip: clip_id(asset::ClipId::Impact),
            clips,
        };
        Ok(Self { assets, model })
    }
    /// Validate complete restored model references before a State becomes an owner.
    /// # Errors
    /// Unknown clips/pictures/trails, invalid frame cursors or zero-count flying occupants.
    pub fn validate_state(&self, state: &beam::State) -> Result<(), Fault> {
        let mut flying = false;
        for shot in state.slots.values() {
            let body = match shot {
                beam::ShotState::Free { .. } => continue,
                beam::ShotState::Flying { body, .. } => {
                    flying = true;
                    body
                }
                beam::ShotState::Impact { body } => body,
            };
            let expected_clip = match shot {
                beam::ShotState::Flying { body, .. } => &self.model.flying_clips[body.heading],
                beam::ShotState::Impact { .. } => &self.model.impact_clip,
                beam::ShotState::Free { .. } => return Err(Fault::Content),
            };
            if &body.clip != expected_clip {
                return Err(Fault::Content);
            }
            let clip = self.model.clips.get(&body.clip).ok_or(Fault::Content)?;
            if body.next_frame.index() > clip.frames.len() {
                return Err(Fault::Content);
            }
            if let Some(picture) = &body.picture {
                self.picture(picture)?;
            }
            if let Some(appearance) = &body.trail_appearance {
                self.trail(appearance)?;
            }
        }
        if flying && state.count.0 == 0 {
            return Err(Fault::Applicability);
        }
        Ok(())
    }

    fn picture(&self, id: &beam::PictureId) -> Result<&data::spritemap::Spritemap, Fault> {
        let index = id.0.parse::<usize>().map_err(|_| Fault::Picture)?;
        self.assets
            .power
            .sheet
            .sprites
            .get(index)
            .ok_or(Fault::Picture)
    }
    fn trail(&self, id: &beam::TrailAppearanceId) -> Result<asset::TrailOffsets, Fault> {
        let index = id.0.parse::<usize>().map_err(|_| Fault::Trail)?;
        match self.assets.power.trails.get(index) {
            Some(asset::TrailAppearance::PowerBeam(offsets)) => Ok(*offsets),
            Some(asset::TrailAppearance::Inactive) | None => Err(Fault::Trail),
        }
    }

    /// Resolve one model picture producer into the shared sprite draw order.
    /// # Errors
    /// An unrecognized restored picture identity.
    pub fn draw(
        &self,
        pictures: &[beam::PictureIntent],
    ) -> Result<crate::intent::SpriteDraw, Fault> {
        let mut draw = crate::intent::SpriteDraw::default();
        for picture in pictures {
            draw.place(
                self.picture(&picture.picture)?,
                types::Pair {
                    x: i32::from(picture.screen.0.x.cast_signed()),
                    y: i32::from(picture.screen.0.y.cast_signed()),
                },
            );
        }
        Ok(draw)
    }

    /// The immutable converted clean-model content.
    #[must_use]
    pub const fn model(&self) -> &beam::PowerBeam {
        &self.model
    }

    /// The typed muzzle of this pose, with all heading offsets already imported.
    /// # Errors
    /// No declared muzzle for the named pose.
    pub fn muzzle(&self, pose: PoseId) -> Result<beam::Muzzle, Fault> {
        let key = map::MAP
            .iter()
            .find(|entry| entry.id == pose)
            .ok_or(Fault::Muzzle)?
            .key;
        let muzzle = self.assets.muzzle(key).ok_or(Fault::Muzzle)?;
        let aim = match muzzle.aim {
            asset::MuzzleAim::Forbidden => beam::MuzzleAim::Forbidden,
            asset::MuzzleAim::PreviousAim => beam::MuzzleAim::PreviousAim,
            asset::MuzzleAim::Aimed(h) => beam::MuzzleAim::Aimed(heading(h)),
        };
        let offsets = |values: &asset::Headings<asset::Offset>| {
            EnumMap::from_fn(|h| {
                let offset = values.get(asset_heading(h));
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
}

/// Air-only geometric World. A failed lookup is latched; the staged model is discarded.
struct AirWorld<'a> {
    room: &'a crate::state::room::RoomScoped,
    fault: Option<Fault>,
}
impl AirWorld<'_> {
    fn cell(&self, contact: &beam::Contact, ordinal: u16) -> Result<(), Fault> {
        if ordinal >= contact.cell_count {
            return Err(Fault::World);
        }
        let width = self.room.width.0;
        let height = self.room.height.0;
        if width == 0 || width > 255 {
            return Err(Fault::World);
        }
        let (mut row, mut column) = match contact.axis {
            beam::Axis::Horizontal => (
                contact.perpendicular_start >> 4,
                contact.leading_boundary >> 4,
            ),
            beam::Axis::Vertical => (
                contact.leading_boundary >> 4,
                contact.perpendicular_start >> 4,
            ),
        };
        if row > 255 {
            return Err(Fault::World);
        }
        match contact.axis {
            beam::Axis::Horizontal => row = row.checked_add(ordinal).ok_or(Fault::World)?,
            beam::Axis::Vertical => column = column.checked_add(ordinal).ok_or(Fault::World)?,
        }
        if row >= height || column >= width {
            return Err(Fault::World);
        }
        let index = row
            .checked_mul(width)
            .and_then(|at| at.checked_add(column))
            .ok_or(Fault::World)?;
        // A checked even byte address has the same cell as the typed geometric request.
        index.checked_mul(2).ok_or(Fault::World)?;
        let block = self
            .room
            .blocks
            .get(usize::from(index))
            .ok_or(Fault::World)?;
        if !matches!(block.kind, data::level::BlockKind::Air(_)) {
            return Err(Fault::World);
        }
        Ok(())
    }
}
impl beam::World for AirWorld<'_> {
    fn contact_cell(&mut self, contact: &beam::Contact, cell: u16) -> beam::ContactResponse {
        if let Err(fault) = self.cell(contact, cell) {
            self.fault.get_or_insert(fault);
        }
        beam::ContactResponse::Passes
    }
}

/// A validated transient trail request. Its destination executes after Flying pictures.
#[derive(Clone, Debug)]
struct TrailRequest {
    position: beam::Axes<beam::FixedPosition>,
    offsets: asset::TrailOffsets,
}
/// Returned requests remain transient until their ordered caller destinations consume them.
#[derive(Clone, Debug, Default)]
pub struct FrameRequests {
    /// The cannon consumed pending aim on an available attempt.
    pub pending_aim_consumed: bool,
    trails: Vec<TrailRequest>,
}
/// One real trail request resolved and terminated without a sprite-producing frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrailDelivery {
    /// Resolved left and right origins before immediate termination.
    pub origins: [types::Pair<u16>; 2],
}

impl FrameRequests {
    /// Require every transient request to have reached its declared drawing destination.
    /// # Errors
    /// A trail was left pending across a completed drawing pass.
    pub const fn finish(&self) -> Result<(), Fault> {
        if self.trails.is_empty() {
            Ok(())
        } else {
            Err(Fault::Trail)
        }
    }

    /// Complete the checked immediate-termination disposition at the layer3 trail point.
    /// # Errors
    /// More than five ordinary requests, which exceeds this bounded producer capability.
    pub fn deliver_trails(&mut self) -> Result<Vec<TrailDelivery>, Fault> {
        if self.trails.len() > 5 {
            return Err(Fault::Trail);
        }
        let mut delivered = Vec::with_capacity(self.trails.len());
        for trail in self.trails.drain(..) {
            // Resolve both real event origins before their empty animation terminates.
            // No origin is retained: there is no later reader in this bounded route.
            delivered.push(TrailDelivery {
                origins: [trail.offsets.left, trail.offsets.right].map(|offset| types::Pair {
                    x: trail.position.x.integer().wrapping_add_signed(offset.x),
                    y: trail.position.y.integer().wrapping_add_signed(offset.y),
                }),
            });
        }
        Ok(delivered)
    }
}

/// Stage an entire clean update; unsupported World or event destination leaves its owner intact.
/// # Errors
/// Unavailable Air-only geometry or unsupported restored content/event capabilities.
pub fn update(
    owner: &mut crate::state::projectile::OrdinaryPower,
    input: &beam::UpdateInput,
    room: &crate::state::room::RoomScoped,
    content: &PowerContent,
) -> Result<FrameRequests, Fault> {
    if input.time_stopped {
        return Err(Fault::Applicability);
    }
    content.validate_state(&owner.model.0)?;
    let mut staged = owner.model.0.clone();
    let mut world = AirWorld { room, fault: None };
    let events = beam::update(
        &mut staged,
        input,
        &content.model,
        beam::Rules::REFERENCE,
        &mut world,
    );
    if let Some(fault) = world.fault {
        return Err(fault);
    }
    let mut requests = FrameRequests::default();
    for event in events {
        match event {
            beam::Event::PendingAimConsumed => requests.pending_aim_consumed = true,
            beam::Event::FireSound(_)
            | beam::Event::ImpactSound(_)
            | beam::Event::Protection(_) => {
                // Sound has no U1 consumer in this no-APU path. Self-shot protection readers
                // are inactive for the owned negative Power/inactive Impact bodies on both
                // protection branches; each request is explicitly disposed here.
            }
            beam::Event::Trail {
                position,
                appearance,
                ..
            } => {
                let appearance = appearance.as_ref().ok_or(Fault::Trail)?;
                let offsets = content.trail(appearance)?;
                match content.assets.power.trail_presentation {
                    asset::TrailPresentation::EndsImmediately => {}
                }
                requests.trails.push(TrailRequest { position, offsets });
            }
        }
    }
    if requests.trails.len() > 5 {
        return Err(Fault::Trail);
    }
    owner.model.0 = staged;
    Ok(requests)
}

/// Revalidate the bounded typed owner against current equipment, selected HUD state and actors.
/// # Errors
/// An unsupported producer or callback-bearing actor cannot remain on the ordinary path.
pub fn validate_path(
    state: &crate::state::State,
    content: &crate::pose::Content,
) -> Result<(), Fault> {
    use crate::state::strand::{Frame, MainLoopAt};
    let Some(owner) = state.projectiles.ordinary() else {
        return Ok(());
    };
    let power = content.ordinary_power.as_ref().ok_or(Fault::Content)?;
    power.validate_state(&owner.model.0)?;
    if !matches!(
        state.main.frames.as_slice(),
        [Frame::MainLoop(
            MainLoopAt::Start | MainLoopAt::AfterHdmaObjects | MainLoopAt::AfterGameState
        )]
    ) {
        return Err(Fault::Applicability);
    }
    let area = usize::from(state.room.width.0)
        .checked_mul(usize::from(state.room.height.0))
        .ok_or(Fault::World)?;
    if area == 0 || area != state.room.blocks.len() {
        return Err(Fault::World);
    }
    if state.room.blocks.iter().any(|block| {
        !matches!(
            block.kind,
            data::level::BlockKind::Air(_)
                | data::level::BlockKind::Slope(_)
                | data::level::BlockKind::Solid(_)
                | data::level::BlockKind::Door(_)
        )
    }) {
        return Err(Fault::Applicability);
    }

    if state.hud.selected.is_some()
        || state.hud.auto_cancel.is_some()
        || state.samus.equipment.equipped != crate::state::samus::Items::default()
        || state.samus.drawing != crate::state::samus::DrawingHandler::Default
    {
        return Err(Fault::Applicability);
    }
    for slot in &state.room.plms.slots {
        if !matches!(slot.occupancy, crate::state::slots::Occupancy::Free) {
            return Err(Fault::Applicability);
        }
    }
    for slot in &state.room.enemies.slots {
        use crate::state::room::{AiHandler, EnemyBody, EnemyExtraProperty};
        use crate::state::slots::Occupancy;
        match slot.occupancy {
            Occupancy::Free | Occupancy::Deleting(_) => {}
            Occupancy::Live(EnemyBody::CeresDoor(door)) => {
                let expected = crate::boss_ceres_ridley_bridge::kind(slot.common.parameter_1)
                    .ok_or(Fault::Applicability)?;
                if (door.0.kind, door.0.facing) != expected
                    || slot.common.ai_handlers
                        != crate::state::slots::FlagSet::<AiHandler>::default()
                    || slot.common.frozen != crate::state::slots::Frames(0)
                    || slot
                        .common
                        .extra_properties
                        .contains(EnemyExtraProperty::Paralysed)
                    || slot
                        .common
                        .extra_properties
                        .contains(EnemyExtraProperty::ExtendedSpritemap)
                {
                    return Err(Fault::Applicability);
                }
            }
            Occupancy::Live(
                EnemyBody::MorphBallEye(_)
                | EnemyBody::Elevator(_)
                | EnemyBody::FaceBlock(_)
                | EnemyBody::Unmodelled(_),
            ) => return Err(Fault::Applicability),
        }
    }
    Ok(())
}

/// Transition eligibility follows the named pose action, including every aimed variant.
const fn transition_eligible(action: types::Action) -> Result<bool, Fault> {
    match action {
        types::Action::Crouching | types::Action::StandingUp => Ok(true),
        types::Action::Morphing | types::Action::Unmorphing => Ok(false),
        types::Action::AgainstWall
        | types::Action::Falling
        | types::Action::Jumping
        | types::Action::Landing(_)
        | types::Action::Moonwalking
        | types::Action::Rolling
        | types::Action::Running
        | types::Action::Spinning
        | types::Action::Still
        | types::Action::TakingOff
        | types::Action::Turning(_)
        | types::Action::WallJumping => Err(Fault::Content),
    }
}
const fn firing_heading(value: types::FiringDirection) -> beam::Heading {
    match value {
        types::FiringDirection::UpFacingRight => beam::Heading::UpFacingRight,
        types::FiringDirection::UpRight => beam::Heading::UpRight,
        types::FiringDirection::Right => beam::Heading::Right,
        types::FiringDirection::DownRight => beam::Heading::DownRight,
        types::FiringDirection::DownFacingRight => beam::Heading::DownFacingRight,
        types::FiringDirection::DownFacingLeft => beam::Heading::DownFacingLeft,
        types::FiringDirection::DownLeft => beam::Heading::DownLeft,
        types::FiringDirection::Left => beam::Heading::Left,
        types::FiringDirection::UpLeft => beam::Heading::UpLeft,
        types::FiringDirection::UpFacingLeft => beam::Heading::UpFacingLeft,
    }
}

/// Supply W1 from the ordinary caller's owned neighbouring receipts.
/// # Errors
/// Missing muzzle content or a caller family beyond the admitted ordinary continuation.
pub fn frame_input(
    state: &crate::state::State,
    content: &PowerContent,
    locked: bool,
) -> Result<beam::UpdateInput, Fault> {
    use data::pose::MovementType;
    use data::pose::transition::Control;
    if state.room.width.0 == 0
        || state.room.height.0 == 0
        || !state.room.width.0.is_multiple_of(16)
        || !state.room.height.0.is_multiple_of(16)
    {
        return Err(Fault::World);
    }
    let pending_aim = state.pending_aim.0.map(firing_heading);
    let pose = map::MAP
        .iter()
        .find(|entry| entry.id == state.samus.pose)
        .ok_or(Fault::Muzzle)?
        .key;
    let eligible = match state.samus.movement {
        MovementType::Standing
        | MovementType::Running
        | MovementType::NormalJumping
        | MovementType::Crouching
        | MovementType::Falling
        | MovementType::Moonwalking
        | MovementType::RanIntoWall => true,
        MovementType::TurningAroundOnGround
        | MovementType::TurningAroundJumping
        | MovementType::TurningAroundFalling => pending_aim.is_some(),
        MovementType::CrouchingTransition => transition_eligible(pose.action)?,
        MovementType::SpinJumping
        | MovementType::WallJumping
        | MovementType::KnockbackOrCrystalFlashEnding
        | MovementType::DamageBoost
        | MovementType::MorphBallOnGround
        | MovementType::MorphBallFalling
        | MovementType::SpringBallOnGround
        | MovementType::SpringBallInAir
        | MovementType::SpringBallFalling => false,
        MovementType::Unused07
        | MovementType::Unused09
        | MovementType::Unused0B
        | MovementType::Unused0C
        | MovementType::Unused0D
        | MovementType::Grappling
        | MovementType::GrabbedByDraygon
        | MovementType::Shinespark => {
            return Err(Fault::Applicability);
        }
    };
    let distance = |direction| {
        let [low, high, whole_low, whole_high] =
            state.movement_travel.0.travels[direction].0.0.to_le_bytes();
        beam::MovementDistance {
            whole: u16::from_le_bytes([whole_low, whole_high]),
            fraction: beam::PixelFraction(u16::from_le_bytes([low, high])),
        }
    };
    let shoot = |buttons| crate::pose::controls(buttons).contains(Control::Shoot);
    Ok(beam::UpdateInput {
        mode: if locked || state.samus.facing == data::pose::PoseFacing::Forward {
            beam::UpdateMode::ShotsOnly
        } else {
            beam::UpdateMode::CannonAndShots
        },
        // The seed capability excludes time-stop creators and is revalidated before PRE.
        time_stopped: false,
        cannon: beam::CannonInput {
            shoot: eligible && shoot(state.controller.held),
            new_press: shoot(state.controller.pressed),
            previous_new_press: shoot(state.controller.prior_pressed),
            muzzle: content.muzzle(state.samus.pose)?,
            previous_muzzle: content.muzzle(state.samus.previous.pose)?.aim,
            moving_muzzle: state.samus.movement == MovementType::Running
                || (pose.action == types::Action::Moonwalking && pose.aim == types::Aim::UpForward),
            pending_aim,
            centre: beam::SamusCentre(beam::Axes {
                x: state.samus.motion.position.x.pixel(),
                y: state.samus.motion.position.y.pixel(),
            }),
        },
        inheritance: beam::Inheritance {
            left: distance(types::Direction::Left),
            right: distance(types::Direction::Right),
            up: distance(types::Direction::Up),
            down: distance(types::Direction::Down),
            camera_vertical_fraction: beam::PixelFraction(state.camera.0.vertical_fraction().0),
        },
        camera: beam::ScreenOrigin(beam::Axes {
            x: state.room.camera.x.pixels,
            y: state.room.camera.y.pixels,
        }),
        room: beam::RoomScreens(beam::Axes {
            x: state.room.width.0 / 16,
            y: state.room.height.0 / 16,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::projectile::{OrdinaryPower, ProjectileOwner};
    use crate::state::strand::{Frame, GameplayAt, MainLoopAt};
    use canonical::{from_bytes, to_bytes};

    enum InvalidBody {
        Clip,
        Cursor,
        /// Unrecognized picture identity.
        Picture,
        /// Unsupported trail appearance or undelivered request.
        Trail,
        Count,
    }

    fn content() -> crate::pose::Content {
        let frame = asset::Frame {
            hold: asset::Hold::try_from(1).unwrap(),
            picture: data::assets::samus::SpriteId(0),
            radius: asset::Radius { x: 1, y: 1 },
            trail_appearance: asset::TrailAppearanceId(0),
        };
        let clip = asset::Clip {
            frames: vec![frame],
            end: asset::ClipEnd::Remove,
        };
        let flight = asset::Headings {
            up_facing_right: clip.clone(),
            up_right: clip.clone(),
            right: clip.clone(),
            down_right: clip.clone(),
            down_facing_right: clip.clone(),
            down_facing_left: clip.clone(),
            down_left: clip.clone(),
            left: clip.clone(),
            up_left: clip.clone(),
            up_facing_left: clip.clone(),
        };
        let assets = asset::Content {
            power: asset::PowerBeam {
                sheet: data::assets::samus::SpriteSheet {
                    sprites: vec![data::spritemap::Spritemap::default()],
                    ..Default::default()
                },
                flight,
                impact: clip,
                damage: asset::Damage::try_from(1).unwrap(),
                trails: vec![asset::TrailAppearance::Inactive],
                trail_presentation: asset::TrailPresentation::EndsImmediately,
            },
            muzzles: vec![],
        };
        crate::pose::Content {
            ordinary_power: Some(PowerContent::new(assets).unwrap()),
            ..Default::default()
        }
    }

    fn seed_facts() -> crate::state::projectile::PowerSeedFacts {
        crate::state::projectile::PowerSeedFacts {
            beams: crate::state::slots::FlagSet::default(),
            charge: crate::state::projectile::ChargeEquipment::Unequipped,
            charge_history: (
                crate::state::slots::Frames(0),
                crate::state::slots::Frames(0),
            ),
            hyper: crate::state::projectile::HyperActivity::Inactive,
            grapple: crate::state::projectile::GrappleActivity::Inactive,
            time: crate::state::projectile::PowerTime::Running,
            sprite_objects: crate::state::projectile::ProducerPool::Empty,
            enemy_projectiles: crate::state::projectile::ProducerPool::Empty,
            trail_effects: crate::state::projectile::ProducerPool::Empty,
            other_player_slots: crate::state::projectile::ProducerPool::Empty,
        }
    }

    fn state() -> crate::state::State {
        let mut state = crate::state::State::default();
        state.main.frames = vec![Frame::MainLoop(MainLoopAt::Start)];
        state.room.width.0 = 1;
        state.room.height.0 = 1;
        state.room.blocks = vec![data::level::Block::default()];
        // Invented fixture: the codec accepts typed values; Session reopening checks applicability.
        state.projectiles = ProjectileOwner::OrdinaryPower(
            OrdinaryPower::from_seed(
                beam::State::reset(EnumMap::from_fn(|_| beam::Axes {
                    x: beam::PixelFraction(0),
                    y: beam::PixelFraction(0),
                })),
                seed_facts(),
                content().ordinary_power.as_ref().unwrap(),
            )
            .unwrap(),
        );
        state
    }

    fn reopen(
        state: &crate::state::State,
        content: crate::pose::Content,
    ) -> Result<crate::Session, crate::session::OpenError> {
        let restored = from_bytes(&to_bytes(state)).unwrap();
        crate::Session::open(
            restored,
            crate::Rules {
                auto_repeat: crate::AutoRepeat {
                    initial: crate::state::slots::Frames(1),
                    subsequent: crate::state::slots::Frames(1),
                },
                switches: crate::session::Switches::REFERENCE,
            },
            std::sync::Arc::new(content),
        )
    }

    #[test]
    fn canonical_reopening_rejects_suspended_caller_equipment_and_hidden_actor() {
        let valid = state();
        assert!(reopen(&valid, content()).is_ok());
        let mut suspended = valid.clone();
        suspended
            .main
            .frames
            .push(Frame::Gameplay(GameplayAt::Start));
        assert!(reopen(&suspended, content()).is_err());
        let mut equipped = valid.clone();
        equipped
            .samus
            .equipment
            .equipped
            .insert(crate::state::samus::Item::Bombs);
        assert!(reopen(&equipped, content()).is_err());
        let mut actor = valid.clone();
        actor.room.enemies.slots[0].occupancy = crate::state::slots::Occupancy::Live(
            crate::state::room::EnemyBody::Unmodelled(crate::state::room::UnmodelledEnemy {
                species: crate::state::room::UnmodelledSpecies::CeresSteam,
                instruction: 0,
                spritemap: 0,
                variables: [0; 6],
            }),
        );
        assert!(reopen(&actor, content()).is_err());
        assert!(reopen(&valid, crate::pose::Content::default()).is_err());
    }

    #[test]
    fn canonical_reopening_rejects_invalid_body_references_and_zero_count_flying() {
        let mut valid = state();
        let owner = match &mut valid.projectiles {
            ProjectileOwner::OrdinaryPower(owner) => Some(owner),
            ProjectileOwner::Legacy(_) => None,
        }
        .unwrap();
        owner.model.0.count = beam::ShotCount(1);
        let body = beam::ShotBody {
            position: beam::Axes {
                x: beam::FixedPosition(0),
                y: beam::FixedPosition(0),
            },
            heading: beam::Heading::Right,
            speed: beam::Axes {
                x: beam::BeamSpeed(0),
                y: beam::BeamSpeed(0),
            },
            radius: beam::Axes {
                x: beam::Radius(1),
                y: beam::Radius(1),
            },
            damage: beam::Damage::new(1).unwrap(),
            trail: beam::FrameWord(0),
            clip: clip_id(asset::ClipId::Flight(asset::Heading::Right)),
            next_frame: beam::ClipFrame::new(0, 1).unwrap(),
            remaining_hold: beam::FrameWord(1),
            picture: Some(picture(0)),
            trail_appearance: None,
        };
        owner.model.0.slots[beam::ShotSlot::Slot0] = beam::ShotState::Flying {
            body,
            consumed: false,
        };
        assert!(reopen(&valid, content()).is_ok());

        for change in [
            InvalidBody::Clip,
            InvalidBody::Cursor,
            InvalidBody::Picture,
            InvalidBody::Trail,
            InvalidBody::Count,
        ] {
            let mut invalid = valid.clone();
            let owner = match &mut invalid.projectiles {
                ProjectileOwner::OrdinaryPower(owner) => Some(owner),
                ProjectileOwner::Legacy(_) => None,
            }
            .unwrap();
            let body = match &mut owner.model.0.slots[beam::ShotSlot::Slot0] {
                beam::ShotState::Flying { body, .. } => Some(body),
                beam::ShotState::Free { .. } | beam::ShotState::Impact { .. } => None,
            }
            .unwrap();
            match change {
                InvalidBody::Clip => body.clip = beam::ClipId("absent".into()),
                InvalidBody::Cursor => body.next_frame = beam::ClipFrame::new(2, 2).unwrap(),
                InvalidBody::Picture => body.picture = Some(beam::PictureId("absent".into())),
                InvalidBody::Trail => {
                    body.trail_appearance = Some(beam::TrailAppearanceId("absent".into()));
                }
                InvalidBody::Count => owner.model.0.count = beam::ShotCount(0),
            }
            assert!(reopen(&invalid, content()).is_err());
        }
    }
    #[test]
    fn seed_rejects_each_occupied_competing_producer_pool() {
        use crate::state::projectile::{OrdinaryCapability, ProducerPool};
        assert!(OrdinaryCapability::from_seed(seed_facts()).is_ok());
        let mut pools = [seed_facts(); 4];
        pools[0].sprite_objects = ProducerPool::Occupied;
        pools[1].enemy_projectiles = ProducerPool::Occupied;
        pools[2].trail_effects = ProducerPool::Occupied;
        pools[3].other_player_slots = ProducerPool::Occupied;
        for facts in pools {
            assert_eq!(
                OrdinaryCapability::from_seed(facts),
                Err(Fault::Applicability)
            );
        }
    }
    #[test]
    fn crouch_stand_and_aim_transitions_allow_shoot_while_morph_transitions_suppress_it() {
        for action in [types::Action::Crouching, types::Action::StandingUp] {
            assert_eq!(transition_eligible(action), Ok(true));
        }
        for action in [types::Action::Morphing, types::Action::Unmorphing] {
            assert_eq!(transition_eligible(action), Ok(false));
        }
    }
    #[test]
    fn complete_power_state_shape_is_pinned() {
        assert_eq!(
            canonical::model::pin_holds::<crate::state::projectile::PowerPin>(),
            Ok(())
        );
    }
    #[test]
    fn ordinary_pending_passage_reopens_and_refuses_dispatch_with_complete_rollback() {
        let mut pending = state();
        pending.room.room = data::id::RoomId(1);
        pending.mode = crate::state::GameMode::PassageEntry {
            doorway: data::id::DoorwayId(1),
        };
        pending.main.waiting = Some(crate::state::strand::Wait::Nmi);
        let mut tables = content();
        let registry = crate::selected_doorway::tests::registered_content();
        tables.world = registry.world;
        tables.room_doors = registry.room_doors;
        let mut session = reopen(&pending, tables).unwrap();
        assert_eq!(
            session
                .step(crate::TickInput {
                    pad: crate::state::Buttons::default(),
                    timing: crate::state::NmiTiming::OnTime,
                })
                .err(),
            Some(crate::SessionFault::Passage(
                crate::selected_doorway::PassageFault::MissingRuntime
            ))
        );
        assert_eq!(session.state(), &pending);
    }
}
