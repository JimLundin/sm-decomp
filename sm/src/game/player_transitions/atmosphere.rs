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
        h.atmospheric_mut().slots[0].frame_and_kind = 0x100;
        h.atmospheric_mut().slots[1].frame_and_kind = 0x100;
        h.atmospheric_mut().slots[0].remaining = 3;
        h.atmospheric_mut().slots[1].remaining = 3;
        h.atmospheric_mut().slots[0].x = h.session().player.x.wrapping_add(4);
        h.atmospheric_mut().slots[1].x = h.session().player.x.wrapping_sub(3);
        h.atmospheric_mut().slots[0].y = bottom.wrapping_sub(4);
        h.atmospheric_created(0)
            .expect("atmospheric producer allocation");
        h.atmospheric_mut().slots[1].y = bottom.wrapping_sub(4);
        h.atmospheric_created(1)
            .expect("atmospheric producer allocation");
    } else {
        h.atmospheric_mut().slots[0].frame_and_kind = 0x300;
        h.atmospheric_mut().slots[0].remaining = 2;
        h.atmospheric_mut().slots[0].x = h.session().player.x;
        h.atmospheric_mut().slots[0].y = h.inputs().fx_y;
        h.atmospheric_created(0)
            .expect("atmospheric producer allocation");
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
        && h.atmospheric().slots[2].frame_and_kind == 0
    {
        h.atmospheric_mut().slots[2].frame_and_kind = 0x500;
        h.atmospheric_mut().slots[2].remaining = 3;
        h.atmospheric_mut().slots[2].x = h.session().player.x;
        // This source uses the live collision radius, unlike the top helper.
        h.atmospheric_mut().slots[2].y = h
            .session()
            .player
            .y
            .wrapping_sub(h.session().player.contact.radius)
            .wrapping_add(6);
        h.atmospheric_created(2)
            .expect("atmospheric producer allocation");
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
        && h.atmospheric().slots[0].frame_and_kind & 0x400 == 0
    {
        for slot in &mut h.atmospheric_mut().slots {
            slot.frame_and_kind = 0x400;
        }
        h.atmospheric_mut().slots[0].remaining = 3;
        h.atmospheric_mut().slots[3].remaining = 3;
        h.atmospheric_mut().slots[1].remaining = 0x8002;
        h.atmospheric_mut().slots[2].remaining = 0x8002;
        for i in 0..4 {
            h.atmospheric_mut().slots[i].y = h.inputs().lava_y;
        }
        h.atmospheric_mut().slots[0].x = h.session().player.x.wrapping_add(6);
        h.atmospheric_created(0)
            .expect("atmospheric producer allocation");
        h.atmospheric_mut().slots[1].x = h.session().player.x;
        h.atmospheric_created(1)
            .expect("atmospheric producer allocation");
        h.atmospheric_mut().slots[2].x = h.session().player.x;
        h.atmospheric_created(2)
            .expect("atmospheric producer allocation");
        h.atmospheric_mut().slots[3].x = h.session().player.x.wrapping_sub(6);
        h.atmospheric_created(3)
            .expect("atmospheric producer allocation");
        if h.inputs().transitions.game_time_frames & 1 == 0 {
            h.sound(2, 6, 0x10);
        }
    }
    clear_buffer_if_suited(h);
}

/// Advance the actual owner through the same ordered core as portable G.
pub(crate) fn draw_atmospheric(h: &mut impl crate::game::visual_host::Host) {
    crate::game::scene::atmosphere::advance(&mut NativeAdvance(h)).unwrap();
}
struct NativeAdvance<'a, H>(&'a mut H);
impl<H: crate::game::visual_host::Host> crate::game::scene::atmosphere::AdvanceHost
    for NativeAdvance<'_, H>
{
    type Error = std::convert::Infallible;
    fn frame_and_kind(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(self.0.atmospheric().slots[slot].frame_and_kind)
    }
    fn set_frame_and_kind(&mut self, slot: usize, value: u16) -> Result<(), Self::Error> {
        self.0.atmospheric_mut().slots[slot].frame_and_kind = value;
        if value == 0 {
            self.0
                .atmospheric_created(slot)
                .expect("atmospheric producer allocation");
        }
        Ok(())
    }
    fn remaining(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(self.0.atmospheric().slots[slot].remaining)
    }
    fn set_remaining(&mut self, slot: usize, value: u16) -> Result<(), Self::Error> {
        self.0.atmospheric_mut().slots[slot].remaining = value;
        Ok(())
    }
    fn x(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(self.0.atmospheric().slots[slot].x)
    }
    fn set_x(&mut self, slot: usize, value: u16) -> Result<(), Self::Error> {
        self.0.atmospheric_mut().slots[slot].x = value;
        Ok(())
    }
    fn y(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(self.0.atmospheric().slots[slot].y)
    }
    fn set_y(&mut self, slot: usize, value: u16) -> Result<(), Self::Error> {
        self.0.atmospheric_mut().slots[slot].y = value;
        Ok(())
    }
    fn camera_x(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.inputs().camera_x)
    }
    fn camera_y(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.inputs().camera_y)
    }
    fn fx_y(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.inputs().fx_y)
    }
    fn duration_pointer(&mut self, kind: u16) -> Result<u16, Self::Error> {
        use crate::game::visual_host::Record;
        Ok(self
            .0
            .table(crate::game::visual_host::Source::Loaded, 0x908b93)
            .native_word(2 * usize::from(kind)))
    }
    fn duration(&mut self, address: u16) -> Result<u16, Self::Error> {
        use crate::game::visual_host::Record;
        Ok(self.0.legacy_native_record(0x90, address).native_word(0))
    }
    fn frame_count(&mut self, kind: u16) -> Result<u16, Self::Error> {
        use crate::game::visual_host::Record;
        Ok(self
            .0
            .table(crate::game::visual_host::Source::Loaded, 0x908bef)
            .native_word(2 * usize::from(kind)))
    }
    fn draw_tile(&mut self, slot: usize, kind: u16) -> Result<(), Self::Error> {
        self.0.draw_atmospheric_tile(slot, kind);
        Ok(())
    }
    fn draw_sprite(&mut self, _: usize, sprite: u16, x: u16, y: u16) -> Result<(), Self::Error> {
        self.0.draw_samus_sprite(sprite, x, y);
        Ok(())
    }
    fn probe(&mut self, name: &'static str) {
        self.0.probe(name);
    }
}
