//! Direct shared-slot access; imported aliases handle exceptional selectors only.
#![forbid(unsafe_code)]
macro_rules! enemy_member {
    (definition) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Definition
    };
    (graphics) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Graphics
    };
    (counter) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Counter
    };
    (pre_action) => {
        $crate::game::enemy_projectiles::host::EnemyMember::PreAction
    };
    (x_fraction) => {
        $crate::game::enemy_projectiles::host::EnemyMember::XFraction
    };
    (x) => {
        $crate::game::enemy_projectiles::host::EnemyMember::X
    };
    (y_fraction) => {
        $crate::game::enemy_projectiles::host::EnemyMember::YFraction
    };
    (y) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Y
    };
    (velocity_x) => {
        $crate::game::enemy_projectiles::host::EnemyMember::VelocityX
    };
    (velocity_y) => {
        $crate::game::enemy_projectiles::host::EnemyMember::VelocityY
    };
    (behavior_a) => {
        $crate::game::enemy_projectiles::host::EnemyMember::BehaviorA
    };
    (behavior_b) => {
        $crate::game::enemy_projectiles::host::EnemyMember::BehaviorB
    };
    (program) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Program
    };
    (spritemap) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Spritemap
    };
    (delay) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Delay
    };
    (radius) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Radius
    };
    (properties) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Properties
    };
    (behavior_c) => {
        $crate::game::enemy_projectiles::host::EnemyMember::BehaviorC
    };
    (angle) => {
        $crate::game::enemy_projectiles::host::EnemyMember::Angle
    };
    (drop_flags) => {
        $crate::game::enemy_projectiles::host::EnemyMember::DropFlags
    };
    (drop_definition) => {
        $crate::game::enemy_projectiles::host::EnemyMember::DropDefinition
    };
    (killed_enemy) => {
        $crate::game::enemy_projectiles::host::EnemyMember::KilledEnemy
    };
}
macro_rules! enemy_field {
    ($slot:ident, definition) => {
        $slot.definition
    };
    ($slot:ident, graphics) => {
        $slot.graphics
    };
    ($slot:ident, counter) => {
        $slot.counter
    };
    ($slot:ident, pre_action) => {
        $slot.pre_action
    };
    ($slot:ident, x_fraction) => {
        $slot.x.fraction
    };
    ($slot:ident, x) => {
        $slot.x.whole
    };
    ($slot:ident, y_fraction) => {
        $slot.y.fraction
    };
    ($slot:ident, y) => {
        $slot.y.whole
    };
    ($slot:ident, velocity_x) => {
        $slot.velocity.x
    };
    ($slot:ident, velocity_y) => {
        $slot.velocity.y
    };
    ($slot:ident, behavior_a) => {
        $slot.behavior_a
    };
    ($slot:ident, behavior_b) => {
        $slot.behavior_b
    };
    ($slot:ident, program) => {
        $slot.program
    };
    ($slot:ident, spritemap) => {
        $slot.spritemap
    };
    ($slot:ident, delay) => {
        $slot.delay
    };
    ($slot:ident, radius) => {
        $slot.radius
    };
    ($slot:ident, properties) => {
        $slot.properties
    };
    ($slot:ident, behavior_c) => {
        $slot.behavior_c
    };
    ($slot:ident, angle) => {
        $slot.angle
    };
    ($slot:ident, drop_flags) => {
        $slot.drop.flags
    };
    ($slot:ident, drop_definition) => {
        $slot.drop.enemy_definition
    };
    ($slot:ident, killed_enemy) => {
        $slot.drop.killed_enemy
    };
}
macro_rules! enemy {
    ($h:ident, $index:expr, $member:ident) => {{
        let index: isize = $index;
        if let Some(slot) = usize::try_from(index).ok().and_then(|i| $h.session().enemy_projectiles.slots.get(i)) {
            $crate::game::enemy_projectiles::access::enemy_field!(slot, $member)
        } else {
            $h.enemy_word($crate::game::enemy_projectiles::access::enemy_member!($member), index)
        }
    }};
    ($h:ident, $index:expr, $member:ident = $value:expr) => {{
        let index: isize = $index;
        let value: u16 = $value;
        $crate::game::enemy_projectiles::access::enemy!(@write $h, index, $member, value);
    }};
    ($h:ident, $value:expr => $index:expr, $member:ident) => {{
        let value: u16 = $value;
        let index: isize = $index;
        $crate::game::enemy_projectiles::access::enemy!(@write $h, index, $member, value);
    }};
    (@write $h:ident, $index:ident, $member:ident, $value:ident) => {{
        if let Some(slot) = usize::try_from($index).ok().and_then(|i| $h.session_mut().enemy_projectiles.slots.get_mut(i)) {
            $crate::game::enemy_projectiles::access::enemy_field!(slot, $member) = $value;
        } else {
            $h.set_enemy_word($crate::game::enemy_projectiles::access::enemy_member!($member), $index, $value);
        }
    }};
}
macro_rules! enemy_byte {
    ($h:ident, $index:expr, $member:ident) => {{
        let index: isize = $index;
        if let Some((slot, lane)) = usize::try_from(index).ok().and_then(|i| {
            $h.session()
                .enemy_projectiles
                .slots
                .get(i / 2)
                .map(|slot| (slot, i & 1))
        }) {
            $crate::game::enemy_projectiles::access::enemy_field!(slot, $member).to_le_bytes()[lane]
        } else {
            $h.enemy_byte(
                $crate::game::enemy_projectiles::access::enemy_member!($member),
                index,
            )
        }
    }};
    ($h:ident, $index:expr, $member:ident = $value:expr) => {{
        let index: isize = $index;
        let value: u8 = $value;
        if let Some((slot, lane)) = usize::try_from(index).ok().and_then(|i| {
            $h.session_mut()
                .enemy_projectiles
                .slots
                .get_mut(i / 2)
                .map(|slot| (slot, i & 1))
        }) {
            let mut bytes =
                $crate::game::enemy_projectiles::access::enemy_field!(slot, $member).to_le_bytes();
            bytes[lane] = value;
            $crate::game::enemy_projectiles::access::enemy_field!(slot, $member) =
                u16::from_le_bytes(bytes);
        } else {
            $h.set_enemy_byte(
                $crate::game::enemy_projectiles::access::enemy_member!($member),
                index,
                value,
            );
        }
    }};
}
pub(crate) use {enemy, enemy_byte, enemy_field, enemy_member};

