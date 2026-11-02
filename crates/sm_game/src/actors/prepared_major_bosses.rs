//! Concrete major-boss callbacks on the current prepared actor aggregate.
//! Original finite category tables for Botwoon, Kraid, Crocomire, Phantoon,
//! Mother Brain and both baby Metroid sequences.
#![forbid(unsafe_code)]
use crate::actors::access::ActorKey;
use crate::actors::botwoon_owned;
use crate::actors::content::ActorRoutine;
use crate::actors::owned_execution::PreparedActorState;
use crate::actors::{
    baby_metroid_cutscene_owned as baby_cutscene, baby_metroid_encounter_owned as baby_encounter,
    crocomire_reactions_owned as croco, kraid_owned as kraid, mother_brain_fight_owned as brain,
    prepared_phantoon,
};
use crate::enemy_script::EnemyScript;

#[path = "prepared_botwoon_bindings.rs"]
mod botwoon_bindings;

pub(crate) fn try_ai(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    if prepared_phantoon::try_ai(routine, host) {
        return true;
    }
    match routine.0 {
        10982544 => kraid::arm_touch(host),
        10982559 => kraid::touch(host),
        10987865 => kraid::initialize(host),
        10988355 => kraid::initialize_arm(host),
        10988392 => kraid::initialize_lint(host, 0x80),
        10988444 => kraid::initialize_lint(host, 0xc0),
        10988490 => kraid::initialize_lint(host, 0x100),
        10988536 => kraid::initialize_foot(host),
        10988577 => kraid::main_ai(host),
        10991549 => kraid::arm_main(host),
        10991617 => kraid::lint_main_ai(host, 0x80),
        10991629 => kraid::lint_main_ai(host, 0xc0),
        10991641 => kraid::lint_main_ai(host, 0x100),
        10992118 => kraid::foot_main(host),
        10992847 => kraid::fingernail_touch(host),
        10992862 => kraid::fingernail_touch(host),
        10992879 => kraid::initialize_fingernail(host),
        10992941 => kraid::initialize_fingernail(host),
        10992946 => kraid::fingernail_main(host, 0x180),
        10992969 => kraid::fingernail_main(host, 0x1c0),
        10783748 => croco::main_ai(host),
        10782343 => croco::hurt(host),
        10811002 => croco::initialize_tongue(host),
        10783322 => croco::initialize(host),
        10795410 => croco::power_bomb(host),
        10795325 => croco::touch(host),
        10795345 => croco::count_hit(host),
        10795368 => croco::impact(host),
        10795700 => croco::impact(host),
        10795525 => croco::hit(host),
        11110023 => brain::initialize_body(host),
        11110149 => brain::initialize_brain(host),
        11110206 => brain::body_main_ai(host),
        11110279 => brain::power_bomb_reaction(host),
        11110283 => brain::brain_main_ai(host),
        11110345 => brain::draw_brain_and_neck_hook(host),
        11110365 => brain::draw_brain_hook(host),
        11121923 => brain::body_shot(host),
        11121927 => brain::brain_shot(host),
        11122118 => brain::brain_touch(host),
        11111221 => crate::actors::mother_brain_tubes_owned::initialize(host),
        11126544 => baby_cutscene::initialize(host),
        11126649 => baby_cutscene::main_ai(host),
        11128579 => baby_cutscene::touch(host),
        11136823 => baby_encounter::initialize(host),
        11136954 => baby_encounter::power_bomb_reaction(host),
        11136965 => baby_encounter::main_ai(host),
        11138953 => baby_encounter::touch(host),
        11139138 => baby_encounter::shot(host),
        // Original species no-op callbacks, including Mother Brain touch/power bomb.
        10982543 | 10982581 | 11001242 | 11110278 | 11122117 => (),
        11769219 => botwoon_owned::initialize(host),
        11769448 => botwoon_owned::main_ai(host),
        11771926 => botwoon_owned::shot(host),
        11771903 | 11771969 => botwoon_owned::contact(routine.0 == 11771969, host),
        _ => return false,
    }
    true
}

