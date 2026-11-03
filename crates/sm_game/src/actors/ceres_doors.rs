//! Ceres station's steam vents and doors (bank $A6:EFB1-$A6:FB71): the
//! steam's activation and its offset in the rotating elevator room, and the
//! doors' functions, the escape's rumbling and explosions and the elevator
//! animations.
//!
//! Was `CeresSteam_*` and `CeresDoor_*` in the original decompilation.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{BossEffects, Death, abs16};
use crate::actors::slot_view::{SlotState, slot_index, slot_view};
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DISABLE_SAMUS_COLL, K_ENEMY_PROPS_INVISIBLE, K_ENEMY_PROPS_TANGIBLE,
};
use crate::display::transfer::{Source, TilemapUpload, queue_tilemap};
use crate::enemy_script::EnemyScript;
use crate::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};
use crate::ram::word;

/// The lease the steam and doors run on, with what they still reach in the
/// game. Their slots are the Session's.
pub trait Lease:
    BossLease<Game: BossEffects> + crate::display::preparation::Access + SlotState
{
    /// The current area's boss bits.
    fn area_boss_flags(&mut self) -> u8;
    /// Queue the bank `bank` mode 7 transfer list at `list`.
    fn queue_mode7_transfer(&mut self, bank: u8, list: u16);
}

slot_view! {
    /// A steam vent's slot: its function, activation timer and kind.
    Steam, SteamEdit {
        record: [function = var_a, activation_timer = var_d, kind = parameter_1],
        extended: [],
    }
}

slot_view! {
    /// A Ceres door's slot: its function, whether Ridley draws it, its
    /// rumbling timer and kind. Enemy 0's `rumble_timer` and `rumble_index`
    /// pace the rotating elevator room's explosions.
    Door, DoorEdit {
        record: [
            function = var_a, drawn_by_ridley = var_b, rumbling_timer = var_d,
            rumble_timer = var_e, rumble_index = var_f, kind = parameter_1,
        ],
        extended: [],
    }
}

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
const OVERLAY_TILES_BANK: u8 = 0xb0;
const OVERLAY_VRAM: u16 = 0x7000;
const OVERLAY_SIZE: u16 = 1024;
/// Bank $86 dust cloud / explosion enemy projectile and its kinds.
const DUST_CLOUD: u16 = 0xe509;
const SMALL_EXPLOSION: u16 = 3;
const SMOKE: u16 = 12;
const BIG_EXPLOSION_SOUND: u8 = 0x25;
/// Ceres status values.
const RIDLEY_ESCAPING: u16 = 1;
const ESCAPE_STARTED: u16 = 2;
const ELEVATOR_ROTATING: u16 = 0x8000;

fn current<L: Lease + ?Sized>(game: &L) -> u16 {
    game.current_enemy()
}

/// Set and then clear enemy `k`'s property bits.
fn change_properties<L: Lease + ?Sized>(k: u16, set: u16, clear: u16, game: &mut L) {
    let mut door = DoorEdit::of(game, k);
    let properties = &mut door.actor().properties;
    *properties = (*properties & !clear) | set;
}

fn branch(taken: bool, script: EnemyScript) -> EnemyScript {
    if taken {
        EnemyScript::jump(script.word(0))
    } else {
        script.offset(1)
    }
}

/// Instruction - hide and show enemy ($A6:F11D, $A6:F135), and the Ceres door instructions
/// ($A6:F63E-$A6:F6C4): go to [[Y]] if Samus is not within 30h pixels, if the area boss is
/// alive or if Ceres Ridley has not escaped; set intangible, tangible, invisible or visible;
/// set or clear the drawn by Ridley flag; queue the door opening sound.
pub fn instruction<L: Lease + ?Sized>(
    ea: u32,
    k: u16,
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    const INTANGIBLE: u16 = K_ENEMY_PROPS_TANGIBLE as u16;
    const INVISIBLE: u16 = K_ENEMY_PROPS_INVISIBLE as u16;
    match ea {
        0xa6_f11d => change_properties(k, VANISH, 0, game),
        0xa6_f135 => change_properties(k, 0, VANISH, game),
        0xa6_f63e => {
            let door = Door::of(game, k).actor();
            let (x, y) = (door.x.whole, door.y.whole);
            let far = abs16(x.wrapping_sub(game.player_x())) >= 0x30
                || abs16(y.wrapping_sub(game.player_y())) >= 0x30;
            return branch(far, script);
        }
        0xa6_f66a => return branch((game.area_boss_flags() & 1) == 0, script),
        0xa6_f678 => return branch(crate::sequence_word!(game.CERES_STATUS) == 0, script),
        0xa6_f68b => change_properties(k, INTANGIBLE, 0, game),
        0xa6_f695 => change_properties(k, 0, INTANGIBLE, game),
        0xa6_f6a6 => change_properties(k, INVISIBLE, 0, game),
        0xa6_f6b3 => change_properties(k, 0, INVISIBLE, game),
        0xa6_f69f | 0xa6_f6b0 => {
            let drawn = u16::from(ea == 0xa6_f69f);
            *DoorEdit::of(game, k).drawn_by_ridley() = drawn;
            if drawn == 0 {
                change_properties(k, 0, INVISIBLE, game);
            }
        }
        _ => game.queue_sound(2, 0x2c, 6),
    }
    script
}

