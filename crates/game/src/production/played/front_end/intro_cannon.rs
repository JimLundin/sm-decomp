//! Carry ordinary movement and movie controller history to the flashback's cannon.

use super::super::{intro::Fault, intro_missiles as missiles, intro_rooms as rooms};
use super::{Button, Pressed};
use interface_hud::{Ammo, SelectableItem, Selection};
use types::save_files::PadButton;
use types::{Action, Body, Direction, DirectionalTravels, FiringDirection, Parts, button};
use weapons_projectiles as w1;

/// The flashback has no scrolling game camera; its shake moves only the background.
pub(super) const VIEW: w1::ScreenOrigin = w1::ScreenOrigin(w1::Axes { x: 0, y: 0 });

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Cannon {
    travels: DirectionalTravels,
    pending: Option<FiringDirection>,
    previous_movie: Pressed,
    /// Protection from an incoming shot, independently of Samus's post-hit protection.
    pub(super) protection: u16,
}

impl Cannon {
    /// The cannon reads the actor and movement history as the current control turn begins.
    pub(super) fn inputs(
        &self,
        actor: &rooms::Actor,
        pad: rooms::DemoPad,
        shoot: PadButton,
        mode: w1::UpdateMode,
        content: &missiles::Content,
    ) -> Result<(w1::UpdateInput, bool), Fault> {
        let body = &actor.samus.movement.0;
        let muzzle = |pose| content.muzzle_for(pose).map_err(Fault::IntroWeapon);
        let distance = |direction: Direction| {
            let (whole, fraction) = self.travels[direction].0.words();
            w1::MovementDistance {
                whole,
                fraction: w1::PixelFraction(fraction),
            }
        };
        let binding = match shoot {
            PadButton::A => Button::A,
            PadButton::B => Button::B,
            PadButton::X => Button::X,
            PadButton::Y => Button::Y,
            PadButton::L => Button::L,
            PadButton::R => Button::R,
            PadButton::Select => Button::Select,
            PadButton::Unrecognized(_) => return Err(Fault::IntroControls),
        };
        Ok((
            w1::UpdateInput {
                mode,
                time_stopped: false,
                cannon: w1::CannonInput {
                    shoot: pad.held.any(button::SHOOT),
                    new_press: pad.pressed.any(button::SHOOT),
                    previous_new_press: self.previous_movie[binding],
                    muzzle: muzzle(body.pose)?,
                    previous_muzzle: muzzle(actor.carry.0.older_pose)?.aim,
                    moving_muzzle: body.pose.action == Action::Running,
                    pending_aim: self.pending.map(heading),
                    centre: w1::SamusCentre(w1::Axes {
                        x: body.position.x.pixel().0,
                        y: body.position.y.pixel().0,
                    }),
                },
                inheritance: w1::Inheritance {
                    left: distance(Direction::Left),
                    right: distance(Direction::Right),
                    up: distance(Direction::Up),
                    down: distance(Direction::Down),
                    camera_vertical_fraction: w1::PixelFraction(0),
                },
                camera: VIEW,
                // The opening retains the empty screen extents when it installs its block grid.
                room: w1::RoomScreens(w1::Axes { x: 0, y: 0 }),
            },
            body.pose.body == Body::Upright && body.pose.action == Action::Still,
        ))
    }

    pub(super) fn supplies(&self, actor: &rooms::Actor) -> missiles::Supplies {
        let selection = actor.carry.0.selection;
        missiles::Supplies {
            ammo: actor.carry.0.missiles,
            selected: selection.item() == Some(SelectableItem::Ammo(Ammo::Missiles)),
            cancel_after_shot: matches!(selection, Selection::OneShot(_)),
            protection: self.protection,
        }
    }

    pub(super) fn update(
        &mut self,
        actor: &mut rooms::Actor,
        input: &w1::UpdateInput,
        eligible: bool,
        content: &missiles::Content,
    ) -> Result<Vec<w1::MissileCandidate>, Fault> {
        let selection = actor.carry.0.selection;
        let mut supplies = self.supplies(actor);
        let mut world = Contacts::default();
        let result = missiles::update(
            &mut actor.shots.0,
            input,
            &mut supplies,
            eligible,
            content,
            w1::Rules::REFERENCE,
            &mut world,
        )
        .map_err(Fault::IntroWeapon)?;
        if world.requested {
            return Err(Fault::IntroShotContact);
        }
        if result.events.contains(&w1::Event::PendingAimConsumed) {
            self.pending = None;
        }
        actor.carry.0.missiles = supplies.ammo;
        if !supplies.selected && selection.item() == Some(SelectableItem::Ammo(Ammo::Missiles)) {
            actor.carry.0.selection = Selection::None;
        }
        self.protection = supplies.protection;
        Ok(result.candidates)
    }

    pub(super) fn moved(&mut self, outcome: &samus_movement::Outcome) {
        self.pending = outcome.pending_aim.or(self.pending);
        self.travels = outcome.travelled;
    }

    pub(super) const fn drawn(&mut self, movie: Pressed) {
        self.previous_movie = movie;
    }
}

/// The intro leaves room width in screens zero. Refuse a point or beam continuation if one
/// reaches the world before the real collision bridge is selected.
#[derive(Default)]
struct Contacts {
    requested: bool,
}

impl w1::World for Contacts {
    fn contact_cell(&mut self, _: &w1::Contact, _: u16) -> w1::ContactResponse {
        self.requested = true;
        w1::ContactResponse::Passes
    }
    fn contact_point(&mut self, _: w1::ShotSlot, _: w1::Axes<u16>) -> w1::ContactResponse {
        self.requested = true;
        w1::ContactResponse::Passes
    }
}

const fn heading(direction: FiringDirection) -> w1::Heading {
    match direction {
        FiringDirection::Left => w1::Heading::Left,
        FiringDirection::Right => w1::Heading::Right,
        FiringDirection::DownLeft => w1::Heading::DownLeft,
        FiringDirection::DownRight => w1::Heading::DownRight,
        FiringDirection::UpLeft => w1::Heading::UpLeft,
        FiringDirection::UpRight => w1::Heading::UpRight,
        FiringDirection::DownFacingLeft => w1::Heading::DownFacingLeft,
        FiringDirection::DownFacingRight => w1::Heading::DownFacingRight,
        FiringDirection::UpFacingLeft => w1::Heading::UpFacingLeft,
        FiringDirection::UpFacingRight => w1::Heading::UpFacingRight,
    }
}
