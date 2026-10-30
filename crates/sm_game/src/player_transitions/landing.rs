//! Landing sounds and region policy over the same owned atmospheric slots.
use super::{
    content::{Content, LandingRegion, LandingRule},
    effects::Host,
    pose,
};
use crate::liquid_surface::LiquidAtHeight;

pub fn handle_landing(h: &mut impl Host) {
    if matches!(
        h.session().player.history.previous.movement_and_facing >> 8,
        3 | 20
    ) && h.inputs().cinematic == 0
    {
        let cue = if matches!(h.session().player.history.previous.pose, 0x81 | 0x82) {
            0x34
        } else {
            0x32
        };
        h.sound(1, 6, cue);
    }
    if h.session().player.contact.y_speed.whole != 0
        && h.session().player.contact.y_speed.whole.wrapping_sub(5) & 0x8000 == 0
    {
        if h.inputs().cinematic == 0 {
            h.sound(3, 6, 4);
        }
    } else {
        if h.session().player.contact.y_speed.fraction == 0 {
            return;
        }
        if h.inputs().cinematic == 0 {
            h.sound(3, 6, 5);
        }
    }
    graphics(h);
}
pub fn graphics(h: &mut impl Host) {
    let region = Content::landing_region(h);
    let rule = if region == LandingRegion::Crateria && h.inputs().cinematic != 0 {
        LandingRule::Clear
    } else {
        Content::landing_room_rule(h, region)
    };
    match rule {
        LandingRule::Dust => particles(h, false),
        LandingRule::WaterSpray => particles(h, true),
        LandingRule::WaterSprayWhenFxTen if h.inputs().transitions.fx_kind == 10 => {
            particles(h, true)
        }
        LandingRule::WaterSprayAtOrBelow944
            if h.session().player.y.wrapping_sub(944) & 0x8000 == 0 =>
        {
            particles(h, true)
        }
        _ => {
            h.atmospheric_mut().slots[2].frame_and_kind = 0;
            h.atmospheric_created(2)
                .expect("atmospheric producer allocation");
            h.atmospheric_mut().slots[3].frame_and_kind = 0;
            h.atmospheric_created(3)
                .expect("atmospheric producer allocation");
        }
    }
}
fn particles(h: &mut impl Host, water: bool) {
    let bottom = pose::loaded_bottom(h);
    if pose::liquid_at_height(h, bottom) != LiquidAtHeight::None {
        return;
    }
    let (kind, right, left, y) = if water {
        (0x100, 4, 3, bottom.wrapping_sub(4))
    } else {
        (0x600, 8, 8, bottom)
    };
    h.atmospheric_mut().slots[2].frame_and_kind = kind;
    h.atmospheric_mut().slots[3].frame_and_kind = kind;
    h.atmospheric_mut().slots[2].remaining = 3;
    h.atmospheric_mut().slots[3].remaining = 3;
    h.atmospheric_mut().slots[2].x = h.session().player.x.wrapping_add(right);
    h.atmospheric_mut().slots[3].x = h.session().player.x.wrapping_sub(left);
    h.atmospheric_mut().slots[2].y = y;
    h.atmospheric_created(2)
        .expect("atmospheric producer allocation");
    h.atmospheric_mut().slots[3].y = y;
    h.atmospheric_created(3)
        .expect("atmospheric producer allocation");
}
