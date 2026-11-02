//! Actual prepared Metroid, Mochtroid and space-pirate callback bodies.
#![forbid(unsafe_code)]
use crate::actors::access::ActorKey;
use crate::actors::content::ActorRoutine;
use crate::actors::owned_corpses;
use crate::actors::owned_execution::PreparedActorState;
use crate::actors::{owned_metroid, owned_mochtroid, owned_space_pirates};
use crate::enemy_script::EnemyScript;

pub(crate) fn try_ai(routine: ActorRoutine, h: &mut PreparedActorState<'_>) -> bool {
    match routine.0 {
        10742351 => owned_metroid::initialize(h),
        10742502 => owned_metroid::frozen(h),
        10742579 => owned_metroid::hurt(h),
        10742680 => owned_metroid::main_ai(h),
        10743275 => owned_metroid::touch(h),
        10743559 => owned_metroid::shot(h),
        10743874 => owned_metroid::power_bomb(h),
        10725245 => owned_mochtroid::initialize(h),
        10725264 => owned_mochtroid::main_ai(h),
        10725715 => owned_mochtroid::touch(h),
        owned_space_pirates::routine::TOUCH
        | owned_space_pirates::routine::SHOT
        | owned_space_pirates::routine::WALKING_INITIALIZATION
        | owned_space_pirates::routine::WALKING_MAIN_AI
        | owned_space_pirates::routine::FROZEN_AI
        | owned_space_pirates::routine::UPDATE_FROZEN
        | owned_space_pirates::routine::POWER_BOMB_REACTION
        | owned_space_pirates::routine::WALL_MAIN_AI
        | owned_space_pirates::routine::NINJA_MAIN_AI
        | owned_space_pirates::routine::WALL_INITIALIZATION
        | owned_space_pirates::routine::NINJA_INITIALIZATION => {
            owned_space_pirates::dispatch_ai(routine.0, h)
        }
        11129608 => owned_corpses::initialize_torizo_corpse(h),
        11129704 => owned_corpses::torizo_corpse_main_ai(h),
        11129898 => owned_corpses::torizo_corpse_power_bomb_reaction(h),
        11129907 => owned_corpses::torizo_corpse_touch_or_shot(h),
        11130806 => owned_corpses::initialize_sidehopper_corpse(h),
        11130953 => owned_corpses::initialize_geemer_corpse(h),
        11130998 => owned_corpses::initialize_ripper_corpse(h),
        11131039 => owned_corpses::initialize_skree_corpse(h),
        11131084 => owned_corpses::sidehopper_corpse_power_bomb_reaction(h),
        11131099 => owned_corpses::corpse_main_ai(h),
        11132141 => owned_corpses::power_bomb_reaction_geemer_corpse(h),
        11132152 => owned_corpses::touch_or_shot_geemer_corpse(h),
        11132157 => owned_corpses::ripper_corpse_power_bomb_reaction(h),
        11132168 => owned_corpses::ripper_corpse_touch_or_shot(h),
        11132173 => owned_corpses::skree_corpse_power_bomb_reaction(h),
        11132184 => owned_corpses::skree_corpse_touch_or_shot(h),
        11132189 => owned_corpses::sidehopper_corpse_shot(h),
        11132228 => owned_corpses::sidehopper_corpse_touch(h),
        _ => return false,
    }
    true
}
pub(crate) fn try_instruction(
    routine: ActorRoutine,
    actor: ActorKey,
    script: EnemyScript,
    h: &mut PreparedActorState<'_>,
) -> Option<EnemyScript> {
    Some(match routine.0 {
        10742437 => owned_metroid::draining_sound(script, h),
        10742449 => owned_metroid::random_cry(script, h),
        owned_space_pirates::instruction::FIRE_LASER_LEFT
        | owned_space_pirates::instruction::FIRE_LASER_RIGHT
        | owned_space_pirates::instruction::WALL_SET_FUNCTION
        | owned_space_pirates::instruction::WALKING_SET_FUNCTION
        | owned_space_pirates::instruction::SET_PALETTE
        | owned_space_pirates::instruction::QUEUE_ATTACK_SOUND
        | owned_space_pirates::instruction::QUEUE_SOUND
        | owned_space_pirates::instruction::SPAWN_CLAW
        | owned_space_pirates::instruction::GO_TO_ACTIVE
        | owned_space_pirates::instruction::FIRE_LASER_LEFT_WITH_Y_OFFSET
        | owned_space_pirates::instruction::FIRE_LASER_RIGHT_WITH_Y_OFFSET
        | owned_space_pirates::instruction::CHOOSE_MOVEMENT
        | owned_space_pirates::instruction::STOP
        | owned_space_pirates::instruction::SET_LEFT_DIVEKICK_SPEED
        | owned_space_pirates::instruction::SET_RIGHT_DIVEKICK_SPEED
        | owned_space_pirates::instruction::MOVE_DOWN_LEFT_WALL
        | owned_space_pirates::instruction::MOVE_DOWN_RIGHT_WALL
        | owned_space_pirates::instruction::RANDOM_DIRECTION_LEFT_WALL
        | owned_space_pirates::instruction::RANDOM_DIRECTION_RIGHT_WALL
        | owned_space_pirates::instruction::PREPARE_WALL_JUMP_RIGHT
        | owned_space_pirates::instruction::PREPARE_WALL_JUMP_LEFT => {
            owned_space_pirates::dispatch_instruction(routine.0, actor.byte_index(), script, h)
        }
        11136208 => owned_corpses::sidehopper_corpse_end_hop(actor.byte_index(), script, h),
        11128756 => crate::actors::baby_metroid_cutscene::go_to_initial(actor.byte_index(), script),
        11128778 => crate::actors::baby_metroid_cutscene::go_to_draining_mother_brain(
            actor.byte_index(),
            script,
        ),
        11139360 => crate::actors::baby_metroid_encounter::go_to_normal(actor.byte_index(), script),
        11139382 => {
            crate::actors::baby_metroid_encounter::go_to_latched_on(actor.byte_index(), script)
        }
        11139472 => {
            crate::actors::baby_metroid_encounter::go_to_remorse(actor.byte_index(), script)
        }
        11139476 => {
            use crate::enemy_projectiles::host::Effects as _;
            use crate::session_access::SessionAccess as _;
            if h.session().timing.random_number & 0x8000 == 0 {
                EnemyScript::jump(script.word(0))
            } else {
                h.queue_sound(2, 6, 0x52);
                script.offset(1)
            }
        }
        _ => return None,
    })
}
pub(crate) fn try_hitbox_touch(routine: ActorRoutine, h: &mut PreparedActorState<'_>) -> bool {
    match routine.0 {
        11700076 => owned_space_pirates::touch(h),
        _ => return false,
    }
    true
}
pub(crate) fn try_hitbox_shot(
    routine: ActorRoutine,
    _argument: u16,
    h: &mut PreparedActorState<'_>,
) -> bool {
    match routine.0 {
        11700089 => owned_space_pirates::shot(h),
        11700168 => owned_space_pirates::check_shot(h),
        11700286 => owned_space_pirates::reflect(h),
        _ => return false,
    }
    true
}

pub(crate) fn try_graphics_hook(routine: ActorRoutine, h: &mut PreparedActorState<'_>) -> bool {
    match routine.0 {
        11129754 => owned_corpses::torizo_corpse_graphics_drawn_hook(h),
        _ => return false,
    }
    true
}
