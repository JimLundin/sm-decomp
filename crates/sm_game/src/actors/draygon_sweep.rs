//! Draygon's swoops: the swoop Y positions, setups, descents, apexes and
//! ascents, mirrored for each side (bank $A5:87F4-$A5:8B09).
//! The native table remains authoritative and may alias other compatibility state.
use crate::actors::draygon_host::Lease;
use crate::actors::host::BossEffects as _;

use crate::menus::prelude::{battle, owned};

use crate::actors::draygon_flight::{body_bubble, fire_wall_turret, move_horizontally};
use crate::actors::draygon_host::{
    ARMS, BODY, Draygon, DraygonEdit, SIGN, abs16, function, instruction_list as list,
};
use crate::ram::word;

/// $7E:9002: the swoop Y positions, one every 4 bytes, up to 800h bytes.
const SWOOP_Y_POSITIONS: usize = 0x9002;
const SWOOP_STEP: u16 = 4;
const SWOOP_TABLE_BYTES: u16 = 0x800;
/// Swoops start descending from Y 180h.
const SWOOP_START_Y: u16 = 0x180;
/// The arms reach out at swoop index 68h.
const ARMS_OUT_INDEX: u16 = 0x68;
/// The right swoop ends at X 2A0h.
const RIGHT_END_X: i32 = 0x2a0;

