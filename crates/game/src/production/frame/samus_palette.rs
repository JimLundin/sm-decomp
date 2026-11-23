//! Retained ordinary colour work between actual `PoseFit` and Maintenance.

use super::{
    Caller, Content, Dependency, State, borrowed_player_preparation::SuitPaletteDisposition,
};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::timing::{Profile, Template, TemplateEffect};
use samus_appearance::{
    ColourEffect, ColourSequences, DryColourFacts, DryColourWork, GrappleActivity, RoomColours,
    ScrewAttack, Suit,
};

pub(super) const WORK: timing::WorkId = timing::WorkId(1311);

pub(super) struct SequencesPin;
impl Pinned for SequencesPin {
    type State = ColourSequences;
    const NAME: &'static str = "frame-colour-sequences";
    const SHAPE: u64 = 2_460_331_385_358_529_755;
}
pub(super) struct WorkPin;
impl Pinned for WorkPin {
    type State = DryColourWork;
    const NAME: &'static str = "frame-dry-colour-work";
    const SHAPE: u64 = 7_504_778_159_706_439_500;
}
// Fixed-length encoding preserves the shared clean palette's complete slot order.
// The clean bank's variable-length validation cannot run during canonical schema tracing.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct Banks {
    current: [u16; types::PALETTE_SLOTS],
    target: [u16; types::PALETTE_SLOTS],
}

impl From<types::Palettes> for Banks {
    fn from(banks: types::Palettes) -> Self {
        Self {
            current: banks.current.0.map(types::Colour::bits),
            target: banks.target.0.map(types::Colour::bits),
        }
    }
}

/// Borrowed original room and weapon authorities qualified through the finite prefix.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    origin: [u8; 32],
}

/// Complete independent physical qualification for one ordinary palette call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    origin: Admission,
    profile: Profile,
}

/// Shared palette banks and the separate retained Appearance operation.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    origin: Option<Admission>,
    sequences: Model<SequencesPin>,
    palettes: Banks,
    work: Model<WorkPin>,
    current_bank: data::dma::BusAddress,
    completed: bool,
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::SamusPalettePreparation))
}

impl Palette {
    /// Admit the checked dry standard origin with both shared banks.
    #[must_use]
    pub fn new(
        origin: [u8; 32],
        sequences: ColourSequences,
        palettes: types::Palettes,
        current_bank: data::dma::BusAddress,
    ) -> Self {
        Self {
            origin: Some(Admission { origin }),
            sequences: Model(sequences),
            palettes: palettes.into(),
            work: Model(DryColourWork::UNOBSERVED),
            current_bank,
            completed: false,
        }
    }

    /// Withdraw borrowed authority after any external write or unknown activity.
    pub const fn invalidate(&mut self) {
        self.origin = None;
    }

    /// Current sequence quantities, retained independently of pending work.
    #[must_use]
    pub const fn sequences(&self) -> ColourSequences {
        self.sequences.0
    }

    /// Complete exact current and target banks for callers and diagnostic comparison.
    /// # Errors
    /// Restored palette words must be valid RGB555.
    pub fn palettes(&self) -> Result<types::Palettes, &'static str> {
        let decode = |bank: &[u16]| {
            let colours = bank
                .iter()
                .copied()
                .map(types::Colour::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            types::Palette::try_from(colours)
        };
        Ok(types::Palettes {
            current: decode(&self.palettes.current)?,
            target: decode(&self.palettes.target)?,
        })
    }

    /// Actual completed sequence state, available only after physical return.
    #[must_use]
    pub const fn completed(&self) -> Option<ColourSequences> {
        if self.completed {
            Some(self.sequences.0)
        } else {
            None
        }
    }

    /// Current shared banks may change independently of pending Appearance work.
    pub fn replace_palettes(&mut self, palettes: types::Palettes) {
        self.palettes = palettes.into();
    }

