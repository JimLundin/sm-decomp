//! Her own frame: her clip's look ahead, her moves, what the frame plays between them and her
//! reaction (the frame's hit), then her appearance from what they report.
//!
//! Movement reports what animation reads "as her moves leave her, before the pose change", so
//! stepping movement whole and then animation gives the values the original reads between
//! them; no pause inside either model is needed.

use super::station::Contact;
use super::{Content, Fault, Rules};
use crate::production::animation;
use crate::production::connected::inputs::Inputs;
use crate::state::owned::{AppearancePin, MovementPin};
use canonical::Canonical;
use canonical::model::{Model, Pinned};
use core::cell::Cell;
use core::num::Wrapping;
use samus_appearance::{
    Ahead, Dash, Figure, FramePart, GrappleActivity, HurtOpportunity, MissingClip, MoveEvent,
    Playhead, Step, Suit,
};
use samus_movement::{
    Dashing, DrawnJumpHistory, JumpButton, JumpHeld, StandingAnimation, WallTest,
};
use types::{Parts, Pose, Px, Set};

pub use crate::production::movement::FigurePin;

/// The previous draw button stored in a checkpoint.
pub struct JumpButtonPin;

impl Pinned for JumpButtonPin {
    type State = JumpButton;
    const NAME: &'static str = "played-samus-jump-button";
    const SHAPE: u64 = 0x2A2A_0720_FF8D_F79B;
}

/// The complete accepted travel interval that the next firing opportunity reads.
pub struct TravelPin;

impl Pinned for TravelPin {
    type State = samus_movement::TravelledDistances;
    const NAME: &'static str = "played-samus-travelled-distances";
    const SHAPE: u64 = 0x614F_299D_A431_20A0;
}

/// Her movement, her animation and the figure she shows, carried frame to frame.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Her movement.
    pub movement: Model<MovementPin>,
    /// Her whole appearance State.
    pub appearance: Model<AppearancePin>,
    /// The figure she is drawn in.
    pub figure: Model<FigurePin>,
    /// The jump button sampled on the last draw that wrote her jump history.
    pub last_draw_jump: Model<JumpButtonPin>,
    /// Her latest completed movement interval, preserved on frames without movement.
    pub travelled: Model<TravelPin>,
}

impl State {
    /// While the pad holds her, her frame changes nothing of her.
    pub(super) const fn held(&self) -> Her {
        Her {
            samus: self.movement.0,
            outcome: None,
            push_at_draw: None,
            appearance: self.appearance.0,
            figure: self.figure.0,
            colours: None,
            sounds: Vec::new(),
        }
    }

    /// Her State after a frame of play: `samus` as the frame leaves her, and what `her` frame
    /// shows.
    pub(super) const fn played(
        &mut self,
        samus: samus_movement::Samus,
        her: &Her,
        jump: JumpButton,
    ) {
        self.movement = Model(samus);
        self.appearance = Model(her.appearance);
        self.figure = Model(her.figure);
        if let Some(outcome) = &her.outcome {
            self.last_draw_jump.0 = jump;
            self.travelled.0.travels = outcome.travelled;
        }
    }
}

/// The jump button sampled on a history-writing draw.
pub(super) const fn jump_button(held: types::Buttons) -> JumpButton {
    if held.any(types::button::JUMP) {
        JumpButton::Held
    } else {
        JumpButton::Up
    }
}

/// External inputs read by movement and appearance in an ordinary turn.
#[derive(Clone, Copy)]
pub(super) struct TurnInputs {
    pub held: types::Buttons,
    pub pressed: types::Buttons,
    pub items: Set<types::Item>,
    pub parity: types::Parity,
    pub energy: u16,
    pub liquid: samus_appearance::Liquid,
    pub sounds_waiting: samus_appearance::SoundsWaiting,
    pub activities: Set<samus_appearance::Activity>,
    pub charge: samus_appearance::Charge,
    pub item_controls: samus_appearance::ItemControls,
    pub view_left: Px,
    pub getaway_slowdown: samus_movement::Slowdown,
    pub getaway_carry: types::Fixed,
}

