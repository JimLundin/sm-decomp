//! Her movement, her appearance and the figure she shows: their seed and compared fields.

use crate::replay::compare::Observation;
use crate::replay::index::ObservationId;
use crate::replay::result::Outcome;
use crate::replay::{appearance, connected, movement};
use crate::source::WramWord;
use canonical::model::Model;
use game::production::played;
use samus_movement::{Phase, Push, Samus};
use types::{Action, PushFrames, ZERO};

/// Solid contact retained by the push's vertical move.
const SOLID_CONTACT: WramWord = WramWord {
    name: "SAMUS_SOLID_CONTACT",
    offset: 0x0DD0,
};

pub(super) fn body_art(
    rom: data::rom::Lorom<'_>,
) -> Result<data::assets::samus::SamusAssets, Outcome> {
    import::samus::import(rom)
        .map_err(|_| crate::replay::data_failure(None, "ordinary Samus body art"))
}

pub(super) fn dust_art(
    rom: data::rom::Lorom<'_>,
) -> Result<data::assets::samus::WallJumpDustArt, Outcome> {
    import::samus::dust::import(rom)
        .map_err(|_| crate::replay::data_failure(None, "wall-jump dust content"))
}

/// Her State as the seed holds it.
pub(in crate::replay) fn seed(
    wram: &[u8],
    content: &appearance::Content,
) -> Result<played::samus::State, Outcome> {
    let (appearance, figure) = appearance::owner(wram, content)?;
    Ok(played::samus::State {
        movement: Model(movement::seed(wram)?),
        last_draw_jump: Model(last_draw(wram)?),
        travelled: Model(samus_movement::TravelledDistances {
            travels: crate::decode::directional_travel(wram)
                .map_err(|_| crate::replay::data_failure(None, "seed travel observation"))?,
        }),
        appearance: Model(appearance),
        figure: Model(figure),
    })
}

pub(in crate::replay) fn last_draw(wram: &[u8]) -> Result<samus_movement::JumpButton, Outcome> {
    let field = |name, offset| super::word(wram, crate::source::WramWord { name, offset });
    let buttons = field("Samus previous draw pad", 0x0dfe)?;
    let bound = field("jump binding", 0x09b4)?;
    Ok(if buttons & bound == 0 {
        samus_movement::JumpButton::Up
    } else {
        samus_movement::JumpButton::Held
    })
}

pub(in crate::replay) fn last_draw_observation(jump: samus_movement::JumpButton) -> Observation {
    Observation::quantity(
        "movement.last-draw-jump",
        i64::from(jump == samus_movement::JumpButton::Held),
    )
}

/// Her fields as the records hold them.
pub(super) fn expected(
    previous: &[u8],
    wram: &[u8],
    content: &appearance::Content,
) -> Result<Vec<Observation>, Outcome> {
    let mut seeded = movement::observed_state(wram)?;
    if seeded.getaway == samus_movement::Getaway::Idle
        && movement::getaway(previous)? == samus_movement::Getaway::Idle
        && seeded.vertical.speed == ZERO
        && matches!(
            (
                seeded.pose.action,
                seeded.knockback.push,
                seeded.vertical.phase
            ),
            (Action::Recoiling, None, Phase::GroundFalling)
                | (Action::Leaping, Some(Push::Down), Phase::Moonfall)
        )
    {
        let before = movement::observed_state(previous)?;
        let stopped = super::word(wram, SOLID_CONTACT)? != 0;
        seeded.vertical.phase = phase_after_push(&before, &seeded, stopped);
    }
    let mut fields = parts(&seeded);
    fields.push(last_draw_observation(last_draw(wram)?));
    fields.extend(movement::expected(
        wram,
        &[ObservationId::PositionX, ObservationId::PositionY],
    )?);
    let (state, playing) = appearance::animation_owner(wram, content)?;
    fields.extend(appearance::initial(state, content));
    fields.push(appearance::hurt_observation(appearance::hurt_count(wram)?));
    fields.push(connected::playing_observation(playing));
    Ok(fields)
}

/// Her fields as the played State holds them.
pub(super) fn actual(
    state: &played::samus::State,
    content: &appearance::Content,
) -> Vec<Observation> {
    let samus = &state.movement.0;
    let mut fields = parts(samus);
    fields.push(last_draw_observation(state.last_draw_jump.0));
    fields.extend([
        Observation::quantity("movement.position.x", i64::from(samus.position.x.0)),
        Observation::quantity("movement.position.y", i64::from(samus.position.y.0)),
    ]);
    fields.extend(appearance::actual(&state.appearance.0, content));
    fields.push(appearance::hurt_observation(state.appearance.0.hurt_count));
    fields.push(connected::playing_observation(state.figure.0));
    fields
}