/// Enemy touch - enemy $E1FF (steam) ($A6:F03F): kept at full health, then the normal reaction.
pub fn steam_touch<L: Lease + ?Sized>(game: &mut L) {
    let steam = current(game);
    SteamEdit::of(game, steam).actor().health = 0x7fff;
    game.react_to_contact(Death::Finalize);
}

/// Initialisation AI - enemy $E1FF (steam) ($A6:EFB1): its kind picks the
/// instruction list and function; it waits 1 to 20h frames to start. Was
/// `CeresSteam_Init`.
pub fn initialize_steam<L: Lease + ?Sized>(game: &mut L) {
    let e = current(game);
    {
        let mut steam = SteamEdit::of(game, e);
        let actor = steam.actor();
        actor.tile = 0;
        actor.properties |= K_ENEMY_PROPS_DISABLE_SAMUS_COLL as u16;
        actor.extra_properties |= 4;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.palette = 2560;
    }
    let timer = (game.roll_random() & 0x1f) + 1;
    *SteamEdit::of(game, e).activation_timer() = timer;
    let kind = crate::ceres_door_data::steam(game, Steam::of(game, e).kind() as isize);
    let mut steam = SteamEdit::of(game, e);
    steam.actor().instruction = kind.instruction_list;
    *steam.function() = kind.function;
}

