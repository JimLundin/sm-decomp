//! Ceres station's steam vents and doors (bank $A6:EFB1-$A6:FB71): the
//! steam's activation and its offset in the rotating elevator room, and the
//! doors' functions, the escape's rumbling and explosions and the elevator
//! animations.
//!
//! Was `CeresSteam_*` and `CeresDoor_*` in the original decompilation. The
//! doors' and steam's shared script steps stay with the compatibility actor
//! records in the host.
#![forbid(unsafe_code)]
use crate::actors::layouts::{EnemyBase, same_words};
use crate::constants::enemy_properties::{kEnemyProps_DisableSamusColl, kEnemyProps_Invisible};
use crate::enemy_script::EnemyScript;
use crate::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// What the steam and doors need beyond the shared boss effects.
pub trait Host: crate::actors::host::BossEffects + crate::records::Records {
    /// Set and then clear the property bits of enemy `k`, through its actor
    /// record.
    fn change_properties(&mut self, k: u16, set: u16, clear: u16);
    /// The script at bank `bank`, `address`, as the original's ROM pointer.
    fn rom_script(&mut self, bank: u8, address: u16) -> EnemyScript;
    /// (`x`, `y`) as projected by the rotating elevator room's mode 7.
    fn project_cinematic_point(&mut self, x: u16, y: u16) -> (u16, u16);
    /// Queue the bank `bank` mode 7 transfer list at `list`.
    fn queue_mode7_transfer(&mut self, bank: u8, list: u16);
}

/// A steam vent's view of its slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Steam {
    pub base: EnemyBase,
    pub function: u16,
    pad_0faa: [u16; 2],
    pub activation_timer: u16,
    pad_0fb0: [u16; 2],
    pub kind: u16,
    pad_0fb6: [u16; 1],
}

same_words!(Steam: EnemyLayout8 {
    function = var_A, activation_timer = var_D, kind = parameter_1,
});

/// A Ceres door's view of its slot. Enemy 0's `rumble_timer` and
/// `rumble_index` pace the rotating elevator room's explosions.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Door {
    pub base: EnemyBase,
    pub function: u16,
    pub drawn_by_ridley: u16,
    pad_0fac: [u16; 1],
    pub rumbling_timer: u16,
    pub rumble_timer: u16,
    pub rumble_index: u16,
    pub kind: u16,
    pad_0fb6: [u16; 1],
}

same_words!(Door: EnemyLayout8 {
    function = var_A, drawn_by_ridley = var_B, rumbling_timer = var_D, rumble_timer = var_E,
    rumble_index = var_F, kind = parameter_1,
});

/// Bank $A6 steam functions, stored in `function`.
pub mod steam_function {
    pub const NONE: u16 = 0xeff4;
    pub const CALCULATE_ROTATING_ELEVATOR_OFFSET: u16 = 0xf019;
}

/// Bank $A6 door functions, stored in `function`.
pub mod door_function {
    pub const HANDLE_ESCAPE_EARTHQUAKE: u16 = 0xf76b;
    pub const HANDLE_RIDLEYS_ROOM_ESCAPE_EARTHQUAKE: u16 = 0xf770;
    pub const RIDLEY_ESCAPE_MODE7_WALL: u16 = 0xf7a5;
    pub const ELEVATOR_DEFAULT: u16 = 0xf7bd;
    pub const RUMBLING_AND_EXPLOSIONS: u16 = 0xf7dc;
    pub const ELEVATOR_ANIMATIONS: u16 = 0xf850;
}

/// Door kinds, the doors' first parameter.
const ELEVATOR_DOOR_OVERLAY: u16 = 2;
const RIDLEYS_ROOM_FACING_RIGHT: u16 = 3;

