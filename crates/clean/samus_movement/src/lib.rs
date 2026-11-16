//! The samus-movement model, built from its behaviour spec alone: a character controller for
//! Samus, stepped one 60 Hz frame at a time, her motion in fixed-point subpixels.
//!
//! Every rule names the spec entry it comes from. `samus` holds her state and a frame's
//! outputs; `step` runs a frame in its order: `moves` her moves, `change` the pose change. `run`
//! is her horizontal motion and `vertical` her vertical motion, each a value with its own moves;
//! `collide` what her moves meet, `fit` pose fit, `jump_held` jump held through, and `chart`
//! with `situation` and `rows` the pose chart; `rules` the tuning parameters and `catalogue` the
//! spec's entries.
//!
//! A caller holds a `Samus`, every part of her state public (her `Run` with its `Dashing`,
//! `Vertical`, pose, `Spin`, `Seat` and `JumpHeld`), and steps her a `Frame` at a time
//! (`Samus::step`) in a `World` under `Rules` (every tuning value the spec's entries list as a
//! parameter, `Rules::default()` the vanilla game's), reading the frame's `Outcome` (what it gives the
//! camera, the world and her animation, §1, §1.4 and `TouchedCells`, her sprite as the shared
//! `types::Sprite`), or `UnknownPose` for a pose no situation names. The pose chart alone answers
//! through `Moment::next_pose`.
//!
//! Spec: `TravelledDistances`. `Outcome::travelled` directly reports a fresh complete
//! `DirectionalTravels` map of signed `TravelDistance`s after collision. A caller retains
//! these in `TravelledDistances::travels`, reads them for firing before refreshing the interval,
//! and can replace the complete map or any named direction with any wrapping signed value.
//! `TravelledDistances::record` replaces ordinary travels while ignoring adjustments and looks;
//! `refresh` clears every direction. Skipping movement does not require a refresh.

//! Spec: `TransitionFiringAim`. Supply named pose content through `Frame::cannon_heading`.
//! `Outcome::pending_aim` is the actual ordinary transition's fresh publication, including
//! changes handed over by ordinary animation ends and excluding refused changes. On `None`,
//! retain the caller's earlier pending aim. `FiringDirection` supports serde checkpoints for
//! every optional direction, including diagonals and each distinct vertical facing. Forced
//! handovers such as shinespark belong to their surrounding domain, not `Samus::step`.