/// Her movement State by part, so a difference names the part that differs; the encoding is
/// only compared, never logged. Her choices of her pose are compared as `movement.choices`:
/// `movement::seed` reads the record's old and old-old poses. This witness checks the wall-jump classes, which the game
/// shifts only on a frame that chooses.
fn parts(samus: &samus_movement::Samus) -> Vec<Observation> {
    let part = |field: &str, value: String| Observation {
        field: field.into(),
        bytes: value.into_bytes(),
    };
    let mut vertical = samus.vertical;
    if let Some(speed) = samus.getaway.downward_speed() {
        vertical = samus_movement::Vertical {
            speed,
            phase: Phase::Falling,
            bounces: vertical.bounces,
        };
    }
    vec![
        part("movement.pose", format!("{:?}", samus.pose)),
        part("movement.run", run(&samus.run)),
        part("movement.vertical", format!("{vertical:?}")),
        part("movement.seat", format!("{:?}", samus.seat)),
        part(
            "movement.choices",
            format!("{:?}", connected::Choices::from(samus.choices)),
        ),
        part("movement.jump-held", format!("{:?}", samus.jump_held)),
        part("movement.knockback", knockback(samus.knockback)),
        part("movement.position", format!("{:?}", samus.position)),
    ]
}

/// The recorded vertical direction leaves footing implicit at two push transitions.
fn phase_after_push(before: &Samus, after: &Samus, stopped: bool) -> Phase {
    let phase = after.vertical.phase;
    if before.pose.action != Action::Recoiling || after.vertical.speed != ZERO {
        return phase;
    }

    match (after.pose.action, after.knockback.push, phase) {
        (Action::Recoiling, None, Phase::GroundFalling)
            if before.knockback.time_left == PushFrames::NONE
                && before.vertical.phase == Phase::Rising
                && (before.knockback.push == Some(Push::Up)
                    || before.knockback.push == Some(Push::Down) && !stopped) =>
        {
            // A spent push clears speed and sets the falling direction in either footing.
            // The preceding rise and the final push's solid contact distinguish them.
            Phase::Falling
        }
        (Action::Leaping, Some(Push::Down), Phase::Moonfall)
            if before.knockback.push == Some(Push::Down)
                && ((before.vertical.phase == Phase::Ground && before.vertical.speed == ZERO)
                    || (stopped && after.run.speed == ZERO)) =>
        {
            // A stopped Down push can touch the floor on the cut that chooses the leap.
            // The leap's first move begins the moonfall on the next cut.
            Phase::Ground
        }
        (_, _, phase) => phase,
    }
}

/// Her knockback as the record states it. The side a hit pushes her to is the hitter's last,
/// kept while no push is under way, so it is compared only while one is.
fn knockback(knockback: samus_movement::Knockback) -> String {
    let samus_movement::Knockback {
        time_left,
        side,
        push,
    } = knockback;
    let side = push.is_some().then_some(side);
    format!("{time_left:?} {side:?} {push:?}")
}

/// Her run as the record states it. While she follows the pad the game keeps no way of its
/// own, so the way her speed points is compared only while she carries on; a way that differs
/// while she follows the pad reaches the compared position if it ever moves her.
fn run(run: &samus_movement::Run) -> String {
    let samus_movement::Run {
        speed,
        way,
        carry,
        dash,
        dashing,
    } = run;
    let way = (*carry == samus_movement::Carry::CarriesOn).then_some(way);
    format!("{speed:?} {way:?} {carry:?} {dash:?} {dashing:?}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn push_exit_in_air_requires_the_preceding_rise_and_no_floor_stop() {
        use super::{Action, Phase, Push, phase_after_push};

        let mut before = super::movement::seed(&vec![0; 0x10000]).unwrap();
        before.pose.action = Action::Recoiling;
        before.knockback.push = Some(Push::Down);
        before.vertical.phase = Phase::Rising;
        let mut after = before;
        after.knockback.push = None;
        after.vertical.phase = Phase::GroundFalling;

        assert_eq!(phase_after_push(&before, &after, false), Phase::Falling);
        assert_eq!(
            phase_after_push(&before, &after, true),
            Phase::GroundFalling
        );
        before.vertical.phase = Phase::Ground;
        assert_eq!(
            phase_after_push(&before, &after, false),
            Phase::GroundFalling
        );
    }

    #[test]
    fn grounded_leap_keeps_its_footing_until_the_first_leap_move() {
        use super::{Action, Phase, Push, phase_after_push};

        let mut before = super::movement::seed(&vec![0; 0x10000]).unwrap();
        before.pose.action = Action::Recoiling;
        before.knockback.push = Some(Push::Down);
        let mut after = before;
        after.pose.action = Action::Leaping;
        after.vertical.phase = Phase::Moonfall;

        assert_eq!(phase_after_push(&before, &after, false), Phase::Ground);
        before.vertical.phase = Phase::Rising;
        assert_eq!(phase_after_push(&before, &after, false), Phase::Moonfall);
        assert_eq!(phase_after_push(&before, &after, true), Phase::Ground);
        before.pose.action = Action::Leaping;
        assert_eq!(phase_after_push(&before, &after, false), Phase::Moonfall);
    }

    #[test]
    fn her_way_is_compared_only_while_she_carries_on() {
        use samus_movement::{Carry, Dashing, Run};
        use types::{Side, ZERO};
        let run = |way, carry| {
            super::run(&Run {
                speed: ZERO,
                way,
                carry,
                dash: ZERO,
                dashing: Dashing::Off,
            })
        };
        assert_eq!(
            run(Side::Left, Carry::FollowsPad),
            run(Side::Right, Carry::FollowsPad)
        );
        assert_ne!(
            run(Side::Left, Carry::CarriesOn),
            run(Side::Right, Carry::CarriesOn)
        );
    }
}