/// The doors' palettes ($A6:F4EC and $A6:F50C), each after a word.
pub const DOOR_PALETTE: u16 = 0xf4ec;
pub const ESCAPE_DOOR_PALETTE: u16 = 0xf50c;
/// The door's empty spritemap ($A6:FAC7).
pub const DOOR_SPRITEMAP: u16 = 0xfac7;
/// The elevator beacon palettes ($A6:F871), 10h bytes each.
const BEACON_PALETTES: u16 = 0xf871;
/// The pre-explosion door overlay tiles in bank $B0, and their VRAM address.
const OVERLAY_TILES: u16 = 0xc400;
const OVERLAY_TILES_BANK: u16 = 0xb000;
const OVERLAY_VRAM: u16 = 0x7000;
const OVERLAY_SIZE: u16 = 1024;
const VRAM_WRITE_QUEUE: usize = 0xd0;
/// Bank $86 dust cloud / explosion enemy projectile and its kinds.
const DUST_CLOUD: u16 = 0xe509;
const SMALL_EXPLOSION: u16 = 3;
const SMOKE: u16 = 12;
const BIG_EXPLOSION_SOUND: u8 = 0x25;
/// Ceres status values.
const RIDLEY_ESCAPING: u16 = 1;
const ESCAPE_STARTED: u16 = 2;
const ELEVATOR_ROTATING: u16 = 0x8000;
/// The enemy spawn records ($7E:7000), 40h bytes per enemy, and their
/// graphical offset words.
const ENEMY_SPAWN_DATA: usize = 0x7000;
const SPAWN_GRAPHICAL_X_OFFSET: usize = 0x10;
const SPAWN_GRAPHICAL_Y_OFFSET: usize = 0x12;

fn current<H: Host + ?Sized>(game: &H) -> i32 {
    word!(game.cur_enemy_index) as i32
}

fn store_word<H: Host + ?Sized>(game: &H, at: usize, value: u16) {
    let bytes = value.to_ne_bytes();
    game.ram().write_byte(at, bytes[0]);
    game.ram().write_byte(at + 1, bytes[1]);
}

/// Initialisation AI - enemy $E1FF (steam) ($A6:EFB1): its kind picks the
/// instruction list and function; it waits 1 to 20h frames to start. Was
/// `CeresSteam_Init`.
pub fn initialize_steam<H: Host + ?Sized>(game: &mut H) {
    let e = current(game);
    slot!(game, Steam[e].base; vram_tiles_index = 0,
        properties = slot!(game, Steam[e].base.properties) | kEnemyProps_DisableSamusColl as u16,
        extra_properties = slot!(game, Steam[e].base.extra_properties) | 4, instruction_timer = 1,
        timer = 0, palette_index = 2560, );
    slot!(
        game,
        Steam[e].activation_timer = (game.next_random() & 0x1f) + 1
    );
    let kind = crate::ceres_door_data::steam(game, slot!(game, Steam[e].kind) as isize);
    slot!(
        game,
        Steam[e].base.current_instruction = kind.instruction_list
    );
    slot!(game, Steam[e].function = kind.function);
}

/// Main AI - enemy $E1FF (steam) ($A6:F00D), at full health. Was
/// `CeresSteam_Main`.
pub fn steam_main<H: Host + ?Sized>(game: &mut H) {
    let e = current(game);
    slot!(game, Steam[e].base.health = 0x7fff);
    let k = word!(game.cur_enemy_index);
    match slot!(game, Steam[e].function) {
        steam_function::NONE => (),
        steam_function::CALCULATE_ROTATING_ELEVATOR_OFFSET => steam_rotating_offset(k, game),
        _ => game.unreachable(),
    }
}

/// Steam function - calculate graphical offset in rotating elevator room
/// ($A6:F019), into the enemy's spawn record. Was `CeresSteam_Func_1`.
pub fn steam_rotating_offset<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    let (x, y) = game.project_cinematic_point(
        slot!(game, Steam[e].base.x_pos),
        slot!(game, Steam[e].base.y_pos),
    );
    let record = ENEMY_SPAWN_DATA + usize::from(word!(game.cur_enemy_index));
    let x_offset = x.wrapping_sub(slot!(game, Steam[e].base.x_pos));
    store_word(game, record + SPAWN_GRAPHICAL_X_OFFSET, x_offset);
    let y_offset = y.wrapping_sub(slot!(game, Steam[e].base.y_pos));
    store_word(game, record + SPAWN_GRAPHICAL_Y_OFFSET, y_offset);
}

/// Instruction - decrement activation timer and go to [[Y]] if non-zero,
/// otherwise show enemy and go to [[Y] + 2] ($A6:F127). Was
/// `CeresSteam_Instr_2`.
pub fn steam_activate<H: Host + ?Sized>(k: u16, script: EnemyScript, game: &mut H) -> EnemyScript {
    let e = i32::from(k);
    let timer = slot!(game, Steam[e].activation_timer);
    slot!(game, Steam[e].activation_timer = timer.wrapping_sub(1));
    if timer != 1 {
        return EnemyScript::jump(script.word(0));
    }
    let next = game.rom_script(0xa6, script.word(2));
    game.change_properties(k, 0, VANISH);
    next
}