impl From<&Inputs> for TurnInputs {
    fn from(inputs: &Inputs) -> Self {
        Self {
            held: inputs.held,
            pressed: inputs.pressed,
            items: inputs.items,
            parity: inputs.parity,
            energy: inputs.energy,
            liquid: inputs.liquid,
            sounds_waiting: inputs.sounds_waiting,
            activities: inputs.activities,
            charge: inputs.charge,
            item_controls: samus_appearance::ItemControls::Skipped,
            view_left: Wrapping(0),
            // The played Ceres rooms have no external horizontal slowdown or carry.
            getaway_slowdown: samus_movement::Slowdown::NONE,
            getaway_carry: types::ZERO,
        }
    }
}

/// Imported content and tuning for the ordinary movement and appearance turn.
#[derive(Clone, Copy)]
pub struct TurnContent<'a> {
    /// Her selected pose bodies and cannon headings.
    pub movement: &'a super::MovementContent,
    /// The selected playhead clips and visual content.
    pub appearance: &'a super::AppearanceContent,
    /// Movement tuning.
    pub movement_rules: &'a samus_movement::Rules,
    /// Appearance tuning.
    pub appearance_rules: &'a samus_appearance::Rules,
}

/// A side-free pass requested by the boss after Samus has completed her ordinary turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GetawayPass;

impl GetawayPass {
    /// Arm the next movement turn while keeping the completed turn's reports.
    pub const fn arm(self, movement: &mut samus_movement::Samus) {
        movement.request_getaway();
    }
}

/// Neighbours for a scoped Samus getaway turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GetawayInputs {
    /// Camera's left edge at the start of the turn.
    pub view_left: Px,
    /// One pass at the end of this turn, when supplied by the boss.
    pub pass: Option<GetawayPass>,
    /// Horizontal speed reduction supplied at the start of the turn.
    pub slowdown: samus_movement::Slowdown,
    /// The room's signed sideways carry for this turn.
    pub carry: types::Fixed,
}

/// Update the count and previous-draw latch while active movement and pose choice are paused.
pub(in crate::production) fn passage_jump_draw(
    held: &mut JumpHeld,
    last_draw: &mut JumpButton,
    jump: JumpButton,
) {
    let mut history = DrawnJumpHistory {
        jump_held: *held,
        last_draw: *last_draw,
    };
    history.draw(jump);
    *held = history.jump_held;
    *last_draw = history.last_draw;
}

impl super::State {
    /// Her own frame: her moves, then what `between` plays once they are made, given her
    /// position after them, giving the frame's hit and the energy it leaves her; then her
    /// reaction to that hit, and her appearance from what both steps report. The played frame's
    /// `between` ignores the position: its hitters read her as the frame starts.
    pub(super) fn her_frame<T>(
        &self,
        inputs: &Inputs,
        content: &Content,
        rules: &Rules,
        between: impl FnOnce(types::Position) -> Result<Between<T>, Fault>,
    ) -> Result<(Her, T), Fault> {
        let world = self.station.world(&self.passage.room.geometry);
        self.samus.turn(
            TurnInputs {
                energy: self.health.0.energy.whole.get(),
                view_left: self.camera.0.position().x.pixel(),
                ..inputs.into()
            },
            &world,
            TurnContent {
                movement: &content.movement,
                appearance: &content.appearance,
                movement_rules: &rules.movement,
                appearance_rules: &rules.appearance,
            },
            between,
        )
    }
}

