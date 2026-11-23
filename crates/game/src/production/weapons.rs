//! The bounded quiet-world weapons update and its ordered late door contacts.

use super::{ProductionFault, Unsupported};
use crate::{
    state::video::{NmiBatch, OAM_SPRITES},
    weapons_projectiles_bridge::PowerContent,
};
use canonical::Canonical;
use serde::{Deserialize, Serialize};
use weapons_projectiles as beam;

const BLOCK_PIXELS: u16 = 16;
const BLOCKS_PER_SCREEN: u16 = 16;
const MAX_GRID_DIMENSION: u16 = 255;
const MAX_CONTACT_CELLS: u16 = 16;
const MAX_SAFE_RADIUS: u8 = 120;
const SPEED_UNITS_PER_PIXEL: u32 = 256;
const MAX_SPEED_MAGNITUDE: u32 = 32768;

/// Physical raw actor slot identity, independent of list position or geometry.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorSlotIdentity(pub u8);

/// One causal opportunity in the actual actor processing order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoorOpportunity {
    /// Occurrence in the supplied processing receipt, including repeated actors.
    pub occurrence: ActorOccurrence,
    /// Physical identity of the current receiver.
    pub actor: ActorSlotIdentity,
    /// Current independently admitted actor and interaction permissions.
    pub conditions: enemy_interactions::ContactConditions,
    /// Current single door figure; no door response is executed by this scope.
    pub door: enemy_interactions::DoorContact,
}

/// An occurrence is distinct from a reusable actor slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorOccurrence(pub usize);

/// The complete returned consumption receipt for one supplied opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoorReceipt {
    /// The exact opportunity consumed, in supplied order.
    pub opportunity: DoorOpportunity,
    /// The selected shot, retaining its body after marking consumption.
    pub consumed: Option<beam::ShotSlot>,
}

/// Complete publications from one weapons/contact advance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeaponsOutputs {
    /// All sound, protection, pending-aim and trail requests, in returned order.
    pub events: Vec<beam::Event>,
    /// Every supplied actor opportunity, including misses and refused contacts.
    pub contacts: Vec<DoorReceipt>,
}

/// Actual inputs at one bounded weapon update and contact opportunity.
#[derive(Clone, Copy)]
pub struct Opportunity<'a> {
    /// Current cannon, motion, camera and room inputs.
    pub input: &'a beam::UpdateInput,
    /// Complete checked terrain.
    pub world: &'a QuietBeamWorld,
    /// Current contact opportunities in processing order.
    pub contacts: &'a [DoorOpportunity],
}

/// Inputs at the flying-picture insertion point.
#[derive(Clone, Copy)]
pub struct FlyingDraw<'a> {
    /// Current presentation phase from the drawing caller.
    pub phase: beam::FlickerPhase,
    /// Current drawing coordinates.
    pub view: &'a beam::DrawView,
    /// The batch already populated by earlier drawing producers.
    pub batch: &'a NmiBatch,
}

/// Publications from the selected weapon, contact and drawing owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShotDrawingOutputs {
    /// All update and contact publications.
    pub weapons: WeaponsOutputs,
    /// Reached flying intents in production order.
    pub pictures: Vec<beam::PictureIntent>,
    /// Incoming index of the exact insertion.
    pub first_sprite: u16,
    /// Batch after exact flight-picture insertion.
    pub batch: NmiBatch,
}

/// A checked complete immutable rectangle of independently admitted terrain.
/// Ordinary air and solid callbacks are supported; a reached other kind is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuietBeamWorld {
    width: u16,
    height: u16,
    cells: Vec<data::level::BlockKind>,
}

impl QuietBeamWorld {
    /// Admit complete independently imported terrain for the bounded quiet recipe.
    ///
    /// # Errors
    /// Missing geometry and dimensions outside the supported rectangle.
    pub fn new(
        width: u16,
        height: u16,
        cells: &[data::level::BlockKind],
    ) -> Result<Self, ProductionFault> {
        let expected = usize::from(width).checked_mul(usize::from(height));
        if width == 0
            || height == 0
            || width > MAX_GRID_DIMENSION
            || height > MAX_GRID_DIMENSION
            || expected != Some(cells.len())
        {
            return Err(unsupported(Unsupported::BeamWorld));
        }
        Ok(Self {
            width,
            height,
            cells: cells.to_vec(),
        })
    }

