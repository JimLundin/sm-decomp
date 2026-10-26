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
// Word and byte operations share one mapping for the resolved slot.
macro_rules! slot_word {
    ($slot:ident, $member:expr, ($($borrow:tt)*)) => {
        match $member {
            EnemyMember::Definition => $($borrow)* $slot.definition,
            EnemyMember::Graphics => $($borrow)* $slot.graphics,
            EnemyMember::Counter => $($borrow)* $slot.counter,
            EnemyMember::PreAction => $($borrow)* $slot.pre_action,
            EnemyMember::XFraction => $($borrow)* $slot.x.fraction,
            EnemyMember::X => $($borrow)* $slot.x.whole,
            EnemyMember::YFraction => $($borrow)* $slot.y.fraction,
            EnemyMember::Y => $($borrow)* $slot.y.whole,
            EnemyMember::VelocityX => $($borrow)* $slot.velocity.x,
            EnemyMember::VelocityY => $($borrow)* $slot.velocity.y,
            EnemyMember::BehaviorA => $($borrow)* $slot.behavior_a,
            EnemyMember::BehaviorB => $($borrow)* $slot.behavior_b,
            EnemyMember::Program => $($borrow)* $slot.program,
            EnemyMember::Spritemap => $($borrow)* $slot.spritemap,
            EnemyMember::Delay => $($borrow)* $slot.delay,
            EnemyMember::Radius => $($borrow)* $slot.radius,
            EnemyMember::Properties => $($borrow)* $slot.properties,
            EnemyMember::BehaviorC => $($borrow)* $slot.behavior_c,
            EnemyMember::Angle => $($borrow)* $slot.angle,
            EnemyMember::DropFlags => $($borrow)* $slot.drop.flags,
            EnemyMember::DropDefinition => $($borrow)* $slot.drop.enemy_definition,
            EnemyMember::KilledEnemy => $($borrow)* $slot.drop.killed_enemy,
        }
    };
}
macro_rules! enemy {
    ($h:ident, $index:expr, $member:ident) => {{
        let index: isize = $index;
        $crate::game::enemy_projectiles::access::read_member(
            $h,
            $crate::game::enemy_projectiles::access::enemy_member!($member),
            index,
        )
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
        $crate::game::enemy_projectiles::access::write_member(
            $h,
            $crate::game::enemy_projectiles::access::enemy_member!($member),
            $index,
            $value,
        );
    }};
}
macro_rules! enemy_byte {
    ($h:ident, $index:expr, $member:ident) => {{
        let index: isize = $index;
        $crate::game::enemy_projectiles::access::read_member_byte(
            $h,
            $crate::game::enemy_projectiles::access::enemy_member!($member),
            index,
        )
    }};
    ($h:ident, $index:expr, $member:ident = $value:expr) => {{
        let index: isize = $index;
        let value: u8 = $value;
        $crate::game::enemy_projectiles::access::write_member_byte(
            $h,
            $crate::game::enemy_projectiles::access::enemy_member!($member),
            index,
            value,
        );
    }};
}
pub(crate) use {enemy, enemy_byte, enemy_member};

use super::host::{EnemyMember, Host};
pub(super) fn read_member(h: &impl Host, member: EnemyMember, index: isize) -> u16 {
    if let Some(slot) = usize::try_from(index)
        .ok()
        .and_then(|i| h.session().enemy_projectiles.slots.get(i))
    {
        slot_word!(slot, member, ())
    } else {
        h.enemy_word(member, index)
    }
}
pub(super) fn write_member(h: &mut impl Host, member: EnemyMember, index: isize, value: u16) {
    if let Some(slot) = usize::try_from(index)
        .ok()
        .and_then(|i| h.session_mut().enemy_projectiles.slots.get_mut(i))
    {
        *slot_word!(slot, member, (&mut)) = value;
    } else {
        h.set_enemy_word(member, index, value);
    }
}
pub(super) fn read_member_byte(h: &impl Host, member: EnemyMember, index: isize) -> u8 {
    if let Some((slot, lane)) = usize::try_from(index).ok().and_then(|i| {
        h.session()
            .enemy_projectiles
            .slots
            .get(i / 2)
            .map(|slot| (slot, i & 1))
    }) {
        slot_word!(slot, member, ()).to_le_bytes()[lane]
    } else {
        h.enemy_byte(member, index)
    }
}
pub(super) fn write_member_byte(h: &mut impl Host, member: EnemyMember, index: isize, value: u8) {
    if let Some((slot, lane)) = usize::try_from(index).ok().and_then(|i| {
        h.session_mut()
            .enemy_projectiles
            .slots
            .get_mut(i / 2)
            .map(|slot| (slot, i & 1))
    }) {
        let word = slot_word!(slot, member, (&mut));
        let mut bytes = word.to_le_bytes();
        bytes[lane] = value;
        *word = u16::from_le_bytes(bytes);
    } else {
        h.set_enemy_byte(member, index, value);
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
