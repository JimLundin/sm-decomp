//! Attached sprite effects and Draygon motion on the single gameplay Session.
#![forbid(unsafe_code)]
use crate::game::{
    session::Point,
    sprite_effect_host::{Host, SpriteMember},
    sprite_effect_state::{
        LegacySpriteDefinition, LegacySpriteHandle, LegacySpriteProgram, LegacySpriteWordIndex,
        SpriteSpawn,
    },
    sprite_objects,
    visual_host::{Record, Source},
};

// Gameplay slots are ordinary fields. Only compatibility selectors leave the
// active pool; the aggregate projection owns tails and cross-owner aliases.
macro_rules! read {
    ($host:ident, $index:expr, $member:ident, $($field:ident).+) => {{
        let index: u16 = $index;
        match $host.session().sprite_effects.active.get(usize::from(index)) {
            Some(slot) => slot.$($field).+,
            None => $host.sprite_word(SpriteMember::$member, LegacySpriteWordIndex(index)),
        }
    }};
}
macro_rules! write {
    ($host:ident, $index:expr, $member:ident, $($field:ident).+, $value:expr) => {{
        let index: u16 = $index;
        let value: u16 = $value;
        match $host.session_mut().sprite_effects.active.get_mut(usize::from(index)) {
            Some(slot) => slot.$($field).+ = value,
            None => $host.set_sprite_word_le_slice(SpriteMember::$member, LegacySpriteWordIndex(index), value),
        }
    }};
}
macro_rules! read_native {
    ($host:ident, $index:expr, $member:ident, $($field:ident).+) => {{
        let index: u16 = $index;
        match $host.session().sprite_effects.active.get(usize::from(index)) {
            Some(slot) => u16::from_ne_bytes(slot.$($field).+.to_le_bytes()),
            None => $host.sprite_word_native(SpriteMember::$member, LegacySpriteWordIndex(index)),
        }
    }};
}
macro_rules! write_native {
    ($host:ident, $index:expr, $member:ident, $($field:ident).+, $value:expr) => {{
        let index: u16 = $index;
        let value: u16 = $value;
        match $host.session_mut().sprite_effects.active.get_mut(usize::from(index)) {
            Some(slot) => slot.$($field).+ = u16::from_le_bytes(value.to_ne_bytes()),
            None => $host.set_sprite_word_native_slice(SpriteMember::$member, LegacySpriteWordIndex(index), value),
        }
    }};
}