    fn response(&self, row: u16, column: u16) -> Result<beam::ContactResponse, ProductionFault> {
        let cell = usize::from(row)
            .checked_mul(usize::from(self.width))
            .and_then(|start| start.checked_add(usize::from(column)))
            .and_then(|index| self.cells.get(index))
            .ok_or_else(|| unsupported(Unsupported::BeamContinuation))?;
        match cell {
            data::level::BlockKind::Air(_) => Ok(beam::ContactResponse::Passes),
            data::level::BlockKind::Solid(_) => Ok(beam::ContactResponse::Blocks),
            data::level::BlockKind::Slope(_)
            | data::level::BlockKind::SpikeAir(_)
            | data::level::BlockKind::SpecialAir(_)
            | data::level::BlockKind::ShootableAir(_)
            | data::level::BlockKind::HorizontalExtension(_)
            | data::level::BlockKind::UnusedAir(_)
            | data::level::BlockKind::BombableAir(_)
            | data::level::BlockKind::Door(_)
            | data::level::BlockKind::Spike(_)
            | data::level::BlockKind::Special(_)
            | data::level::BlockKind::Shootable(_)
            | data::level::BlockKind::VerticalExtension(_)
            | data::level::BlockKind::Grapple(_)
            | data::level::BlockKind::Bombable(_) => {
                Err(unsupported(Unsupported::ReactiveBeamWorld))
            }
        }
    }

    fn validate_room(&self, room: beam::RoomScreens) -> Result<(), ProductionFault> {
        if room.0.x.checked_mul(BLOCKS_PER_SCREEN) != Some(self.width)
            || room.0.y.checked_mul(BLOCKS_PER_SCREEN) != Some(self.height)
        {
            return Err(unsupported(Unsupported::BeamWorld));
        }
        Ok(())
    }

    fn envelope(
        &self,
        centre: beam::Axes<u16>,
        radius: beam::Axes<beam::Radius>,
        speed: beam::Axes<beam::BeamSpeed>,
        rules: beam::Rules,
    ) -> Result<(), ProductionFault> {
        // With arbitrary cell alignment, half-size <=120 touches at most sixteen cells.
        if radius.x.0 > MAX_SAFE_RADIUS || radius.y.0 > MAX_SAFE_RADIUS {
            return Err(unsupported(Unsupported::BeamSpan));
        }
        for (position, radius, speed, blocks) in [
            (centre.x, radius.x, speed.x, self.width),
            (centre.y, radius.y, speed.y, self.height),
        ] {
            // Signed speed units are 1/256 pixel. The envelope bounds a possible acceleration
            // and fractional carry, including wrapping acceleration; it never predicts motion.
            let magnitude = u32::from(speed.0.unsigned_abs())
                .saturating_add(u32::from(rules.acceleration.0.unsigned_abs()));
            let displacement = magnitude
                .min(MAX_SPEED_MAGNITUDE)
                .div_ceil(SPEED_UNITS_PER_PIXEL)
                .saturating_add(1);
            let margin = u32::from(radius.0).saturating_add(displacement).max(1);
            let position = u32::from(position);
            let end = u32::from(blocks)
                .checked_mul(u32::from(BLOCK_PIXELS))
                .ok_or_else(|| unsupported(Unsupported::BeamWorld))?;
            if position.checked_sub(margin).is_none()
                || position.checked_add(margin).is_none_or(|high| high >= end)
            {
                return Err(unsupported(Unsupported::BeamContinuation));
            }
        }
        Ok(())
    }
}

const fn unsupported(reason: Unsupported) -> ProductionFault {
    ProductionFault::Unsupported(reason)
}

fn largest_radius(content: &PowerContent) -> beam::Axes<beam::Radius> {
    content
        .model()
        .clips
        .values()
        .flat_map(|clip| &clip.frames)
        .fold(
            beam::Axes {
                x: beam::Radius(0),
                y: beam::Radius(0),
            },
            |largest, frame| beam::Axes {
                x: beam::Radius(largest.x.0.max(frame.radius.x.0)),
                y: beam::Radius(largest.y.0.max(frame.radius.y.0)),
            },
        )
}

fn validate_geometry(
    state: &beam::State,
    input: &beam::UpdateInput,
    content: &PowerContent,
    rules: beam::Rules,
    world: &QuietBeamWorld,
) -> Result<(), ProductionFault> {
    world.validate_room(input.room)?;
    let largest = largest_radius(content);
    for shot in state.slots.values() {
        if let Some(body) = shot.body() {
            let radius = beam::Axes {
                x: beam::Radius(largest.x.0.max(body.radius.x.0)),
                y: beam::Radius(largest.y.0.max(body.radius.y.0)),
            };
            world.envelope(
                beam::Axes {
                    x: body.position.x.integer(),
                    y: body.position.y.integer(),
                },
                radius,
                body.speed,
                rules,
            )?;
        }
    }
    if input.mode == beam::UpdateMode::CannonAndShots
        && !input.time_stopped
        && input.cannon.shoot
        && let Some((heading, centre)) = beam::cannon_origin(&input.cannon)
    {
        world.envelope(
            centre.0,
            largest,
            beam::inherited_motion(heading, input.inheritance),
            rules,
        )?;
    }
    Ok(())
}

