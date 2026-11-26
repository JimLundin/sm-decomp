//! The production flight to Zebes, from the station blackout to the Landing Site load request.

use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::assets::zebes::{Effect, Zebes};
use screens_zebes::{ColourEffectRequest, FrameAdvance, FrameReport, Rules};
use types::zebes::{ColourEffectRef, SubtitleChoice, ZebesStage, ZebesState};

use crate::intent::SpriteDraw;
use crate::state::video::{Colour, NmiBatch, Oam};
use types::Pair;

/// Canonical representation of the complete clean flight owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePin;

impl Pinned for ScenePin {
    type State = ZebesState;
    const NAME: &'static str = "production-zebes-scene";
    const SHAPE: u64 = 0x9197_892A_716D_DFAC;
}

/// Destination held by the scheduler before the room loader selects an entrance.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadDestination {
    /// The Landing Site cutscene station on Zebes.
    LandingSite,
    /// The station where a new game starts on Ceres.
    CeresStart,
}

/// Pending scheduler work and its carried destination.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    /// The flight is still running.
    Flight(LoadDestination),
    /// The scene has published its load request.
    Loading(LoadDestination),
}

impl Flow {
    /// The context the room loader will read.
    #[must_use]
    pub const fn destination(self) -> LoadDestination {
        match self {
            Self::Flight(destination) | Self::Loading(destination) => destination,
        }
    }
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
struct ColourTurn {
    effect: u16,
    sample: u16,
    remaining: u16,
}

impl ColourTurn {
    const fn starts(effect: ColourEffectRef) -> Self {
        Self {
            effect: effect.0,
            sample: 0,
            remaining: 0,
        }
    }

    fn advance(&mut self, effect: &Effect, palette: &mut [Colour; 256]) -> Result<bool, Fault> {
        if self.remaining == 0 {
            if usize::from(self.sample) == effect.samples.len() {
                if !effect.repeats {
                    return Ok(false);
                }
                self.sample = 0;
            }
            let sample = effect
                .samples
                .get(usize::from(self.sample))
                .ok_or(Fault::ColourEffect)?;
            let end = effect.first.saturating_add(sample.colours.len());
            let row = palette
                .get_mut(effect.first..end)
                .ok_or(Fault::ColourEffect)?;
            for (slot, &colour) in row.iter_mut().zip(&sample.colours) {
                *slot = Colour(colour);
            }
            self.remaining = sample.frames.get();
            self.sample = self.sample.saturating_add(1);
        }
        self.remaining = self.remaining.saturating_sub(1);
        Ok(true)
    }
}

/// Complete carried scene, colour effects, drawing and receiving owners through the hand-off.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The complete F4 model state.
    pub scene: Model<ScenePin>,
    /// What the scheduler will dispatch.
    pub flow: Flow,
    /// The scheduler's carried count of finished work.
    pub finished: u16,
    /// Complete carried Health, including its energy and protection.
    pub health: Model<crate::production::health_tail::HealthPin>,
    /// The stored energy capacity.
    pub maximum: u16,
    /// Current published palette.
    pub palette: [Colour; 256],
    /// The current sprite buffer.
    pub oam: Oam,
    japanese_subtitles: bool,
    active: [Option<ColourTurn>; 2],
}

/// Fresh scene and consumer outputs for one cut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// F4's complete publication and owner requests.
    pub scene: FrameReport,
    /// The three ordered writes installed by the drawing owner on setup.
    pub pages: Option<Box<Pages>>,
}

/// Resolved drawing writes. The affine map writes only the low byte of each cell word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pages {
    /// Rear ship map, addressed in Mode 7 map-cell order.
    pub ship_map: Vec<u8>,
    /// Mode 7 graphics shared with the preceding scene and kept installed.
    pub inherited_ship_tiles: Vec<u8>,
    /// The flat page's tile words.
    pub flat_map: Vec<u16>,
    /// Graphics words used by the flat page and sprites.
    pub zebes_tiles: Vec<u16>,
}

