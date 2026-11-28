#![cfg(not(doctest))]
//! The boss-ceres-ridley model, built from its behaviour spec alone. The parts of the Ceres
//! station so far: the doors, the shaft as she arrives, Ridley's room as she enters, Ridley's
//! waking, his tail at rest and its hit, his take-off, flight and real retreat, and the escape's steam. Each
//! piece has its own state, inputs and outputs, and its caller steps it on its own schedule.
//!
//! **The doors.** A Ceres door stands between two of the station's rooms. It slides open as
//! Samus comes near and shuts behind her once she has gone, hissing as it opens. The shaft's door
//! becomes an unseen wall once Ridley has fled, and during the escape every door keeps the
//! station shaking.
//!
//! A caller holds each [`CeresDoor`] between frames. When a room starts, [`CeresDoor::place`]
//! places it and says which colour set its palette takes, and which palette. Each frame the
//! enemies run,
//! [`CeresDoor::run`] takes the door and what it reads as the frame starts ([`Inputs`]) and
//! answers with the door as its run leaves it, the sound it asks for and the shake it starts
//! ([`Run`]).
//!
//! **The shaft as she arrives.** A new game starts with her at the top of the shaft on the
//! elevator's pad. The pad holds her for a second, lowers her a pixel a frame to the shaft's
//! floor and lets her go; a cover hides the floor below until she stands on it. Before the
//! escape the shaft's beacon pulses through its colours and the platform below turns.
//!
//! The pad and the cover hold state: a caller holds the [`PadAndCover`] between frames and
//! steps it on the frame the arrival places them ([`PadAndCover::arrival`]) and on each frame
//! they run ([`PadAndCover::run`]). Each step answers with them as it leaves them, her height
//! and what is drawn of them ([`PadAndCoverStep`]).
//!
//! The lights remember nothing: on each light turn, [`Lights::step`] answers with the beacon's
//! step and the platform's look. Outside the escape, every overlay turn is a light turn.
//!
//! **The shaft during the escape.** Place the actor with [`place_shaft_overlay`] and give it
//! turns with [`turn_shaft_overlay`]. The caller applies its art, light and burst requests,
//! and passes the rotation start to [`turn_shaft`] after motion and the room actors. The shaft
//! publishes its view and can lock Samus on reaching the elevator. [`MountedVent::run`] reads
//! the incoming view and projects only the drawing origin, not the contact position.
//!
//! **Ridley's room as she enters.** Its door is a Ceres door of its own kind: it stands open
//! behind her a moment, slams shut and stays locked, drawn by Ridley, until he is beaten; then
//! it is an ordinary door. The two walls his getaway puts up are Ceres doors too, shown during
//! the getaway only. Ridley waits in the dark with the baby, which now and then cries and
//! glows, until he wakes.
//!
//! Ridley holds state: a caller holds the [`CeresRidley`] between frames and steps it on the
//! frame the room places him ([`CeresRidley::place`]) and on each frame he runs
//! ([`CeresRidley::run`]), with what he reads. The placing answers with what
//! he hands the rest of the game ([`RidleyPlacing`]); a run, with the baby's cry and colours
//! ([`RidleyRun`]).
//!
//! **Ceres Ridley wakes.** His waiting over, his eye lights up out of the dark, then his body
//! fades in; with its last colours he turns tangible and the fight's music starts. A moment
//! later he roars, shuts his jaws and sits a while before he takes off. The same
//! [`CeresRidley::run`] carries him through it, reading his content ([`RidleyContent`]): the
//! run answers with his eye's and body's colours, the fight's music and the roar's sound too.
//!
//! **His tail at rest and its hit.** From the moment he starts moving until he takes off, his
//! tail hangs still behind him ([`CeresRidley::tail`]), its curl changing now and then at
//! random. Once he is tangible its tip hits her when her body comes within its reach and her
//! protection has run out: the run reads her ([`Her`]) and answers with the frame's hit, its
//! side and its push time ([`types::Hit`]). Her motion reads the hit as it stands;
//! samus-health reads it as its `Impact` input, worth 15 whole energy.
//!
//! **His take-off and flight.** His roar over, he crouches and hops twice: first up and one
//! pixel to the right, then up and to the left. He springs into the air, rises fast, slows, and
//! hovers, his speed drawn to his point ([`HIS_POINT`]), until she is nearly out of energy or
//! his hits taken reach their limit and he gives up, or he reaches his point or tires of
//! hovering and turns to an attack. The same
//! [`CeresRidley::run`] carries him, reading how the fight stands ([`Fight`]); every run he
//! moves by his speed within the room's bounds, and the run answers with the bound he met and
//! the shake he starts as he meets the room's left side fast.
//!
//! **His tail in flight.** The caller carries the seven-link [`FlightTail`] between runs.
//! On take-off it comes loose; from that run on it swings after his move and before his
//! take-off figure advances. [`RidleyRun`] reports the seven places in base-to-tip order,
//! any swish, and the same [`types::Hit`] as at rest, now judged from the moved tip.
//! Aiming describes the ordinary Ceres loadout, without missiles.
//!
//! **The ordinary fight picture.** Save [`FightWings`] with the fight. Supply facing, tail
//! visibility and the actor framework's flash inputs on [`RidleyFrame`]. [`RidleyRun`] reports
//! the pre-hop parts and any accepted-shot timer refresh; the actor framework spends the timer.
//! Save [`FightRibs`] alongside the wings. Each ribs reveal hands the renderer a [`RibLook`];
//! the renderer owns the art, queued uploads and published picture.
//! Use [`CeresRidley::picture_order`] with the door owner's attachment state to schedule the
//! existing baby and door pictures around the room layers. The flyby keeps a separate clock.
//!
//! **His real retreat.** `Hovers` starts the pull away on the same run that selects it.
//! The caller can restore [`RidleyPhase::Retreating`], [`RidleyPhase::CoastingAbove`]
//! with its remaining [`CoastRuns`], or [`GetawayState::GetawayReady`] with its flyby beats.
//! The stopped run reports [`GetawayBackdrop`] for the wall actors and drawing without
//! changing the station's stage. On the next run [`CeresRidley::run`] starts the flyby using
//! the room's [`FlybyContent`] and the caller's game clock. [`GetawayEffects`] reports the
//! stage for the walls, the picture, and any sound, movement and quake requests. The fake
//! retreat is a separate handoff.
//!
//! **The self-destruct warning.** On the next ordinary run after `WarningReady`,
//! [`step_warning`] starts loading the caller's [`WarningContent`]. The caller saves
//! [`WarningState`] alongside the drawing owner's room picture and applies each run's
//! [`WarningEffects`]. The last art batch places the heading, which holds before English typing.
//! With Japanese enabled, a subtitle can appear during typing. When the warning is finished,
//! the following run publishes the escape stage, Ceres defeat and countdown start together.
//!
//! **The escape's steam.** On her way out, steam vents in the station's floors, walls and
//! ceilings blow puffs. A caller holds one [`Vent`] for each vent the room holds and places each
//! as the room starts ([`Vent::place`]); each waits a random while, then puffs, rests and puffs
//! again, only while it is in view. On each frame the enemies run, [`run_vents`] runs the room's
//! vents together in the room's order, reading the camera and her ([`Her`]), and answers with the
//! frame's one hit, the same shared [`types::Hit`] his tail gives. As a samus-health `Impact`,
//! it is worth zero energy but still grants protection.
//!
//! The tuning values of all of them are in [`Rules`], whose default holds the spec's values.