fn write_y_position<S: Lease>(index: u16, value: u16, game: &mut S) {
    let at = SWOOP_Y_POSITIONS + usize::from(index);
    let bytes = value.to_ne_bytes();
    game.set_slot_byte(at, bytes[0]);
    game.set_slot_byte(at + 1, bytes[1]);
}
/// Calculate Draygon swoop Y positions ($A5:8817): fall with the swoop Y
/// acceleration to the reset Y, then set the X speed that reaches Samus.
fn calculate_swoop_y_positions<L: Lease>(game: &mut L) {
    let (mut height, mut velocity, mut index) = (SWOOP_START_Y, 0u16, 0u16);
    loop {
        velocity = velocity.wrapping_add(Draygon::of(game, BODY).swoop_y_acceleration());
        let next = height.wrapping_sub(u16::from(velocity.to_ne_bytes()[1]));
        if (next as i16).wrapping_sub(Draygon::of(game, BODY).reset_y() as i16) < 0 {
            break;
        }
        height = next;
        write_y_position(index, next, game);
        index = index.wrapping_add(SWOOP_STEP);
        if index.wrapping_sub(SWOOP_TABLE_BYTES) & SIGN == 0 {
            game.foreign(|game| game.unreachable());
        }
    }
    let distance = abs16(
        Draygon::of(game, BODY)
            .left_reset_x()
            .wrapping_sub(game.player_x()),
    );
    let speed = (u32::from(distance) << 16)
        .checked_div(u32::from(index >> 2))
        .unwrap_or(0);
    {
        let mut draygon = DraygonEdit::of(game, BODY);
        *draygon.x_subspeed() = speed as u16;
        *draygon.x_speed() = (speed >> 16) as u16;
    }
    write_y_position(index, Draygon::of(game, BODY).actor().y.whole, game);
    {
        let mut draygon = DraygonEdit::of(game, BODY);
        *draygon.function_timer() = index;
        *draygon.swoop_duration() = index;
    }
    // The swoop's speed, last Y position and index stay in $0E22..$0E26.
    word! {game; REMOVED_VAR_E22 = velocity; REMOVED_VAR_E24 = height; REMOVED_VAR_E26 = index;}
}
/// Draygon body function - swoop right ($A5:87F4) or left ($A5:89B3) - setup.
/// Was `begin`.
pub fn swoop_setup<L: Lease>(right: bool, game: &mut L) {
    if right {
        fire_wall_turret(game);
        calculate_swoop_y_positions(game);
    } else {
        // The duration is $0E26's quarter: the last swoop Y position index or
        // chase X subspeed stored there, not the swoop duration meant.
        let distance = abs16(
            Draygon::of(game, BODY)
                .right_reset_x()
                .wrapping_sub(game.player_x()),
        );
        let speed = (u32::from(distance) << 16)
            .checked_div(u32::from(word!(game.REMOVED_VAR_E26) >> 2))
            .unwrap_or(0);
        {
            let mut draygon = DraygonEdit::of(game, BODY);
            *draygon.x_subspeed() = speed as u16;
            *draygon.x_speed() = (speed >> 16) as u16;
        }
    }
    *DraygonEdit::of(game, BODY).function() = if right {
        function::SWOOP_RIGHT_DESCENDING
    } else {
        function::SWOOP_LEFT_DESCENDING
    };
    {
        let mut draygon = DraygonEdit::of(game, BODY);
        let actor = draygon.actor();
        actor.instruction = if right {
            list::BODY_RIGHT_RESET
        } else {
            list::BODY_LEFT_RESET
        };
        actor.instruction_delay = 1;
    }
    *DraygonEdit::of(game, BODY).facing_right() = u16::from(right);
    if !right {
        let copied = Draygon::of(game, BODY).swoop_duration();
        *DraygonEdit::of(game, BODY).function_timer() = copied;
    }
}
/// Draygon body function - swoop right ($A5:8922) or left ($A5:8A50) - apex:
/// set the X speed that reaches the far side in the swoop's duration. Was `turn`.
pub fn swoop_apex<S: Lease>(right: bool, game: &mut S) {
    fire_wall_turret(game);
    let count = u32::from(Draygon::of(game, BODY).swoop_duration() >> 2);
    let x = Draygon::of(game, BODY).actor().x.whole as i32;
    let distance = if right {
        RIGHT_END_X - x
    } else {
        x - Draygon::of(game, BODY).left_reset_x() as i32
    };
    let speed = ((distance << 16) as u32).checked_div(count).unwrap_or(0);
    if right {
        {
            let mut draygon = DraygonEdit::of(game, BODY);
            *draygon.x_subspeed() = speed as u16;
            *draygon.x_speed() = (speed >> 16) as u16;
        }
    } else {
        // The left apex writes high then low; other speed setters write low first.
        {
            let mut draygon = DraygonEdit::of(game, BODY);
            *draygon.x_speed() = (speed >> 16) as u16;
            *draygon.x_subspeed() = speed as u16;
        }
    }
    *DraygonEdit::of(game, BODY).function() = if right {
        function::SWOOP_RIGHT_ASCENDING
    } else {
        function::SWOOP_LEFT_ASCENDING
    };
}
/// Draygon body function - swoop right or left - descending ($A5:88B1,
/// $A5:8A00) or, with `ascending`, ascending ($A5:8951, $A5:8A90): follow the
/// swoop Y positions, then pick the next attack. Was `advance`.
pub fn swoop<L: Lease>(right: bool, ascending: bool, game: &mut L) {
    let turret = fire_wall_turret(game);
    if !right || !ascending {
        body_bubble(turret, game);
    }
    let index = Draygon::of(game, BODY).function_timer();
    if index == ARMS_OUT_INDEX {
        let instruction = match (right, ascending) {
            (true, false) => list::ARMS_RIGHT_NEAR_SWOOP_APEX,
            (true, true) => list::ARMS_RIGHT_IDLE,
            (false, false) => list::ARMS_LEFT_NEAR_SWOOP_APEX,
            (false, true) => list::ARMS_LEFT_IDLE,
        };
        {
            let mut draygon = DraygonEdit::of(game, ARMS);
            let actor = draygon.actor();
            actor.instruction = instruction;
            actor.instruction_delay = 1;
        }
    }
    let at = SWOOP_Y_POSITIONS + usize::from(index);
    let height = u16::from_ne_bytes([game.slot_byte(at), game.slot_byte(at + 1)]);
    let k = game.current_enemy();
    DraygonEdit::of(game, k).actor().y.whole = height;
    // A broad selected slot can alias the body's index: reread after its Y store.
    let index = Draygon::of(game, BODY).function_timer();
    let next = if ascending {
        index.wrapping_add(SWOOP_STEP)
    } else {
        index.wrapping_sub(SWOOP_STEP)
    };
    *DraygonEdit::of(game, BODY).function_timer() = next;
    if !ascending {
        if next == 0 {
            *DraygonEdit::of(game, BODY).function() = if right {
                function::SWOOP_RIGHT_APEX
            } else {
                function::SWOOP_LEFT_APEX
            };
        } else {
            move_horizontally(right, game);
        }
    } else if next != Draygon::of(game, BODY).swoop_duration() {
        move_horizontally(right, game);
    } else if right {
        *DraygonEdit::of(game, BODY).function_timer() = 0;
        *DraygonEdit::of(game, BODY).function() = if battle!(game.Random) & 1 != 0 {
            function::SWOOP_LEFT_SETUP
        } else {
            function::GOOP_LEFT_SETUP
        };
    } else if battle!(game.Random) & 1 != 0 {
        {
            let mut draygon = DraygonEdit::of(game, BODY);
            *draygon.function() = function::SWOOP_RIGHT_SETUP;
            *draygon.function_timer() = 0;
        }
        DraygonEdit::of(game, BODY).actor().x.whole = Draygon::of(game, BODY).left_reset_x();
        DraygonEdit::of(game, BODY).actor().y.whole = Draygon::of(game, BODY).reset_y();
    } else {
        *DraygonEdit::of(game, BODY).function() = function::GOOP_RIGHT_SETUP;
        let copied = Draygon::of(game, BODY).left_reset_x();
        DraygonEdit::of(game, BODY).actor().x.whole = copied;
    }
}
