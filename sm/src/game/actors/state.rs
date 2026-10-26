//! Common actor state; one codec defines its byte-compatible projection.
#![forbid(unsafe_code)]
use crate::game::session::{Point, Position};

#[derive(Debug, Default, PartialEq)]
pub(in crate::game) struct Actor {
    pub(in crate::game) definition: u16,
    pub(in crate::game) x: Position,
    pub(in crate::game) y: Position,
    pub(in crate::game) radius: Point,
    pub(in crate::game) properties: u16,
    pub(in crate::game) extra_properties: u16,
    pub(in crate::game) ai_handlers: u16,
    pub(in crate::game) health: u16,
    pub(in crate::game) spritemap: u16,
    pub(in crate::game) timer: u16,
    pub(in crate::game) instruction: u16,
    pub(in crate::game) instruction_delay: u16,
    pub(in crate::game) palette: u16,
    pub(in crate::game) tile: u16,
    pub(in crate::game) layer: u16,
    pub(in crate::game) flash: u16,
    pub(in crate::game) frozen: u16,
    pub(in crate::game) invincibility: u16,
    pub(in crate::game) shake: u16,
    pub(in crate::game) frame: u16,
    pub(in crate::game) bank: u8,
    pub(in crate::game) hurt_time: u8,
    pub(in crate::game) variables: [u16; 5],
    pub(in crate::game) pre_instruction: u16,
    pub(in crate::game) parameters: [u16; 2],
}
#[derive(Debug, Default, PartialEq)]
pub(super) struct SpawnRecord {
    pub(super) contact_mode: u16,
    pub(super) death_cause: u16,
    pub(super) scratch_04: u16,
    pub(super) tile: u16,
    pub(super) palette: u16,
    pub(super) scratch_0a: [u16; 3],
    pub(super) draw_offset: Point,
    pub(super) scratch_14: [u16; 5],
    pub(super) definition: u16,
    pub(super) position: Point,
    pub(super) instruction: u16,
    pub(super) properties: u16,
    pub(super) extra_properties: u16,
    pub(super) parameters: [u16; 2],
    pub(super) name: [u8; 18],
}
#[derive(Debug, Default, PartialEq)]
pub(in crate::game) struct Actors {
    pub(super) slots: [Actor; 32],
    pub(super) saved: [SpawnRecord; 32],
    pub(in crate::game) controls: ActorControls,
    pub(super) combat: ActorCombat,
    pub(super) staging: ActorStaging,
    pub(super) graphics: ActorGraphics,
    pub(super) drawing: ActorDrawing,
    pub(in crate::game) schedule: ActorSchedule,
    pub(super) compatibility: ActorCompatibility,
    pub(super) diagnostics: ActorDiagnostics,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::game) struct ActorControls {
    pub(super) phase_counter: u16,
    pub(super) selection_visits: u16,
    pub(super) population_scratch: u16,
    pub(super) compatibility_new_index: u16,
    pub(super) first_free: u16,
    pub(super) population_count: u16,
    pub(super) selected: u16,
    pub(super) compatibility_selected_backup: u16,
    pub(super) compatibility_data_pointer: u16,
    pub(in crate::game) shake_request: u16,
    pub(super) projectile_contacts_disabled: u16,
    pub(super) process_all: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ActorCombat {
    pub(super) last_power: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ActorStaging {
    pub(super) descriptor_prefix: [u8; 32],
    pub(super) descriptor_write_position: u16,
    pub(super) transfer_source: u16,
    pub(super) transfer_destination: u16,
    pub(super) gap: [u8; 4],
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ActorGraphics {
    pub(super) definitions: [u16; 4],
    pub(super) tile_offsets: [u16; 4],
    pub(super) palette_routes: [u16; 4],
    pub(super) compatibility_write_position: u16,
    pub(super) compatibility_next_tile: u16,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ActorDrawing {
    pub(super) queued_indices: [u16; 114],
    pub(super) queue_sizes: [u16; 8],
    pub(super) hook_address: [u8; 4],
    pub(super) hook_work: [u16; 3],
    pub(super) tilemap_size: u16,
    pub(super) tilemap_pending: u16,
}
impl Default for ActorDrawing {
    fn default() -> Self {
        Self {
            queued_indices: [0; 114],
            queue_sizes: [0; 8],
            hook_address: [0; 4],
            hook_work: [0; 3],
            tilemap_size: 0,
            tilemap_pending: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::game) struct ActorSchedule {
    pub(super) boss_id: u16,
    pub(super) compatibility_179e: [u16; 3],
    pub(super) active_write_position: u16,
    pub(in crate::game) interactive_write_position: u16,
    pub(super) compatibility_active_cursor: u16,
    pub(super) compatibility_interactive_cursor: u16,
    pub(super) active: [u16; 32],
    pub(in crate::game) interactive: [u16; 32],
    pub(in crate::game) colliding: [u16; 4],
    pub(super) compatibility_distances: [u16; 4],
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ActorCompatibility {
    pub(super) pre_hook: [u8; 8],
    pub(super) post_hook: [u8; 4],
    pub(super) debug_enemy_index: u16,
    pub(super) log_index: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ActorDiagnostics {
    pub(super) index: u16,
    pub(super) freeze: u16,
    pub(super) text: Point,
    pub(super) population_entry: u16,
    pub(super) population_key: u16,
    pub(super) spawn: Point,
    pub(super) gap: [u8; 2],
    pub(super) sprite_contacts_disabled: u16,
}

impl Actor {
    pub(in crate::game) fn restart_animation(&mut self, instruction: u16) {
        self.instruction = instruction;
        self.instruction_delay = 1;
        self.timer = 0;
    }

    pub(super) fn duplicate_for_storage(&self) -> Self {
        Self {
            definition: self.definition,
            x: self.x,
            y: self.y,
            radius: self.radius,
            properties: self.properties,
            extra_properties: self.extra_properties,
            ai_handlers: self.ai_handlers,
            health: self.health,
            spritemap: self.spritemap,
            timer: self.timer,
            instruction: self.instruction,
            instruction_delay: self.instruction_delay,
            palette: self.palette,
            tile: self.tile,
            layer: self.layer,
            flash: self.flash,
            frozen: self.frozen,
            invincibility: self.invincibility,
            shake: self.shake,
            frame: self.frame,
            bank: self.bank,
            hurt_time: self.hurt_time,
            variables: self.variables,
            pre_instruction: self.pre_instruction,
            parameters: self.parameters,
        }
    }
}

impl Clone for Actors {
    fn clone(&self) -> Self {
        Self {
            slots: core::array::from_fn(|i| self.slots[i].duplicate_for_storage()),
            saved: core::array::from_fn(|i| self.saved[i].duplicate_for_storage()),
            controls: self.controls.clone(),
            combat: self.combat.clone(),
            staging: self.staging.clone(),
            graphics: self.graphics.clone(),
            drawing: self.drawing.clone(),
            schedule: self.schedule.clone(),
            compatibility: self.compatibility.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }
}

impl SpawnRecord {
    // Storage/guard duplication is private; read views cannot detach publicly.
    pub(super) fn duplicate_for_storage(&self) -> Self {
        Self {
            contact_mode: self.contact_mode,
            death_cause: self.death_cause,
            scratch_04: self.scratch_04,
            tile: self.tile,
            palette: self.palette,
            scratch_0a: self.scratch_0a,
            draw_offset: self.draw_offset,
            scratch_14: self.scratch_14,
            definition: self.definition,
            position: self.position,
            instruction: self.instruction,
            properties: self.properties,
            extra_properties: self.extra_properties,
            parameters: self.parameters,
            name: self.name,
        }
    }
}
