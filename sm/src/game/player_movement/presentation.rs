//! Shared footstep particles and ordered speed-echo publication.
#![forbid(unsafe_code)]
use super::{
    content::{Content, FootstepRegion, FootstepRule},
    effects::{EchoAxis, Host},
};
use crate::game::{liquid_surface::LiquidAtHeight, player_transitions as transitions};

pub(super) fn footsteps(h: &mut impl Host) {
    let region = Content::footstep_region(h);
    let rule = if region == FootstepRegion::Crateria && h.inputs().cinematic == 0 {
        Content::footstep_room_rule(h)
    } else if region == FootstepRegion::Sand {
        FootstepRule::Sand
    } else {
        FootstepRule::Default
    };
    let sand = match rule {
        FootstepRule::Sand => true,
        FootstepRule::SandWhenFxTen => h.inputs().transitions.fx_kind == 10,
        FootstepRule::SandAtOrBelow944 => h.session().player.y.wrapping_sub(944) & 0x8000 == 0,
        FootstepRule::Default => false,
    };
    if !sand && h.session().player.contact.speed_boost & 0xff00 != 1024 {
        return;
    }
    let bottom = transitions::loaded_bottom(h);
    if transitions::liquid_at_height(h, bottom) != LiquidAtHeight::None {
        return;
    }
    // Sand and boost dust differ only in particle kind; their positions/timing agree.
    let s = h.session_mut();
    if s.player.movement_and_facing as u8 == 4 {
        s.atmospheric.slots[0].x = s.player.x.wrapping_sub(12);
        s.atmospheric.slots[1].x = s.player.x.wrapping_add(8);
    } else {
        s.atmospheric.slots[0].x = s.player.x.wrapping_add(12);
        s.atmospheric.slots[1].x = s.player.x.wrapping_sub(8);
    }
    s.atmospheric.slots[0].y = s.player.y.wrapping_add(16);
    s.atmospheric.slots[1].y = s.player.y.wrapping_add(16);
    let kind = if sand { 256 } else { 1792 };
    s.atmospheric.slots[0].frame_and_kind = kind;
    s.atmospheric.slots[1].frame_and_kind = kind;
    s.atmospheric.slots[0].remaining = 0x8002;
    s.atmospheric.slots[1].remaining = 3;
}

pub(crate) fn record_echo(h: &mut impl Host) {
    if h.session().player.contact.speed_boost & 0xff00 == 1024
        && h.session().projectiles.echoes.index & 0x8000 == 0
        && h.inputs().transitions.game_time_frames & 3 == 0
    {
        let original = h.session().projectiles.echoes.index;
        let index = usize::from(h.session().projectiles.echoes.index >> 1);
        let x = h.session().player.x;
        if index < 4 {
            h.session_mut().projectiles.echoes.x[index] = x;
        } else {
            h.write_echo_coordinate(EchoAxis::X, index, x);
        }
        // Exceptional X destinations may alias player Y. Read it after publishing X.
        let y = h.session().player.y;
        if index < 4 {
            h.session_mut().projectiles.echoes.y[index] = y;
        } else {
            h.write_echo_coordinate(EchoAxis::Y, index, y);
        }
        let mut next = original.wrapping_add(2);
        if (next as i16).wrapping_sub(4) >= 0 {
            next = 0;
        }
        h.session_mut().projectiles.echoes.index = next;
    }
}
