//! Owned player animation clock, commands and running-speed timing.
#![forbid(unsafe_code)]
use super::{
    atmosphere,
    content::{AnimationProgram, AnimationTable, Content},
    effects::Host,
    pose,
    state::{AnimationStep, HurtAction, InputMode, MovementMode, PoseRequestWord},
};
#[cfg(sm_verify_dll)]
use crate::player_transitions_rust_observer as observer;
use sm_game::{player_collision::effects::Content as CollisionContent, visual_host::Record};

pub(crate) fn advance_animation(h: &mut impl Host, step: AnimationStep) {
    if step == AnimationStep::Gameplay {
        atmosphere::advance(h);
    }
    let pose = h.session().player.pose;
    let mut timer = h.session().player.animation_clock.remaining;
    if step == AnimationStep::Gameplay
        && matches!(pose, 0x4d | 0x4e)
        && h.session().player.contact.y_direction != 2
        && h.session().player.animation_clock.frame == 1
        && timer == 1
    {
        timer = 4;
    }
    timer = timer.wrapping_sub(1);
    h.session_mut().player.animation_clock.remaining = timer;
    h.probe(match step {
        AnimationStep::Gameplay => "transitions.phase.220",
        AnimationStep::Death => "transitions.phase.222",
    });
    if timer == 0 || timer & 0x8000 != 0 {
        h.session_mut().player.animation_clock.frame =
            h.session().player.animation_clock.frame.wrapping_add(1);
        h.probe(match step {
            AnimationStep::Gameplay => "transitions.phase.221",
            AnimationStep::Death => "transitions.phase.223",
        });
        interpret(h);
    }
}
fn table_word(h: &mut impl Host, table: AnimationTable, offset: usize) -> u16 {
    let record = Content::animation_table(h, table);
    u16::from_le_bytes(record.bytes::<2>(offset))
}
fn transition(h: &mut impl Host, pose: u8, _command: u8) {
    #[cfg(sm_verify_dll)]
    let before_pose = h.session().player.transitions.transitional.to_raw();
    h.session_mut().player.transitions.transitional = PoseRequestWord::from_raw(u16::from(pose));
    #[cfg(sm_verify_dll)]
    match _command {
        8 | 13 => h.probe("transitions.phase.229"),
        9 => h.probe("transitions.phase.232"),
        10 => h.probe("transitions.phase.234"),
        12 => observer::publication(
            9,
            0xa2c,
            before_pose,
            h.session().player.transitions.transitional.to_raw(),
        ),
        _ => unreachable!("transition command observation"),
    }
    #[cfg(sm_verify_dll)]
    let before_hurt = h.session().player.transitions.hurt.to_raw();
    h.session_mut().player.transitions.hurt = HurtAction::from_raw(3);
    #[cfg(sm_verify_dll)]
    match _command {
        8 | 13 => h.probe("transitions.phase.230"),
        9 => h.probe("transitions.phase.233"),
        10 => h.probe("transitions.phase.235"),
        12 => {
            observer::publication(
                10,
                0xa32,
                before_hurt,
                h.session().player.transitions.hurt.to_raw(),
            );
            h.probe("transitions.phase.31");
        }
        _ => unreachable!("transition command observation"),
    }
}
fn speed_command<H: Host>(
    h: &mut H,
    program: &<H as CollisionContent>::Record,
    frame: usize,
) -> u8 {
    h.probe("transitions.phase.25");
    let momentum = h.session().player.contact.momentum;
    let run_button = h.inputs().run_button;
    if momentum == 0
        || run_button & h.inputs().held == 0
        || h.session().player.movement_and_facing >> 8 != 1
    {
        return program.byte(frame);
    }
    if h.session().inventory.equipment.active & 0x2000 == 0 {
        h.session_mut().player.animation_clock.frame = 0;
        let buffer = h.session().player.transitions.animation_buffer;
        let list = table_word(h, AnimationTable::RunningWithoutBooster, 0);
        let running = Content::animation_program(h, AnimationProgram(list), 0);
        let delay = running.byte(0);
        h.session_mut().player.animation_clock.remaining = buffer.wrapping_add(u16::from(delay));
        return 0;
    }
    let counter = h.session().player.contact.speed_boost.wrapping_sub(1);
    h.session_mut().player.contact.speed_boost = counter;
    if counter as u8 != 0 {
        return program.byte(frame);
    }
    let mut staged_counter = counter;
    if counter & 0x400 == 0 {
        staged_counter = counter.wrapping_add(0x100);
        h.session_mut().player.contact.speed_boost = staged_counter;
        if staged_counter & 0x400 != 0 {
            h.session_mut().projectiles.controls.echo_sound = 1;
            h.sound(3, 6, 3);
        }
    }
    // Keep the source stage across sound, but acquire its table afterward.
    let stage_offset = usize::from(staged_counter >> 8) * 2;
    let duration = table_word(h, AnimationTable::BoosterStageDurations, stage_offset);
    let live_counter = h.session().player.contact.speed_boost;
    h.session_mut().player.contact.speed_boost = duration | (live_counter & 0xff00);
    h.session_mut().player.animation_clock.frame = 0;
    let buffer = h.session().player.transitions.animation_buffer;
    let list = table_word(h, AnimationTable::RunningBoosterStages, stage_offset);
    let running = Content::animation_program(h, AnimationProgram(list), 0);
    let delay = running.byte(0);
    h.session_mut().player.animation_clock.remaining = buffer.wrapping_add(u16::from(delay));
    0
}
fn walljump_delay(h: &mut impl Host) {
    let gravity = h.session().inventory.equipment.active & 0x20 != 0;
    let liquid = if gravity {
        false
    } else {
        let top = pose::loaded_top(h);
        pose::liquid_at_height(h, top) != sm_game::liquid_surface::LiquidAtHeight::None
    };
    let (sound, advance) = if liquid {
        (0x31, 1)
    } else if h.session().inventory.equipment.active & 8 != 0 {
        (0x33, 21)
    } else if h.session().inventory.equipment.active & 0x200 != 0 {
        (0x3e, 11)
    } else {
        (0x31, 1)
    };
    h.sound(1, 6, sound);
    h.session_mut().player.animation_clock.frame = h
        .session()
        .player
        .animation_clock
        .frame
        .wrapping_add(advance);
}
pub(super) fn interpret(h: &mut impl Host) {
    let pose_index = usize::from(h.session().player.pose) * 2;
    let list = table_word(h, AnimationTable::PosePrograms, pose_index);
    let mut program = Content::animation_program(h, AnimationProgram(list), 0);
    let frame = usize::from(h.session().player.animation_clock.frame);
    if program.byte(frame) & 0x80 != 0 {
        // Program ownership and the first instruction position survive sound.
        let command = speed_command(h, &program, frame);
        let frame = usize::from(h.session().player.animation_clock.frame);
        let update_timer = match command & 15 {
            6 => {
                let frame = if h.session().inventory.health.current.wrapping_sub(30) & 0x8000 != 0 {
                    h.session().player.animation_clock.frame.wrapping_add(1)
                } else {
                    0
                };
                h.session_mut().player.animation_clock.frame = frame;
                h.probe("transitions.phase.226");
                true
            }
            7 => {
                h.session_mut().player.movement_handler = MovementMode::DrainedCrouching;
                h.probe("transitions.phase.227");
                h.session_mut().player.animation_clock.frame =
                    h.session().player.animation_clock.frame.wrapping_add(1);
                h.probe("transitions.phase.228");
                true
            }
            8 => {
                if h.session().player.transitions.input_mode.to_raw() == InputMode::Demo.to_raw() {
                    transition(h, program.byte(frame + 1), command & 15);
                } else {
                    let pending = h.session().player.transitions.ordinary.to_raw();
                    if !matches!(pending, 0x4b | 0x4c | 0x19 | 0x1a) {
                        h.session_mut().player.transitions.input_mode = InputMode::AutoJump;
                        h.probe("transitions.phase.231");
                        transition(h, program.byte(frame + 1), command & 15);
                    }
                }
                false
            }
            9 => {
                // This opcode checks each byte independently in source order.
                let mask = u16::from_le_bytes([program.byte(frame + 1), program.byte(frame + 2)]);
                let has_item = mask & h.session().inventory.equipment.active != 0;
                let moving = h.session().player.contact.y_speed.whole != 0
                    || h.session().player.contact.y_speed.fraction != 0;
                let offset = match (has_item, moving) {
                    (false, false) => 3,
                    (false, true) => 4,
                    (true, false) => 5,
                    (true, true) => 6,
                };
                transition(h, program.byte(frame + offset), command & 15);
                false
            }
            10 => {
                let moving = h.session().player.contact.y_speed.whole != 0
                    || h.session().player.contact.y_speed.fraction != 0;
                transition(
                    h,
                    program.byte(frame + if moving { 2 } else { 1 }),
                    command & 15,
                );
                false
            }
            11 => {
                walljump_delay(h);
                true
            }
            12 => {
                h.probe("transitions.phase.26");
                let mask = program.native_word(frame + 1);
                #[cfg(sm_verify_dll)]
                observer::scalar(3, mask);
                h.probe("transitions.phase.30");
                let matched = mask & h.session().inventory.equipment.active != 0;
                let offset = if matched { 4 } else { 3 };
                #[cfg(sm_verify_dll)]
                {
                    observer::scalar(5, u16::from(matched));
                    observer::scalar(
                        6,
                        u16::try_from(frame + offset).expect("observed operand offset"),
                    );
                }
                transition(h, program.byte(frame + offset), command & 15);
                false
            }
            13 => {
                transition(h, program.byte(frame + 1), command & 15);
                false
            }
            14 => {
                let back = program.byte(frame + 1);
                h.session_mut().player.animation_clock.frame = h
                    .session()
                    .player
                    .animation_clock
                    .frame
                    .wrapping_sub(u16::from(back));
                h.probe("transitions.phase.238");
                true
            }
            15 => {
                h.session_mut().player.animation_clock.frame = 0;
                h.probe("transitions.phase.239");
                true
            }
            _ => false,
        };
        #[cfg(sm_verify_dll)]
        observer::scalar(4, u16::from(update_timer));
        h.probe("transitions.phase.32");
        if !update_timer {
            return;
        }
    } else if h.session().player.contact.momentum != 0
        && h.session().player.movement_and_facing >> 8 == 1
    {
        let list = if h.session().inventory.equipment.active & 0x2000 != 0 {
            let stage = usize::from(h.session().player.contact.speed_boost >> 8);
            table_word(h, AnimationTable::RunningBoosterStages, stage * 2)
        } else {
            table_word(h, AnimationTable::RunningWithoutBooster, 0)
        };
        program = Content::animation_program(h, AnimationProgram(list), 0);
    }
    let buffer = h.session().player.transitions.animation_buffer;
    let delay = program.byte(usize::from(h.session().player.animation_clock.frame));
    h.session_mut().player.animation_clock.remaining = buffer.wrapping_add(u16::from(delay));
    h.probe("transitions.phase.240");
}
