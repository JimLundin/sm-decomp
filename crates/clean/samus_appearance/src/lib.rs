//! The samus-appearance model, built from its behaviour spec alone: her animation's timing,
//! appearance opportunities and independent cannon readiness.
//!
//! Retain [`Look`] for animation and cannon readiness: its `animation` is [`State`],
//! the four animation quantities, and its `cannon` is the independent [`CannonDelay`].
//! For ordinary dry-movement colours, also retain [`ColourSequences`] and the shared [`Palettes`].
//! Spec: `CannonItemDelay`, `RetainedCannonReadiness`. Keeping animation alone is not enough
//! to preserve the whole look; cannon readiness cannot be reconstructed from animation or HUD
//! selection. It advances only at actual eligible item-control completion, not animation or
//! drawing, and resets only at full Samus initialization (`CannonReadinessReset`). Interrupted
//! callers also preserve the actual [`CannonObservation`] alongside the look, interrupted
//! animation work, independently retained preparation inputs and their own input/read/output
//! delivery progress. Save and restore these together; model completion is not external
//! delivery acknowledgement.
//!
//! For an ordinary animation frame, a caller runs two things, in this order:
//!
//! 1. Before samus-movement's step, `Ahead::of`, on the playhead it kept and the clip of her
//!    figure: the end her clip reaches this frame, and whether her spin has turned far enough to
//!    kick off a wall. Both follow from the frame's start alone.
//! 2. After it, `update`, with the `State` it kept, the frame's `Inputs` (the figure
//!    samus-movement picked among them), her clips and the run's paces (`Content`) and the tuning
//!    (`Rules`, the spec's parameters: `Rules::VANILLA` is the game). It gives back the `State`
//!    to keep for the next frame, and the sound asked for, which only the frame itself can tell.
//!
//! The `State` is all the animation remembers: her step and the time left on it (`Playhead`), the
//! boost stage and the laps left (`LapsLeft`). A caller may set each to any value the spec
//! allows, and gives it back as `update` left it. Her clips are data: each `Clip` is a list of
//! steps, each one shown for a `Hold` or a `Control`; `Playhead::run_down` tells what her clip
//! does as the time runs down (`RunDown`).

