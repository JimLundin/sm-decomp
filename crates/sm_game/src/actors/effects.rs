//! Finite retained actor effects. Existing sound/damage/player-control ports are reused.
#![forbid(unsafe_code)]
use super::{
    access::{ActorAccess, ActorCollections, ActorKey, SpawnAccess},
    content::{ActorCaptureReads, ActorContent, ActorRoutine},
    scene::{ActorDrawRequest, ActorTileChunkSource, ActorTileDestination, ActorTilemapSource},
};
use crate::enemy_script::EnemyScript;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorAssertion {
    AnimationInstructionHighBit,
    DrawingQueueAddress,
}
pub trait ActorEffects {
    /// Verification-only wire events, dormant for normal hosts/routes.
    #[cfg(sm_verify_dll)]
    fn actor_reference_stage(&mut self, _event: u16, _key: u16) {}
    #[cfg(sm_verify_dll)]
    fn actor_reference_opcode(&mut self, _event: u16, _address: u32) {}
    fn actor_ai(&mut self, routine: ActorRoutine);
    fn actor_instruction(
        &mut self,
        routine: ActorRoutine,
        actor: ActorKey,
        script: EnemyScript,
    ) -> EnemyScript;
    fn actor_hitbox_touch(&mut self, routine: ActorRoutine);
    fn actor_hitbox_shot(&mut self, routine: ActorRoutine, argument: u16);
    fn actor_graphics_hook(&mut self, routine: ActorRoutine);
    fn draw_player_scene(&mut self);
    fn draw_actor_visual(&mut self, request: ActorDrawRequest);
    fn actor_assertion(&mut self, site: ActorAssertion);
    fn copy_actor_tilemap(&mut self, source: ActorTilemapSource);
    fn copy_actor_tile_chunk(
        &mut self,
        source: ActorTileChunkSource,
        destination: ActorTileDestination,
    );
}
pub trait ActorHost:
    ActorAccess + SpawnAccess + ActorCollections + ActorContent + ActorCaptureReads + ActorEffects
{
}
