//! Jump and shinespark initialization share the owned motion state.
#![forbid(unsafe_code)]
use super::{
    effects::Host,
    pose,
    state::{BombJump, JumpKind, MovementMode},
};
use crate::liquid_surface::LiquidAtHeight;

pub fn initialize_jump(h: &mut impl Host, kind: JumpKind) {
    h.probe(match kind {
        JumpKind::Normal => "samus.jump.normal",
        JumpKind::Wall => "samus.jump.wall",
        JumpKind::Knockback => "samus.jump.knockback",
        JumpKind::Bomb => "samus.jump.bomb",
    });
    let environment = if h.session().inventory.equipment.active & 0x20 != 0 {
        0
    } else {
        let bottom = pose::loaded_bottom(h);
        match pose::liquid_at_height(h, bottom) {
            LiquidAtHeight::None => 0,
            LiquidAtHeight::Water => 1,
            LiquidAtHeight::LavaAcid => 2,
        }
    };
    let high = h.session().inventory.equipment.active & 0x100 != 0;
    let speeds = match kind {
        JumpKind::Normal if high => [(0, 6), (0x8000, 2), (0x8000, 3)],
        JumpKind::Normal => [(0xe000, 4), (0xc000, 1), (0xc000, 2)],
        JumpKind::Wall if high => [(0x8000, 5), (0x8000, 0), (0x8000, 3)],
        JumpKind::Wall => [(0xa000, 4), (0x4000, 0), (0xa000, 2)],
        JumpKind::Knockback => [(0, 5), (0, 2), (0, 2)],
        JumpKind::Bomb => [(0xc000, 2), (0x1000, 0), (0x1000, 0)],
    };
    let (fraction, whole) = speeds[environment];
    h.session_mut().player.contact.y_speed.fraction = fraction;
    h.session_mut().player.contact.y_speed.whole = whole;
    if matches!(kind, JumpKind::Normal | JumpKind::Wall)
        && h.session().inventory.equipment.active & 0x2000 != 0
    {
        let c = &mut h.session_mut().player.contact;
        c.y_speed.fraction = c.y_speed.fraction.wrapping_add(c.extra_run.fraction);
        c.y_speed.whole = c.y_speed.whole.wrapping_add(c.extra_run.whole >> 1);
    }
    let p = &mut h.session_mut().player;
    p.transitions.grapple_walljump_timer = 0;
    p.transitions.elevator_fade_timer = 0;
    p.contact.y_direction = 1;
}
pub fn handle_transition(h: &mut impl Host) {
    let movement = usize::from(h.session().player.movement_and_facing >> 8);
    sm_data::table::check_index(movement, 28);
    match movement {
        2 => {
            let pose = h.session().player.pose;
            if matches!(pose, 0x4b | 0x4c)
                || (pose.wrapping_sub(0x55) & 0x8000 == 0 && pose.wrapping_sub(0x5b) & 0x8000 != 0)
            {
                if matches!(h.session().player.history.previous.pose, 0x27 | 0x28) {
                    h.session_mut().player.y = h.session().player.y.wrapping_sub(10);
                }
                initialize_jump(h, JumpKind::Normal);
            }
        }
        3 if !matches!(
            h.session().player.history.previous.movement_and_facing >> 8,
            3 | 20
        ) =>
        {
            initialize_jump(h, JumpKind::Normal)
        }
        18 if matches!(h.session().player.pose, 0x7f | 0x80)
            && h.session().player.history.previous.movement_and_facing >> 8 == 17 =>
        {
            initialize_jump(h, JumpKind::Normal)
        }
        20 if h.session().player.history.previous.movement_and_facing >> 8 != 20 => {
            initialize_jump(h, JumpKind::Wall)
        }
        13 if (h.session().player.pose == 0x65
            && h.session().player.history.previous.pose == 0x64)
            || (h.session().player.pose == 0x66
                && h.session().player.history.previous.pose == 0x63) =>
        {
            initialize_jump(h, JumpKind::Normal)
        }
        _ => {}
    }
}
pub fn start_shinespark(h: &mut impl Host) {
    h.probe("transitions.phase.22");
    let s = h.session_mut();
    s.player.movement_handler = MovementMode::ShinesparkWindup;
    s.player.contact.y_direction = 1;
    s.player.contact.speed_boost = 1024;
    s.player.contact.y_speed.fraction = 0;
    s.player.contact.y_speed.whole = 0;
    s.player.transitions.knockback = 0;
    s.player.contact.extra_run.whole = 8;
    s.player.contact.extra_run.fraction = 0;
    s.player.contact.base_run.whole = 0;
    s.player.contact.base_run.fraction = 0;
    s.shared_sequence.stage = 7;
    s.shared_sequence.value = 0;
    s.projectiles.controls.cooldown = 0;
    s.player.transitions.shinespark_start_stop_timer = 30;
    s.player.shine_timer = 60;
    s.player.shine_tick = 6;
    s.player.palette_frame = 0;
    s.player.transitions.bomb_jump = BombJump::from_raw(0);
    if h.session().projectiles.controls.charge != 0 {
        if h.session().projectiles.controls.charge.wrapping_sub(16) & 0x8000 == 0 {
            h.sound(1, 9, 2);
        }
        h.session_mut().projectiles.controls.charge = 0;
        h.session_mut().projectiles.controls.clear_flare_animation();
    }
}