/// The dependencies that can stop the flight recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The requested drawing pages are absent or differ from the imported references.
    Pages,
    /// The requested picture or its palette is absent.
    Sprite,
    /// A requested palette effect is missing or has no usable next sample.
    ColourEffect,
    /// The flight cannot start with this loading context.
    Destination(LoadDestination),
    /// The scene's references or inherited state do not match the admitted recipe.
    State,
    /// The next scheduled frame needs the Landing Site loader.
    LandingSiteLoader,
}

impl State {
    /// Accept the station's blank hand-off, carrying energy and colours unchanged.
    #[must_use]
    pub fn start(
        finished: u16,
        health: samus_health::State,
        maximum: u16,
        palette: [Colour; 256],
        oam: Oam,
        subtitles: SubtitleChoice,
        destination: LoadDestination,
    ) -> Self {
        Self {
            scene: Model(ZebesState::pending()),
            flow: Flow::Flight(destination),
            finished,
            health: Model(health),
            maximum,
            palette,
            oam,
            japanese_subtitles: subtitles == SubtitleChoice::WithJapanese,
            active: [None, None],
        }
    }

    /// The carried caption option used by the flight.
    #[must_use]
    pub const fn subtitles_enabled(&self) -> bool {
        self.japanese_subtitles
    }

    pub(crate) fn validate(&self, content: &Zebes) -> Result<(), Fault> {
        if self.flow.destination() != LoadDestination::LandingSite {
            return Err(Fault::Destination(self.flow.destination()));
        }
        if matches!(self.flow, Flow::Loading(_)) != (self.scene.0.stage == ZebesStage::Complete)
            || self.health.0.energy.whole.get() > self.maximum
            || self.maximum > i16::MAX.cast_unsigned()
        {
            return Err(Fault::State);
        }
        for turn in self.active.iter().flatten() {
            let effect = effect(content, ColourEffectRef(turn.effect))?;
            if usize::from(turn.sample) > effect.samples.len() {
                return Err(Fault::ColourEffect);
            }
        }
        Ok(())
    }

    /// Advance the scheduled flight and publish the model, palette, sprite and hand-off outputs.
    ///
    /// # Errors
    /// The loader is outside this recipe. Missing consumer content rejects the frame.
    pub(crate) fn advance(
        &mut self,
        content: &Zebes,
        rules: Rules,
        cut: FrameAdvance,
    ) -> Result<Output, Fault> {
        if let Flow::Loading(destination) = self.flow {
            return Err(match destination {
                LoadDestination::LandingSite => Fault::LandingSiteLoader,
                LoadDestination::CeresStart => Fault::Destination(destination),
            });
        }
        let subtitles = if self.japanese_subtitles {
            SubtitleChoice::WithJapanese
        } else {
            SubtitleChoice::EnglishOnly
        };
        let scene = screens_zebes::step(&mut self.scene.0, &content.scene, rules, subtitles, cut);
        if cut == FrameAdvance::Finished {
            self.finished = self.finished.wrapping_add(1);
            self.apply_colours(content, scene.requests.colour_effect)?;
            self.draw(content, &scene)?;
        }
        if scene.requests.handoff.is_some() {
            self.health.0.energy.whole =
                samus_health::WholeEnergy::new(self.maximum).ok_or(Fault::State)?;
            self.flow = Flow::Loading(self.flow.destination());
        }
        let pages = scene
            .requests
            .pages
            .map(|pages| {
                if pages != content.scene.pages {
                    return Err(Fault::Pages);
                }
                Ok(Box::new(Pages {
                    ship_map: content.ship_map.clone(),
                    inherited_ship_tiles: content.ship_tiles.clone(),
                    flat_map: content.flat_map.clone(),
                    zebes_tiles: content.zebes_tiles.clone(),
                }))
            })
            .transpose()?;
        Ok(Output { scene, pages })
    }