impl State {
    /// Play her complete movement and appearance turn with a scoped boss and camera input.
    /// Actor contact is outside this scope. A pass is applied after the ordinary turn.
    ///
    /// # Errors
    /// Missing content or malformed geometry refuses the turn before State changes.
    pub fn step_getaway(
        &mut self,
        inputs: &Inputs,
        getaway: GetawayInputs,
        doors: &[boss_ceres_ridley::CeresDoor],
        content: TurnContent<'_>,
    ) -> Result<samus_movement::Outcome, Fault> {
        inputs.geometry.validate().map_err(|_| Fault::Geometry)?;
        let world = super::ceres_doors::WithDoors::new(&inputs.geometry, doors);
        let (her, ()) = self.turn(
            TurnInputs {
                view_left: getaway.view_left,
                getaway_slowdown: getaway.slowdown,
                getaway_carry: getaway.carry,
                ..inputs.into()
            },
            &world,
            content,
            |_| {
                Ok(Between {
                    played: (),
                    hit: None,
                    energy: inputs.energy,
                })
            },
        )?;
        let mut moved = her.samus;
        if let Some(pass) = getaway.pass {
            pass.arm(&mut moved);
        }
        let outcome = her.outcome.clone().ok_or(Fault::Geometry)?;
        self.played(moved, &her, jump_button(inputs.held));
        Ok(outcome)
    }

    pub(super) fn turn<T>(
        &self,
        inputs: TurnInputs,
        world: &dyn types::World,
        content: TurnContent<'_>,
        between: impl FnOnce(types::Position) -> Result<Between<T>, Fault>,
    ) -> Result<(Her, T), Fault> {
        let start = self.figure.0;
        let (samus, outcome, between) = self.movement(inputs, world, &content, between)?;
        let push_at_draw = (self.movement.0.getaway == samus_movement::Getaway::Idle).then(|| {
            between
                .hit
                .map_or(self.movement.0.knockback.time_left, |hit| hit.push().push)
        });
        let after = Figure {
            pose: samus.pose,
            sprite: outcome.sprite,
        };
        let mut appearance = self.appearance.0;
        let mut seen = turn_animation_inputs(start, after, &outcome, inputs);
        // Hurt colours read the hit's energy change before the frame's periodic loss.
        seen.energy = between.energy;

        // The loop refuses her other suits, and no grapple is in it.
        let hurt = HurtOpportunity::Observed {
            suit: Suit::Power,
            grapple: GrappleActivity::Dormant,
        };
        let mut frame = animation::frame_of_play(seen, hurt);
        frame.item_controls = inputs.item_controls;
        if outcome.deferred_motion == samus_movement::DeferredMotion::Clear {
            // The forced recoil pose is selected before this cut's clip visit. Preparation
            // has already seen the old figure, which drawing still uses for the spin end.
            let hold = content
                .appearance
                .get(after)
                .and_then(|clip| clip.hold(Step::FIRST))
                .ok_or(Fault::Clip(after))?;
            appearance.kept_action = start.pose.action;
            appearance.animation.playhead = Playhead {
                step: Step::FIRST,
                time_left: hold.frames().slowed(inputs.liquid.start_slowdown),
            };
            frame.parts[FramePart::Preparation] = false;
            frame.inputs.figure = after;
        }
        let shown = animation::play(
            &mut appearance,
            content.appearance,
            content.appearance_rules,
            &frame,
        )
        .map_err(|fault| match fault {
            animation::ClipFault::Missing(figure) | animation::ClipFault::Conflicting(figure) => {
                Fault::Clip(figure)
            }
        })?;
        Ok((
            Her {
                samus,
                outcome: Some(outcome),
                push_at_draw,
                appearance,
                figure: after,
                colours: shown.her_colours,
                sounds: shown.sounds,
            },
            between.played,
        ))
    }

