//! Today's block and solid-enemy collision, answering motion's requests
//! ([`model::collision::Collision`](crate::model::collision::Collision)) until the collision
//! domain replaces it.

use super::solid_enemy::{Contact, Direction};
use super::{Body, Heading as Way, Surroundings, Touches, move_x, move_y, scan_x};
use crate::model::collision::{
    Blocker, Body as Extent, Collision, Enemies, EnemyHit, Heading, Moved, Stop, WallHit,
};
use crate::model::num::Fixed;
use crate::state::samus::{Point, Samus, Speed};

/// Samus's X radius, which only her initialisation sets (`$90:EC39`).
pub const X_RADIUS: u16 = 5;

/// The room's blocks and solid enemies as one frame's moves meet them.
pub struct World<'a, 't> {
    /// Where the moves happen.
    pub surroundings: Surroundings<'a>,
    /// The scroll triggers and touch PLMs the moves reach.
    pub touches: &'t mut Touches,
    /// Her Y speed was 0.0 at the frame's start, which a walk along a slope reads.
    pub still_vertically: bool,
    /// The "position was adjusted by a slope" flag, as the vertical moves leave it.
    pub slope_adjusted: bool,
}

const fn direction(heading: Heading) -> Direction {
    match heading {
        Heading::Left => Direction::Left,
        Heading::Right => Direction::Right,
        Heading::Up => Direction::Up,
        Heading::Down => Direction::Down,
    }
}

const fn negated(value: Fixed) -> Fixed {
    Fixed(value.0.wrapping_neg())
}

/// The move from `from` to `to`; a block stop takes the end's subpixel and the whole pixels.
const fn between(from: Fixed, to: Fixed, side: Heading, blocked: bool) -> Moved {
    if !blocked {
        return Moved {
            distance: Fixed(to.0.wrapping_sub(from.0)),
            stop: None,
        };
    }
    Moved {
        distance: Fixed::new(to.pixel().wrapping_sub(from.pixel()), 0),
        stop: Some(Stop {
            side,
            blocker: Blocker::Block { subpixel: to.sub() },
        }),
    }
}

impl<'a, 't> World<'a, 't> {
    /// The world for `samus` as she stands now.
    #[must_use]
    pub fn new(surroundings: Surroundings<'a>, touches: &'t mut Touches, samus: &Samus) -> Self {
        Self {
            surroundings,
            touches,
            still_vertically: samus.motion.vertical_speed == Fixed::default(),
            slope_adjusted: samus.slope_adjusted,
        }
    }
}

/// Samus as her block collision sees her: her X radius, her Y radius (`$0B00`) and whether
/// she has no Y speed.
#[must_use]
pub fn body(samus: &Samus) -> Body {
    Body {
        x_radius: X_RADIUS,
        y_radius: samus.y_radius,
        still_vertically: samus.motion.vertical_speed == Fixed::default(),
    }
}

impl World<'_, '_> {
    const fn body(&self, extent: Extent) -> Body {
        Body {
            x_radius: extent.radius.x,
            y_radius: extent.radius.y,
            still_vertically: self.still_vertically,
        }
    }

    /// A solid enemy within `distance` on `heading`: the whole pixels she can still move, and
    /// whether she already touches it.
    fn enemy(&self, extent: Extent, heading: Heading, distance: Fixed) -> Option<(Fixed, bool)> {
        let mut at = Point::from(extent.position);
        let checked = self.surroundings.enemies.check(
            direction(heading),
            self.body(extent),
            Speed(distance.0),
            &mut at,
        );
        match checked {
            Contact::Clear => None,
            Contact::Touching { .. } => Some((Fixed::ZERO, true)),
            Contact::Blocked { distance, .. } => Some((Fixed::new(distance, 0), false)),
        }
    }

    /// A move cut by a solid enemy: `cut` pixels the way the distance goes.
    const fn cut(cut: Fixed, touching: bool, backwards: bool, side: Heading) -> Moved {
        Moved {
            distance: if backwards { negated(cut) } else { cut },
            stop: Some(Stop {
                side,
                blocker: Blocker::Enemy { touching },
            }),
        }
    }

    /// The vertical move by the signed `distance`, testing `heading`'s side.
    fn vertical(&mut self, extent: Extent, heading: Heading, distance: Fixed) -> Moved {
        let way = match heading {
            Heading::Up => Way::Up,
            Heading::Down | Heading::Left | Heading::Right => Way::Down,
        };
        let mut at = Point::from(extent.position);
        let Surroundings { room, order, .. } = self.surroundings;
        let body = self.body(extent);
        let moved = move_y(
            room,
            body,
            &mut at,
            Speed(distance.0),
            way,
            order,
            self.touches,
        );
        self.slope_adjusted = moved.slope.apply(self.slope_adjusted);
        between(extent.position.y, at.y.into(), heading, moved.collided)
    }
}

impl Collision for World<'_, '_> {
    fn move_horizontal(
        &mut self,
        extent: Extent,
        heading: Heading,
        distance: Fixed,
        enemies: Enemies,
    ) -> Moved {
        let backwards = distance.negative();
        if enemies == Enemies::Test {
            let ahead = if backwards {
                negated(distance)
            } else {
                distance
            };
            if let Some((cut, touching)) = self.enemy(extent, heading, ahead) {
                return Self::cut(cut, touching, backwards, heading);
            }
        }
        let mut at = Point::from(extent.position);
        let body = self.body(extent);
        let room = self.surroundings.room;
        let moved = move_x(room, body, &mut at, Speed(distance.0), self.touches);
        // A block counts only when the move goes the way the request tests (`$90:93A0`).
        let blocked = moved.collided && backwards == (heading == Heading::Left);
        between(extent.position.x, at.x.into(), heading, blocked)
    }

    fn move_down(
        &mut self,
        extent: Extent,
        heading: Heading,
        distance: Fixed,
        enemies: Enemies,
    ) -> Moved {
        if enemies == Enemies::Test
            && let Some((cut, touching)) = self.enemy(extent, heading, distance)
        {
            return Self::cut(cut, touching, false, heading);
        }
        self.vertical(extent, heading, distance)
    }

    fn move_up(&mut self, extent: Extent, distance: Fixed) -> Moved {
        if let Some((cut, touching)) = self.enemy(extent, Heading::Up, distance) {
            return Self::cut(cut, touching, true, Heading::Up);
        }
        self.vertical(extent, Heading::Up, negated(distance))
    }

    fn enemy_ahead(
        &mut self,
        extent: Extent,
        heading: Heading,
        distance: Fixed,
    ) -> Option<EnemyHit> {
        self.enemy(extent, heading, distance)
            .map(|(_, touching)| EnemyHit { touching })
    }

    fn probe_wall(&mut self, extent: Extent, heading: Heading, reach: u16) -> Option<WallHit> {
        let ahead = Fixed::new(reach, 0);
        if let Some((cut, _)) = self.enemy(extent, heading, ahead) {
            return Some(WallHit {
                distance_px: cut.pixel(),
                subpixel: None,
            });
        }
        let signed = match heading {
            Heading::Left => negated(ahead),
            Heading::Right | Heading::Up | Heading::Down => ahead,
        };
        let mut at = Point::from(extent.position);
        let body = self.body(extent);
        let room = self.surroundings.room;
        let scanned = scan_x(room, body, &mut at, Speed(signed.0), self.touches);
        let distance = Fixed(scanned.distance.0);
        scanned.collided.then(|| WallHit {
            distance_px: if distance.negative() {
                negated(distance).pixel()
            } else {
                distance.pixel()
            },
            subpixel: Some(at.x.subpixels),
        })
    }
}