mod baby;
mod catalogue;
mod door;
mod fight_picture;
mod fight_ribs;
mod fight_wings;
mod figure;
mod flight;
mod flight_tail;
mod flyby;
mod getaway;
mod her;
mod inputs;
mod lights;
mod look;
mod pad;
mod pad_wait;
mod phase;
mod puff;
mod reach;
mod ridley;
mod ridley_phase;
mod ridley_wait;
mod roar;
mod rules;
mod shaft_overlay;
mod shaft_rotation;
mod shown;
mod station;
mod steam;
mod tail;
mod tail_aim;
mod takeoff;
mod turn;
mod waking;
mod warning;
mod warning_picture;

pub use baby::{
    Baby, BabyColours, BabyCry, BabyFigure, BabyLook, BabyRoutine, BabyRun, FidgetFigure,
    GlowFigure,
};
pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use door::{BOX_RADIUS, CeresDoor, DoorKind, OpeningSound, Placement, Run};
pub use fight_picture::{
    FightColours, FightFacing, FightFlash, FightParts, FightPictureInputs, FightPictureOrder,
    FightPictureStage, FlashHalf, RoomLayerOrder, TailIntent, TailTipLook, WingIntent,
};
pub use fight_ribs::{FightRibs, RibBeat, RibWait};
pub use fight_wings::{FightWings, WingCount};
pub use figure::{Figure, FigureLook};
pub use flight::{Bound, Fight, HIS_POINT, HoverRuns, Speed};
pub use flight_tail::{
    FlightTail, LagCount, SwingLag, SwingPace, SwingWay, SwishRuns, TAIL_SEGMENTS, TailAngle,
    TailLength, TailMark, TailSegment, TailSwish,
};
pub use flyby::{
    FlybyBabyLook, FlybyContent, FlybyFrame, FlybyPath, FlybyTransform, FlybyZoom, GetawayEffects,
    GetawaySound, GetawayView,
};
pub use getaway::{GetawayBackdrop, GetawayColours, GetawayContent};
pub use her::Her;
pub use inputs::Inputs;
pub use lights::{BeaconStep, FadeSteps, Lights, LightsInputs};
pub use look::AlternatingLook;
pub use pad::{Cover, Pad, PadAndCover, PadAndCoverFigures, PadAndCoverStep, PadFigure};
pub use pad_wait::PadWait;
pub use phase::{Phase, Quarter, Wait};
pub use puff::{Puff, PuffFigure, VentFacing};
pub use ridley::{
    Boss, CeresRidley, Drawn, GoalColours, MusicStop, RidleyDrawing, RidleyFrame, RidleyPlacing,
    RidleyRun, RoomColours, ShakeStop,
};
pub use ridley_phase::{Attack, CoastRuns, FightMusic, FlightPhase, RidleyPhase, RoarSound};
pub use ridley_wait::{RidleyWait, WakeWait};
pub use roar::{RidleyLook, RoarFigure, TimedRoarFigure};
pub use rules::Rules;
pub use shaft_overlay::{
    OverlayPlacement, OverlaySpriteUpload, OverlayTurn, RumbleBurst, RumbleParticle,
    StartShaftRotation, overlay_figure, place_shaft_overlay, turn_shaft_overlay,
};
pub use shaft_rotation::{
    EarlierShaftPose, ElevatorAnimation, ElevatorLock, ShaftPlacement, ShaftTurn, TiltSample,
    TiltTrack, elevator_reached, turn_shaft, turn_shaft_rotation,
};
pub use shown::{FigureLength, FigureSequence, NextFigure, ShownFigure};
pub use station::{
    ColourSet, DoorColours, DoorPalette, Shake, ShakeLayers, ShakeMotion, ShakeStart, Stage,
};
pub use steam::{FirstPuffWait, MountedVent, Vent, VentPhase, run_vents};
pub use tail::{TAIL_DAMAGE, TailCurl};
pub use takeoff::{TakeOffFigure, TimedTakeOffFigure};
pub use types::ceres_fight::{RibLook, WingBeat};
pub use types::ceres_getaway::{
    FlybyBabyBeat, FlybyBeats, FlybyProgress, FlybyTurn, FlybyWingBeat, GetawayState, OutOfTheWay,
};
pub use types::ceres_shaft::{
    BurstPoint, BurstWait, RumbleTurns, ShaftOverlay, ShaftOverlayPhase, ShaftRotation, ShaftView,
    TiltDirection, VentDrawOffset,
};
pub use types::ceres_warning::{
    HeadingHold, StrokeBeat, SubtitleHold, TypingWait, WARNING_GLOW_LOOKS, WarningArtProgress,
    WarningGlow, WarningHold, WarningPhase, WarningState, WarningTypingProgress,
};
pub use waking::{
    BodyColours, EyeColours, FadeSets, ReadyWait, RemainingTurns as RunsLeft, RidleyContent,
    RoarWait,
};
pub use warning::{
    CountdownStart, EscapeActivation, EscapeTrack, WarningEffects, WarningInputs, WarningStroke,
    step_warning,
};
pub use warning_picture::{
    PositionedGlyph, WarningArtUpdate, WarningColourPair, WarningColourWrite, WarningContent,
    WarningEnglishCell, WarningForeground, WarningPalette, WarningPictureUpdates, WarningSheet,
    WarningTextWrite,
};