    /// Her movement's two steps: her moves, after her clip's look ahead from the figure she
    /// starts the frame in; `between`, given her position after them, giving the frame's hit
    /// (the played frame judges the hit before her moves and ignores the position); then her
    /// reaction to it.
    fn movement<T>(
        &self,
        inputs: TurnInputs,
        world: &dyn types::World,
        content: &TurnContent<'_>,
        between: impl FnOnce(types::Position) -> Result<Between<T>, Fault>,
    ) -> Result<(samus_movement::Samus, samus_movement::Outcome, Between<T>), Fault> {
        let start = self.figure.0;
        let ahead = Ahead::of(&self.appearance.0, start, content.appearance)
            .map_err(|MissingClip(figure)| Fault::Clip(figure))?;
        let mut samus = self.movement.0;
        let missing_body = Cell::new(None);
        let body = |pose: Pose| {
            content.movement.radius(pose).unwrap_or_else(|| {
                missing_body.set(Some(pose));
                types::Pair {
                    x: Wrapping(0),
                    y: Wrapping(0),
                }
            })
        };
        let heading = |pose| content.movement.cannon_heading(pose);
        let frame = samus_movement::Frame {
            sprite_before_movement: start.sprite,
            cinematic_active: inputs.activities[samus_appearance::Activity::Cinematic],
            pad: inputs.held,
            pressed: inputs.pressed,
            items: inputs.items,
            spin_turned_far: ahead.turned_far,
            elevator_moving: false,
            parity: inputs.parity,
            animation_ends: ahead.end,
            view_left: inputs.view_left,
            getaway_slowdown: inputs.getaway_slowdown,
            getaway_carry: inputs.getaway_carry,
            body: &body,
            cannon_heading: &heading,
        };
        let moves = samus
            .moves(&frame, world, content.movement_rules)
            .map_err(|samus_movement::UnknownPose(pose)| Fault::Pose(pose))?;
        let between = between(moves.position_after_moves())?;
        let outcome =
            samus.react_after_draw(moves, between.hit.map(Contact::push), self.last_draw_jump.0);
        if let Some(pose) = outcome.pose_change
            && samus.getaway != samus_movement::Getaway::Idle
            && content.movement.radius(pose).is_none()
        {
            return Err(Fault::Body(pose));
        }
        missing_body
            .get()
            .map_or(Ok((samus, outcome, between)), |pose| Err(Fault::Body(pose)))
    }
}

/// What a frame plays between her moves and her reaction: the frame's hit, the whole energy it
/// leaves her before the frame's loss, and the rest of what it played.
pub(super) struct Between<T> {
    /// What else it played.
    pub(super) played: T,
    /// The frame's hit, which her reaction reads.
    pub(super) hit: Option<Contact>,
    /// Her whole energy as the hit leaves it, which her appearance reads.
    pub(super) energy: u16,
}

/// Her own frame's results: her movement, its outcome, her appearance, the figure she shows, the
/// colours her row takes and the sounds it asks for.
pub(super) struct Her {
    pub(super) samus: samus_movement::Samus,
    /// None while the pad holds her.
    pub(super) outcome: Option<samus_movement::Outcome>,
    // Movement spends this time before returning the settled state.
    pub(super) push_at_draw: Option<types::PushFrames>,
    appearance: samus_appearance::AppearanceState,
    figure: Figure,
    /// The colours her row of the screen's colours takes; none leaves it as it is.
    pub(super) colours: Option<types::RowColours>,
    pub(super) sounds: Vec<samus_appearance::SoundOffer>,
}

pub(super) const fn standing_pose(side: types::Side) -> Pose {
    Pose {
        body: types::Body::Upright,
        action: types::Action::Still,
        facing: match side {
            types::Side::Left => types::Facing::Left,
            types::Side::Right => types::Facing::Right,
        },
        aim: types::Aim::Forward,
        gun: types::Gun::Stowed,
        legs: types::Legs::Together,
    }
}

