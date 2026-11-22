//! The scoped ordinary shot owner joins current fire, door runs and following presentation.

mod continuation;
pub use continuation::{Completion, Content, Continuation, DrawOp, DrawStore, Environment, Inputs};

use super::{
    borrowed::ActorEligibilityOrigin,
    borrowed_player_preparation::{InputBindings, PoseInputCompletion},
    borrowed_preparation::ActorEligibilityReceipt,
    frame_history,
    pose_input::Configuration,
    random::{RandomSeed, Sequence},
};
use crate::production::{
    MovementContent, MovementInputs, ProductionFault, Unsupported, camera_semantic,
    entries::ceres_advance,
    gameplay_sounds::{GameplaySounds, Offer},
    movement, weapons,
};
use crate::{state::video::NmiBatch, weapons_projectiles_bridge::PowerContent};
use audio_transport::{CueId, QueueAllowance};
use boss_ceres_ridley::{CeresDoor, Inputs as CeresInputs, OpeningSound, Run, Stage};
use data::assets::samus::{key::Dress, map};
use types::{Action, Direction, FiringDirection, Pair, Parts, Pose, button};
use weapons_projectiles as beam;

/// One independently imported ordinary origin at a supported current shot opportunity.
#[derive(Clone, Debug)]
pub struct Origin {
    /// The entire carried projectile pool before its current update.
    pub shots: beam::State,
    /// Current pose from the retained player.
    pub pose: Pose,
    /// The previous pose that would supply the fallback muzzle.
    pub prior_pose: Pose,
    /// Current horizontal base speed read by the pose chart.
    pub horizontal_speed: types::Fixed,
    /// Previous held and previous newly pressed physical words.
    pub previous_pad: [u16; 2],
    /// Current configured physical actions.
    pub bindings: InputBindings,
    /// Current pre-motion geometry and imported cannon descriptors.
    pub weapon: beam::UpdateInput,
    /// Checked reached terrain.
    pub room: weapons::QuietBeamWorld,
    /// Current ordered door owners.
    pub doors: [CeresDoor; 2],
    /// Current contact permissions and geometry of the same owners.
    pub contacts: [weapons::DoorOpportunity; 2],
    /// Current roster predicates from the same actor origin.
    pub roster: ActorEligibilityOrigin,
    /// Current Ceres producer dependencies.
    pub door_inputs: CeresInputs,
    /// Shared sequence before the current draw.
    pub random: RandomSeed,
}

impl Origin {
    fn same_immutable(&self, other: &Self) -> bool {
        self.bindings == other.bindings
            && self.room == other.room
            && self.contacts == other.contacts
            && self.roster == other.roster
            && self.door_inputs.stage == other.door_inputs.stage
            && self.door_inputs.random == other.door_inputs.random
            && self.door_inputs.shake_frames_left == other.door_inputs.shake_frames_left
            && self.weapon.mode == other.weapon.mode
            && self.weapon.time_stopped == other.weapon.time_stopped
            && self.weapon.room == other.weapon.room
            && self.weapon.cannon.shoot == other.weapon.cannon.shoot
            && self.weapon.cannon.new_press == other.weapon.cannon.new_press
            && self.weapon.cannon.previous_new_press == other.weapon.cannon.previous_new_press
            && self.doors.iter().zip(other.doors).all(|(left, right)| {
                left.kind == right.kind
                    && left.facing == right.facing
                    && left.position == right.position
            })
    }
}

impl PartialEq for Origin {
    fn eq(&self, other: &Self) -> bool {
        let left = &self.weapon;
        let right = &other.weapon;
        self.shots == other.shots
            && self.pose == other.pose
            && self.prior_pose == other.prior_pose
            && self.horizontal_speed == other.horizontal_speed
            && self.previous_pad == other.previous_pad
            && self.bindings == other.bindings
            && self.room == other.room
            && self.doors == other.doors
            && self.contacts == other.contacts
            && self.roster == other.roster
            && self.door_inputs == other.door_inputs
            && self.random == other.random
            && left.mode == right.mode
            && left.time_stopped == right.time_stopped
            && left.inheritance == right.inheritance
            && left.camera == right.camera
            && left.room == right.room
            && same_cannon(&left.cannon, &right.cannon)
    }
}

impl Eq for Origin {}

