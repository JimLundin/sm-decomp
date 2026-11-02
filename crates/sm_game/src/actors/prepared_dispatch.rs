//! Concrete actor callbacks. Each registry owns actual species bodies.
//! The root must compose every listed child before qualifying this host.
#![forbid(unsafe_code)]
use crate::actors::access::ActorKey;
use crate::actors::content::ActorRoutine;
use crate::actors::effects::{ActorAssertion, ActorEffects, ActorHost};
use crate::actors::host::{ActorRuntime, Death, Shot};
use crate::actors::owned_execution::{self as execution, CommonAi, PreparedActorState};
use crate::enemy_script::EnemyScript;

fn common_ai(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    if let Some(kind) = CommonAi::from_routine(routine) {
        execution::common_ai(kind, host);
        return true;
    }
    match routine.0 {
        11763747 | 10715171 | 10649635 | 10780707 | 10911779 | 11042851 | 10846243 => {
            host.contact_reaction(Death::Finalize)
        }
        11763757 | 10725800 | 10715181 | 10649645 | 10780717 | 10911789 | 10977325 | 11042861
        | 10846253 => host.shot_reaction(Shot::Finalize),
        10518583 | 10649655 | 10715191 | 10780727 | 10846263 | 10911799 | 10977335 | 11042871
        | 11108407 | 11173943 | 11698231 | 11763767 => host.power_bomb_reaction(Death::Finalize),
        10977340 => host.power_bomb_reaction(Death::LeavePending),
        10715212 | 10723079 | 10738715 | 10780748 | 10795344 | 10811067 | 10846284 | 10864042
        | 10864068 | 10874354 | 10911820 | 10942752 | 10977356 | 10982543 | 10982581 | 11001242
        | 11001873 | 11042892 | 11062374 | 11108428 | 11122117 | 11173964 | 11196359 | 11200475
        | 11200476 | 11698252 | 11763788 | 11765518 | 11791130 => (),
        _ => return false,
    }
    true
}
fn small_ai(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    execution::etecoon_ai(routine, host)
        || execution::dachora_ai(routine, host)
        || execution::mini_kraid_ai(routine, host)
        || execution::zebetites_ai(routine, host)
        || execution::rippers_ai(routine, host)
        || execution::choot_ai(routine, host)
        || execution::zoa_ai(routine, host)
        || execution::roach_ai(routine, host)
        || execution::tripper_ai(routine, host)
        || execution::sidehopper_ai(routine, host)
}
fn common_touch(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    match routine.0 {
        10518563 | 10780707 => host.contact_reaction(Death::Finalize),
        10936153 => host.contact_reaction(Death::LeavePending),
        10795344 | 10846284 | 10977356 | 11122117 => (),
        _ => return false,
    }
    true
}
fn common_shot(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    match routine.0 {
        10518573 | 10780717 | 10846253 => host.shot_reaction(Shot::Finalize),
        10649676 | 10846284 | 10911820 | 10977356 | 10982581 | 11192769 => (),
        10846278 => crate::actors::combat::create_dud(host),
        _ => return false,
    }
    true
}
impl ActorEffects for PreparedActorState<'_> {
    fn actor_ai(&mut self, routine: ActorRoutine) {
        self.guarded_actor_call(|host| {
            if small_ai(routine, host)
                || crate::actors::owned_acquired_boss_bindings::try_ai(routine, host)
                || crate::actors::prepared_major_bosses::try_ai(routine, host)
                || crate::actors::prepared_combat_families::try_ai(routine, host)
                || crate::actors::prepared_remaining_species::try_ai(routine, host)
                || common_ai(routine, host)
            {
                return;
            }
            host.unknown_function();
        });
    }
    fn actor_instruction(
        &mut self,
        routine: ActorRoutine,
        actor: ActorKey,
        script: EnemyScript,
    ) -> EnemyScript {
        self.guarded_actor_call(|host| {
            let script = match crate::actors::animation::dispatch_common_instruction(
                host, routine.0, actor, script,
            ) {
                Ok(next) => return next,
                Err(script) => script,
            };
            let script = match execution::mini_kraid_instruction(routine, script, host) {
                Ok(next) => return next,
                Err(script) => script,
            };
            let script = match execution::zoa_instruction(routine, script, host) {
                Ok(next) => return next,
                Err(script) => script,
            };
            let script = match execution::tripper_instruction(routine, script, host) {
                Ok(next) => return next,
                Err(script) => script,
            };
            let script = match execution::sidehopper_instruction(routine, script, host) {
                Ok(next) => return next,
                Err(script) => script,
            };
            if let Some(next) = crate::actors::owned_acquired_boss_bindings::try_instruction(
                routine,
                actor,
                script.clone(),
                host,
            ) {
                return next;
            }
            if let Some(next) = crate::actors::prepared_major_bosses::try_instruction(
                routine,
                actor,
                script.clone(),
                host,
            ) {
                return next;
            }
            if let Some(next) = crate::actors::prepared_combat_families::try_instruction(
                routine,
                actor,
                script.clone(),
                host,
            ) {
                return next;
            }
            if let Some(next) = crate::actors::prepared_remaining_species::try_instruction(
                routine, actor, script, host,
            ) {
                return next;
            }
            host.unknown_function();
        })
    }
    fn actor_hitbox_touch(&mut self, routine: ActorRoutine) {
        self.guarded_actor_call(|host| {
            if execution::mini_kraid_ai(routine, host)
                || execution::zebetites_ai(routine, host)
                || crate::actors::owned_acquired_boss_bindings::try_hitbox_touch(routine, host)
                || crate::actors::prepared_major_bosses::try_hitbox_touch(routine, host)
                || crate::actors::prepared_combat_families::try_hitbox_touch(routine, host)
                || crate::actors::prepared_remaining_species::try_hitbox_touch(routine, host)
                || common_touch(routine, host)
            {
                return;
            }
            host.unknown_function();
        });
    }
    fn actor_hitbox_shot(&mut self, routine: ActorRoutine, argument: u16) {
        self.guarded_actor_call(|host| {
            if execution::mini_kraid_ai(routine, host)
                || execution::zebetites_ai(routine, host)
                || crate::actors::owned_acquired_boss_bindings::try_hitbox_shot(
                    routine, argument, host,
                )
                || crate::actors::prepared_major_bosses::try_hitbox_shot(routine, argument, host)
                || crate::actors::prepared_combat_families::try_hitbox_shot(routine, argument, host)
                || crate::actors::prepared_remaining_species::try_hitbox_shot(
                    routine, argument, host,
                )
                || common_shot(routine, host)
            {
                return;
            }
            host.unknown_function();
        });
    }
    fn actor_graphics_hook(&mut self, routine: ActorRoutine) {
        self.guarded_actor_call(|host| {
            if matches!(routine.0, 10518604 | 11042892 | 11110278) {
                return;
            }
            if crate::actors::owned_acquired_boss_bindings::try_graphics_hook(routine, host)
                || crate::actors::prepared_major_bosses::try_graphics_hook(routine, host)
                || crate::actors::prepared_combat_families::try_graphics_hook(routine, host)
                || crate::actors::prepared_remaining_species::try_graphics_hook(routine, host)
            {
                return;
            }
            host.unknown_function();
        });
    }
    fn draw_player_scene(&mut self) {
        self.guarded_actor_call(crate::player_drawing::draw_player_frame);
    }
    fn draw_actor_visual(&mut self, request: crate::actors::scene::ActorDrawRequest) {
        self.draw_current_actor_visual(request);
    }
    fn actor_assertion(&mut self, site: ActorAssertion) {
        self.owner.poison();
        std::panic::panic_any(execution::ActorExecutionFault::Assertion(site));
    }
    fn copy_actor_tilemap(&mut self, source: crate::actors::scene::ActorTilemapSource) {
        self.copy_current_actor_tilemap(source);
    }
    fn copy_actor_tile_chunk(
        &mut self,
        source: crate::actors::scene::ActorTileChunkSource,
        destination: crate::actors::scene::ActorTileDestination,
    ) {
        self.copy_current_actor_chunk(source, destination);
    }
}
impl ActorHost for PreparedActorState<'_> {}
impl PreparedActorState<'_> {
    pub fn advance_actor_selected(&mut self) {
        self.guarded_actor_call(crate::actors::scheduling::advance_selected);
    }
    pub fn advance_actor_animation(&mut self) {
        self.guarded_actor_call(crate::actors::animation::advance_animation);
    }
    pub fn draw_actor_scene(&mut self) {
        self.guarded_actor_call(crate::actors::drawing::draw_scene);
    }
}