impl Her {
    pub(super) fn stand_at_elevator(
        &mut self,
        samus: &mut samus_movement::Samus,
        reach: boss_ceres_ridley::ElevatorLock,
        appearance: &super::AppearanceContent,
    ) -> Result<(), Fault> {
        let pose = standing_pose(reach.standing);
        let figure = Figure {
            pose,
            sprite: types::Sprite::Plain,
        };
        if reach.animation == boss_ceres_ridley::ElevatorAnimation::StartStanding {
            let hold = appearance
                .get(figure)
                .and_then(|clip| clip.hold(Step::FIRST))
                .ok_or(Fault::Clip(figure))?;
            self.appearance.animation.playhead = Playhead {
                step: Step::FIRST,
                time_left: hold.frames(),
            };
        }
        samus.pose = pose;
        self.figure = figure;
        Ok(())
    }

    pub(super) fn body_picture(
        &self,
        art: &data::assets::samus::SamusAssets,
        camera: types::Position,
        rotation: Option<crate::state::video::Mode7>,
    ) -> Result<crate::intent::SpriteDraw, Fault> {
        crate::exact::samus::draw(
            art,
            self.figure,
            self.appearance.animation.playhead.step,
            crate::exact::samus::project(self.samus.position, camera, rotation),
        )
        .ok_or(Fault::SamusBody(self.figure))
    }

    pub(super) fn tiles(
        &self,
        content: &super::Content,
    ) -> Result<Option<crate::state::video::SamusTileUploads>, Fault> {
        content
            .samus_tiles
            .as_ref()
            .filter(|_| self.outcome.is_some())
            .map(|(tiles, _)| {
                super::samus_tiles::uploads(
                    tiles,
                    self.figure,
                    self.appearance.animation.playhead.step,
                )
            })
            .transpose()
    }

    /// Where her moves leave her, which the mini-map reads.
    pub(super) fn after_moves(&self) -> types::Position {
        self.outcome
            .as_ref()
            .map_or(self.samus.position, |outcome| outcome.position_after_moves)
    }
}

/// What animation reads of this frame's moves.
pub(in crate::production) fn animation_inputs(
    start: Figure,
    after: Figure,
    outcome: &samus_movement::Outcome,
    inputs: &Inputs,
) -> samus_appearance::Inputs {
    turn_animation_inputs(start, after, outcome, inputs.into())
}

fn turn_animation_inputs(
    start: Figure,
    after: Figure,
    outcome: &samus_movement::Outcome,
    inputs: TurnInputs,
) -> samus_appearance::Inputs {
    let mut moves = Set::default();
    moves[MoveEvent::StandingRestart] = outcome.standing_animation == StandingAnimation::Restarts;
    moves[MoveEvent::WallJumpReadied] = outcome.wall_test == WallTest::Readied;
    moves[MoveEvent::HandedOver] = outcome.handed_over;
    moves[MoveEvent::PushStarted] = outcome.push_started;
    samus_appearance::Inputs {
        figure: start,
        figure_after: after,
        moves,
        dash: dash(
            outcome.dashing_after_moves,
            outcome.dash_began,
            outcome.dash_lost,
        ),
        run_held: inputs.held.any(types::button::RUN),
        heading_down: outcome.heading_down,
        vertical_speed: outcome.vertical_speed_after_moves,
        energy: inputs.energy,
        previous_energy: inputs.energy,
        items: inputs.items,
        liquid: inputs.liquid,
        sounds_waiting: inputs.sounds_waiting,
        activities: inputs.activities,
        shoot_held: inputs.held.any(types::button::SHOOT),
        charge: inputs.charge,
    }
}

/// The dash as the clip moves on, from movement's report of her moves.
///
/// Whether she is dashing after her moves decides first: movement reports a dash begun into a
/// wall as begun and lost with her not dashing after her moves, and the original cancels it before
/// the clip moves on, so it is stopped.
const fn dash(dashing_after_moves: Dashing, began: bool, lost: bool) -> Dash {
    match (dashing_after_moves, began, lost) {
        (Dashing::Off, _, true) => Dash::Stopped,
        (Dashing::Off, _, false) => Dash::Off,
        (Dashing::On, true, true) => Dash::BeganAndLost,
        (Dashing::On, true, false) => Dash::Began,
        (Dashing::On, false, true) => Dash::Lost,
        (Dashing::On, false, false) => Dash::On,
    }
}

