//! The room's self-destruct warning, from the flyby's handoff to the escape.

use types::GameFrames;
use types::ceres_progress::CeresDefeat;
use types::ceres_warning::{
    HeadingHold, StrokeBeat, SubtitleHold, TypingWait, WarningArtProgress, WarningPhase,
    WarningState, WarningTypingProgress,
};
use types::options::Language;

use crate::station::Stage;
use crate::warning_picture::{
    PositionedGlyph, WarningContent, WarningForeground, WarningPalette, WarningPictureUpdates,
    WarningTextWrite,
};

const GLOW_PERIOD: u16 = 4;

/// Spec: `WarningEnds`. Start the countdown at one minute; the HUD owns its subsequent state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountdownStart {
    /// Whole minutes.
    pub minutes: u8,
    /// Seconds within the minute.
    pub seconds: u8,
    /// Hundredths within the second.
    pub hundredths: u8,
}

/// Spec: `WarningEnds`. Hand these off together on the first escape run, before room actors finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscapeActivation {
    /// The new station stage for doors, walls and the quake.
    pub stage: Stage,
    /// Progress to record before any later load of Ridley's room.
    pub defeat: CeresDefeat,
    /// Initial time for the countdown owner.
    pub countdown: CountdownStart,
}

const START_ESCAPE: EscapeActivation = EscapeActivation {
    stage: Stage::Escape,
    defeat: CeresDefeat::MainBoss,
    countdown: CountdownStart {
        minutes: 1,
        seconds: 0,
        hundredths: 0,
    },
};

/// Spec: `WarningArt`. Request the escape track on the last art run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscapeTrack;

impl EscapeTrack {
    /// Frames before the track starts.
    pub const DELAY: u8 = 8;
}

/// Spec: `WarningTyping`. Request a stroke on alternate printed letters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarningStroke;

impl WarningStroke {
    /// The second effects channel.
    pub const EFFECTS_CHANNEL: u8 = 2;
    /// Audio's queue limit on this request.
    pub const QUEUE_LIMIT: u8 = 3;
}

/// Spec: `WarningArt`, `WarningHeading`, `WarningGlow`, `WarningEnds`. This cut's room inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarningInputs {
    /// Whether the scheduler runs the ordinary room update on this cut.
    pub ordinary: bool,
    /// Wrapping game clock as the cut starts.
    pub finished: GameFrames,
    /// Whether a palette fade is active, suppressing the glow.
    pub fade: bool,
    /// Whether a door passage is active, suppressing the glow.
    pub passage: bool,
    /// The file's language, sampled when the heading hold ends.
    pub language: Language,
    /// Current foreground colours, sampled on the first art run.
    pub foreground: WarningForeground,
}

impl WarningInputs {
    const fn allows_glow(self) -> bool {
        !self.fade && !self.passage && self.finished.0.is_multiple_of(GLOW_PERIOD)
    }
}

/// Spec: `WarningArt`, `WarningHeading`, `WarningTyping`, `WarningGlow`, `WarningEnds`.
/// A cut's ordered drawing updates and one-shot requests. The state itself stays with the caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarningEffects<'a, T> {
    /// Changes to the room picture; anything not named stays in place.
    pub picture: WarningPictureUpdates<'a, T>,
    /// Escape music request on the last art run.
    pub music: Option<EscapeTrack>,
    /// Effects request on an audible letter.
    pub stroke: Option<WarningStroke>,
    /// Present only on the transition from `EscapeReady` to `Escape`.
    pub activation: Option<EscapeActivation>,
}

impl<'a, T> WarningEffects<'a, T> {
    const fn quiet() -> Self {
        Self {
            picture: WarningPictureUpdates::unchanged(),
            music: None,
            stroke: None,
            activation: None,
        }
    }

    /// Spec: `WarningArt`. Load the next batch; the final batch also places the heading.
    fn load_art(&mut self, loaded: usize, content: &WarningContent<'a, T>) -> WarningPhase {
        let Some(batch) = content.schedule.get(loaded) else {
            return WarningPhase::WarningReady;
        };
        self.picture.art = batch;
        if let Some(progress) =
            WarningArtProgress::new(loaded.saturating_add(1), content.schedule.len())
        {
            return WarningPhase::SettingArt(progress);
        }
        self.picture.words = Some(WarningTextWrite {
            palette: WarningPalette::HeadingText,
            cells: content.heading,
        });
        self.music = Some(EscapeTrack);
        WarningPhase::Heading(HeadingHold::FULL)
    }

