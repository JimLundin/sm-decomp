//! Ordinary gameplay state shared by player, room mechanisms and enemy projectiles.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrepareEntry {
    Normal,
    Demo,
    ProjectilesOnly,
    Inactive,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Shot {
    pub x: u16,
    pub y: u16,
    pub x_fraction: u16,
    pub y_fraction: u16,
    pub x_radius: u16,
    pub y_radius: u16,
    pub x_speed: u16,
    pub y_speed: u16,
    pub direction: u16,
    pub kind: u16,
    pub damage: u16,
    pub cursor: u16,
    pub remaining: u16,
    pub motion: u16,
    pub variable: u16,
    pub trail_time: u16,
    pub auxiliary: u16,
    pub sprite: u16,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Trail {
    pub remaining: u16,
    pub cursor: u16,
    pub tile: u16,
    pub x: u16,
    pub y: u16,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct TrailPair {
    pub left: Trail,
    pub right: Trail,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Controls {
    pub cooldown: u16,
    pub spread_charge: u16,
    pub orbit_step: u16,
    pub charge: u16,
    pub previous_charge: u16,
    pub glow: u16,
    pub charge_palette: u16,
    pub resume_sound: u16,
    pub echo_sound: u16,
    pub pending_aim: u16,
    pub protection: u16,
    pub selected_weapon: u16,
    pub automatic_weapon: u16,
    pub weapon_changed: u16,
    pub flare_frames: [u16; 3],
    pub flare_timers: [u16; 3],
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Preparation {
    pub next_pose: u16,
    pub interrupted_pose: u16,
    pub transitional_pose: u16,
    pub momentum_routine: u16,
    pub special_graphics: u16,
    pub hurt_switch: u16,
    pub demo_held: u16,
    pub demo_pressed: u16,
    pub demo_filtered_held: u16,
    pub demo_filtered_pressed: u16,
    pub animation_skip: u16,
    pub aim_history: u16,
    pub previous_movement_and_facing: u16,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Echoes {
    pub index: u16,
    pub x: [u16; 4],
    pub y: [u16; 4],
    pub x_speed: [u16; 4],
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct PowerBomb {
    pub flag: u16,
    pub x: u16,
    pub y: u16,
}

use crate::game::item_pickups::Inventory;
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Session {
    pub projectiles: Projectiles,
    pub enemy_projectiles: EnemyProjectiles,
    pub player: Player,
    pub inventory: Inventory,
    pub mechanisms: Mechanisms,
    pub room: Room,
    pub feedback: Feedback,
    pub messages: MessageRequests,
    pub spawn_scratch: SpawnScratch,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Projectiles {
    pub shots: [Shot; 10],
    pub trails: [TrailPair; 18],
    pub controls: Controls,
    pub selected: u16,
    pub count: u16,
    pub bomb_count: u16,
    pub preparation: Preparation,
    pub echoes: Echoes,
    pub power_bomb: PowerBomb,
    pub launch: crate::game::launch_motion::LaunchMotion,
}
// Implementation retains existing LaunchMotion rather than a second algorithm.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Player {
    pub x: u16,
    pub y: u16,
    pub pose: u16,
    pub previous_pose: u16,
    pub movement_and_facing: u16,
    pub grapple: u16,
    pub movement_handler: u16,
    pub shine_timer: u16,
    pub shine_tick: u16,
    pub palette_frame: u16,
    pub special_palette_flags: u16,
    pub contact: ContactPhysics,
    pub invincibility: u16, // 18A8, distinct from projectile protection18AC.
    pub periodic_damage: u16,
    pub speed_divisor: u16,
    pub knockback_timer: u16,
    pub knockback_x_direction: u16,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FixedMotion {
    pub fraction: u16,
    pub whole: u16,
}
impl FixedMotion {
    // Overlapping middle words are computed views, never separately stored.
    pub fn middle(&self) -> u16 {
        (self.fraction >> 8) | (self.whole << 8)
    }
    pub fn set_middle(&mut self, value: u16) {
        self.fraction = (self.fraction & 0xff) | (value << 8);
        self.whole = (self.whole & 0xff00) | (value >> 8);
    }
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct ContactPhysics {
    pub radius: u16,
    pub deceleration: u16,
    pub momentum: u16,
    pub speed_boost: u16,
    pub extra_run: FixedMotion,
    pub base_run: FixedMotion,
    pub y_direction: u16,
    pub y_speed: FixedMotion,
    pub external_x: FixedMotion,
    pub external_y: FixedMotion,
    pub y_acceleration: FixedMotion,
    pub auto_jump: u16,
    pub collision_direction: u16,
    pub contact_attack: u16,
    pub reaction_point: u16,
    pub sand_visits: u16,
    pub movement_speed_table: u16,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Mechanism {
    pub header: u16,
    pub block: u16,
    pub pre_action: u16,
    pub program: u16,
    pub counter: u16,
    pub room_argument: u16,
    pub work0: u16,
    pub instruction_delay: u16,
    pub draw_program: u16,
    pub return_program: u16,
    pub work1: u16,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Mechanisms {
    pub slots: [Mechanism; 40],
    pub flags: u16,
    pub draw_cursor: u16,
    pub selected: u16,
    pub block_x: u16,
    pub block_y: u16,
    pub next_graphics: u16,
    pub saved_graphics: [u16; 4],
    pub tilemap: [u16; 256],
    pub custom_draw: [u16; 3],
    pub save_locked: u16,
    pub statue_finished: u16,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Cell {
    pub level: u16,
    pub behavior: u8,
}
pub const TERRAIN_CELLS: usize = 12_800;
pub const TILE_DEFINITIONS: usize = 1_024;
#[derive(Clone, Debug, PartialEq)]
pub struct Terrain {
    cells: Box<[Cell]>,
    tiles: Box<[[u16; 4]]>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeError {
    TerrainCapacity,
    TileCapacity,
}
impl Terrain {
    pub fn from_owned(cells: Box<[Cell]>, tiles: Box<[[u16; 4]]>) -> Result<Self, ShapeError> {
        if cells.len() != TERRAIN_CELLS {
            return Err(ShapeError::TerrainCapacity);
        }
        if tiles.len() != TILE_DEFINITIONS {
            return Err(ShapeError::TileCapacity);
        }
        Ok(Self { cells, tiles })
    }
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }
    pub fn tiles(&self) -> &[[u16; 4]] {
        &self.tiles
    }
    pub fn tiles_mut(&mut self) -> &mut [[u16; 4]] {
        &mut self.tiles
    }
}
// Capacity is physical disjoint range, not width*height. Imported accesses
// beyond it use the finite alias protocol; never clamp/reject by dimensions.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Room {
    pub terrain: Terrain,
    pub area: u16,
    pub width: u16,
    pub height: u16,
    pub scroll_width: u16,
    pub scroll_height: u16,
    pub size_bytes: u16,
    pub collision_cursor: u16,
    pub liquid_options: u16,
    pub liquid_height: u16,
    pub liquid_target: u16,
    pub liquid_velocity: u16,
    pub liquid_delay: u16,
    pub enemy_deaths: u16,
    pub enemy_quota: u16,
    pub loaded_items: u16,
    pub power_bomb_radius: u16,
    pub enemy_processing_stage: u16,
    pub collision_detection_index: u16,
    pub slope_behavior: u16,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Feedback {
    pub earthquake_kind: u16,
    pub earthquake_timer: u16,
    pub sound_disabled: u16,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct MessageRequests {
    pub queued: u16,
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            cells: vec![Cell::default(); TERRAIN_CELLS].into_boxed_slice(),
            tiles: vec![[0; 4]; TILE_DEFINITIONS].into_boxed_slice(),
        }
    }
}
impl Default for Mechanisms {
    fn default() -> Self {
        Self {
            slots: [Mechanism::default(); 40],
            flags: 0,
            draw_cursor: 0,
            selected: 0,
            block_x: 0,
            block_y: 0,
            next_graphics: 0,
            saved_graphics: [0; 4],
            tilemap: [0; 256],
            custom_draw: [0; 3],
            save_locked: 0,
            statue_finished: 0,
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct SpawnScratch {
    pub header: u16,
    pub coordinates: u16,
    pub argument: u16,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Position {
    pub whole: u16,
    pub fraction: u16,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rectangle {
    pub origin: Point,
    pub extent: Point,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct DropMetadata {
    pub flags: u16,
    pub enemy_definition: u16,
    pub killed_enemy: u16,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct EnemyProjectile {
    pub definition: u16,
    pub graphics: u16,
    pub counter: u16,
    pub pre_action: u16,
    pub x: Position,
    pub y: Position,
    pub velocity: Point,
    // These words have different meanings for different projectile behaviors.
    // Keep their full stored values until each behavior's valid-state closure is known.
    pub behavior_a: u16,
    pub behavior_b: u16,
    pub program: u16,
    pub spritemap: u16,
    pub delay: u16,
    pub radius: u16,
    pub properties: u16,
    pub behavior_c: u16,
    pub angle: u16,
    pub drop: DropMetadata,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct SpawnContext {
    pub primary: u16,
    pub secondary: u16,
    pub angle: u8,
    pub rectangle: Rectangle,
    pub variant: u16,
    pub point: Point,
    pub drop_definition: u16,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct EnemyProjectiles {
    pub slots: [EnemyProjectile; 18],
    pub flags: u16,
    pub selected: u16,
    pub spawn: SpawnContext,
    pub drop_bias: u16,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Vector32 {
    pub x: u32,
    pub y: u32,
}