use super::host::{EnemyMember, Host};
pub(super) fn read_member(h: &impl Host, member: EnemyMember, index: isize) -> u16 {
    match member {
        EnemyMember::Definition => enemy!(h, index, definition),
        EnemyMember::Graphics => enemy!(h, index, graphics),
        EnemyMember::Counter => enemy!(h, index, counter),
        EnemyMember::PreAction => enemy!(h, index, pre_action),
        EnemyMember::XFraction => enemy!(h, index, x_fraction),
        EnemyMember::X => enemy!(h, index, x),
        EnemyMember::YFraction => enemy!(h, index, y_fraction),
        EnemyMember::Y => enemy!(h, index, y),
        EnemyMember::VelocityX => enemy!(h, index, velocity_x),
        EnemyMember::VelocityY => enemy!(h, index, velocity_y),
        EnemyMember::BehaviorA => enemy!(h, index, behavior_a),
        EnemyMember::BehaviorB => enemy!(h, index, behavior_b),
        EnemyMember::Program => enemy!(h, index, program),
        EnemyMember::Spritemap => enemy!(h, index, spritemap),
        EnemyMember::Delay => enemy!(h, index, delay),
        EnemyMember::Radius => enemy!(h, index, radius),
        EnemyMember::Properties => enemy!(h, index, properties),
        EnemyMember::BehaviorC => enemy!(h, index, behavior_c),
        EnemyMember::Angle => enemy!(h, index, angle),
        EnemyMember::DropFlags => enemy!(h, index, drop_flags),
        EnemyMember::DropDefinition => enemy!(h, index, drop_definition),
        EnemyMember::KilledEnemy => enemy!(h, index, killed_enemy),
    }
}
pub(super) fn write_member(h: &mut impl Host, member: EnemyMember, index: isize, value: u16) -> () {
    match member {
        EnemyMember::Definition => enemy!(h, index, definition = value),
        EnemyMember::Graphics => enemy!(h, index, graphics = value),
        EnemyMember::Counter => enemy!(h, index, counter = value),
        EnemyMember::PreAction => enemy!(h, index, pre_action = value),
        EnemyMember::XFraction => enemy!(h, index, x_fraction = value),
        EnemyMember::X => enemy!(h, index, x = value),
        EnemyMember::YFraction => enemy!(h, index, y_fraction = value),
        EnemyMember::Y => enemy!(h, index, y = value),
        EnemyMember::VelocityX => enemy!(h, index, velocity_x = value),
        EnemyMember::VelocityY => enemy!(h, index, velocity_y = value),
        EnemyMember::BehaviorA => enemy!(h, index, behavior_a = value),
        EnemyMember::BehaviorB => enemy!(h, index, behavior_b = value),
        EnemyMember::Program => enemy!(h, index, program = value),
        EnemyMember::Spritemap => enemy!(h, index, spritemap = value),
        EnemyMember::Delay => enemy!(h, index, delay = value),
        EnemyMember::Radius => enemy!(h, index, radius = value),
        EnemyMember::Properties => enemy!(h, index, properties = value),
        EnemyMember::BehaviorC => enemy!(h, index, behavior_c = value),
        EnemyMember::Angle => enemy!(h, index, angle = value),
        EnemyMember::DropFlags => enemy!(h, index, drop_flags = value),
        EnemyMember::DropDefinition => enemy!(h, index, drop_definition = value),
        EnemyMember::KilledEnemy => enemy!(h, index, killed_enemy = value),
    }
}
pub(super) fn read_member_byte(h: &impl Host, member: EnemyMember, index: isize) -> u8 {
    match member {
        EnemyMember::Definition => enemy_byte!(h, index, definition),
        EnemyMember::Graphics => enemy_byte!(h, index, graphics),
        EnemyMember::Counter => enemy_byte!(h, index, counter),
        EnemyMember::PreAction => enemy_byte!(h, index, pre_action),
        EnemyMember::XFraction => enemy_byte!(h, index, x_fraction),
        EnemyMember::X => enemy_byte!(h, index, x),
        EnemyMember::YFraction => enemy_byte!(h, index, y_fraction),
        EnemyMember::Y => enemy_byte!(h, index, y),
        EnemyMember::VelocityX => enemy_byte!(h, index, velocity_x),
        EnemyMember::VelocityY => enemy_byte!(h, index, velocity_y),
        EnemyMember::BehaviorA => enemy_byte!(h, index, behavior_a),
        EnemyMember::BehaviorB => enemy_byte!(h, index, behavior_b),
        EnemyMember::Program => enemy_byte!(h, index, program),
        EnemyMember::Spritemap => enemy_byte!(h, index, spritemap),
        EnemyMember::Delay => enemy_byte!(h, index, delay),
        EnemyMember::Radius => enemy_byte!(h, index, radius),
        EnemyMember::Properties => enemy_byte!(h, index, properties),
        EnemyMember::BehaviorC => enemy_byte!(h, index, behavior_c),
        EnemyMember::Angle => enemy_byte!(h, index, angle),
        EnemyMember::DropFlags => enemy_byte!(h, index, drop_flags),
        EnemyMember::DropDefinition => enemy_byte!(h, index, drop_definition),
        EnemyMember::KilledEnemy => enemy_byte!(h, index, killed_enemy),
    }
}
pub(super) fn write_member_byte(
    h: &mut impl Host,
    member: EnemyMember,
    index: isize,
    value: u8,
) -> () {
    match member {
        EnemyMember::Definition => enemy_byte!(h, index, definition = value),
        EnemyMember::Graphics => enemy_byte!(h, index, graphics = value),
        EnemyMember::Counter => enemy_byte!(h, index, counter = value),
        EnemyMember::PreAction => enemy_byte!(h, index, pre_action = value),
        EnemyMember::XFraction => enemy_byte!(h, index, x_fraction = value),
        EnemyMember::X => enemy_byte!(h, index, x = value),
        EnemyMember::YFraction => enemy_byte!(h, index, y_fraction = value),
        EnemyMember::Y => enemy_byte!(h, index, y = value),
        EnemyMember::VelocityX => enemy_byte!(h, index, velocity_x = value),
        EnemyMember::VelocityY => enemy_byte!(h, index, velocity_y = value),
        EnemyMember::BehaviorA => enemy_byte!(h, index, behavior_a = value),
        EnemyMember::BehaviorB => enemy_byte!(h, index, behavior_b = value),
        EnemyMember::Program => enemy_byte!(h, index, program = value),
        EnemyMember::Spritemap => enemy_byte!(h, index, spritemap = value),
        EnemyMember::Delay => enemy_byte!(h, index, delay = value),
        EnemyMember::Radius => enemy_byte!(h, index, radius = value),
        EnemyMember::Properties => enemy_byte!(h, index, properties = value),
        EnemyMember::BehaviorC => enemy_byte!(h, index, behavior_c = value),
        EnemyMember::Angle => enemy_byte!(h, index, angle = value),
        EnemyMember::DropFlags => enemy_byte!(h, index, drop_flags = value),
        EnemyMember::DropDefinition => enemy_byte!(h, index, drop_definition = value),
        EnemyMember::KilledEnemy => enemy_byte!(h, index, killed_enemy = value),
    }
}