//! ## Caller example: animation and its shown opportunities
//!
//! This example is scoped to animation, stride/selection sounds and frozen drawing/passage
//! opportunities. It does not show cannon item controls or complete persistent look retention,
//! nor a full scheduler; callers still retain cannon readiness as described above.
//!
//! Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. The caller supplies real
//! opportunity-time inputs from their owners. The two functions below show one ordinary work
//! and one first-passage work in that scope; neither retries a sound or performs movement.
//! Movement reads `Ahead::of` before producing `Inputs`; stride/selection inputs are captured
//! at their named opportunities. Part 2 receives every retained graphical contact, with
//! room/liquid context.
//! The sound owner receives every request in order, including the existing animation request.
//! This example leaves `Boosting` in the existing animation interface rather than inventing
//! an admission limit for it. Engine admission predicates for other drawing effects must be
//! established by those owners, not manufactured by these calls.
//!
//! ```rust
//! use samus_appearance::*;
//! use types::{Action, AnimationSound, Item, Set};
//!
//! // A caller-owned output buffer; these are distinct requests, not a last-sound slot.
//! enum Request { Animation(AnimationSound), Appearance(SoundRequest) }
//! struct Outputs { sounds: Vec<Request>, graphics: Vec<GraphicalContact> }
//! impl Outputs {
//!     fn retain(&mut self, output: OpportunityOutput) {
//!         self.sounds.extend(output.sound.map(Request::Appearance));
//!         self.graphics.extend(output.contact);
//!     }
//! }
//!
//! fn ordinary<C: Content>(
//!     state: State, inputs: &Inputs, content: &C, stride: Opportunity<'_>,
//!     selection: Opportunity<'_>, previous: Action,
//! ) -> (State, Outputs) {
//!     let mut out = Outputs { sounds: Vec::new(), graphics: Vec::new() };
//!     // Constructed by the movement owner after running position changes, before update:
//!     // Opportunity::Contact { figure: stride_figure, playhead: stride_playhead,
//!     //     stage: state.stage, clip: content.clip(&stride_figure), suppressors }.
//!     out.retain(stride.read());
//!     let (after, animation_sound) = update(state, inputs, content, &Rules::VANILLA);
//!     out.sounds.extend(animation_sound.map(Request::Animation));
//!     // Constructed for the actual selection, including both complete figures:
//!     // Opportunity::PoseSelected { from, selected, items, submerged, cinematic }.
//!     out.retain(selection.read());
//!     out.retain(Opportunity::NormalDraw {
//!         previous, current: inputs.figure_after.pose.action,
//!     }.read());
//!     // `after` is the sole animation update.
//!     (after, out)
//! }
//!
//! fn first_passage_frozen(
//!     state: State, previous: Action, current: Action, shoot_held: bool, charge: Charge,
//! ) -> (State, Outputs) {
//!     let mut out = Outputs { sounds: Vec::new(), graphics: Vec::new() };
//!     out.retain(Opportunity::PassageEntry { current, shoot_held, charge }.read());
//!     // Then the passage sound hold; normal drawing can still occur in quieting.
//!     out.retain(Opportunity::NormalDraw { previous, current }.read());
//!     (state, out) // No update, no movement: all four quantities unchanged.
//! }
//!
//! // Invented content for a complete ordinary-work call. In a game the importer supplies
//! // Content's figure-to-clip lookup; here both figures intentionally share one clip.
//! struct ExampleContent { clip: Clip, paces: Paces }
//! impl Content for ExampleContent {
//!     fn clip(&self, _: &Figure) -> &Clip { &self.clip }
//!     fn paces(&self) -> &Paces { &self.paces }
//! }
//! let hold = Hold::new(7).unwrap();
//! let content = ExampleContent {
//!     clip: Clip {
//!         instructions: vec![Instruction::Show(hold), Instruction::Show(hold),
//!             Instruction::Control(Control::Again(Step::FIRST))],
//!         kick: None, fast_fall: None, rising: None,
//!         contacts: [Step::FIRST].into_iter().collect(),
//!     },
//!     paces: Paces { plain: Pace(vec![hold, hold]), boosted:
//!         enum_map::EnumMap::from_array(std::array::from_fn(|_| Pace(vec![hold, hold]))) },
//! };
//! let running = Figure {
//!     pose: types::Pose { body: types::Body::Upright, action: Action::Running,
//!         facing: types::Facing::Right, aim: types::Aim::Forward,
//!         gun: types::Gun::Stowed, legs: types::Legs::Together },
//!     sprite: types::Sprite::Plain,
//! };
//! let selected = Figure { pose: types::Pose { action: Action::Spinning, ..running.pose },
//!     ..running };
//! // Multiple requests are retained even when animation is frozen:
//! let state = State {
//!     playhead: Playhead { step: Step::FIRST, time_left: Frames::LAST },
//!     stage: types::BoostStage::Stage0, laps_left: LapsLeft::WithoutBooster,
//! };
//! let (unchanged, out) = first_passage_frozen(
//!     state, Action::Spinning, Action::Running, false, Charge::new(0).unwrap(),
//! );
//! assert_eq!(unchanged, state);
//! assert_eq!(out.sounds.len(), 2); // SamusSilence then SpinEnded, distinct opportunities.
//!
//! let ahead = Ahead::of(state.playhead, content.clip(&running)); // Before movement.
//! // Now use the movement owner's results. No movement is performed by appearance.
//! let inputs = Inputs {
//!     figure: running, figure_after: selected, moves: Set::default(), dash: Dash::Off,
//!     run_held: false, heading_down: false, vertical_speed: types::ZERO,
//!     energy: 99, items: Set::<Item>::default(), liquid: Liquid::NONE,
//!     sounds_waiting: SoundsWaiting::NONE,
//! };
//! let stride = Opportunity::Contact { figure: running, playhead: state.playhead,
//!     stage: state.stage, clip: content.clip(&running), suppressors: Set::default() };
//! let selection = Opportunity::PoseSelected { from: running, selected,
//!     items: inputs.items, submerged: false, cinematic: CinematicActivity::Inactive };
//! let (after, out) = ordinary(state, &inputs, &content, stride, selection, Action::Running);
//! assert_eq!(ahead.end, None);
//! assert_eq!(after.playhead.time_left, hold.frames());
//! assert_eq!(out.graphics.len(), 1);
//! assert!(matches!(out.sounds.as_slice(), [Request::Appearance(SoundRequest::Footstep),
//!     Request::Appearance(SoundRequest::PlainSpin)]));
//! // Hand out.graphics to part 2 and out.sounds to sound work without dropping either.
//! ```

//! # Interrupted work
//!
//! Spec: `PerformedAnimationDecisions`, `SuspendedAppearance`. Build owned `AnimationInputs`
//! and a `SuspendedAppearance` with `animation_steps()`, inserting `WorkStep::Appearance`
//! reads at their actual opportunities. The caller preserves animation order, places contact
//! before playing and supplies actual accepted selection before `SelectedClip` is offered.
//! Update unmade owner inputs in `animation` or unconsumed appearance inputs in `steps` at
//! those opportunities; never replace a pending answer or completed receipt.
//!
//! `offer()` chooses and retains an applicable animation answer without applying it, consuming
//! skipped rules. `None` can mean the next work is an appearance read or that work is finished;
//! `complete()` distinguishes these cases, applying an answer once or completing the read.
//! `finish()` is appropriate only once all remaining inputs are available at their opportunities.
//! Produced receipts, sounds and graphics are retained in order, along with consumed work progress.
//! Serialize the entire work with serde, discard it and deserialize fresh work to continue.
//! The caller owns external delivered positions and pending publications and saves those with
//! this work and the rest of its continuation, including independently retained preparation inputs.
//! `update()` uses this same engine without pauses. Appearance-only frozen work has no
//! animation inputs and contains only the actual drawing or passage opportunities.