    fn apply_colours(
        &mut self,
        content: &Zebes,
        request: Option<ColourEffectRequest>,
    ) -> Result<(), Fault> {
        let [accent, caption] = &mut self.active;
        // Only the active effects are replaced. Their first samples apply to the carried palette.
        if let Some(ColourEffectRequest::ReplaceActive { accent: id, .. }) = request {
            effect(content, id)?;
            *accent = Some(ColourTurn::starts(id));
            *caption = None;
        }
        if let Some(
            ColourEffectRequest::CaptionBrightens(id) | ColourEffectRequest::CaptionDims(id),
        ) = request
        {
            *caption = Some(ColourTurn::starts(id));
        }
        for slot in &mut self.active {
            if let Some(turn) = slot
                && !turn.advance(
                    effect(content, ColourEffectRef(turn.effect))?,
                    &mut self.palette,
                )?
            {
                *slot = None;
            }
        }
        Ok(())
    }

    fn draw(&mut self, content: &Zebes, report: &FrameReport) -> Result<(), Fault> {
        let mut batch = NmiBatch {
            oam: self.oam,
            ..NmiBatch::default()
        };
        batch.clear_high_oam();
        for intent in &report.picture.sprites {
            let mut drawn = SpriteDraw::default();
            drawn.place(
                &content
                    .pictures
                    .iter()
                    .find(|(id, _)| *id == intent.picture)
                    .ok_or(Fault::Sprite)?
                    .1,
                Pair {
                    x: i32::from(intent.anchor.x),
                    y: i32::from(intent.anchor.y),
                },
            );
            let palette = u8::try_from(intent.palette.0)
                .ok()
                .filter(|&p| p < 8)
                .ok_or(Fault::Sprite)?;
            for piece in &mut drawn.pieces {
                piece.look.palette = palette;
                // Preserve OAM slots and move out-of-range pieces outside the visible picture.
                if !(-32..224).contains(&piece.at.y) {
                    piece.at = Pair { x: 384, y: 224 };
                }
            }
            batch.draw_sprites(&drawn);
        }
        batch.finalise_oam();
        self.oam = batch.oam;
        Ok(())
    }
}

fn effect(content: &Zebes, id: ColourEffectRef) -> Result<&Effect, Fault> {
    content
        .effects
        .iter()
        .find(|effect| effect.id == id)
        .ok_or(Fault::ColourEffect)
}

impl Pages {
    /// Install the flight's three drawing writes in live VRAM. Shared ship graphics are inherited.
    pub fn install(&self, vram: &mut [u16]) {
        use crate::commit::write_words;
        use crate::state::video::VramWord;

        for (word, &low) in vram.iter_mut().zip(&self.ship_map) {
            let [_, high] = word.to_le_bytes();
            *word = u16::from_le_bytes([low, high]);
        }
        write_words(vram, VramWord(0x5c00), 1, self.flat_map.iter().copied());
        write_words(vram, VramWord(0x6000), 1, self.zebes_tiles.iter().copied());
    }
}

#[cfg(test)]
mod tests {
    use super::{Flow, LoadDestination, State};
    use crate::state::video::{Colour, Oam};
    use samus_health::{EnergyAmount, EnergyLoss, ProtectionFrames, Warning};
    use types::zebes::SubtitleChoice;

    #[test]
    fn distinct_seed_contexts_survive_checkpoint_encoding() {
        let health = samus_health::State {
            energy: EnergyAmount::ZERO,
            pending_loss: EnergyLoss::ZERO,
            warning: Warning::Off,
            protection: ProtectionFrames::ZERO,
        };
        for destination in [LoadDestination::LandingSite, LoadDestination::CeresStart] {
            let seeded = State::start(
                0,
                health,
                99,
                [Colour(0); 256],
                Oam::default(),
                SubtitleChoice::EnglishOnly,
                destination,
            );
            let encoded = canonical::to_bytes(&seeded);
            let restored: State = canonical::from_bytes(&encoded).unwrap();
            assert_eq!(restored.flow, Flow::Flight(destination));
        }
    }
}