pub(super) fn shot_word(h: &impl Host, member: super::host::ShotMember, index: isize) -> u16 {
    if let Some(shot) = usize::try_from(index)
        .ok()
        .and_then(|i| h.session().projectiles.shots.get(i))
    {
        match member {
            super::host::ShotMember::Kind => shot.kind,
            super::host::ShotMember::Direction => shot.direction,
            super::host::ShotMember::Variable => shot.variable,
            super::host::ShotMember::X => shot.x,
            super::host::ShotMember::Y => shot.y,
            super::host::ShotMember::XRadius => shot.x_radius,
            super::host::ShotMember::YRadius => shot.y_radius,
            super::host::ShotMember::Damage => shot.damage,
        }
    } else {
        h.shot_word(member, index)
    }
}
pub(super) fn set_shot_direction(h: &mut impl Host, index: isize, value: u16) {
    if let Some(shot) = usize::try_from(index)
        .ok()
        .and_then(|i| h.session_mut().projectiles.shots.get_mut(i))
    {
        shot.direction = value;
    } else {
        h.set_shot_direction(index, value);
    }
}

/// Arithmetic arguments are captured before reading the old field value.
macro_rules! enemy_add {
    ($h:ident, $index:expr, $member:ident, $amount:expr) => {{
        let index: isize = $index;
        let amount: u16 = $amount;
        let value = $crate::game::enemy_projectiles::access::enemy!($h, index, $member)
            .wrapping_add(amount);
        $crate::game::enemy_projectiles::access::enemy!($h, index, $member = value);
        value
    }};
}
macro_rules! enemy_decrement {
    ($h:ident, $index:expr, $member:ident) => {{
        let index: isize = $index;
        let old = $crate::game::enemy_projectiles::access::enemy!($h, index, $member);
        $crate::game::enemy_projectiles::access::enemy!($h, index, $member = old.wrapping_sub(1));
        old
    }};
}
pub(crate) use {enemy_add, enemy_decrement};
