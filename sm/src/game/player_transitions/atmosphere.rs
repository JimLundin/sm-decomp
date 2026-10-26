//! Gameplay liquid effects over the Session's four atmospheric slots.
#![forbid(unsafe_code)]
use super::{effects::Host, pose};
use crate::game::samus_palette;

pub(super) fn advance(h: &mut impl Host) {
    match (h.inputs().transitions.fx_kind & 15) >> 1 {
        0 => no_fx(h),
        1 => lava(h),
        2 => acid(h),
        3 => water(h),
        _ => {}
    }
}
fn no_fx(h: &mut impl Host) {
    let bottom = pose::loaded_bottom(h);
    h.session_mut().player.transitions.animation_buffer = h.session().player.speed_divisor;
    let liquid = h.session().player.transitions.liquid_physics;
    if liquid == 0 {
        return;
    }
    h.session_mut().player.transitions.liquid_physics = 0;
    if liquid & 1 == 0 {
        return;
    }
    h.sound(2, 6, 0xe);
    let suit = h.session().player.palette.suit_index;
    if suit & 4 == 0 && matches!(h.session().player.movement_and_facing >> 8, 3 | 20) {
        h.sound(1, 6, 0x30);
    }
    splash(h, bottom);
}
fn water(h: &mut impl Host) {
    let bottom = pose::loaded_bottom(h);
    if h.inputs().fx_y & 0x8000 == 0
        && h.inputs().fx_y.wrapping_sub(bottom) & 0x8000 != 0
        && h.session().room.liquid_options & 4 == 0
    {
        h.session_mut().player.transitions.animation_buffer = 3;
        if h.session().player.transitions.liquid_physics == 1 {
            bubbles(h);
        } else {
            h.session_mut().player.transitions.liquid_physics = 1;
            h.sound(2, 6, 0xd);
            splash(h, bottom);
        }
    } else {
        no_fx(h);
    }
}
fn splash(h: &mut impl Host, bottom: u16) {
    const TWO_SLOTS: [u8; 28] = [
        1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    ];
    if TWO_SLOTS[usize::from(h.session().player.movement_and_facing >> 8)] != 0 {
        h.session_mut().atmospheric.slots[0].frame_and_kind = 0x100;
        h.session_mut().atmospheric.slots[1].frame_and_kind = 0x100;
        h.session_mut().atmospheric.slots[0].remaining = 3;
        h.session_mut().atmospheric.slots[1].remaining = 3;
        h.session_mut().atmospheric.slots[0].x = h.session().player.x.wrapping_add(4);
        h.session_mut().atmospheric.slots[1].x = h.session().player.x.wrapping_sub(3);
        h.session_mut().atmospheric.slots[0].y = bottom.wrapping_sub(4);
        h.session_mut().atmospheric.slots[1].y = bottom.wrapping_sub(4);
    } else {
        h.session_mut().atmospheric.slots[0].frame_and_kind = 0x300;
        h.session_mut().atmospheric.slots[0].remaining = 2;
        h.session_mut().atmospheric.slots[0].x = h.session().player.x;
        h.session_mut().atmospheric.slots[0].y = h.inputs().fx_y;
    }
    bubbles(h);
}
fn clear_buffer_if_suited(h: &mut impl Host) {
    if matches!(h.session().player.pose, 0 | 0x9b)
        || h.session().inventory.equipment.active & 0x20 != 0
    {
        h.session_mut().player.transitions.animation_buffer = 0;
    }
}
fn bubbles(h: &mut impl Host) {
    let top = pose::loaded_top(h);
    if top.wrapping_sub(24).wrapping_sub(h.inputs().fx_y) & 0x8000 == 0
        && h.inputs().frame & 0x7f == 0
        && h.session().atmospheric.slots[2].frame_and_kind == 0
    {
        h.session_mut().atmospheric.slots[2].frame_and_kind = 0x500;
        h.session_mut().atmospheric.slots[2].remaining = 3;
        h.session_mut().atmospheric.slots[2].x = h.session().player.x;
        // This source uses the live collision radius, unlike the top helper.
        h.session_mut().atmospheric.slots[2].y = h
            .session()
            .player
            .y
            .wrapping_sub(h.session().player.contact.radius)
            .wrapping_add(6);
        let random = h.random_word();
        h.sound(2, 6, if random & 1 != 0 { 15 } else { 17 });
    }
    clear_buffer_if_suited(h);
}
fn damage(h: &mut impl Host, whole: u16) {
    let (fraction, carry) = h
        .session()
        .player
        .periodic_damage_fraction
        .overflowing_add(0x8000);
    let damage = h
        .session()
        .player
        .periodic_damage
        .wrapping_add(whole)
        .wrapping_add(u16::from(carry));
    h.session_mut().player.periodic_damage_fraction = fraction;
    h.session_mut().player.periodic_damage = damage;
}
fn damage_sound(h: &mut impl Host) {
    if h.inputs().transitions.game_time_frames & 7 == 0
        && h.session().inventory.health.current.wrapping_sub(71) & 0x8000 == 0
    {
        h.sound(3, 3, 0x2d);
    }
}
fn lava(h: &mut impl Host) {
    let bottom = pose::loaded_bottom(h);
    if h.inputs().lava_y & 0x8000 == 0 && h.inputs().lava_y.wrapping_sub(bottom) & 0x8000 != 0 {
        if h.session().player.contact.speed_boost != 0 {
            samus_palette::cancel_speed_boost(h);
            h.session_mut().player.contact.extra_run.whole = 0;
            h.session_mut().player.contact.extra_run.fraction = 0;
        }
        if h.session().inventory.equipment.active & 0x20 != 0 {
            h.session_mut().player.transitions.animation_buffer = 0;
            h.session_mut().player.transitions.liquid_physics = 2;
        } else {
            damage_sound(h);
            damage(h, 0);
            submerged_lava_acid(h);
        }
    } else {
        no_fx(h);
    }
}
fn acid(h: &mut impl Host) {
    let bottom = pose::loaded_bottom(h);
    if h.inputs().lava_y & 0x8000 == 0 && h.inputs().lava_y.wrapping_sub(bottom) & 0x8000 != 0 {
        damage_sound(h);
        damage(h, 1);
        submerged_lava_acid(h);
    } else {
        no_fx(h);
    }
}
fn submerged_lava_acid(h: &mut impl Host) {
    let top = pose::loaded_top(h);
    h.session_mut().player.transitions.animation_buffer = 2;
    h.session_mut().player.transitions.liquid_physics = 2;
    if top.wrapping_sub(h.inputs().lava_y) & 0x8000 != 0
        && h.session().atmospheric.slots[0].frame_and_kind & 0x400 == 0
    {
        for slot in &mut h.session_mut().atmospheric.slots {
            slot.frame_and_kind = 0x400;
        }
        h.session_mut().atmospheric.slots[0].remaining = 3;
        h.session_mut().atmospheric.slots[3].remaining = 3;
        h.session_mut().atmospheric.slots[1].remaining = 0x8002;
        h.session_mut().atmospheric.slots[2].remaining = 0x8002;
        for i in 0..4 {
            h.session_mut().atmospheric.slots[i].y = h.inputs().lava_y;
        }
        h.session_mut().atmospheric.slots[0].x = h.session().player.x.wrapping_add(6);
        h.session_mut().atmospheric.slots[1].x = h.session().player.x;
        h.session_mut().atmospheric.slots[2].x = h.session().player.x;
        h.session_mut().atmospheric.slots[3].x = h.session().player.x.wrapping_sub(6);
        if h.inputs().transitions.game_time_frames & 1 == 0 {
            h.sound(2, 6, 0x10);
        }
    }
    clear_buffer_if_suited(h);
}

