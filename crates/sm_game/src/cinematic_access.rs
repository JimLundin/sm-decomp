//! Unpublished cinematic drafts over the current Session's existing owners.
//! Enemy-projectile aliases and Samus's frame meet between transactions.
#![forbid(unsafe_code)]
use crate::cinematic::{FadePlanes, PaletteBuffer, ShotWords};
use crate::cinematic_state::Cinematic;
use crate::explosion_window::Explosion;
use crate::liquid::LiquidWords;
use crate::session::Session;

/// Compatibility codecs are retained only for nonadmitted callers and hold checks.
pub trait Value: crate::session_cohort::Cohort {
    fn current(session: &Session) -> Self;
    fn merge(before: &Self, after: &Self, session: &mut Session);
}

macro_rules! changed {
    ($target:expr, $before:expr, $after:expr; $($($field:ident).+),+ $(,)?) => {
        $(if $before.$($field).+ != $after.$($field).+ {
            $target.$($field).+ = $after.$($field).+;
        })+
    };
}

impl Value for Cinematic {
    fn current(session: &Session) -> Self {
        Self(session.enemy_projectiles.clone())
    }
    fn merge(before: &Self, after: &Self, session: &mut Session) {
        let target = &mut session.enemy_projectiles;
        // Original cohort traversal: scalar words, then each slot's fields.
        changed!(target, before.0, after.0;
            flags, zoom, selected, spawn.primary, spawn.secondary,
            drop_origin.x, drop_origin.y);
        for ((target, before), after) in target
            .slots
            .iter_mut()
            .zip(&before.0.slots)
            .zip(&after.0.slots)
        {
            changed!(target, before, after;
                definition, graphics, counter, pre_action,
                x.fraction, x.whole, y.fraction, y.whole,
                velocity.x, velocity.y, behavior_a, behavior_b,
                program, spritemap, delay, radius, properties, behavior_c,
                angle, drop.flags, drop.enemy_definition, drop.killed_enemy);
        }
    }
}

fn merge_words<'a>(
    target: impl Iterator<Item = &'a mut u16>,
    before: impl Iterator<Item = &'a u16>,
    after: impl Iterator<Item = &'a u16>,
) {
    for ((target, before), after) in target.zip(before).zip(after) {
        if before != after {
            *target = *after;
        }
    }
}

impl Value for FadePlanes {
    fn current(session: &Session) -> Self {
        let mut planes = Box::new([0; 2048]);
        for (target, source) in planes
            .iter_mut()
            .zip(session.actors.extended.tilemaps.iter().flatten())
        {
            *target = *source;
        }
        Self(planes)
    }
    fn merge(before: &Self, after: &Self, session: &mut Session) {
        merge_words(
            session.actors.extended.tilemaps.iter_mut().flatten(),
            before.0.iter(),
            after.0.iter(),
        );
    }
}

impl Value for PaletteBuffer {
    fn current(session: &Session) -> Self {
        Self(session.presentation.visual().colors)
    }
    fn merge(before: &Self, after: &Self, session: &mut Session) {
        merge_words(
            session.presentation.visual_mut().colors.iter_mut(),
            before.0.iter(),
            after.0.iter(),
        );
    }
}

impl Value for ShotWords {
    fn current(session: &Session) -> Self {
        Self {
            x: core::array::from_fn(|index| session.projectiles.shots[index].x),
            kind: core::array::from_fn(|index| session.projectiles.shots[index].kind),
        }
    }
    fn merge(before: &Self, after: &Self, session: &mut Session) {
        // All X words precede all kind words in the original traversal.
        merge_words(
            session.projectiles.shots.iter_mut().map(|shot| &mut shot.x),
            before.x.iter(),
            after.x.iter(),
        );
        merge_words(
            session
                .projectiles
                .shots
                .iter_mut()
                .map(|shot| &mut shot.kind),
            before.kind.iter(),
            after.kind.iter(),
        );
    }
}

impl Value for Explosion {
    fn current(session: &Session) -> Self {
        Self {
            bomb: session.projectiles.power_bomb.clone(),
            radius: session.room.power_bomb_radius,
        }
    }
    fn merge(before: &Self, after: &Self, session: &mut Session) {
        // The original cohort publishes these ten words in this order.
        let target = &mut session.projectiles.power_bomb;
        changed!(target, before.bomb, after.bomb;
            explosion_status, x, y, window_x, window_y);
        if before.radius != after.radius {
            session.room.power_bomb_radius = after.radius;
        }
        changed!(target, before.bomb, after.bomb;
            pre_flash_radius, flag, radius_speed, shape_definition);
    }
}

impl Value for LiquidWords {
    fn current(session: &Session) -> Self {
        Self::of(&session.room)
    }
    fn merge(before: &Self, after: &Self, session: &mut Session) {
        // Preserve the liquid codec traversal, including separated Room fields.
        let liquid = &mut session.room.liquid;
        changed!(liquid, before.liquid, after.liquid;
            surface.fraction, surface.whole, lava_acid.fraction, lava_acid.whole,
            tilemap, rising, kind, offset.fraction, offset.whole,
            tide_phase, base_fraction);
        for ((target, previous), next) in [
            &mut session.room.liquid_height,
            &mut session.room.liquid_target,
            &mut session.room.liquid_velocity,
            &mut session.room.liquid_options,
            &mut session.room.liquid_delay,
        ]
        .into_iter()
        .zip([
            before.height,
            before.target,
            before.velocity,
            before.options,
            before.delay,
        ])
        .zip([
            after.height,
            after.target,
            after.velocity,
            after.options,
            after.delay,
        ]) {
            if previous != next {
                *target = next;
            }
        }
    }
}

impl<A: Value, B: Value> Value for (A, B) {
    fn current(session: &Session) -> Self {
        (A::current(session), B::current(session))
    }
    fn merge(before: &Self, after: &Self, session: &mut Session) {
        A::merge(&before.0, &after.0, session);
        B::merge(&before.1, &after.1, session);
    }
}

pub trait Access {
    /// Read the actual current owner, without opening an unrelated lease.
    fn cinematic_value<C: Value>(&self) -> C;
    /// Only changed drafts admit a lease, at the original cohort commit boundary.
    fn commit_cinematic<C: Value>(&mut self, before: &C, after: C);
}

pub fn load<C: Value, H: Access + ?Sized>(game: &H) -> C {
    game.cinematic_value()
}

pub fn with<C: Value, H: Access + ?Sized, T>(
    game: &mut H,
    action: impl FnOnce(&mut H, &mut C) -> T,
) -> T {
    let before = load::<C, H>(game);
    let mut working = before.clone();
    #[cfg(sm_verify_dll)]
    let held = crate::session_cohort::hold(&mut working);
    let result = action(game, &mut working);
    #[cfg(sm_verify_dll)]
    crate::session_cohort::release(held);
    if working != before {
        game.commit_cinematic(&before, working);
    }
    result
}