    /// Spec: `WarningHeading`. Place the subtitle before typing on the last hold run.
    fn subtitle_and_type(
        &mut self,
        left: SubtitleHold,
        typing: WarningTypingProgress,
        content: &WarningContent<'a, T>,
    ) -> WarningPhase {
        let hold = left.spend();
        if hold.is_none() {
            self.picture.words = Some(WarningTextWrite {
                palette: WarningPalette::Japanese,
                cells: content.subtitle,
            });
        }
        self.type_next(typing, hold, content)
    }

    /// Spec: `WarningTyping`. Spend the wait, consume one cell, or finish the English text.
    fn type_next(
        &mut self,
        mut typing: WarningTypingProgress,
        hold: Option<SubtitleHold>,
        content: &WarningContent<'a, T>,
    ) -> WarningPhase {
        if let Some(wait) = typing.wait.spend() {
            typing.wait = wait;
        } else if let Some(cell) = content.english.get(typing.consumed) {
            typing.wait = if typing.consumed == 0 {
                TypingWait::Ready
            } else {
                TypingWait::TwoRuns
            };
            typing.consumed = typing.consumed.saturating_add(1);
            if let Some(glyph) = cell.glyph {
                self.picture.letter = Some(PositionedGlyph {
                    cell: cell.cell,
                    glyph,
                });
                self.stroke = (typing.beat == StrokeBeat::Audible).then_some(WarningStroke);
                typing.beat = typing.beat.flip();
            }
        } else {
            return if hold.is_some() {
                WarningPhase::Typing(WarningTypingProgress {
                    wait: TypingWait::TwoRuns,
                    ..typing
                })
            } else {
                WarningPhase::EscapeReady
            };
        }
        hold.map_or(WarningPhase::Typing(typing), |left| {
            WarningPhase::SubtitleWait { left, typing }
        })
    }
}

/// Advance the warning on an ordinary run and return this cut's updates. A stopped cut
/// leaves both the warning and the caller's picture untouched (`WarningEnds`).
#[must_use]
pub fn step_warning<'a, T>(
    state: &mut WarningState,
    inputs: WarningInputs,
    content: &WarningContent<'a, T>,
) -> WarningEffects<'a, T> {
    let mut effects = WarningEffects::quiet();
    if !inputs.ordinary {
        return effects;
    }
    step_glow(state, inputs, content, &mut effects.picture);
    state.phase = match state.phase {
        WarningPhase::WarningReady => {
            effects.picture.copy_colours(inputs.foreground);
            effects.load_art(0, content)
        }
        WarningPhase::SettingArt(progress) => effects.load_art(progress.get(), content),
        WarningPhase::Heading(left) => hold_heading(left, inputs.language),
        WarningPhase::SubtitleWait { left, typing } => {
            effects.subtitle_and_type(left, typing, content)
        }
        WarningPhase::Typing(typing) => effects.type_next(typing, None, content),
        WarningPhase::EscapeReady => {
            effects.activation = Some(START_ESCAPE);
            WarningPhase::Escape
        }
        WarningPhase::Escape => WarningPhase::Escape,
    };
    effects
}

/// Spec: `WarningGlow`. Use the phase at the start of the run, never the new phase.
fn step_glow<'a, T>(
    state: &mut WarningState,
    inputs: WarningInputs,
    content: &WarningContent<'a, T>,
    picture: &mut WarningPictureUpdates<'a, T>,
) {
    if matches!(
        state.phase,
        WarningPhase::WarningReady | WarningPhase::SettingArt(_)
    ) || !inputs.allows_glow()
    {
        return;
    }
    state.glow = state.glow.advance();
    if let Some(colours) = content.glow_palettes.get(usize::from(state.glow.get())) {
        picture.glow_colours(*colours);
    }
}

/// Spec: `WarningHeading`. Arm the first (silent) cell for the next run when the hold ends.
const fn hold_heading(left: HeadingHold, language: Language) -> WarningPhase {
    if let Some(left) = left.spend() {
        return WarningPhase::Heading(left);
    }
    let typing = WarningTypingProgress::START;
    match language {
        Language::English => WarningPhase::Typing(typing),
        Language::Japanese => WarningPhase::SubtitleWait {
            left: SubtitleHold::FULL,
            typing,
        },
    }
}