pub(crate) fn position(host: &impl Host, handle: LegacySpriteHandle) -> Point {
    let index = handle.0 >> 1;
    Point {
        x: read!(host, index, X, x.whole),
        y: read!(host, index, Y, y.whole),
    }
}
pub(crate) fn set_position(host: &mut impl Host, handle: LegacySpriteHandle, position: Point) {
    let index = handle.0 >> 1;
    write!(host, index, X, x.whole, position.x);
    write!(host, index, Y, y.whole, position.y);
}
pub(crate) fn retire(host: &mut impl Host, handle: LegacySpriteHandle) {
    write!(host, handle.0 >> 1, Instruction, instruction, 0);
}
/// Raw Metroid retirement preserves native-store provenance at the legacy edge.
pub(crate) fn retire_native(host: &mut impl Host, handle: LegacySpriteHandle) {
    write_native!(host, handle.0 >> 1, Instruction, instruction, 0);
}
pub(crate) fn freeze_attachment(
    host: &mut impl Host,
    handle: LegacySpriteHandle,
    appearance: u16,
    program: LegacySpriteProgram,
) {
    let index = handle.0 >> 1;
    write!(host, index, Appearance, appearance, appearance);
    write!(host, index, DisableFlags, disable_flags, 1);
    write!(host, index, Instruction, instruction, program.0);
}
/// Retain the actor's live Y observation after the sprite X write, including
/// defined broad selectors that alias the actor's position.
pub(crate) fn attach_actor(
    host: &mut impl Host,
    handle: LegacySpriteHandle,
    enemy: u16,
    appearance: u16,
) {
    let index = handle.0 >> 1;
    let x = host.attachment_x(enemy);
    write!(host, index, X, x.whole, x);
    let y = host.attachment_y(enemy);
    write!(host, index, Y, y.whole, y);
    write!(host, index, Appearance, appearance, appearance);
    write!(host, index, DisableFlags, disable_flags, 0);
}
pub(crate) fn flash_attachment(host: &mut impl Host, handle: LegacySpriteHandle, appearance: u16) {
    let index = handle.0 >> 1;
    let old = read!(host, index, Appearance, appearance);
    write!(
        host,
        index,
        Appearance,
        appearance,
        appearance | (old & 0xf1ff)
    );
}
pub(crate) fn maw_appearance(
    host: &mut impl Host,
    handle: LegacySpriteHandle,
    appearance: u16,
    enemy: u16,
) {
    let index = handle.0 >> 1;
    let old = read!(host, index, Appearance, appearance);
    write!(
        host,
        index,
        Appearance,
        appearance,
        appearance | (old & 0xf1ff)
    );
    if host.attachment_frozen_timer(enemy) != 0 {
        let old = read!(host, index, Appearance, appearance);
        write!(host, index, Appearance, appearance, (old & 0xf1ff) | 0xc00);
        let frozen_timer = host.attachment_frozen_timer(enemy);
        if frozen_timer as i32 - 90 & 0x8000 != 0 && frozen_timer & 2 == 0 {
            let old = read!(host, index, Appearance, appearance);
            write!(
                host,
                index,
                Appearance,
                appearance,
                appearance | (old & 0xf1ff)
            );
        }
    }
}
#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
}
fn displace_axis(host: &mut impl Host, index: u16, axis: Axis, delta: u32) {
    let fraction = match axis {
        Axis::X => read_native!(host, index, XFraction, x.fraction),
        Axis::Y => read_native!(host, index, YFraction, y.fraction),
    };
    let whole = match axis {
        Axis::X => read_native!(host, index, X, x.whole),
        Axis::Y => read_native!(host, index, Y, y.whole),
    };
    let value = (u32::from(fraction) | (u32::from(whole) << 16)).wrapping_add(delta);
    match axis {
        Axis::X => {
            write_native!(host, index, XFraction, x.fraction, value as u16);
            write_native!(host, index, X, x.whole, (value >> 16) as u16);
        }
        Axis::Y => {
            write_native!(host, index, YFraction, y.fraction, value as u16);
            write_native!(host, index, Y, y.whole, (value >> 16) as u16);
        }
    }
}
fn loaded_word(host: &impl Host, address: u32, index: usize) -> u16 {
    host.table(Source::Loaded, address).native_word(
        index
            .checked_mul(2)
            .expect("sprite motion table offset overflow"),
    )
}
pub(crate) fn draygon_motion(host: &mut impl Host, horizontal_bias: u16) {
    let mut handle = 62u16;
    for offset in (0usize..=20).rev().step_by(4) {
        let index = offset >> 1;
        let slot = handle >> 1;
        let direction = loaded_word(host, 0xa5a1df, index);
        let speed = u32::from(loaded_word(host, 0xa5a1af, index));
        let x = if (u32::from(direction) + 64) & 0x80 != 0 {
            speed
        } else {
            speed
                .wrapping_neg()
                .wrapping_sub(u32::from(horizontal_bias) << 16)
        };
        displace_axis(host, slot, Axis::X, x);
        // The source performs this additional LE fraction observation before
        // recapturing the direction table for Y.
        let _fraction = read!(host, slot, YFraction, y.fraction);
        let direction = loaded_word(host, 0xa5a1df, index);
        let speed = u32::from(loaded_word(host, 0xa5a1af, index + 1));
        let y = if (u32::from(direction) + 128) & 0x80 != 0 {
            speed
        } else {
            speed.wrapping_neg()
        };
        displace_axis(host, slot, Axis::Y, y);
        handle = handle.wrapping_sub(2);
    }
}
pub(crate) fn draygon_spawn_ring(host: &mut impl Host) {
    sprite_objects::retire_all(host.session_mut());
    let mut offset = 20u16;
    for definition in [59, 60] {
        for _ in 0..3 {
            let index = usize::from(offset >> 1);
            let x = loaded_word(host, 0xa5a1c7, index);
            let y = loaded_word(host, 0xa5a1c7, index + 1);
            sprite_objects::spawn(
                host,
                SpriteSpawn {
                    position: Point { x, y },
                    definition: LegacySpriteDefinition(definition),
                    appearance: 0xe00,
                },
            );
            offset = offset.wrapping_sub(4);
        }
    }
}
pub(crate) fn draygon_spawn_trails(host: &mut impl Host) {
    for _ in 0..4 {
        sprite_objects::spawn(
            host,
            SpriteSpawn {
                position: Point { x: 16, y: 384 },
                definition: LegacySpriteDefinition(59),
                appearance: 0xe00,
            },
        );
    }
}
pub(crate) fn draygon_trajectory(host: &mut impl Host, cursor: u16) -> u16 {
    for handle in (56u16..=62).rev().step_by(2) {
        #[cfg(sm_verify_dll)]
        host.observe_draygon(1, handle, 0);
        let offset = loaded_word(host, 0xa5a19f, usize::from((handle.wrapping_sub(56)) >> 1));
        let position = cursor.wrapping_add(offset);
        if position & 0x8000 != 0 {
            #[cfg(sm_verify_dll)]
            {
                host.observe_draygon(3, handle, 0);
                host.observe_draygon(6, handle, 0);
            }
            continue;
        }
        // Native source captures are intentionally separate. The physical
        // displacement stays wide; it does not bank-wrap CE07 + position.
        let sentinel = host
            .record(Source::Native, 0xa5, 0xce07)
            .native_word(usize::from(position));
        #[cfg(sm_verify_dll)]
        host.observe_draygon(2, handle, sentinel);
        if sentinel == 0x8080 {
            #[cfg(sm_verify_dll)]
            host.observe_draygon(3, handle, 1);
            retire(host, LegacySpriteHandle(handle));
        } else {
            #[cfg(sm_verify_dll)]
            host.observe_draygon(3, handle, 2);
            let slot = handle >> 1;
            let x = read!(host, slot, X, x.whole);
            let dx = host
                .record(Source::Native, 0xa5, 0xce07)
                .byte(usize::from(position)) as i8 as i16;
            #[cfg(sm_verify_dll)]
            host.observe_draygon(4, handle, dx as u8 as u16);
            write!(host, slot, X, x.whole, x.wrapping_add_signed(dx));
            let y = read!(host, slot, Y, y.whole);
            let dy = host
                .record(Source::Native, 0xa5, 0xce07)
                .byte(usize::from(position) + 1) as i8 as i16;
            #[cfg(sm_verify_dll)]
            host.observe_draygon(5, handle, dy as u8 as u16);
            write!(host, slot, Y, y.whole, y.wrapping_add_signed(dy));
        }
        #[cfg(sm_verify_dll)]
        host.observe_draygon(6, handle, 0);
    }
    cursor.wrapping_add(4)
}