//! # Cannon readiness
//!
//! Spec: `CannonItemDelay`, `CannonReadinessReset`, `RetainedCannonReadiness`.
//! Carry `Look { animation, cannon }` for animation and cannon readiness. Construct
//! `CannonDelay::new(n)` with `n` in 0..=2 for part-way starts, or use `CannonDelay::INITIAL`
//! at full initialization. `initialize()` resets only readiness; the existing
//! animation initialization remains its own contract.
//!
//! At actual item controls, supply `ItemControlAnswer` identities and eligibility.
//! Ordinary work calls `controls_completed()` once after effects complete. Interrupted
//! work carries `CannonObservation::Unmade`, calls `choose()` at the actual choice,
//! then `complete(&mut look)` only when those effects complete. Retain the observation
//! alongside `Look` and existing `SuspendedAppearance` and caller-owned progress;
//! serialize all of them, discard them and deserialize fresh instances. Completed
//! observations cannot apply again, and chosen answers cannot be replaced.
//! No observation is made for skipped controls. A new observation belongs only to
//! the next actual controls, never a redraw, pose change or sound retry.
//! At drawing, read readiness after those actual controls have completed.
//! Read `remaining()` or `may_follow()` without effects. HUD one-shot mode, switch
//! events, outside selection changes, gun pictures and external delivery acknowledgements
//! are not readiness inputs. Neither readiness update nor inspection advances animation
//! or publishes a sound/graphic.

//! # Ordinary dry-movement colours
//!
//! Spec: `OrdinaryDryColours`, `RetainedDryColours`. For each actual palette opportunity,
//! carry [`DryColourWork::UNOBSERVED`] until the current figure, position and suit are selected.
//! Before health maintenance, gather the actual neighbour answers in [`DryColourFacts`],
//! including whether the screw attack is switched on now ([`ScrewAttack`], from progress), and
//! call [`DryColourWork::observe`].
//!
//! The reset admits an upright figure that is running, standing still, moonwalking, in a
//! straight jump, turning in one or landing from one, or spinning while the screw attack is off,
//! with every neighbour fact ordinary. `Err(UnsupportedDryColours)` means the opportunity is
//! outside that arm: report it unsupported; the work, sequences and palettes are left as they
//! were. A skipped opportunity needs no call. Once observed, the work keeps the facts it
//! admitted: a later screw attack, suit or effect change does not alter or cancel it.
//!
//! While admitted work is pending, consumers still read the old [`ColourSequences`].
//! Other colour owners can write either bank in the same shared [`Palettes`] at any time.
//! [`DryColourWork::complete`] publishes the prepared sequences once, before later drawing or
//! palette upload; it returns `true` only for that completion. It needs neither fresh facts nor
//! palette access. Video reads the current bank; transitions can also read the target bank. No
//! figure, animation, cannon or other look work is consumed, and there are no requests to
//! deliver from the reset.
//!
//! Save the sequences, both live banks, and the work together with the selected figure, the
//! rest of the look and caller-owned opportunity and delivery progress. Restore fresh instances
//! of all of them. Unobserved work still needs current facts; pending work keeps its own admitted
//! facts; completed work cannot reset again. Reading or saving any of these does not apply work.

mod ahead;
mod cannon;
mod catalogue;
pub use cannon::{
    CannonDelay, CannonObservation, ControlEligibility, ItemControlAnswer, Look, SelectedItem,
};
mod clip;
mod colours;
pub use colours::{
    ChargeGlowPosition, ColourEffect, ColourSequences, DryColourFacts, DryColourWork,
    GrappleActivity, LiquidLevel, RoomColours, ScrewAttack, Suit, UnsupportedDryColours,
    VisorDelay, VisorStep,
};
pub use types::{PALETTE_SLOTS, Palette, Palettes};
mod inputs;
mod moves;
mod opportunity;
mod play;
mod rules;
mod run;
mod starting;
mod state;
mod suspended;
mod update;
pub use suspended::{
    AnimationDecision, AnimationInputs, Answer, AppearanceRead, OfferedDecision, PublishedSound,
    Receipt, SuspendedAppearance, WorkStep,
};

pub use ahead::Ahead;
pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use clip::{Branch, Clip, Control, Frames, Hold, Instruction, Pace, Paces, Slowdown, Step};
pub use inputs::{Content, Dash, Figure, Inputs, Liquid, MoveEvent, SoundsWaiting};
pub use opportunity::{
    Charge, CinematicActivity, ContactSuppressor, GraphicalContact, Opportunity, OpportunityOutput,
    SoundQueue, SoundRequest,
};
pub use rules::Rules;
pub use state::{Laps, LapsLeft, Playhead, RunDown, State};
pub use update::update;