/// Main AI - enemy $E1FF (steam) ($A6:F00D), at full health. Was
/// `CeresSteam_Main`.
pub fn steam_main<L: Lease + ?Sized>(game: &mut L) {
    let e = current(game);
    SteamEdit::of(game, e).actor().health = 0x7fff;
    let k = game.current_enemy();
    match Steam::of(game, e).function() {
        steam_function::NONE => (),
        steam_function::CALCULATE_ROTATING_ELEVATOR_OFFSET => steam_rotating_offset(k, game),
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// Steam function - calculate graphical offset in rotating elevator room
/// ($A6:F019), into the enemy's spawn record. Was `CeresSteam_Func_1`.
pub fn steam_rotating_offset<L: Lease + ?Sized>(k: u16, game: &mut L) {
    let steam = Steam::of(game, k).actor();
    let point = crate::geometry::Point {
        x: steam.x.whole,
        y: steam.y.whole,
    };
    let crate::geometry::Point { x, y } = crate::cinematic::project_cinematic_point(point, &*game);
    let record = slot_index(game.current_enemy());
    let x_offset = x.wrapping_sub(Steam::of(game, k).actor().x.whole);
    game.session_mut().actors.saved[record].draw_offset.x =
        u16::from_le_bytes(x_offset.to_ne_bytes());
    let y_offset = y.wrapping_sub(Steam::of(game, k).actor().y.whole);
    game.session_mut().actors.saved[record].draw_offset.y =
        u16::from_le_bytes(y_offset.to_ne_bytes());
}

/// Instruction - decrement activation timer and go to [[Y]] if non-zero,
/// otherwise show enemy and go to [[Y] + 2] ($A6:F127). Was
/// `CeresSteam_Instr_2`.
pub fn steam_activate<L: Lease + ?Sized>(k: u16, script: EnemyScript, game: &mut L) -> EnemyScript {
    let timer = Steam::of(game, k).activation_timer();
    *SteamEdit::of(game, k).activation_timer() = timer.wrapping_sub(1);
    if timer != 1 {
        return EnemyScript::jump(script.word(0));
    }
    let record = game.actor_record(0xa6, script.word(2));
    let next = EnemyScript::at(record.image, record.base);
    change_properties(k, 0, VANISH, game);
    next
}

/// The steam's hidden state: intangible and invisible.
pub const VANISH: u16 = 0x0400 | 0x0100;

/// Initialisation AI - enemy $E23F (Ceres door) ($A6:F6C5): its kind picks
/// the function and instruction list; the palette depends on the escape.
/// Was `CeresDoor_Init`.
pub fn initialize_door<L: Lease + ?Sized>(game: &mut L) {
    let e = current(game);
    {
        let mut door = DoorEdit::of(game, e);
        let actor = door.actor();
        actor.spritemap = DOOR_SPRITEMAP;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.tile = 0;
        actor.palette = 1024;
    }
    let kind = Door::of(game, e).kind() as isize;
    let function = crate::ceres_door_data::door_function(game, kind);
    *DoorEdit::of(game, e).function() = function;
    let list = crate::ceres_door_data::door_instructions(game, kind);
    {
        let mut door = DoorEdit::of(game, e);
        door.actor().instruction = list;
        *door.drawn_by_ridley() = 0;
    }
    load_overlay_tiles(game.current_enemy(), game);
    let palette = if crate::sequence_word!(game.CERES_STATUS) != 0 {
        ESCAPE_DOOR_PALETTE.wrapping_add(2)
    } else {
        if Door::of(game, e).kind() == RIDLEYS_ROOM_FACING_RIGHT {
            copy_rom_colors_to_target_palette(0xa6, 0x142, DOOR_PALETTE.wrapping_add(2), 0xf, game);
            return;
        }
        DOOR_PALETTE.wrapping_add(2)
    };
    let door = current(game);
    DoorEdit::of(game, door).actor().palette = 3584;
    copy_rom_colors_to_target_palette(0xa6, 0x1e2, palette, 0xf, game);
}

/// Load rotating elevator room pre-explosion door overlay tiles if needed
/// ($A6:F739): queue $B0:C400..C7FF to VRAM $7000. Was `CeresDoor_Func_1`.
pub fn load_overlay_tiles<L: Lease + ?Sized>(k: u16, game: &mut L) {
    if Door::of(game, k).kind() != ELEVATOR_DOOR_OVERLAY {
        return;
    }
    // The original stores the bank's word before the source's, whose high byte
    // then overwrites the bank's low byte: the record is the plain upload.
    queue_tilemap(
        game,
        TilemapUpload {
            bytes: OVERLAY_SIZE,
            source: Source {
                address: OVERLAY_TILES,
                bank: OVERLAY_TILES_BANK,
            },
            column: false,
            vram_address: OVERLAY_VRAM,
        },
    );
}

/// Main AI - enemy $E23F (Ceres door) ($A6:F765). Was `CeresDoor_Main`.
pub fn door_main<L: Lease + ?Sized>(game: &mut L) {
    let e = current(game);
    let k = game.current_enemy();
    match Door::of(game, e).function() {
        door_function::HANDLE_ESCAPE_EARTHQUAKE => escape_rumble(0x14, game),
        door_function::HANDLE_RIDLEYS_ROOM_ESCAPE_EARTHQUAKE => escape_rumble(0x1d, game),
        door_function::RIDLEY_ESCAPE_MODE7_WALL => mode7_wall(game),
        door_function::ELEVATOR_DEFAULT => elevator_default(k, game),
        door_function::RUMBLING_AND_EXPLOSIONS => rumbling_and_explosions(k, game),
        door_function::ELEVATOR_ANIMATIONS => elevator_animations(game),
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// Handle Ceres door earthquake during escape ($A6:F773): a rare stronger
/// earthquake type replaces `earthquake`. Was `escape_rumble`.
pub fn escape_rumble<L: Lease + ?Sized>(earthquake: u16, game: &mut L) {
    if crate::sequence_word!(game.CERES_STATUS) >= ESCAPE_STARTED
        && word!(game.EARTHQUAKE_TIMER) == 0
    {
        if (word!(game.RANDOM_NUMBER) & 0xfff) < 0x80 {
            word! {game; EARTHQUAKE_TIMER = 4; EARTHQUAKE_TYPE = earthquake.wrapping_add(6); }
        } else {
            word! {game; EARTHQUAKE_TIMER = 2; EARTHQUAKE_TYPE = earthquake; }
        }
    }
}

/// Ceres door function - Ridley escape mode 7 wall ($A6:F7A5): visible only
/// during the getaway cutscene. Was `CeresDoor_Func_4`.
pub fn mode7_wall<L: Lease + ?Sized>(game: &mut L) {
    change_properties(
        game.current_enemy(),
        K_ENEMY_PROPS_INVISIBLE as u16,
        0,
        game,
    );
    if crate::sequence_word!(game.CERES_STATUS) & RIDLEY_ESCAPING != 0 {
        let door = current(game);
        DoorEdit::of(game, door).actor().palette = 3584;
        change_properties(
            game.current_enemy(),
            0,
            K_ENEMY_PROPS_INVISIBLE as u16,
            game,
        );
    }
}

/// Ceres door function - rotating elevator room - default ($A6:F7BD): the
/// elevator animates; once the escape starts, 48 frames of rumbling. Was
/// `CeresDoor_Func_5`.
pub fn elevator_default<L: Lease + ?Sized>(k: u16, game: &mut L) {
    elevator_animations(game);
    if crate::sequence_word!(game.CERES_STATUS) >= ESCAPE_STARTED {
        {
            let mut door = DoorEdit::of(game, k);
            *door.function() = door_function::RUMBLING_AND_EXPLOSIONS;
            *door.rumbling_timer() = 48;
        }
        let mut first = DoorEdit::of(game, 0);
        *first.rumble_timer() = 0;
        *first.rumble_index() = 0;
    }
}

/// Ceres door function - rotating elevator room - rumbling and explosions
/// ($A6:F7DC): an explosion or smoke at the next of four offsets every five
/// frames, then the elevator room rotates. Was `CeresDoor_Func_6`.
pub fn rumbling_and_explosions<L: Lease + ?Sized>(k: u16, game: &mut L) {
    {
        let mut door = DoorEdit::of(game, k);
        let timer = door.rumbling_timer();
        *timer = timer.wrapping_sub(1);
        if (*timer & 0x8000) != 0 {
            door.actor().properties |= K_ENEMY_PROPS_INVISIBLE as u16;
            *door.function() = door_function::ELEVATOR_ANIMATIONS;
            drop(door);
            start_elevator_rotation(game);
            return;
        }
    }
    let pair = {
        let mut first = DoorEdit::of(game, 0);
        let timer = first.rumble_timer();
        *timer = timer.wrapping_sub(1);
        if (*timer & 0x8000) == 0 {
            return;
        }
        *timer = 4;
        let index = first.rumble_index();
        *index = index.wrapping_sub(1);
        if (*index & 0x8000) != 0 {
            *index = 3;
        }
        (2 * *index) as isize
    };
    let x = Door::of(game, k)
        .actor()
        .x
        .whole
        .wrapping_add(crate::ceres_door_data::explosion_offset(game, pair));
    let y = Door::of(game, k)
        .actor()
        .y
        .whole
        .wrapping_add(crate::ceres_door_data::explosion_offset(game, pair + 1));
    game.place_projectile_spawn(x, y);
    let kind = if game.roll_random() < 0x4000 {
        SMOKE
    } else {
        SMALL_EXPLOSION
    };
    game.spawn_room_eproj(DUST_CLOUD, kind);
    game.queue_sound(1, BIG_EXPLOSION_SOUND, 6);
}

/// Set Ceres elevator room to rotate if Ridley has escaped ($A6:F67F). Was
/// `finish_door_explosion`.
pub fn start_elevator_rotation<L: Lease + ?Sized>(game: &mut L) {
    if crate::sequence_word!(game.CERES_STATUS) != 0 {
        crate::sequence_word!(game.CERES_STATUS = ELEVATOR_ROTATING);
    }
}

/// Ceres door function - rotating elevator room - handle Ceres elevator
/// animations ($A6:F850): the platform, and the beacon palette every eighth
/// frame. Was `CeresDoor_Func_7`.
pub fn elevator_animations<L: Lease + ?Sized>(game: &mut L) {
    animate_elevator_platform(game);
    if crate::sequence_word!(game.PALETTE_CHANGE_NUM) == 0 {
        let frame = 2 * (crate::frame_display_word!(game.NmiFrameWordPair) & 0x38);
        copy_rom_colors_to_palette(0x52, 0xa6, BEACON_PALETTES.wrapping_add(frame), 6, game);
    }
}

/// Animate Ceres elevator platform ($A6:F8F1): light and dark frames, two
/// frames each. Was `CeresDoor_Func_8`.
pub fn animate_elevator_platform<L: Lease + ?Sized>(game: &mut L) {
    let frame = ((crate::frame_display_word!(game.NmiFrameWordPair) & 2) >> 1) as isize;
    let list = crate::ceres_door_data::platform_frame(game, frame);
    game.queue_mode7_transfer(0xa6, list);
}

// The door functions, palettes and spritemap are the dispatch IDs and the
// decompilation's addresses; the steam's hidden state is the doors' one.
const _: () = {
    use crate::actors::ceres_doors::{self as c, door_function as f};
    use crate::constants::bank_a6::*;
    assert!((0xa6_0000 | f::RUMBLING_AND_EXPLOSIONS as u32) == 10942428);
    assert!((0xa6_0000 | f::ELEVATOR_ANIMATIONS as u32) == 10942544);
    assert!(c::DOOR_PALETTE == ADDR_K_CERES_DOOR_PALETTE as u16);
    assert!(c::ESCAPE_DOOR_PALETTE == ADDR_WORD_A6F50C as u16);
    assert!(c::DOOR_SPRITEMAP == ADDR_K_CERES_DOOR_SPRMAP_FAC7 as u16);
};
