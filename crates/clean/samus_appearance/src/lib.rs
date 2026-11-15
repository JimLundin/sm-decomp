//! The samus-appearance model, built from its behaviour spec alone: her animation's timing.
//!
//! Each frame a caller runs two things, in this order:
//!
//! 1. Before samus-movement's step, `Ahead::of`, on the playhead it kept and the clip of her
//!    figure: the end her clip reaches this frame, and whether her spin has turned far enough to
//!    kick off a wall. Both follow from the frame's start alone.
//! 2. After it, `update`, with the `State` it kept, the frame's `Inputs` (the figure
//!    samus-movement picked among them), her clips and the run's paces (`Content`) and the tuning
//!    (`Rules`, the spec's parameters: `Rules::VANILLA` is the game). It gives back the `State`
//!    to keep for the next frame, and the sound asked for, which only the frame itself can tell.
//!
//! The `State` is all the model remembers: her step and the time left on it (`Playhead`), the
//! boost stage and the laps left (`LapsLeft`). A caller may set each to any value the spec
//! allows, and gives it back as `update` left it. Her clips are data: each `Clip` is a list of
//! steps, each one shown for a `Hold` or a `Control`; `Playhead::run_down` tells what her clip
//! does as the time runs down (`RunDown`).

//! ## Caller example: ordinary work and frozen drawing
//!
//! Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. The caller supplies real
//! opportunity-time inputs from their owners. The two functions below show one ordinary work
//! and one first-passage work; neither retries a sound or performs movement. Movement reads
//! `Ahead::of` before producing `Inputs`; stride/selection inputs are captured at their named
//! opportunities. Part 2 receives every retained graphical contact, with room/liquid context.
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

mod ahead;
mod catalogue;
mod clip;
mod inputs;
mod moves;
mod opportunity;
mod play;
mod rules;
mod run;
mod starting;
mod state;
mod update;

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
