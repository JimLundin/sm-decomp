//! Phantoon's actual host and callbacks on the current prepared actor owner.
#![forbid(unsafe_code)]
use crate::actors::access::ActorKey;
use crate::actors::content::ActorRoutine;
use crate::actors::owned_execution::PreparedActorState;
use crate::actors::phantoon_owned::{self as phantoon, Callback};
use crate::actors::remaining_core::PreInstruction as _;
use crate::enemy_script::EnemyScript;

impl phantoon::Host for PreparedActorState<'_> {
    fn dispatch_a7_function(&mut self, address: u32) {
        self.remaining_run_pre_instruction(address);
    }
    fn spawn_hdma(&mut self, bank: u8, args: &crate::display::hdma_spawn::HdmaObjectDefinition) {
        crate::actors::boss_hdma_owned::spawn_hdma_owned(bank, args, self);
    }
    fn start_wave(&mut self, direction: u16, speed: u16) {
        crate::actors::boss_hdma_owned::initialize_phantoon_motion(direction, speed, self);
    }
    fn disable_minimap(&mut self) {
        crate::player_update::minimap::disable_for_boss(self);
    }
    fn guard<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        self.guarded_actor_call(action)
    }
    fn spawn_plm(&mut self, x: u8, y: u8, header: u16) {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Hardcoded { x, y, header },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
    fn queue_music_delayed(&mut self, track: u16) {
        crate::music_requests::queue_delayed_8(track, self);
    }
    fn unreachable(&mut self) -> ! {
        self.unknown_function()
    }
}
pub(crate) fn try_ai(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    let callback = match routine.0 {
        10997235 => Callback::Initialize,
        10997333 => Callback::InitializePart,
        10997414 => Callback::Main,
        11001151 => Callback::Hurt,
        11001237 => Callback::Touch,
        11001243 => Callback::Shot,
        _ => return false,
    };
    phantoon::callback(host, callback);
    true
}
pub(crate) fn try_instruction(
    routine: ActorRoutine,
    actor: ActorKey,
    script: EnemyScript,
    host: &mut PreparedActorState<'_>,
) -> Option<EnemyScript> {
    match routine.0 {
        10977418 => {
            let address = u32::from(script.word(0)) | 0xa70000;
            host.guarded_actor_call(|host| {
                phantoon::callback(
                    host,
                    Callback::Instruction {
                        address,
                        actor: actor.byte_index(),
                    },
                )
            });
            Some(script.offset(1))
        }
        _ => None,
    }
}
pub(crate) fn try_hitbox_touch(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    if routine.0 != 11001237 {
        return false;
    }
    phantoon::callback(host, Callback::Touch);
    true
}
pub(crate) fn try_hitbox_shot(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    if routine.0 != 11001243 {
        return false;
    }
    phantoon::callback(host, Callback::Shot);
    true
}