    /// Current bank as a live hardware publication.
    #[must_use]
    pub fn region(&self) -> data::timing::ContentRegion {
        data::timing::ContentRegion {
            start: self.current_bank,
            bytes: self
                .palettes
                .current
                .iter()
                .flat_map(|colour| colour.to_le_bytes())
                .collect(),
        }
    }

    pub(super) const fn retire_opportunity(&mut self) {
        self.work.0 = DryColourWork::UNOBSERVED;
        self.completed = false;
    }

    pub(super) fn holds(&self, state: &State) -> bool {
        let mut active = state
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(WORK));
        let pending = self.work.0.pending().is_some();
        active.next().is_some() == pending
            && active.next().is_none()
            && self.completed == (!pending && self.work.0 != DryColourWork::UNOBSERVED)
            && self
                .palettes
                .current
                .iter()
                .chain(&self.palettes.target)
                .all(|bits| types::Colour::new(*bits).is_some())
    }
}

impl Contract {
    /// Bind a complete source-qualified quiet template to its original image.
    /// # Errors
    /// Invalid templates or hardware effects require another qualified arm.
    pub fn new(origin: [u8; 32], template: &Template) -> Result<Self, ProductionFault> {
        if template
            .runs
            .iter()
            .any(|run| run.effect != TemplateEffect::None)
        {
            return Err(missing());
        }
        Ok(Self {
            origin: Admission { origin },
            profile: template.bind(&[]).map_err(|_| missing())?,
        })
    }
}

/// Facts for a qualified Power-suit dry opportunity in a standard room with a dormant grapple.
/// The screw attack is the frame's switched-on equipment seeded at the origin, not live progress.
fn dry_facts(
    figure: samus_appearance::Figure,
    position: types::Position,
    equipment: &types::Set<types::Item>,
    speed_glow: bool,
) -> DryColourFacts {
    let screw_attack = if equipment[types::Item::ScrewAttack] {
        ScrewAttack::On
    } else {
        ScrewAttack::Off
    };
    let mut effects = types::Set::default();
    effects[ColourEffect::SpeedGlow] = speed_glow;
    DryColourFacts {
        figure,
        position,
        screw_attack,
        suit: Suit::Power,
        room_colours: RoomColours::Standard,
        levels: types::Set::default(),
        grapple: GrappleActivity::Dormant,
        effects,
    }
}

impl State {
    fn palette_facts(&self, content: &Content) -> Result<DryColourFacts, ProductionFault> {
        let contract = content.samus_palette.as_ref().ok_or_else(missing)?;
        let palette = self.samus_palette.as_ref().ok_or_else(missing)?;
        let Caller::PlayerContinuation {
            completed,
            run: super::movement_caller::Run::Complete(run),
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        if palette.origin.as_ref() != Some(&contract.origin)
            || !matches!(
                self.interruption,
                super::interruption_caller::Stage::Complete(_)
            )
        {
            return Err(missing());
        }
        let origin = content
            .liquid_history
            .as_ref()
            .filter(|origin| origin.origin == contract.origin.origin)
            .ok_or_else(missing)?;
        let liquid = self
            .liquid_history
            .as_ref()
            .ok_or_else(missing)?
            .at_animation(origin, completed, run)
            .map_err(|_| missing())?;
        let suit = completed.0.suit_palette.as_ref().ok_or_else(missing)?;
        let environment = completed.0.environment.as_ref().ok_or_else(missing)?;
        let (samus, outcome) = self.movement.finished().ok_or_else(missing)?;
        if liquid != samus_appearance::Liquid::NONE
            || suit.terminal != SuitPaletteDisposition::PowerSuit
            || suit.dependency.equipment != environment.dependency.equipment
            || suit.dependency.equipment[types::Item::VariaSuit]
            || suit.dependency.equipment[types::Item::GravitySuit]
            || self.playing_figure.0.pose != samus.pose
            || self.playing_figure.0.sprite != outcome.sprite
            || content.appearance.get(self.playing_figure.0).is_none()
        {
            return Err(missing());
        }
        let look = &self.appearance.model.0.state;
        let speed_glow = look.stage != types::BoostStage::Stage0
            || look.laps_left != samus_appearance::LapsLeft::WithoutBooster;
        // Both preparations consumed the same switched-on items, checked equal above.
        Ok(dry_facts(
            self.playing_figure.0,
            samus.position,
            &suit.dependency.equipment,
            speed_glow,
        ))
    }

    pub(super) fn advance_palette(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let facts = self.palette_facts(content)?;
        let contract = content.samus_palette.as_ref().ok_or_else(missing)?;
        let palette = self.samus_palette.as_mut().ok_or_else(missing)?;
        if let Some(pending) = palette.work.0.pending() {
            let mut active = self
                .timing
                .work
                .iter()
                .filter(|work| work.kind == timing::WorkKind::Semantic(WORK));
            if *pending != facts
                || !active
                    .next()
                    .is_some_and(|work| work.profile == contract.profile)
                || active.next().is_some()
            {
                return Err(missing());
            }
        } else {
            if !palette.work.0.observe(Some(facts)).map_err(|_| missing())? {
                return Err(missing());
            }
            self.timing
                .begin(WORK, contract.profile.clone())
                .map_err(ProductionFault::FrameHardware)?;
        }
        Ok(())
    }

    pub(super) fn complete_palette(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let facts = self.palette_facts(content)?;
        let palette = self.samus_palette.as_mut().ok_or_else(missing)?;
        if palette.work.0.pending() != Some(&facts)
            || !palette.work.0.complete(&mut palette.sequences.0)
        {
            return Err(missing());
        }
        palette.completed = true;
        Ok(())
    }
}

#[cfg(test)]
mod shape_tests {
    use super::*;