#[cfg(test)]
mod tests {
    use super::{Between, Contact, dash};
    use crate::production::played::tests::{content_with, inputs, rules, state};
    use data::assets::art::{Colour, Palette};
    use data::assets::samus::palettes::SamusPalettes;
    use samus_appearance::Dash;
    use samus_movement::{Dashing, Hit};
    use types::{Action, PushFrames, Side};

    /// A hit on her frame throws her back and pulses her hurt colours, and she plays on through
    /// the push's next frame: every pose the push takes her into has its body and its clip.
    #[test]
    fn a_hit_throws_her_back_in_her_hurt_colours() {
        let palettes = SamusPalettes {
            hurt: Palette(core::array::from_fn(|place| {
                Colour(0x1000 + u16::try_from(place).unwrap())
            })),
            ..SamusPalettes::default()
        };
        let content = content_with(&palettes);
        let hurt = crate::production::pulse_colours(&palettes).unwrap().hurt;
        let mut state = state(Vec::new());
        let frame_inputs = inputs(types::Buttons::default());
        let hit = Hit {
            side: Side::Left,
            push: PushFrames::new(5).unwrap(),
        };
        let (her, ()) = state
            .her_frame(&frame_inputs, &content, &rules(), |_| {
                Ok(Between {
                    played: (),
                    hit: Some(Contact::Tail(hit)),
                    energy: 84,
                })
            })
            .unwrap();
        assert!(her.outcome.as_ref().unwrap().push_started);
        assert_eq!(her.samus.knockback.time_left, PushFrames::new(4).unwrap());
        assert_eq!(her.samus.knockback.side, Side::Left);
        assert!(her.samus.knockback.push.is_some());
        assert_eq!(her.samus.pose.action, Action::Recoiling);
        assert_eq!(her.figure.pose, her.samus.pose);
        assert_eq!(her.appearance.hurt_count.frames(), 2);
        assert_eq!(her.colours, Some(hurt));

        state
            .samus
            .played(her.samus, &her, samus_movement::JumpButton::Up);
        let (next, ()) = state
            .her_frame(&frame_inputs, &content, &rules(), |_| {
                Ok(Between {
                    played: (),
                    hit: None,
                    energy: 99,
                })
            })
            .unwrap();
        assert_eq!(next.samus.knockback.time_left, PushFrames::new(3).unwrap());
        assert_eq!(next.samus.pose.action, Action::Recoiling);
        assert_eq!(next.appearance.hurt_count.frames(), 3);
    }

    /// Each report of her moves gives the dash the clip reads, decided first by whether she is
    /// dashing after them.
    #[test]
    fn dash_follows_her_dashing_after_moves() {
        let cases = [
            (Dashing::Off, false, true, Dash::Stopped),
            (Dashing::Off, true, false, Dash::Off),
            (Dashing::Off, false, false, Dash::Off),
            (Dashing::On, true, true, Dash::BeganAndLost),
            (Dashing::On, true, false, Dash::Began),
            (Dashing::On, false, true, Dash::Lost),
            (Dashing::On, false, false, Dash::On),
        ];
        for (dashing, began, lost, expected) in cases {
            assert_eq!(
                dash(dashing, began, lost),
                expected,
                "{dashing:?}, began {began}, lost {lost}"
            );
        }
    }

    /// A run into a wall that starts her dash leaves her not dashing after her moves: stopped, so
    /// the clip holds its own pace.
    #[test]
    fn dash_begun_into_a_wall_is_stopped() {
        assert_eq!(dash(Dashing::Off, true, true), Dash::Stopped);
    }
}

#[cfg(test)]
mod getaway_tests;
