//! Her movement, her appearance and the figure she shows: their seed and compared fields.

use crate::replay::compare::Observation;
use crate::replay::index::ObservationId;
use crate::replay::result::Outcome;
use crate::replay::{appearance, connected, movement};
use canonical::model::Model;
use game::production::played;

/// Her State as the seed holds it.
pub(in crate::replay) fn seed(
    wram: &[u8],
    content: &appearance::Content,
) -> Result<played::samus::State, Outcome> {
    let (appearance, figure) = appearance::owner(wram, content)?;
    Ok(played::samus::State {
        movement: Model(movement::seed(wram)?),
        last_draw_jump: Model(last_draw(wram)?),
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

/// Her fields as the record at `cut` of `row` holds them.
pub(super) fn expected(
    wram: &[u8],
    content: &appearance::Content,
    row: &str,
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let seeded = movement::seed(wram)?;
    let mut fields = parts(&seeded);
    fields.push(last_draw_observation(last_draw(wram)?));
    fields.extend(movement::expected(
        wram,
        &[ObservationId::PositionX, ObservationId::PositionY],
        row,
        cut,
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
/// `movement::seed` reads them from the record's old and old-old pose types, which the game
/// shifts only on a frame that chooses.
fn parts(samus: &samus_movement::Samus) -> Vec<Observation> {
    let part = |field: &str, value: String| Observation {
        field: field.into(),
        bytes: value.into_bytes(),
    };
    vec![
        part("movement.pose", format!("{:?}", samus.pose)),
        part("movement.run", run(&samus.run)),
        part("movement.vertical", format!("{:?}", samus.vertical)),
        part("movement.seat", format!("{:?}", samus.seat)),
        part("movement.choices", format!("{:?}", samus.choices)),
        part("movement.jump-held", format!("{:?}", samus.jump_held)),
        part("movement.knockback", knockback(samus.knockback)),
        part("movement.position", format!("{:?}", samus.position)),
    ]
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
