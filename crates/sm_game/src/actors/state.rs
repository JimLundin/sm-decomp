//! Common actor state; one codec defines its byte-compatible projection.
use crate::geometry::{Point, Position};

#[derive(Debug, Default, PartialEq)]
pub struct Actor {
    pub definition: u16,
    pub x: Position,
    pub y: Position,
    pub radius: Point,
    pub properties: u16,
    pub extra_properties: u16,
    pub ai_handlers: u16,
    pub health: u16,
    pub spritemap: u16,
    pub timer: u16,
    pub instruction: u16,
    pub instruction_delay: u16,
    pub palette: u16,
    pub tile: u16,
    pub layer: u16,
    pub flash: u16,
    pub frozen: u16,
    pub invincibility: u16,
    pub shake: u16,
    pub frame: u16,
    pub bank: u8,
    pub hurt_time: u8,
    pub variables: [u16; 5],
    pub pre_instruction: u16,
    pub parameters: [u16; 2],
}
#[derive(Debug, Default, PartialEq)]
pub struct SpawnRecord {
    pub contact_mode: u16,
    pub death_cause: u16,
    pub scratch_04: u16,
    pub tile: u16,
    pub palette: u16,
    pub scratch_0a: [u16; 3],
    pub draw_offset: Point,
    pub scratch_14: [u16; 5],
    pub definition: u16,
    pub position: Point,
    pub instruction: u16,
    pub properties: u16,
    pub extra_properties: u16,
    pub parameters: [u16; 2],
    pub name: [u8; 18],
}
#[derive(Debug, Default, PartialEq)]
pub struct Actors {
    pub slots: [Actor; 32],
    pub saved: [SpawnRecord; 32],
    pub controls: ActorControls,
    pub combat: ActorCombat,
    pub staging: ActorStaging,
    pub graphics: ActorGraphics,
    pub drawing: ActorDrawing,
    pub schedule: ActorSchedule,
    pub compatibility: ActorCompatibility,
    pub diagnostics: ActorDiagnostics,
    pub extended: ExtendedRam,
    pub crocomire_melt: CrocomireMelt,
}
/// Crocomire's melt ($7E:0688..$7E:071B): the words of
/// `crocomire_reactions::melt::Word` by address, then each pixel column's height.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CrocomireMelt {
    pub words: [u16; 10],
    pub column_heights: [[u8; 32]; 4],
}
/// The slots' extended RAM ($7E:7800..$7E:8FFF): three blocks of 32 words per
/// slot, the original's `var_00`..`var_5f`, as the words of each block in RAM
/// order (40h bytes per slot). It lives beside the slot records rather than in
/// `Actor`, because clearing or spawning a slot leaves it untouched, as the
/// original does. A fourth block holds the bosses' shared RAM ($7E:9000..$7E:97DB)
/// and the last two the BG2 tilemap staging RAM ($7E:2000..$7E:2FFF).
#[derive(Clone, Debug, PartialEq)]
pub struct ExtendedRam {
    pub blocks: [[u16; 1024]; 6],
}
impl Default for ExtendedRam {
    fn default() -> Self {
        Self {
            blocks: [[0; 1024]; 6],
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorControls {
    pub phase_counter: u16,
    pub selection_visits: u16,
    pub population_scratch: u16,
    pub compatibility_new_index: u16,
    pub first_free: u16,
    pub population_count: u16,
    pub selected: u16,
    pub compatibility_selected_backup: u16,
    pub compatibility_data_pointer: u16,
    pub shake_request: u16,
    pub projectile_contacts_disabled: u16,
    pub process_all: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorCombat {
    pub last_power: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorStaging {
    pub descriptor_prefix: [u8; 32],
    pub descriptor_write_position: u16,
    pub transfer_source: u16,
    pub transfer_destination: u16,
    pub gap: [u8; 4],
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorGraphics {
    pub definitions: [u16; 4],
    pub tile_offsets: [u16; 4],
    pub palette_routes: [u16; 4],
    pub compatibility_write_position: u16,
    pub compatibility_next_tile: u16,
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ActorDrawing {
    pub hook_address: [u8; 4],
    pub hook_work: [u16; 3],
    pub tilemap_size: u16,
    pub tilemap_pending: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorSchedule {
    pub boss_id: u16,
    pub compatibility_179e: [u16; 3],
    pub active_write_position: u16,
    pub interactive_write_position: u16,
    pub compatibility_active_cursor: u16,
    pub compatibility_interactive_cursor: u16,
    pub active: [u16; 32],
    pub interactive: [u16; 32],
    pub colliding: [u16; 4],
    pub compatibility_distances: [u16; 4],
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorCompatibility {
    pub pre_hook: [u8; 8],
    pub post_hook: [u8; 4],
    pub debug_enemy_index: u16,
    pub log_index: u16,
    pub scratch: [u16; 5],
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorDiagnostics {
    pub index: u16,
    pub freeze: u16,
    pub text: Point,
    pub population_entry: u16,
    pub population_key: u16,
    pub spawn: Point,
    pub gap: [u8; 2],
    pub sprite_contacts_disabled: u16,
}

impl Actor {
    pub fn restart_animation(&mut self, instruction: u16) {
        self.instruction = instruction;
        self.instruction_delay = 1;
        self.timer = 0;
    }

    pub fn duplicate_for_storage(&self) -> Self {
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
            controls: self.controls,
            combat: self.combat,
            staging: self.staging,
            graphics: self.graphics,
            drawing: self.drawing.clone(),
            schedule: self.schedule,
            compatibility: self.compatibility,
            diagnostics: self.diagnostics,
            extended: self.extended.clone(),
            crocomire_melt: self.crocomire_melt.clone(),
        }
    }
}

impl SpawnRecord {
    // Storage/guard duplication is private; read views cannot detach publicly.
    pub fn duplicate_for_storage(&self) -> Self {
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