struct CandidateWorld<'a> {
    geometry: &'a QuietBeamWorld,
    fault: Option<ProductionFault>,
}

impl beam::World for CandidateWorld<'_> {
    fn contact_cell(&mut self, contact: &beam::Contact, ordinal: u16) -> beam::ContactResponse {
        let start = contact.perpendicular_start >> 4;
        let perpendicular = start.checked_add(ordinal);
        let along = contact.leading_boundary >> 4;
        let (row, column) = match contact.axis {
            beam::Axis::Horizontal => (perpendicular, Some(along)),
            beam::Axis::Vertical => (Some(along), perpendicular),
        };
        if ordinal >= contact.cell_count
            || contact.cell_count > MAX_CONTACT_CELLS
            || row.is_none_or(|row| row >= self.geometry.height)
            || column.is_none_or(|column| column >= self.geometry.width)
        {
            self.fault
                .get_or_insert_with(|| unsupported(Unsupported::BeamContinuation));
            return beam::ContactResponse::Passes;
        }
        let Some((row, column)) = row.zip(column) else {
            self.fault
                .get_or_insert_with(|| unsupported(Unsupported::BeamContinuation));
            return beam::ContactResponse::Passes;
        };
        match self.geometry.response(row, column) {
            Ok(response) => response,
            Err(fault) => {
                self.fault.get_or_insert(fault);
                // The model finishes only the discarded candidate; no unsupported response escapes.
                beam::ContactResponse::Passes
            }
        }
    }
}

pub(super) fn advance(
    state: &mut beam::State,
    input: &beam::UpdateInput,
    content: &PowerContent,
    rules: beam::Rules,
    world: &QuietBeamWorld,
    opportunities: &[DoorOpportunity],
) -> Result<WeaponsOutputs, ProductionFault> {
    let events = update(state, input, content, rules, world)?;
    let contacts = opportunities
        .iter()
        .map(|&opportunity| contact(state, opportunity))
        .collect();
    Ok(WeaponsOutputs { events, contacts })
}

pub(super) fn update(
    state: &mut beam::State,
    input: &beam::UpdateInput,
    content: &PowerContent,
    rules: beam::Rules,
    world: &QuietBeamWorld,
) -> Result<Vec<beam::Event>, ProductionFault> {
    content
        .validate_state(state)
        .map_err(ProductionFault::Weapons)?;
    validate_geometry(state, input, content, rules, world)?;
    let mut candidate_world = CandidateWorld {
        geometry: world,
        fault: None,
    };
    let events = beam::update(state, input, content.model(), rules, &mut candidate_world);
    if let Some(fault) = candidate_world.fault {
        return Err(fault);
    }
    // Check every retained body even when a span/room guard suppressed all callbacks.
    content
        .validate_state(state)
        .map_err(ProductionFault::Weapons)?;
    validate_geometry(state, input, content, rules, world)?;
    Ok(events)
}

pub(super) fn contact(state: &mut beam::State, opportunity: DoorOpportunity) -> DoorReceipt {
    DoorReceipt {
        opportunity,
        consumed: crate::enemy_interactions_bridge::contact_door(
            state,
            opportunity.conditions,
            opportunity.door,
        ),
    }
}

pub(super) fn advance_with_drawing(
    state: &mut beam::State,
    shot: Opportunity<'_>,
    content: &PowerContent,
    rules: beam::Rules,
    drawing: FlyingDraw<'_>,
) -> Result<ShotDrawingOutputs, ProductionFault> {
    let weapons = advance(state, shot.input, content, rules, shot.world, shot.contacts)?;
    draw_flying(state, weapons, content, drawing)
}

pub(super) fn draw_flying(
    state: &beam::State,
    weapons: WeaponsOutputs,
    content: &PowerContent,
    drawing: FlyingDraw<'_>,
) -> Result<ShotDrawingOutputs, ProductionFault> {
    let beam::PictureIntents { flying, impact } =
        beam::pictures(state, drawing.phase, drawing.view);
    if !impact.is_empty() {
        return Err(unsupported(Unsupported::BeamDrawing));
    }
    let draw = content.draw(&flying).map_err(ProductionFault::Weapons)?;
    let mut batch = drawing.batch.clone();
    let first_sprite = batch.sprites.0;
    if usize::from(first_sprite)
        .checked_add(draw.pieces.len())
        .is_none_or(|end| end >= OAM_SPRITES)
    {
        return Err(unsupported(Unsupported::BeamDrawing));
    }
    batch.draw_sprites(&draw);
    Ok(ShotDrawingOutputs {
        weapons,
        pictures: flying,
        first_sprite,
        batch,
    })
}