pub(crate) fn try_instruction(
    routine: ActorRoutine,
    actor: ActorKey,
    script: EnemyScript,
    host: &mut PreparedActorState<'_>,
) -> Option<EnemyScript> {
    if let Some(next) = prepared_phantoon::try_instruction(routine, actor, script.clone(), host) {
        return Some(next);
    }
    if crocomire_instruction(routine.0) {
        return Some(croco::instruction(
            routine.0,
            actor.byte_index(),
            script,
            host,
        ));
    }
    if let Some(next) = body_instruction(routine.0, script.clone(), host) {
        return Some(next);
    }
    Some(match routine.0 {
        10991158 => {
            kraid::step_up(host);
            script
        }
        10991164 => {
            kraid::step_down_and_shake(host);
            script
        }
        10991182 => {
            kraid::stomp_sound(host);
            script
        }
        10991194 => {
            kraid::step_back(host);
            script
        }
        10991207 => {
            kraid::step_back(host);
            script
        }
        10991220 => {
            kraid::step_forward(host);
            script
        }
        10991235 => {
            kraid::lunge_right(host);
            script
        }
        10979983 => match kraid::arm_swipe_when_hurt(host) {
            Some(list) => EnemyScript::jump(list),
            None => script,
        },
        10782374 => {
            crate::actors::crocomire_attack_owned::fight_ai(actor.byte_index(), script, host)
        }
        10782546 => crate::actors::crocomire_attack_owned::maybe_start_projectile_attack(
            actor.byte_index(),
            script,
            host,
        ),
        11115208 => brain::spawn_dust_cloud_at_offset(actor.byte_index(), script, host),
        11115247 => brain::spawn_hand_beam_projectile(actor.byte_index(), script, host),
        11115269 => brain::increment_hand_beam_attack_phase(actor.byte_index(), script, host),
        11139476 => baby_encounter::branch_or_queue_remorse_sound(actor.byte_index(), script, host),
        11128756 => baby_cutscene::go_to_initial(actor.byte_index(), script),
        11128778 => baby_cutscene::go_to_draining_mother_brain(actor.byte_index(), script),
        11139360 => baby_encounter::go_to_normal(actor.byte_index(), script),
        11139382 => baby_encounter::go_to_latched_on(actor.byte_index(), script),
        11139472 => baby_encounter::go_to_remorse(actor.byte_index(), script),
        10991155 => script, // Original kraid_instruction no-op at A7:B533.

        11769031 | 11769095 | 11769111 | 11769175 => botwoon_owned::set_hitbox(8, 16, script, host),
        11769047 | 11769079 | 11769127 | 11769159 => {
            botwoon_owned::set_hitbox(12, 12, script, host)
        }
        11769063 | 11769143 => botwoon_owned::set_hitbox(16, 8, script, host),
        11769191 => botwoon_owned::set_spitting(script, host),
        // The shared spit sound does not consume script data.
        11769202 => {
            crate::effects::SoundEffects::queue_sound(host, 1, 0x7c, 6);
            script
        }
        _ => return None,
    })
}

pub(crate) fn try_pre_instruction(
    routine: ActorRoutine,
    actor: ActorKey,
    host: &mut PreparedActorState<'_>,
) -> bool {
    // Original remaining_bindings pre_instruction's exhaustive Phantoon table.
    match routine.0 {
        10999114 | 10999271 | 10999309 | 10999416 | 10999481 | 10999522 | 10999597 | 10999615
        | 10999655 | 10999765 | 10999850 | 10999900 | 10999953 | 10999980 | 11000086 | 11000110
        | 11000136 | 11000401 | 11000454 | 11000535 | 11000637 | 10999388 | 10998953 | 10999022
        | 10999048 | 10999190 | 10999508 | 10999688 | 10999799 | 10999924 | 11000203 => {
            let [low, high, _, _] = routine.0.to_le_bytes();
            crate::actors::phantoon_owned::callback(
                host,
                crate::actors::phantoon_owned::Callback::PreInstruction {
                    function: u16::from_le_bytes([low, high]),
                    actor: actor.byte_index(),
                },
            );
            true
        }
        11763835 | 11767039 | 11771328 | 11771517 | 11771616 | 11771700 | 11771770 => {
            botwoon_owned::pre_instruction(routine.0, actor.byte_index(), host);
            true
        }
        _ => false,
    }
}