fn same_cannon(left: &beam::CannonInput, right: &beam::CannonInput) -> bool {
    left.shoot == right.shoot
        && left.new_press == right.new_press
        && left.previous_new_press == right.previous_new_press
        && left.previous_muzzle == right.previous_muzzle
        && left.moving_muzzle == right.moving_muzzle
        && left.pending_aim == right.pending_aim
        && left.centre == right.centre
        && left.muzzle.aim == right.muzzle.aim
        && left.muzzle.lift == right.muzzle.lift
        && left.muzzle.stationary == right.muzzle.stationary
        && left.muzzle.moving == right.muzzle.moving
}

/// Live results through the actor handoff before the next movement owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AfterActors {
    /// Actual selected deferred placement. Its dependency remains the current muzzle pose.
    pub pose: PoseInputCompletion,
    /// The current table selected a pose application, possibly repeating the current pose.
    pub pose_requested: bool,
    /// Physical previous-new word used by the weapon branch.
    pub previous_pressed: u16,
    /// Actual complete W1 state, including in-frame contact marks.
    pub shots: beam::State,
    /// Actual events and per-door contacts in order.
    pub weapons: weapons::WeaponsOutputs,
    /// Current actors after the two actual production runs.
    pub doors: [CeresDoor; 2],
    /// Complete outputs at the real actor runs.
    pub runs: [Run; 2],
    /// Actual carried result of this opportunity's Random operation.
    pub random: RandomSeed,
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::ShotOpportunity)
}

fn accepts(origin: &Origin, roster: &ActorEligibilityReceipt) -> bool {
    let [first, second] = &origin.contacts;
    let [first_door, second_door] = origin.doors;
    let centre = origin.weapon.cannon.centre.0;
    roster.active.as_slice() == [first.actor, second.actor]
        && first.occurrence == weapons::ActorOccurrence(0)
        && second.occurrence == weapons::ActorOccurrence(1)
        && first.conditions.processing_membership
        && second.conditions.processing_membership
        && first_door.position != second_door.position
        && origin.door_inputs.stage == Stage::Calm
        && origin.door_inputs.her
            == Pair {
                x: core::num::Wrapping(centre.x),
                y: core::num::Wrapping(centre.y),
            }
        && origin.weapon.mode == beam::UpdateMode::CannonAndShots
        && !origin.weapon.time_stopped
}

fn current_contact(
    mut opportunity: weapons::DoorOpportunity,
    door: CeresDoor,
    roster: &ActorEligibilityReceipt,
) -> weapons::DoorOpportunity {
    opportunity.door = if !roster.interactive.contains(&opportunity.actor) || !door.phase.solid() {
        enemy_interactions::DoorContact::Passable
    } else if door.figure().is_none() {
        enemy_interactions::DoorContact::WithoutFigure
    } else {
        let hitbox = door.hitbox();
        enemy_interactions::DoorContact::Box(enemy_interactions::ContactBox {
            centre: enemy_interactions::Centre {
                x: hitbox.centre.x.0,
                y: hitbox.centre.y.0,
            },
            radii: enemy_interactions::Radii {
                x: hitbox.radius.x.0,
                y: hitbox.radius.y.0,
            },
        })
    };
    opportunity
}

