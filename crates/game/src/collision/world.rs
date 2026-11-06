//! The room as the motion model asks it: [`model::collision::Collision`] over today's block
//! collision and solid-enemy check, so a move means what the engine's moves mean.
//!
//! The collision domain gets its own spec later; until then this is the bridge, and every move
//! goes through [`collision::move_x`], [`collision::move_y`] and [`SolidEnemies::check`].

use crate::collision::solid_enemy::{Contact, Direction};
use crate::collision::{self, Heading, Surroundings, Touches};
use crate::model::collision::{Body, Collision, Enemies, Moved, Side, Stop};
use crate::model::num::Fixed;
use crate::model::samus::ground::Position;
use crate::state::samus::{self as state, Point, Speed};

/// The room, its enemies and what the frame's moves left behind them.
pub struct World<'a, 't> {
    surroundings: Surroundings<'a>,
    /// Her vertical speed is zero, which a walk along a non-square slope needs.
    still_vertically: bool,
    touches: &'t mut Touches,
    slope_adjusted: bool,
}

impl<'a, 't> World<'a, 't> {
    /// The world of a frame's moves, with the "adjusted by a slope" flag the last frame left.
    pub const fn new(
        surroundings: Surroundings<'a>,
        still_vertically: bool,
        touches: &'t mut Touches,
        slope_adjusted: bool,
    ) -> Self {
        Self {
            surroundings,
            still_vertically,
            touches,
            slope_adjusted,
        }
    }

    /// The "adjusted by a slope" flag as the moves leave it.
    #[must_use]
    pub const fn slope_adjusted(&self) -> bool {
        self.slope_adjusted
    }

    const fn body(&self, body: Body) -> collision::Body {
        collision::Body {
            x_radius: body.radius.x,
            y_radius: body.radius.y,
            still_vertically: self.still_vertically,
        }
    }
}

const fn fixed(value: state::Fixed) -> Fixed {
    Fixed::new(value.pixels, value.subpixels)
}

const fn word(value: Fixed) -> state::Fixed {
    state::Fixed {
        pixels: value.pixel(),
        subpixels: value.sub(),
    }
}

const fn point(position: Position) -> Point {
    Point {
        x: word(position.x),
        y: word(position.y),
    }
}

/// The move from `old` to `new` on one axis, as the model applies it: a stop sets the subpixel
/// the collision left and adds whole pixels, any other move adds the whole difference.
const fn moved(old: Fixed, new: state::Fixed, stop: Option<Side>) -> Moved {
    match stop {
        Some(side) => Moved {
            distance: Fixed::new(new.pixels.wrapping_sub(old.pixel()), 0),
            stop: Some(Stop {
                side,
                subpixel: Some(new.subpixels),
            }),
        },
        None => Moved {
            distance: Fixed(fixed(new).0.wrapping_sub(old.0)),
            stop: None,
        },
    }
}

/// The enemy check's distance: the move's size, without its sign.
const fn magnitude(distance: Fixed, backwards: bool) -> Speed {
    if backwards {
        Speed(distance.0.wrapping_neg())
    } else {
        Speed(distance.0)
    }
}

/// The distance an enemy cut the move to, with the move's sign.
const fn signed(cut: Speed, backwards: bool) -> Speed {
    if backwards {
        Speed(cut.0.wrapping_neg())
    } else {
        cut
    }
}

impl Collision for World<'_, '_> {
    fn move_horizontal(&mut self, body: Body, distance: Fixed, enemies: Enemies) -> Moved {
        let mut position = point(body.position);
        let backwards = distance.pixel().cast_signed().is_negative();
        let (direction, side) = if backwards {
            (Direction::Left, Side::Left)
        } else {
            (Direction::Right, Side::Right)
        };
        let collision_body = self.body(body);
        let contact = match enemies {
            Enemies::Test => self.surroundings.enemies.check(
                direction,
                collision_body,
                magnitude(distance, backwards),
                &mut position,
            ),
            Enemies::Skip => Contact::Clear,
        };
        let stopped = if let Some(cut) = contact.distance() {
            position.x = collision::advance(position.x, signed(cut, backwards));
            true
        } else {
            let room = self.surroundings.room;
            let step = Speed(distance.0);
            collision::move_x(room, collision_body, &mut position, step, self.touches).collided
        };
        moved(body.position.x, position.x, stopped.then_some(side))
    }

    fn move_down(&mut self, body: Body, distance: Fixed, enemies: Enemies) -> Moved {
        let mut position = point(body.position);
        let Surroundings {
            room,
            order,
            enemies: solid,
        } = self.surroundings;
        let collision_body = self.body(body);
        let step = Speed(distance.0);
        let collided = match enemies {
            Enemies::Test => {
                let contact = solid.check(Direction::Down, collision_body, step, &mut position);
                if let Some(cut) = contact.distance() {
                    position.y = collision::advance(position.y, cut);
                    true
                } else {
                    let touches = &mut *self.touches;
                    let moved = collision::move_y(
                        room,
                        collision_body,
                        &mut position,
                        step,
                        Heading::Down,
                        order,
                        touches,
                    );
                    self.slope_adjusted = moved.slope.apply(self.slope_adjusted);
                    moved.collided
                }
            }
            // The only move that skips the enemies is the elevator's, whose request says up.
            Enemies::Skip => {
                let touches = &mut *self.touches;
                let moved = collision::move_y(
                    room,
                    collision_body,
                    &mut position,
                    step,
                    Heading::Up,
                    order,
                    touches,
                );
                self.slope_adjusted = moved.slope.apply(self.slope_adjusted);
                moved.collided
            }
        };
        moved(body.position.y, position.y, collided.then_some(Side::Below))
    }

    fn enemy_blocks(&self, body: Body, side: Side, distance: Fixed) -> bool {
        let direction = match side {
            Side::Left => Direction::Left,
            Side::Right => Direction::Right,
            Side::Below => Direction::Down,
        };
        let mut position = point(body.position);
        let contact = self.surroundings.enemies.check(
            direction,
            self.body(body),
            Speed(distance.0),
            &mut position,
        );
        contact != Contact::Clear
    }
}