//! # An ordinary call and consumer lifetime
//!
//! Restore the caller's complete serde `Samus` (position, pose, Run, Vertical including
//! bounces/phase, Spin, Seat and `JumpHeld`). Before the new movement interval, weapons read
//! caller-held previous `TravelledDistances` and pending firing aim; the cannon pass clears
//! pending aim. Appearance's first half supplies this frame's clip end and spin-turned-far
//! result, and its actual already-selected `Sprite` supplies `sprite_before_movement`.
//! Progress supplies switched-on items, the mode supplies cinematic/elevator status, input
//! supplies held/just-pressed actions, and the game supplies parity and pose-body/cannon content.
//! None of these inputs is inferred from this frame's final pose or current items.
//!
//! Build `Frame` with those inputs and call `samus.step(&frame, &world, &rules)` exactly once.
//! It mutates the persistent Samus; retain/save all of that value for next frame. Save
//! `outcome.travelled` for the next cannon opportunity and replace caller pending aim only
//! when `outcome.pending_aim` publishes a heading. Skipped intervals preserve caller memory.
//! The world receives `inside` and ordered `reached` reactions, not collision-work records.
//!
//! CPU timing receives `movement_work`, `movement_skip` and ordered `collision_tests`
//! at step completion as fresh semantic performed work. Fit requests are nested in the
//! actual `BodyFitConsideration`, preserving request order rather than flattening it away.
//! An enemy search owns its `CandidateCount` and considered prefix; block-only looks have
//! no enemy-search fields. Support is floor then ceiling, using any floor-updated centre.
//! A zero request is present; skipped ordinary admission is not a zero request.
//!
//! CPU timing assigns any timing costs outside movement. Sound transport receives every ordered
//! `touchdown_requests` record (identity, library and allowance), without deduplication.
//! Appearance's second half receives start/end pose, actual `floor_response` and its
//! pre-response `VerticalSpeed`, `touchdown_graphics` permission, and the existing animation
//! outputs. It evaluates its own graphics admission rules and can append normal-drawing sound
//! requests *after* movement's requests, preserving even repeated `SpinEnded` identities.
//! These new outputs are fresh per-call publications, not movement State and not claims of
//! sound playback or visible dust/splash. A subsequent call with no contact reports none.
//! Spec: `AnimationEnds`, `PerformedBodyFit`. An ordinary animation end accepts its
//! handed-over target without body-fit consideration, clearance, refusal, substitution or
//! fit placement, even for taller supplied bodies. Initialization and the end's effects
//! still apply. An end that gives way uses ordinary selection and fitting instead.
//! `floor_response` reports accepted contact, never handover or held-jump history.
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
//! `Run::speed` and `Run::dash` are magnitudes; carry/facing supplies their direction.
//! Moonwalking's facing is the way she travels, while her sprite/gun looks backwards.
//! A speed-derived intent is submitted to collision, the horizontal whole-part limit keeps
//! its fraction, then `MotionAttempt::intended` publishes the signed limited request before
//! slope shortening. Eligible enemies see that complete limited request first; an enemy
//! stop skips terrain and its factors. Otherwise terrain tests use the unshortened target
//! line, applying each reached floor's factor in order until a stop. `RequestEffect::Travel`
//! is signed accepted travel; `RequestEffect::Clearance` is available whole-pixel room,
//! not net displacement. `SlopeFactor` is dimensionless, and cells are column/row coordinates.
//! All pixel/subpixel carries and signed travel borrows retain their wrapping meanings.
//! Gravity grows the speed after vertical travel and before the response. Thus a ceiling
//! reports the pulled pre-response rise, then ordinarily clears it; changing-body ceiling
//! contact retains that pulled speed. Selected `handed_over` metadata is not an accepted
//! floor effect. Each accepted floor response carries its own kind and pre-response speed.
//!
//! The complete compiling ordinary example is `tests/ordinary.rs`: fresh initialization
//! is separate from serde restoration, including complete carry, phase, spin and seating.
//! It shows caller-owned pending aim/travel at the firing and publication boundaries.
//!
//! The integration test `ordinary_call_delivers_outputs_and_comparison_checks_transport_fields`
//! performs this ordinary flow through the public entry point with supplied room/body data.

pub mod catalogue;
mod change;
mod chart;
mod collide;
mod fit;
mod jump_held;
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
pub use types::{
    BodyFitConsideration, BodyFitResult, CandidateCount, CandidateIndex, CandidateResult,
    ClearanceRoom, CollisionPurpose, CollisionTest, CollisionTests, EnemyCandidate, EnemySearch,
    FloorResponse, FloorResponseKind, MotionAttempt, MotionResult, MotionSkipReason, MovementWork,
    PixelGap, PlacementDistance, QueueAllowance, RequestEffect, RequestedWorld, SoundIdentity,
    SoundLibrary, SoundRequest, SupportCheck, SupportResult, SupportSurface, VerticalMotionAtTest,
    VerticalSpeed,
};
mod vertical;

pub use travel::{TravelKind, TravelledDistances};
pub use types::{Direction, DirectionalTravels, FiringDirection, TravelDistance};

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use chart::Moment;
pub use jump_held::{FrameBefore, JumpHeld};
pub use rows::Met;
pub use rules::{Bounces, Rules, SpeedCap, WithItem};
pub use run::{Carry, Dashing, Run};
pub use samus::{Outcome, Samus, Seat, Spin, StandingAnimation, WallTest};
pub use situation::UnknownPose;
pub use tick::Frame;
pub use vertical::{Phase, Vertical};
