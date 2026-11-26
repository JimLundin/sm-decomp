//! The scoped cinematic weapon recipe. Room play offers the encounter callback between the
//! shared work and drawing entries. Publication can finish on a later cut.

use super::{ProductionFault, Scope, Unsupported, played::intro_missiles, weapons::QuietBeamWorld};
use crate::{
    state::{
        owned::PowerPin,
        video::{NmiBatch, Oam},
    },
    weapons_projectiles_bridge::Fault,
};
use canonical::{Canonical, model::Model};
use weapons_projectiles as w1;

/// Complete cinematic weapon State and any sprite batch waiting for a completed cut.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The five weapon slots and exhaust pool.
    pub weapons: Model<PowerPin>,
    /// Missile supply, selection and incoming-shot protection.
    pub supplies: intro_missiles::Supplies,
    /// The exact local OAM produced by both visits, while publication is awaited.
    pub pending: Option<Oam>,
}

/// How the drawing opportunity fits around this cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presentation {
    /// There is no new drawing or publication.
    Hold,
    /// Take both weapon visits. Their OAM will publish on a later cut.
    Prepare(w1::ScreenOrigin),
    /// Draw and publish at this cut.
    DrawAndPublish(w1::ScreenOrigin),
    /// Publish already prepared OAM at a completed cut.
    Publish,
    /// This completed cut carries a closed draw gate.
    Hidden,
}

/// An authorized weapons work turn and its external encounter and terrain.
pub struct Turn<'a> {
    /// Pose, control and scene inputs at the cannon/shot work phase.
    pub update: &'a w1::UpdateInput,
    /// Cannon eligibility after the demo's action.
    pub eligible: bool,
    /// The caller's checked block grid.
    pub world: &'a QuietBeamWorld,
    /// The encounter's decision on this turn's candidates.
    pub contact: &'a dyn Fn(&[w1::MissileCandidate]) -> Option<w1::ShotSlot>,
}

/// The scoped actor and drawing opportunities of a cut.
pub struct Input<'a> {
    /// Any new actor turn before drawing.
    pub turn: Option<Turn<'a>>,
    /// Drawing and completion opportunities supplied by the scene.
    pub presentation: Presentation,
}

/// Requests delivered before drawing and the sprite publication, when complete.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The weapon turn's reports and candidate shots.
    pub turn: Option<w1::IntroFightUpdate>,
    /// The encounter's requested slot, applied to the complete State.
    pub impact: Option<w1::ShotSlot>,
    /// Number of point contacts at the caller's grid.
    pub point_contacts: usize,
    /// Local exact OAM on a completed drawing cut.
    pub sprites: Option<Oam>,
}

struct Terrain<'a> {
    grid: &'a QuietBeamWorld,
    fault: Option<ProductionFault>,
    point_contacts: usize,
}

impl w1::World for Terrain<'_> {
    fn contact_cell(&mut self, _: &w1::Contact, _: u16) -> w1::ContactResponse {
        self.fault
            .get_or_insert(ProductionFault::Unsupported(Unsupported::BeamContinuation));
        w1::ContactResponse::Passes
    }

    fn contact_point(&mut self, _: w1::ShotSlot, centre: w1::Axes<u16>) -> w1::ContactResponse {
        self.point_contacts = self.point_contacts.saturating_add(1);
        match self.grid.missile_point(centre) {
            Ok(response) => response,
            Err(fault) => {
                self.fault.get_or_insert(fault);
                w1::ContactResponse::Passes
            }
        }
    }
}

pub(super) fn advance(
    state: &mut State,
    input: Input<'_>,
    content: &intro_missiles::Content,
    rules: w1::Rules,
) -> Result<Output, ProductionFault> {
    let mut output = Output {
        turn: None,
        impact: None,
        point_contacts: 0,
        sprites: None,
    };
    if let Some(turn) = input.turn {
        let mut terrain = Terrain {
            grid: turn.world,
            fault: None,
            point_contacts: 0,
        };
        let result = intro_missiles::update(
            &mut state.weapons.0,
            turn.update,
            &mut state.supplies,
            turn.eligible,
            content,
            rules,
            &mut terrain,
        )
        .map_err(ProductionFault::Weapons)?;
        if let Some(fault) = terrain.fault {
            return Err(fault);
        }
        output.impact = (turn.contact)(&result.candidates);
        if let Some(slot) = output.impact
            && !intro_missiles::hit(&mut state.weapons.0, slot, content)
        {
            return Err(ProductionFault::Weapons(Fault::Applicability));
        }
        output.point_contacts = terrain.point_contacts;
        output.turn = Some(result);
    }
    output.sprites = match input.presentation {
        Presentation::Hold => None,
        Presentation::Prepare(view) => {
            state.pending = Some(prepare(state, view, content)?);
            None
        }
        Presentation::DrawAndPublish(view) => {
            state.pending = None;
            Some(prepare(state, view, content)?)
        }
        Presentation::Hidden => {
            state.pending = None;
            let mut batch = NmiBatch::default();
            batch.finalise_oam();
            Some(batch.oam)
        }
        Presentation::Publish => Some(state.pending.take().ok_or(ProductionFault::Unsupported(
            Unsupported::MissingInput(Scope::IntroMissiles),
        ))?),
    };
    Ok(output)
}

fn prepare(
    state: &mut State,
    view: w1::ScreenOrigin,
    content: &intro_missiles::Content,
) -> Result<Oam, ProductionFault> {
    let visits = intro_missiles::draw_twice(&mut state.weapons.0, view, content)
        .map_err(ProductionFault::Weapons)?;
    let mut batch = NmiBatch::default();
    for visit in &visits {
        batch.draw_sprites(visit);
    }
    batch.finalise_oam();
    Ok(batch.oam)
}
