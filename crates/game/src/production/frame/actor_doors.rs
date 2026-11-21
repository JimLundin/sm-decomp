//! Carry the ordinary Ceres doors through their qualified `ActorUpdate` opportunities.

use super::borrowed_player_preparation::{
    ActorContentDisposition, ActorGeometry, ActorScene, ActorSeed, ActorSolidity, ActorTouch,
    ActorUnselectedOrigin, ActorUpdateCompletion, ActorViewDisposition, AdministrativeCounter,
    CurrentActor, GraphicsUpdated,
};
use super::borrowed_preparation::{ActorEligibilityReceipt, ActorSlotIdentity};
use boss_ceres_ridley::{CeresDoor, DoorKind, Inputs, Phase, Rules, Stage, Wait};
use canonical::Canonical;
use core::num::Wrapping;
use types::{Pair, Parts, Px, Side};

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Figure {
    Open,
    Closed,
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Facing {
    Left,
    Right,
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
struct Body {
    position: [u32; 2],
    radius: [u16; 2],
    content: ActorContentDisposition,
    touch: ActorTouch,
    solidity: ActorSolidity,
    invisible: bool,
}

impl Body {
    const fn new(seed: &ActorSeed, invisible: bool) -> Self {
        let geometry = seed.geometry;
        Self {
            position: [geometry.position.x.0, geometry.position.y.0],
            radius: [geometry.radius.x.0, geometry.radius.y.0],
            content: seed.content,
            touch: seed.touch,
            solidity: seed.solidity,
            invisible,
        }
    }

    const fn geometry(self) -> ActorGeometry {
        let [x, y] = self.position;
        let [rx, ry] = self.radius;
        ActorGeometry {
            position: Pair {
                x: Wrapping(x),
                y: Wrapping(y),
            },
            radius: Pair {
                x: Wrapping(rx),
                y: Wrapping(ry),
            },
        }
    }

    fn centre(self) -> Pair<Px> {
        let position = self.geometry().position;
        Pair {
            x: position.x.pixel(),
            y: position.y.pixel(),
        }
    }

    fn view(self, camera: Pair<u16>) -> Option<ActorViewDisposition> {
        let centre = self.centre();
        let x = relative(centre.x.0, camera.x);
        let y = relative(centre.y.0, camera.y);
        let radius = i32::from(self.geometry().radius.x.0);
        if x < radius.wrapping_neg() || x > radius.wrapping_add(256) || y < -8 {
            return None;
        }
        if y > 248 {
            Some(ActorViewDisposition::BelowPaddedView)
        } else {
            self.invisible
                .then_some(ActorViewDisposition::WithinPaddedViewButInvisible)
        }
    }
}

fn relative(position: u16, reference: u16) -> i32 {
    i32::from(i16::from_le_bytes(
        position.wrapping_sub(reference).to_le_bytes(),
    ))
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
struct Door {
    identity: ActorSlotIdentity,
    facing: Facing,
    figure: Figure,
    wait: u16,
    frames: u16,
    graphics_updated: bool,
    body: Body,
}

impl Door {
    fn admit(identity: ActorSlotIdentity, door: CeresDoor, body: Body) -> Option<Self> {
        let (figure, wait) = match door.phase {
            Phase::Open { wait } => (Figure::Open, wait),
            Phase::Closed { wait } => (Figure::Closed, wait),
            Phase::Placed
            | Phase::Starting { .. }
            | Phase::Opening { .. }
            | Phase::Closing { .. }
            | Phase::Wall => return None,
        };
        if door.kind != DoorKind::Door || door.position != body.centre() {
            return None;
        }
        Some(Self {
            identity,
            facing: match door.facing {
                Side::Left => Facing::Left,
                Side::Right => Facing::Right,
            },
            figure,
            wait: wait.frames(),
            frames: 0,
            graphics_updated: false,
            body,
        })
    }

    fn run(self, inputs: &Inputs, rules: &Rules) -> Option<Self> {
        let wait = Wait::new(self.wait)?;
        let door = CeresDoor {
            kind: DoorKind::Door,
            facing: match self.facing {
                Facing::Left => Side::Left,
                Facing::Right => Side::Right,
            },
            position: self.body.centre(),
            phase: match self.figure {
                Figure::Open => Phase::Open { wait },
                Figure::Closed => Phase::Closed { wait },
            },
        };
        let result = door.run(inputs, rules);
        let mut next = Self::admit(self.identity, result.door, self.body)?;
        if result.sound.is_some() || result.shake.is_some() || next.figure != self.figure {
            return None;
        }
        next.frames = self.frames.wrapping_add(1);
        next.graphics_updated = self.wait == 1;
        Some(next)
    }

    fn bind(mut self, actor: &ActorSeed) -> Option<Self> {
        if self.wait != actor.instruction_timer || actor.flash_timer != 0 {
            return None;
        }
        self.frames = actor.frame_counter.0;
        self.graphics_updated = actor.graphics_updated == GraphicsUpdated::Set;
        Some(self)
    }

    const fn graphics(self) -> GraphicsUpdated {
        if self.graphics_updated {
            GraphicsUpdated::Set
        } else {
            GraphicsUpdated::Clear
        }
    }

    #[cfg(test)]
    fn matches(self, actor: &CurrentActor) -> bool {
        self.identity == actor.identity
            && self.body.geometry() == actor.geometry
            && self.body.content == actor.content
            && self.body.touch == actor.touch
            && self.body.solidity == actor.solidity
            && actor.flash_timer == 0
            && self.wait == actor.instruction_timer
            && self.frames == actor.frame_counter.0
            && self.graphics() == actor.graphics_updated
    }
}

/// Complete carried state for the two scoped ordinary doors.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Doors([Door; 2]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Path {
    Waiting,
    Polling,
}

impl Doors {
    /// Admit two checked current actor identities and complete door placements.
    #[must_use]
    pub fn new(
        [first, second]: [(ActorSlotIdentity, CeresDoor); 2],
        [first_actor, second_actor]: &[ActorSeed; 2],
        [first_invisible, second_invisible]: [bool; 2],
    ) -> Option<Self> {
        let first = Door::admit(first.0, first.1, Body::new(first_actor, first_invisible))?
            .bind(first_actor)?;
        let second = Door::admit(
            second.0,
            second.1,
            Body::new(second_actor, second_invisible),
        )?
        .bind(second_actor)?;
        if first.identity == second.identity
            || first.figure != Figure::Open
            || second.figure != Figure::Closed
        {
            return None;
        }
        Some(Self([first, second]))
    }

    pub(super) fn current_actors(self) -> [ActorSeed; 2] {
        self.0.map(|door| ActorSeed {
            content: door.body.content,
            touch: door.body.touch,
            solidity: door.body.solidity,
            geometry: door.body.geometry(),
            frame_counter: AdministrativeCounter(door.frames),
            instruction_timer: door.wait,
            graphics_updated: door.graphics(),
            flash_timer: 0,
        })
    }

    pub(super) fn views(self, camera: Pair<u16>) -> Option<[ActorViewDisposition; 2]> {
        let [first, second] = self.0;
        let views = [first.body.view(camera)?, second.body.view(camera)?];
        (views
            == [
                ActorViewDisposition::WithinPaddedViewButInvisible,
                ActorViewDisposition::BelowPaddedView,
            ])
        .then_some(views)
    }

    pub(super) fn contact_misses(self, centre: Px, radius: Px) -> bool {
        let [_, second] = self.0;
        let distance = relative(second.body.centre().x.0, centre.0).abs();
        distance >= i32::from(radius.0).wrapping_add(i32::from(second.body.geometry().radius.x.0))
    }

    pub(super) const fn path(self) -> Option<Path> {
        let [first, second] = self.0;
        match (first.wait, second.wait) {
            (2, 2) => Some(Path::Waiting),
            (1, 1) => Some(Path::Polling),
            _ => None,
        }
    }

    pub(super) fn run(
        self,
        current: &ActorUnselectedOrigin,
        views: [ActorViewDisposition; 2],
        membership: &ActorEligibilityReceipt,
        her: Pair<Px>,
        random: u16,
    ) -> Option<(Self, ActorUpdateCompletion)> {
        self.path()?;
        current.validate_quiet_update().ok()?;
        if current.scene != ActorScene::PreEscape
            || !membership
                .active
                .iter()
                .copied()
                .eq(self.0.iter().map(|door| door.identity))
            || membership.active.get(1..) != Some(membership.interactive.as_slice())
            || self.0.iter().any(|door| {
                let touch = if membership.interactive.contains(&door.identity) {
                    ActorTouch::Tangible
                } else {
                    ActorTouch::Intangible
                };
                door.body.touch != touch
            })
        {
            return None;
        }
        let inputs = Inputs {
            her,
            stage: Stage::Calm,
            random,
            shake_frames_left: 0,
        };
        let rules = Rules::default();
        let [a, b] = self.0;
        let next = Self([a.run(&inputs, &rules)?, b.run(&inputs, &rules)?]);
        let actors = self
            .0
            .into_iter()
            .zip(next.0)
            .zip(current.actors.iter())
            .zip(views)
            .map(|(((door, returned), actor), view)| {
                if door.body != Body::new(actor, door.body.invisible) || actor.flash_timer != 0 {
                    return None;
                }
                Some(CurrentActor {
                    identity: door.identity,
                    content: actor.content,
                    touch: actor.touch,
                    solidity: actor.solidity,
                    geometry: actor.geometry,
                    frame_counter: AdministrativeCounter(returned.frames),
                    instruction_timer: returned.wait,
                    graphics_updated: returned.graphics(),
                    flash_timer: actor.flash_timer,
                    view,
                })
            })
            .collect::<Option<Vec<_>>>()?
            .try_into()
            .ok()?;
        let mut dependency = current.clone();
        for (door, actor) in self.0.into_iter().zip(&mut dependency.actors) {
            actor.instruction_timer = door.wait;
            actor.frame_counter = AdministrativeCounter(door.frames);
            actor.graphics_updated = door.graphics();
        }
        let output = ActorUpdateCompletion {
            dependency,
            actors,
            interactive: membership.interactive.clone(),
        };
        Some((next, output))
    }

    #[cfg(test)]
    pub(super) fn matches(self, output: &ActorUpdateCompletion) -> bool {
        self.0
            .into_iter()
            .zip(&output.actors)
            .all(|(door, actor)| door.matches(actor))
    }
}
