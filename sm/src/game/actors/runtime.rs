//! Actor compatibility implementations on the single aggregate Runtime.
#![forbid(unsafe_code)]
use super::{
    access::*,
    state::{Actor, SpawnRecord},
};
use crate::game::session_runtime::Runtime;
impl ActorAccess for Runtime<'_> {
    #[cfg(sm_verify_dll)]
    fn actor_movement_observe(&mut self, tag: u8) {
        self.observe_actor_movement(tag);
    }
    fn resolve_actor<'a>(&'a self, key: ActorKey, mode: AccessMode) -> ActorRead<'a> {
        resolve_actor_with(self, key, mode, Runtime::byte)
    }
    fn edit_actor<R>(
        &mut self,
        key: ActorKey,
        mode: AccessMode,
        edit: impl for<'edit> FnOnce(&'edit mut Actor) -> R,
    ) -> R {
        edit_actor_with(self, key, mode, Runtime::byte, Runtime::set_byte, edit)
    }
}
impl SpawnAccess for Runtime<'_> {
    fn resolve_spawn<'a>(&'a self, key: ActorKey, mode: AccessMode) -> SpawnRead<'a> {
        resolve_spawn_with(self, key, mode, Runtime::byte)
    }
    fn edit_spawn<R>(
        &mut self,
        key: ActorKey,
        mode: AccessMode,
        edit: impl for<'edit> FnOnce(&'edit mut SpawnRecord) -> R,
    ) -> R {
        edit_spawn_with(self, key, mode, Runtime::byte, Runtime::set_byte, edit)
    }
}
impl ActorCollections for Runtime<'_> {
    fn collection_word(&self, window: CollectionWindow, offset: usize, mode: AccessMode) -> u16 {
        collection_word_with(self, window, offset, mode, Runtime::byte)
    }
    fn collection_byte(&self, window: CollectionWindow, offset: usize) -> u8 {
        collection_byte_with(self, window, offset, Runtime::byte)
    }
    fn write_collection_word(
        &mut self,
        window: CollectionWindow,
        offset: usize,
        value: u16,
        store: CollectionStore,
    ) {
        write_collection_word_with(
            self,
            window,
            offset,
            value,
            store,
            Runtime::set_byte,
            Runtime::write_slice,
        )
    }
    fn write_collection_byte(&mut self, window: CollectionWindow, offset: usize, value: u8) {
        write_collection_byte_with(self, window, offset, value, Runtime::set_byte)
    }
    fn write_tile_source(&mut self, offset: usize, source: [u8; 3]) {
        write_tile_source_with(self, offset, source, Runtime::write_slice)
    }
    fn copy_palette_row(&mut self, row: u8, colors: [u8; 32]) {
        copy_palette_row_with(self, row, colors, Runtime::write_slice)
    }
}

use super::{
    content::ActorRoutine,
    effects::{ActorAssertion, ActorEffects, ActorHost},
    scene::{ActorDrawRequest, ActorTileChunkSource, ActorTileDestination, ActorTilemapSource},
};
use crate::game::{enemy_script::EnemyScript, session_access::SessionAccess};
impl ActorEffects for Runtime<'_> {
    #[cfg(sm_verify_dll)]
    fn actor_reference_stage(&mut self, event: u16, key: u16) {
        super::composed_checks::stage(self, event, key);
        if crate::actor_population_animation_observer::active() {
            self.foreign(|game| {
                crate::actor_population_animation_observer::stage(event, key, game);
            });
        }
    }
    #[cfg(sm_verify_dll)]
    fn actor_reference_opcode(&mut self, event: u16, address: u32) {
        if crate::actor_population_animation_observer::active() {
            self.foreign(|game| {
                crate::actor_population_animation_observer::opcode(
                    event,
                    address,
                    game.cur_enemy_index(),
                    game,
                );
            });
        }
    }
    fn actor_ai(&mut self, routine: ActorRoutine) {
        self.actor_ai_legacy(routine.0);
    }
    fn actor_instruction(
        &mut self,
        routine: ActorRoutine,
        actor: ActorKey,
        script: EnemyScript,
    ) -> EnemyScript {
        self.actor_instruction_legacy(routine.0, actor.byte_index(), script)
    }
    fn actor_hitbox_touch(&mut self, routine: ActorRoutine) {
        self.actor_touch_legacy(routine.0);
    }
    fn actor_hitbox_shot(&mut self, routine: ActorRoutine, argument: u16) {
        self.actor_shot_legacy(routine.0, argument);
    }
    fn actor_graphics_hook(&mut self, routine: ActorRoutine) {
        self.actor_hook_legacy(routine.0);
    }
    fn draw_player_scene(&mut self) {
        self.actor_player_scene_legacy();
    }
    fn draw_actor_visual(&mut self, request: ActorDrawRequest) {
        #[cfg(sm_verify_dll)]
        super::composed_checks::draw(self, &request);
        super::media_runtime::draw(self, request).expect("actor visual binding invalid");
    }
    fn actor_assertion(&mut self, site: ActorAssertion) {
        self.actor_assertion_legacy(matches!(site, ActorAssertion::DrawingQueueAddress));
    }
    fn copy_actor_tilemap(&mut self, source: ActorTilemapSource) {
        super::media_runtime::copy(self, source).expect("actor tilemap binding invalid");
        let pending = self
            .session()
            .actors
            .drawing
            .tilemap_pending
            .wrapping_add(1);
        self.session_mut().actors.drawing.tilemap_pending = pending;
    }
    fn copy_actor_tile_chunk(
        &mut self,
        source: ActorTileChunkSource,
        destination: ActorTileDestination,
    ) {
        let bytes: [u8; 8] =
            core::array::from_fn(|i| source.record.image.read(source.record.base + i));
        self.write_slice(0x7000 + usize::from(destination.0), &bytes);
    }
}
impl ActorHost for Runtime<'_> {}