pub(crate) fn try_hitbox_touch(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    if prepared_phantoon::try_hitbox_touch(routine, host) {
        return true;
    }
    match routine.0 {
        10982539 => kraid::touch_arm_or_foot(host),
        10982544 => kraid::arm_touch(host),
        _ => return false,
    }
    true
}
pub(crate) fn try_hitbox_shot(
    routine: ActorRoutine,
    argument: u16,
    host: &mut PreparedActorState<'_>,
) -> bool {
    if prepared_phantoon::try_hitbox_shot(routine, host) {
        return true;
    }
    match routine.0 {
        10982582 => kraid::arm_shot(host, argument),
        _ => return false,
    }
    true
}
pub(crate) fn try_graphics_hook(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    match routine.0 {
        11110345 => brain::draw_brain_and_neck_hook(host),
        11110365 => brain::draw_brain_hook(host),
        _ => return false,
    }
    true
}
fn crocomire_instruction(address: u32) -> bool {
    use croco::instruction::*;
    matches!(
        address,
        QUEUE_CRY
            | QUEUE_BIG_EXPLOSION
            | QUEUE_SKELETON_COLLAPSES
            | SHAKE_SCREEN
            | MOVE_LEFT_4PX
            | MOVE_LEFT_4PX_AND_SPAWN_DUST
            | MOVE_LEFT_4PX_AND_SPAWN_DUST_2
            | MOVE_LEFT_4PX_SPAWN_DUST_AND_HANDLE_SPIKE_WALL
            | MOVE_RIGHT_4PX_IF_ON_SCREEN
            | MOVE_RIGHT_4PX
            | MOVE_RIGHT_4PX_IF_ON_SCREEN_AND_SPAWN_DUST
            | MOVE_RIGHT_4PX_AND_SPAWN_DUST
    ) || ((SPAWN_DUST_FIRST..=SPAWN_DUST_LAST).contains(&address)
        && (address - SPAWN_DUST_FIRST).is_multiple_of(5))
}
/// Full original addresses; the low word is extracted only after recognition.
pub fn try_pause(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    if routine.0 != 0xa7c325 {
        return false;
    }
    crate::actors::kraid_pause_owned::pause(host);
    true
}
pub fn try_unpause(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> Option<u8> {
    Some(match routine.0 {
        0xa7c1fb => crate::actors::kraid_pause_owned::unpause_dead(host),
        0xa7c24e => crate::actors::kraid_pause_owned::unpause_alive(host),
        0xa7c2a0 => crate::actors::kraid_pause_owned::unpause_sinking(host),
        11110243 => crate::actors::mother_brain_fight_owned::body_unpause_hook(host),
        _ => return None,
    })
}

// Original sm/src/game/mother_brain_body.rs finite movement and pose tables.
struct BodyStep {
    dx: u16,
    dy: u16,
    scroll_x: u16,
    footstep: bool,
}
fn step(dx: i16, dy: i16, scroll_x: i16, footstep: bool) -> BodyStep {
    BodyStep {
        dx: u16::from_ne_bytes(dx.to_ne_bytes()),
        dy: u16::from_ne_bytes(dy.to_ne_bytes()),
        scroll_x: u16::from_ne_bytes(scroll_x.to_ne_bytes()),
        footstep,
    }
}
fn body_instruction(
    address: u32,
    script: EnemyScript,
    host: &mut PreparedActorState<'_>,
) -> Option<EnemyScript> {
    use crate::actors::access::{AccessMode, ActorAccess};
    use crate::session_access::SessionAccess;
    let movement = match address {
        11113910 => step(0, -10, 4, false),
        11113920 => step(0, -16, 4, false),
        11113930 => step(0, -12, -2, false),
        11113950 => step(0, 12, 4, false),
        11113960 => step(0, 16, -2, false),
        11113970 => step(0, 10, -2, false),
        11113980 | 11114224 => step(1, -2, 0, false),
        11113996 => step(2, 0, 0, false),
        11114012 => step(0, 1, 0, false),
        11114018 => step(3, 1, 0, true),
        11114040 => step(15, -2, 0, false),
        11114056 => step(6, -4, 0, false),
        11114072 => step(-2, 4, 0, false),
        11114088 | 11114110 => step(-1, 2, 0, true),
        11114132 => step(-2, 0, 0, false),
        11114148 => step(0, -1, 0, false),
        11114154 => step(-3, -1, 0, false),
        11114170 => step(-15, 2, 0, true),
        11114192 => step(-6, 4, 0, false),
        11114208 => step(2, -4, 0, false),
        _ => return body_pose(address, script, host),
    };
    if movement.footstep {
        host.write_current_word_le(0x183e, 1);
        host.write_current_word_le(0x1840, 4);
        if crate::actors::mother_brain_state_owned::part_get(
            host,
            0,
            crate::actors::mother_brain_state_owned::Field::MotherBrainBodyForm,
        ) == 3
        {
            // Original returning diagnostic: no mutation or fatal edge.
            println!("Write to rom!");
        }
    }
    let x = host.edit_actor(ActorKey::from_legacy(0), AccessMode::Native, |body| {
        body.x.whole = body.x.whole.wrapping_add(movement.dx);
        body.y.whole = body.y.whole.wrapping_add(movement.dy);
        body.x.whole
    });
    let old = host.session().room.camera.bg2_scroll[1];
    host.write_current_slice(0xb7, &old.wrapping_sub(movement.dy).to_le_bytes());
    host.write_current_slice(
        0xb5,
        &movement
            .scroll_x
            .wrapping_add(34)
            .wrapping_sub(x)
            .to_le_bytes(),
    );
    Some(script)
}
fn body_pose(
    address: u32,
    script: EnemyScript,
    host: &mut PreparedActorState<'_>,
) -> Option<EnemyScript> {
    let pose = match address {
        11114240 => 0,
        11114248 => 1,
        11114256 => 3,
        11114264 => 2,
        11114272 => 4,
        11114280 => 6,
        _ => return None,
    };
    crate::actors::mother_brain_state_owned::part_set(
        host,
        0,
        crate::actors::mother_brain_state_owned::Field::MotherBrainBodyPose,
        pose,
    );
    Some(script)
}