/// Run the mapped standing input, current W1 and the interleaved C1/door joins.
///
/// # Errors
/// Rejects an unsupported current pose, weapon or actor relation before returning live results.
fn advance(
    origin: &Origin,
    sampled: u16,
    movement: &MovementContent,
    power: &PowerContent,
    mut audio: Option<&mut GameplaySounds>,
) -> Result<(AfterActors, Vec<Offer>), ProductionFault> {
    let roster = ActorEligibilityReceipt::marked_offscreen(&origin.roster).ok_or_else(missing)?;
    if !accepts(origin, &roster) {
        return Err(missing());
    }
    let [previous_held, previous_pressed] = origin.previous_pad;
    let pressed = frame_history::newly_pressed(sampled, previous_held);
    let pose = Configuration::new(origin.bindings)
        .ordinary_shot(
            origin.pose,
            [sampled, pressed],
            origin.horizontal_speed,
            movement,
        )
        .ok_or_else(missing)?;
    let mut weapon = origin.weapon.clone();
    let cannon = &mut weapon.cannon;
    cannon.shoot = pose.terminal.held_actions.0 & button::SHOOT.0 != 0;
    cannon.new_press = pose.terminal.pressed_actions.0 & button::SHOOT.0 != 0;
    cannon.previous_new_press = previous_pressed & origin.bindings.shoot.0 != 0;
    let mut shots = origin.shots.clone();
    let events = weapons::update(
        &mut shots,
        &weapon,
        power,
        beam::Rules::REFERENCE,
        &origin.room,
    )?;
    let mut offers = Vec::new();
    if let Some(audio) = audio.as_deref_mut() {
        for event in &events {
            match event {
                beam::Event::FireSound(_) => {
                    offers.push(audio.offer(CueId::Beam, QueueAllowance::Fifteen)?);
                }
                beam::Event::ImpactSound(_) => {
                    return Err(ProductionFault::Unsupported(Unsupported::WeaponImpactSound));
                }
                beam::Event::Protection(_)
                | beam::Event::PendingAimConsumed
                | beam::Event::Trail { .. } => {}
            }
        }
    }
    let mut sequence = Sequence::new(origin.random);
    sequence.draw();
    let mut door_inputs = origin.door_inputs;
    door_inputs.random = sequence.current().0;
    let [first, second] = origin.contacts;
    let mut doors = origin.doors;
    let [first_door, second_door] = &mut doors;
    let rules = boss_ceres_ridley::Rules::default();
    let first_contact = weapons::contact(&mut shots, current_contact(first, *first_door, &roster));
    let first_run = ceres_advance(first_door, &door_inputs, &rules);
    if let (Some(audio), Some(OpeningSound)) = (audio.as_deref_mut(), first_run.sound) {
        offers.push(audio.offer(CueId::Door, QueueAllowance::Six)?);
    }
    let second_contact =
        weapons::contact(&mut shots, current_contact(second, *second_door, &roster));
    let second_run = ceres_advance(second_door, &door_inputs, &rules);
    if let (Some(audio), Some(OpeningSound)) = (audio, second_run.sound) {
        offers.push(audio.offer(CueId::Door, QueueAllowance::Six)?);
    }
    let pose_requested = pose.terminal.prospective != origin.pose
        || (origin.pose.action == Action::Running
            && pose.terminal.held_actions == button::RUN
            && origin.horizontal_speed != types::ZERO);
    Ok((
        AfterActors {
            pose,
            pose_requested,
            previous_pressed,
            shots,
            doors,
            weapons: weapons::WeaponsOutputs {
                events,
                contacts: vec![first_contact, second_contact],
            },
            runs: [first_run, second_run],
            random: sequence.current(),
        },
        offers,
    ))
}

/// Apply POST with the same mapped actions used for the earlier deferred choice.
///
/// # Errors
/// Rejects a mismatched current pose or an unsupported Movement decision.
fn place(
    actors: &AfterActors,
    samus: &mut samus_movement::Samus,
    mut input: MovementInputs,
    world: &dyn types::World,
    content: &MovementContent,
    rules: &samus_movement::Rules,
) -> Result<samus_movement::Outcome, ProductionFault> {
    if samus.pose != actors.pose.dependency.pose {
        return Err(missing());
    }
    input.pad = actors.pose.terminal.held_actions;
    input.pressed = actors.pose.terminal.pressed_actions;
    movement::advance_scoped(samus, content, rules, &input, world)
}

/// Follow the player that POST actually placed for the current presentation.
fn follow(
    camera: &mut world_camera::Camera,
    samus: &samus_movement::Samus,
    movement: &samus_movement::Outcome,
    room: &world_camera::Room,
    lead: world_camera::LeadSetting,
    rules: &world_camera::Rules,
) {
    camera_semantic::step(
        camera,
        rules,
        &world_camera::Frame {
            samus: camera_semantic::completed_samus(samus, movement),
            room,
            lead,
            time_stopped: false,
        },
    );
}

/// Insert actual flight pictures into the carried batch, then close out OAM.
///
/// # Errors
/// Rejects a drawing unsupported by the exact encoder.
fn draw(
    actors: &AfterActors,
    camera: &world_camera::Camera,
    phase: beam::FlickerPhase,
    batch: &NmiBatch,
    power: &PowerContent,
) -> Result<(weapons::ShotDrawingOutputs, u16), ProductionFault> {
    let camera = camera.position();
    let view = beam::DrawView::Normal(beam::ScreenOrigin(beam::Axes {
        x: camera.x.pixel().0,
        y: camera.y.pixel().0,
    }));
    let mut drawing = weapons::draw_flying(
        &actors.shots,
        actors.weapons.clone(),
        power,
        weapons::FlyingDraw {
            phase,
            view: &view,
            batch,
        },
    )?;
    let after_power = drawing.batch.sprites.0;
    drawing.batch.finalise_oam();
    Ok((drawing, after_power))
}

