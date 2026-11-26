//! The samus-movement model, built from its behaviour spec alone: a character controller for
//! Samus, stepped one 60 Hz frame at a time, her motion in fixed-point subpixels.
//!
//! Every rule names the spec entry it comes from. `samus` holds her state and a frame's
//! outputs; `step` plays a frame in its two steps (§12): `moves` her moves as she posed and the
//! pose they ask for, then, once the frame's hit is known, `change` the pose she takes, with
//! `fit` its pose fit, and `knockback` a hit's push. `run` is her horizontal motion and
//! `vertical` her vertical motion, each a value with its own rules; `collide` what her moves
//! meet, `jump_held` jump held through, and `chart` with `situation` and `rows` the pose chart;
//! `rules` the tuning parameters and `catalogue` the spec's entries.
//!
//! A caller holds a `Samus`, every part of her state public (her `Run` with its `Dashing`,
//! `Vertical`, pose, `Choices`, `Seat`, `JumpHeld` and `Knockback`), and steps her a `Frame` at a
//! time in a `World` under `Rules` (every tuning value the spec's entries list as a parameter,
//! `Rules::default()` the vanilla game's). A frame is two calls in order: `Samus::moves`, whose
//! `HerMoves` gives her position after her moves, from which the hitter judges the frame's hit
//! (`types::Hit`, or none); then `Samus::react` with that hit, which plays the rest of the frame
//! and returns its `Outcome` (what it gives the camera, the world and her animation, §1, §1.4,
//! §12's push started and `TouchedCells`, her sprite as the shared `types::Sprite`). `moves`
//! gives `UnknownPose` for a pose no situation names. The pose chart alone answers through
//! `Moment::next_pose`.
//!
//! Spec: `WallJump`, her choices. `Outcome::recorded_choice` reports what the frame's choice of
//! her pose recorded in `Samus::choices`: `Some` wherever the frame chose anew, the pose she
//! already had included, and `None` where it chose nothing and her choices stayed as they were.
//!
//! Spec: `TravelledDistances`. `Outcome::travelled` directly reports a fresh complete
//! `DirectionalTravels` map of signed `TravelDistance`s after collision. A caller retains
//! these in `TravelledDistances::travels`, reads them for firing before refreshing the interval,
//! and can replace the complete map or any named direction with any wrapping signed value.
//! `TravelledDistances::record` replaces ordinary travels while ignoring adjustments and looks;
//! `refresh` clears every direction. Skipping movement does not require a refresh.
//!
//! Spec: `TransitionFiringAim`. Supply named pose content through `Frame::cannon_heading`.
//! `Outcome::pending_aim` is the fresh publication of the pose change she took, a change an
//! animation's end handed over included and a refused change excluded. On `None`, keep the
//! caller's earlier pending aim. `FiringDirection` saves and restores with serde, every
//! direction included. Forced handovers such as the shinespark's belong to their own domain.
//!
//! Spec: `Jumping`, jump history during a passage. While her motion and pose choice are
//! paused, draws that update jump history go on without her frame: the caller holds a
//! `DrawnJumpHistory` (her `JumpHeld` and the jump button of the previous such draw, on an
//! active frame the jump the frame's pad holds) and calls `DrawnJumpHistory::draw` with each
//! history-writing draw's jump, and nothing between them. The count goes on against the
//! previous draw, whatever the controller's poll pressed; the frame before stays as the last
//! active frame left it, so on the next active frame the caller puts `jump_held` back on her
//! and ends that frame with `Samus::react_after_draw`, giving it the history's `last_draw`:
//! its count goes on against that draw's jump, while her controls and pose read the frame's
//! own pad. Jump held now after a draw with jump up puts the count at 0, even with no press
//! reported this frame. `Samus::react` is the same reaction for a frame with no passage
//! before it, where the frame's own pad tells the previous draw's jump.
//!
//! # An ordinary call
//!
//! Restore the caller's complete serde `Samus` (position, pose, `Run`, `Vertical` with its
//! bounces and phase, `Choices`, `Seat`, `JumpHeld` and `Knockback`); a start mid-play with no
//! history gives `Choices::starting_in` its pose. Before the frame's moves, weapons read the
//! caller-held `TravelledDistances` and pending firing aim; the cannon pass clears pending aim.
//! Appearance's first half supplies the frame's clip end and spin-turned-far result, and the
//! sprite it selected before movement; progress supplies the items switched on, the mode
//! cinematic and elevator status, input the held and just-pressed actions, and the game the
//! frame's parity and the poses' bodies and cannon headings.
//!
//! Build a `Frame` with those inputs and call `samus.moves(&frame, &world, &rules)`; the hitter
//! judges the frame's hit from `position_after_moves()` and the body of the pose she started
//! in, samus-health takes its damage, and `samus.react(moves, hit)` plays the rest of the frame
//! and leaves `samus` her State at the frame's end: save all of it for the next frame. Between
//! the two calls `samus` is still her State at the frame's start. Keep `outcome.travelled` for
//! the next cannon opportunity, and replace the
//! pending aim only when `outcome.pending_aim` publishes a heading. A frame the surrounding mode
//! skips calls nothing and keeps the caller's memory as it is.
//!
//! The world receives `inside` and the ordered `reached` cells; the mini-map
//! `position_after_moves`; sound transport every ordered `touchdown_requests` record (identity,
//! library and allowance), repeats kept; appearance's second half the start and end poses,
//! `floor_response` with its speed, `touchdown_graphics`, the §1.4 outputs and
//! `push_started` (`HurtColours`), and it may
//! append its own sound requests after movement's. None of these is State, and a frame with no
//! touchdown reports none.
//!
//! # Content boundary and numerical meanings
//!
//! The importer validates pose/action relations, coverage for every selectable body, and
//! world stand-in chains before handing the model content. `Frame::body` returns an actual
//! `Radius` (whole-pixel half-sizes), not an unavailable-content sentinel. Missing body
//! content is an importer error, never silently replaced by zero geometry. The spec gives
//! no universal radius range; the model is total for admitted typed radii, including zero,
//! while the importer checks that its content represents the intended game's boxes.
//! `cannon_heading` may be unavailable (`None`), distinct from every valid firing direction.
//!
//! `Run::speed` and `Run::dash` are magnitudes; carry and facing give their direction.
//! Moonwalking's facing is the way she travels, while her sprite and gun look backwards.
//! A horizontal move is limited in whole pixels, its fraction kept; solid enemies judge the
//! limited move before a sloped floor shortens it, and an enemy that stops it leaves no
//! shortening. `SlopeFactor` is dimensionless, and cells are column and row coordinates. Every
//! pixel and subpixel carry, and every signed travel's borrow, keeps its wrapping meaning.
//! The pull grows her vertical speed after her vertical move and before what it met acts, so
//! a ceiling's frame reports the pulled rise, which the ceiling then ends.
//!
//! The complete compiling ordinary example is `tests/ordinary.rs`: a fresh start apart from a
//! serde restoration, and the caller's pending aim and travels at the firing and publication
//! boundaries.

pub mod catalogue;
mod change;
mod chart;
mod collide;
mod fit;
mod jump_held;
mod knockback;
mod moves;
mod rows;
pub mod rules;
mod run;
mod samus;
mod situation;
mod step;
mod tick;
mod touchdown;
mod travel;
mod vertical;

pub use types::{
    FloorResponse, FloorResponseKind, QueueAllowance, SoundIdentity, SoundLibrary, SoundRequest,
    VerticalSpeed,
};

pub use travel::{TravelKind, TravelledDistances};
pub use types::{Direction, DirectionalTravels, FiringDirection, TravelDistance};

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use chart::Moment;
pub use jump_held::{DrawnJumpHistory, FrameBefore, JumpButton, JumpHeld};
pub use knockback::{Knockback, Push};
pub use rows::Met;
pub use rules::{Bounces, Rules, SpeedCap, WithItem};
pub use run::{Carry, Dashing, Run};
pub use samus::{Choices, Chosen, Outcome, Samus, Seat, StandingAnimation, WallTest};
pub use situation::UnknownPose;
pub use step::HerMoves;
pub use tick::Frame;
pub use types::{Hit, PushFrames};
pub use vertical::{Phase, Vertical};