/// The steam's hidden state: intangible and invisible.
pub const VANISH: u16 = 0x0400 | 0x0100;

/// Initialisation AI - enemy $E23F (Ceres door) ($A6:F6C5): its kind picks
/// the function and instruction list; the palette depends on the escape.
/// Was `CeresDoor_Init`.
pub fn initialize_door<H: Host + ?Sized>(game: &mut H) {
    let e = current(game);
    slot!(game, Door[e].base.spritemap_pointer = DOOR_SPRITEMAP);
    slot!(game, Door[e].base.instruction_timer = 1);
    slot!(game, Door[e].base; timer = 0, vram_tiles_index = 0);
    slot!(game, Door[e].base.palette_index = 1024);
    let kind = slot!(game, Door[e].kind) as isize;
    slot!(
        game,
        Door[e].function = crate::ceres_door_data::door_function(game, kind)
    );
    slot!(
        game,
        Door[e].base.current_instruction = crate::ceres_door_data::door_instructions(game, kind)
    );
    slot!(game, Door[e].drawn_by_ridley = 0);
    load_overlay_tiles(word!(game.cur_enemy_index), game);
    let palette = if word!(game.ceres_status) != 0 {
        ESCAPE_DOOR_PALETTE.wrapping_add(2)
    } else {
        if slot!(game, Door[e].kind) == RIDLEYS_ROOM_FACING_RIGHT {
            copy_rom_colors_to_target_palette(0xa6, 0x142, DOOR_PALETTE.wrapping_add(2), 0xf, game);
            return;
        }
        DOOR_PALETTE.wrapping_add(2)
    };
    slot!(game, Door[current(game)].base.palette_index = 3584);
    copy_rom_colors_to_target_palette(0xa6, 0x1e2, palette, 0xf, game);
}