    #[test]
    fn frame_palette_shapes() {
        let shapes = (
            canonical::model::shape::<ColourSequences>(),
            canonical::model::shape::<DryColourWork>(),
        );
        assert_eq!(shapes, (Ok(SequencesPin::SHAPE), Ok(WorkPin::SHAPE)));
    }
}

#[cfg(test)]
mod screw_attack_tests {
    use super::*;
    use samus_appearance::{
        ChargeGlowPosition, Figure, UnsupportedDryColours, VisorDelay, VisorStep,
    };
    use types::{Action, Aim, Body, Facing, Gun, Item, Legs, Pair, Pose, Set, Sprite, ZERO};

    const STALE: ColourSequences = ColourSequences {
        visor_delay: VisorDelay(6),
        visor_step: VisorStep(1),
        charge_glow_position: ChargeGlowPosition(4),
    };

    fn spin_with(screw_attack: bool) -> DryColourFacts {
        let spin = Figure {
            pose: Pose {
                body: Body::Upright,
                action: Action::Spinning,
                facing: Facing::Right,
                aim: Aim::Forward,
                gun: Gun::Stowed,
                legs: Legs::Together,
            },
            sprite: Sprite::Plain,
        };
        let mut equipment = Set::default();
        equipment[Item::ScrewAttack] = screw_attack;
        dry_facts(spin, Pair { x: ZERO, y: ZERO }, &equipment, false)
    }

    #[test]
    fn a_spin_with_the_screw_attack_switched_on_is_refused() {
        let mut work = DryColourWork::UNOBSERVED;
        assert_eq!(
            work.observe(Some(spin_with(true))),
            Err(UnsupportedDryColours)
        );
        assert_eq!(work, DryColourWork::UNOBSERVED);
    }

    #[test]
    fn a_spin_without_the_screw_attack_gets_the_reset() {
        let mut work = DryColourWork::UNOBSERVED;
        assert_eq!(work.observe(Some(spin_with(false))), Ok(true));
        let mut sequences = STALE;
        assert!(work.complete(&mut sequences));
        assert_eq!(
            sequences,
            ColourSequences {
                visor_delay: VisorDelay(1),
                visor_step: VisorStep(3),
                charge_glow_position: ChargeGlowPosition(0),
            }
        );
    }
}