/// Advance the four owned effects in their original descending draw order.
pub(crate) fn draw_atmospheric(h: &mut impl crate::game::visual_host::Host) {
    use crate::game::visual_host::{Record, Source};
    h.probe("atmosphere.advance");
    for slot in (0..4).rev() {
        let initial = h.session().atmospheric.slots[slot].frame_and_kind;
        if initial == 0 {
            continue;
        }
        let kind = initial >> 8;
        let frame_offset = 2 * u16::from(initial as u8);
        let remaining = h.session().atmospheric.slots[slot]
            .remaining
            .wrapping_sub(1);
        h.session_mut().atmospheric.slots[slot].remaining = remaining;
        if remaining == 0 || remaining == 0x8000 {
            h.probe("atmosphere.reload");
            let pointer = h
                .table(Source::Loaded, 0x908b93)
                .native_word(2 * usize::from(kind));
            let duration = h
                .legacy_native_record(0x90, pointer.wrapping_add(frame_offset))
                .native_word(0);
            h.session_mut().atmospheric.slots[slot].remaining = duration;
            if remaining == 0 {
                let next = h.session().atmospheric.slots[slot]
                    .frame_and_kind
                    .wrapping_add(1);
                h.session_mut().atmospheric.slots[slot].frame_and_kind = next;
                let count = h
                    .table(Source::Loaded, 0x908bef)
                    .native_word(2 * usize::from(kind));
                if i16::from(next as u8).wrapping_sub(count as i16) >= 0 {
                    h.session_mut().atmospheric.slots[slot].frame_and_kind = 0;
                    h.probe("atmosphere.retire");
                    continue;
                }
            }
        } else if remaining & 0x8000 != 0 {
            continue;
        }
        // Preserve the checked dispatch and null slot after the timer's effects.
        match [0, 1, 1, 3, 4, 5, 6, 6][usize::from(kind)] {
            0 => panic!("non-null function pointer"),
            1 => {
                h.probe("atmosphere.footstep");
                h.draw_atmospheric_tile(slot, kind);
            }
            3 | 5 => {
                let frame = u16::from(h.session().atmospheric.slots[slot].frame_and_kind as u8);
                let sprite = if kind == 3 {
                    h.probe("atmosphere.dive");
                    h.session_mut().atmospheric.slots[slot].y = h.inputs().fx_y;
                    frame.wrapping_add(399)
                } else {
                    h.probe("atmosphere.bubbles");
                    frame.wrapping_add(390)
                };
                if h.session().atmospheric.slots[slot]
                    .y
                    .wrapping_sub(h.inputs().camera_y)
                    & 0xff00
                    == 0
                {
                    h.draw_samus_sprite(
                        sprite,
                        h.session().atmospheric.slots[slot]
                            .x
                            .wrapping_sub(h.inputs().camera_x),
                        h.session().atmospheric.slots[slot]
                            .y
                            .wrapping_sub(h.inputs().camera_y),
                    );
                }
            }
            _ => {
                if kind == 4 {
                    h.probe("atmosphere.lava");
                    let x = h.session().atmospheric.slots[slot].x;
                    h.session_mut().atmospheric.slots[slot].x = if slot & 2 != 0 {
                        x.wrapping_sub(1)
                    } else {
                        x.wrapping_add(1)
                    };
                } else {
                    h.probe("atmosphere.dust");
                }
                h.session_mut().atmospheric.slots[slot].y =
                    h.session().atmospheric.slots[slot].y.wrapping_sub(1);
                h.draw_atmospheric_tile(slot, kind);
            }
        }
    }
}