/// Load rotating elevator room pre-explosion door overlay tiles if needed
/// ($A6:F739): queue $B0:C400..C7FF to VRAM $7000. Was `CeresDoor_Func_1`.
pub fn load_overlay_tiles<H: Host + ?Sized>(k: u16, game: &mut H) {
    if slot!(game, Door[i32::from(k)].kind) != ELEVATOR_DOOR_OVERLAY {
        return;
    }
    let tail = word!(game.vram_write_queue_tail);
    let entry = VRAM_WRITE_QUEUE + usize::from(word!(game.vram_write_queue_tail));
    store_word(game, entry, OVERLAY_SIZE);
    store_word(game, entry + 3, OVERLAY_TILES_BANK);
    store_word(game, entry + 2, OVERLAY_TILES);
    store_word(game, entry + 5, OVERLAY_VRAM);
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

/// Main AI - enemy $E23F (Ceres door) ($A6:F765). Was `CeresDoor_Main`.
pub fn door_main<H: Host + ?Sized>(game: &mut H) {
    let e = current(game);
    let k = word!(game.cur_enemy_index);
    match slot!(game, Door[e].function) {
        door_function::HANDLE_ESCAPE_EARTHQUAKE => escape_rumble(0x14, game),
        door_function::HANDLE_RIDLEYS_ROOM_ESCAPE_EARTHQUAKE => escape_rumble(0x1d, game),
        door_function::RIDLEY_ESCAPE_MODE7_WALL => mode7_wall(game),
        door_function::ELEVATOR_DEFAULT => elevator_default(k, game),
        door_function::RUMBLING_AND_EXPLOSIONS => rumbling_and_explosions(k, game),
        door_function::ELEVATOR_ANIMATIONS => elevator_animations(game),
        _ => game.unreachable(),
    }
}

/// Handle Ceres door earthquake during escape ($A6:F773): a rare stronger
/// earthquake type replaces `earthquake`. Was `escape_rumble`.
pub fn escape_rumble<H: Host + ?Sized>(earthquake: u16, game: &mut H) {
    if word!(game.ceres_status) >= ESCAPE_STARTED && word!(game.earthquake_timer) == 0 {
        if (word!(game.random_number) & 0xfff) < 0x80 {
            word! {game; earthquake_timer = 4; earthquake_type = earthquake.wrapping_add(6); }
        } else {
            word! {game; earthquake_timer = 2; earthquake_type = earthquake; }
        }
    }
}

/// Ceres door function - Ridley escape mode 7 wall ($A6:F7A5): visible only
/// during the getaway cutscene. Was `CeresDoor_Func_4`.
pub fn mode7_wall<H: Host + ?Sized>(game: &mut H) {
    game.change_properties(word!(game.cur_enemy_index), kEnemyProps_Invisible as u16, 0);
    if word!(game.ceres_status) & RIDLEY_ESCAPING != 0 {
        slot!(game, Door[current(game)].base.palette_index = 3584);
        game.change_properties(word!(game.cur_enemy_index), 0, kEnemyProps_Invisible as u16);
    }
}

/// Ceres door function - rotating elevator room - default ($A6:F7BD): the
/// elevator animates; once the escape starts, 48 frames of rumbling. Was
/// `CeresDoor_Func_5`.
pub fn elevator_default<H: Host + ?Sized>(k: u16, game: &mut H) {
    elevator_animations(game);
    if word!(game.ceres_status) >= ESCAPE_STARTED {
        let e = i32::from(k);
        slot!(
            game,
            Door[e].function = door_function::RUMBLING_AND_EXPLOSIONS
        );
        slot!(game, Door[e].rumbling_timer = 48);
        slot!(game, Door[0]; rumble_timer = 0, rumble_index = 0);
    }
}

/// Ceres door function - rotating elevator room - rumbling and explosions
/// ($A6:F7DC): an explosion or smoke at the next of four offsets every five
/// frames, then the elevator room rotates. Was `CeresDoor_Func_6`.
pub fn rumbling_and_explosions<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    slot!(
        game,
        Door[e].rumbling_timer = slot!(game, Door[e].rumbling_timer).wrapping_sub(1)
    );
    if (slot!(game, Door[e].rumbling_timer) & 0x8000) != 0 {
        slot!(
            game,
            Door[e].base.properties =
                slot!(game, Door[e].base.properties) | kEnemyProps_Invisible as u16
        );
        slot!(game, Door[e].function = door_function::ELEVATOR_ANIMATIONS);
        start_elevator_rotation(game);
        return;
    }
    slot!(
        game,
        Door[0].rumble_timer = slot!(game, Door[0].rumble_timer).wrapping_sub(1)
    );
    if (slot!(game, Door[0].rumble_timer) & 0x8000) == 0 {
        return;
    }
    slot!(game, Door[0].rumble_timer = 4);
    slot!(
        game,
        Door[0].rumble_index = slot!(game, Door[0].rumble_index).wrapping_sub(1)
    );
    if (slot!(game, Door[0].rumble_index) & 0x8000) != 0 {
        slot!(game, Door[0].rumble_index = 3);
    }
    let pair = (2 * slot!(game, Door[0].rumble_index)) as isize;
    let x = slot!(game, Door[e].base.x_pos)
        .wrapping_add(crate::ceres_door_data::explosion_offset(game, pair));
    let y = slot!(game, Door[e].base.y_pos)
        .wrapping_add(crate::ceres_door_data::explosion_offset(game, pair + 1));
    game.set_projectile_spawn_point(x, y);
    let kind = if game.next_random() < 0x4000 {
        SMOKE
    } else {
        SMALL_EXPLOSION
    };
    game.spawn_room_projectile(DUST_CLOUD, kind);
    game.queue_sound(1, BIG_EXPLOSION_SOUND, 6);
}

/// Set Ceres elevator room to rotate if Ridley has escaped ($A6:F67F). Was
/// `finish_door_explosion`.
pub fn start_elevator_rotation<H: Host + ?Sized>(game: &mut H) {
    if word!(game.ceres_status) != 0 {
        word!(game.ceres_status = ELEVATOR_ROTATING);
    }
}

/// Ceres door function - rotating elevator room - handle Ceres elevator
/// animations ($A6:F850): the platform, and the beacon palette every eighth
/// frame. Was `CeresDoor_Func_7`.
pub fn elevator_animations<H: Host + ?Sized>(game: &mut H) {
    animate_elevator_platform(game);
    if word!(game.palette_change_num) == 0 {
        let frame = 2 * (word!(game.nmi_frame_counter_word) & 0x38);
        copy_rom_colors_to_palette(0x52, 0xa6, BEACON_PALETTES.wrapping_add(frame), 6, game);
    }
}

/// Animate Ceres elevator platform ($A6:F8F1): light and dark frames, two
/// frames each. Was `CeresDoor_Func_8`.
pub fn animate_elevator_platform<H: Host + ?Sized>(game: &mut H) {
    let frame = ((word!(game.nmi_frame_counter_word) & 2) >> 1) as isize;
    let list = crate::ceres_door_data::platform_frame(game, frame);
    game.queue_mode7_transfer(0xa6, list);
}