/// Carry the actual owners after POST and Camera into the next pre-motion opportunity.
///
/// # Errors
/// Rejects a pose whose cannon descriptor is outside imported content.
fn carry(
    origin: &mut Origin,
    actors: &AfterActors,
    samus: &samus_movement::Samus,
    movement: &samus_movement::Outcome,
    camera: &world_camera::Camera,
    dress: Dress,
    power: &PowerContent,
) -> Result<(), ProductionFault> {
    let prior_pose = if actors.pose_requested {
        origin.pose
    } else {
        origin.prior_pose
    };
    let muzzles = muzzles(samus.pose, prior_pose, dress, power)?;
    origin.prior_pose = prior_pose;
    origin.shots = actors.shots.clone();
    origin.doors = actors.doors;
    origin.random = actors.random;
    origin.previous_pad = [
        actors.pose.dependency.held.0,
        actors.pose.dependency.pressed.0,
    ];
    refresh(origin, samus, camera, muzzles);
    origin.weapon.cannon.pending_aim = movement.pending_aim.map(pending_heading);
    let distance = |dir| {
        let fixed = movement.travelled[dir].0;
        beam::MovementDistance {
            whole: fixed.pixel().0,
            fraction: beam::PixelFraction(fixed.sub()),
        }
    };
    origin.weapon.inheritance = beam::Inheritance {
        left: distance(Direction::Left),
        right: distance(Direction::Right),
        up: distance(Direction::Up),
        down: distance(Direction::Down),
        camera_vertical_fraction: beam::PixelFraction(camera.vertical_fraction().0),
    };
    Ok(())
}

fn muzzles(
    pose: Pose,
    prior_pose: Pose,
    dress: Dress,
    power: &PowerContent,
) -> Result<(beam::Muzzle, beam::MuzzleAim), ProductionFault> {
    let import = |pose| {
        let id = map::id_of(pose, dress).ok_or_else(missing)?;
        power.muzzle(id).map_err(ProductionFault::Weapons)
    };
    Ok((import(pose)?, import(prior_pose)?.aim))
}

fn refresh(
    origin: &mut Origin,
    samus: &samus_movement::Samus,
    camera: &world_camera::Camera,
    (muzzle, previous): (beam::Muzzle, beam::MuzzleAim),
) {
    origin.pose = samus.pose;
    origin.horizontal_speed = samus.run.speed;
    let centre = Pair {
        x: samus.position.x.pixel(),
        y: samus.position.y.pixel(),
    };
    origin.door_inputs.her = centre;
    let cannon = &mut origin.weapon.cannon;
    cannon.centre = beam::SamusCentre(beam::Axes {
        x: centre.x.0,
        y: centre.y.0,
    });
    cannon.previous_muzzle = previous;
    cannon.muzzle = muzzle;
    cannon.moving_muzzle = samus.pose.action == Action::Running;
    let view = camera.position();
    origin.weapon.camera = beam::ScreenOrigin(beam::Axes {
        x: view.x.pixel().0,
        y: view.y.pixel().0,
    });
}

const fn pending_heading(direction: FiringDirection) -> beam::Heading {
    match direction {
        FiringDirection::Left => beam::Heading::Left,
        FiringDirection::Right => beam::Heading::Right,
        FiringDirection::DownLeft => beam::Heading::DownLeft,
        FiringDirection::DownRight => beam::Heading::DownRight,
        FiringDirection::UpLeft => beam::Heading::UpLeft,
        FiringDirection::UpRight => beam::Heading::UpRight,
        FiringDirection::DownFacingLeft => beam::Heading::DownFacingLeft,
        FiringDirection::DownFacingRight => beam::Heading::DownFacingRight,
        FiringDirection::UpFacingLeft => beam::Heading::UpFacingLeft,
        FiringDirection::UpFacingRight => beam::Heading::UpFacingRight,
    }
}
