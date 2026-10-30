//! Mother Brain's body effects: its rainbow beam palette cycle and the
//! explosions around it as it dies (bank $A9:B03E-$A9:BD44).
#![forbid(unsafe_code)]
use crate::actors::layouts::{EnemyLayout72 as Body, part_view};
use crate::actors::mother_brain_fight::{Lease, State};
use crate::actors::session_slots::session_slot;
use crate::cartridge::NativeDiagnostics;
use crate::cartridge::native_record;
use crate::cartridge::rom_table_word;
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::ram::CompatibilityRam;
use sm_data::cartridge::lorom_offset;

/// What Mother Brain's effects still reach in the compatibility runtime.
pub trait Host {
    /// Where the next enemy projectile spawns.
    fn set_projectile_spawn_point(&mut self, x: u16, y: u16);
    /// Spawn a death explosion of `kind` at the projectile spawn point.
    fn spawn_death_explosion(&mut self, kind: u16);
}

part_view! {
    /// The brain's slot, which holds the body's explosion and rainbow state.
    Brain: EnemyLayout72 {
        words: [
            var_a, var_b, var_c, var_d, explosion_timer, explosion_index, parameter_1,
            parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, rainbow_palette_index, var_02, var_03, var_04, var_05, var_06, var_07, var_08,
            var_09, var_0a, var_0b, var_0c, var_0d, var_0e, var_0f, var_10, var_11, var_12, var_13,
            var_14, var_15, var_16, var_17, var_18, var_19, var_1a, var_1b, var_1c, var_1d, var_1e,
            var_1f,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, var_21, var_22, var_23, var_24, var_25, var_26, var_27, var_28, var_29, var_2a,
            var_2b, var_2c, var_2d, var_2e, var_2f, var_30, var_31, var_32, var_33, var_34, var_35,
            var_36, var_37, var_38, var_39, var_3a, var_3b, var_3c, var_3d, var_3e, var_3f,
        ],
    }
}

/// The body is slot 0 and the brain slot 1.
const BODY: i32 = 0;
const BRAIN: i32 = 0x40;

/// Mother Brain's palettes are in bank $AD.
const PALETTE_BANK: u8 = 0xad;
/// $AD:E434: the rainbow beam palette animation, a zero-separated chain whose
/// nonzero words name each step's palette.
const RAINBOW_PALETTES: u32 = 0xad_e434;
/// Each palette is 15 colours (1-15): the body's BG1/2 palette 4, the sprite
/// palette 1 and, from the 16th colour on, the back leg's sprite palette 3.
const BODY_PALETTE: u16 = 0x82;
const SPRITE_PALETTE_1: u16 = 0x122;
const BACK_LEG_PALETTE: u16 = 0x162;
const COLORS: u16 = 15;
const BACK_LEG_COLORS: u16 = 30;
/// The rainbow advances on frames whose counter has this bit set.
const RAINBOW_FRAME_BIT: u16 = 2;

/// $A9:B099: X and Y offsets from the body, rows 0-6 of four explosions.
const EXPLOSION_OFFSETS: u32 = 0xa9_b099;
const LAST_EXPLOSION_ROW: u16 = 6;
const EXPLOSION_ROW_BYTES: u16 = 16;
const EXPLOSION_BANK: u8 = 0xa9;
/// The random draws from which the second and third explosion kinds are chosen.
const SECOND_KIND: u16 = 0x4000;
const THIRD_KIND: u16 = 0xe000;
/// Mother Brain exploding: sound 13h, library 3, at most 3 queued.
const EXPLODING_SOUND: u8 = 0x13;

/// Write Mother Brain palette ($A9:BD1D) from `source` in bank $AD. Was
/// `MotherBrain_WritePalette`.
pub fn write_palette(source: u16, game: &mut (impl CompatibilityRam + NativeDiagnostics + ?Sized)) {
    copy_rom_colors_to_palette(BODY_PALETTE, PALETTE_BANK, source, COLORS, game);
    copy_rom_colors_to_palette(SPRITE_PALETTE_1, PALETTE_BANK, source, COLORS, game);
    copy_rom_colors_to_palette(
        BACK_LEG_PALETTE,
        PALETTE_BANK,
        source.wrapping_add(BACK_LEG_COLORS),
        COLORS,
        game,
    );
}

/// Handle Mother Brain body rainbow beam palette ($A9:BCFD): every other frame,
/// follow the chain from the animation index to its next nonzero link, store
/// the link plus 2 and write the palette it names. Was
/// `MotherBrain_HandleRainbowBeamPalette`.
pub fn handle_rainbow_palette(game: &mut (impl State + ?Sized)) {
    if session_slot!(game, Body[BODY].base.frame_counter) & RAINBOW_FRAME_BIT == 0 {
        return;
    }
    let mut cursor = session_slot!(game, Brain[BRAIN].rainbow_palette_index);
    let mut link;
    loop {
        link = cursor;
        cursor = rom_table_word!(game, RAINBOW_PALETTES, (cursor >> 1) as isize);
        if cursor != 0 {
            break;
        }
    }
    let next = link.wrapping_add(2);
    session_slot!(game, Brain[BRAIN].rainbow_palette_index = next);
    // The palette word is read from the native image two bytes before `next`.
    let image = game.native_image();
    let at = (lorom_offset(RAINBOW_PALETTES) + next as isize - 2) as usize;
    write_palette(
        u16::from_ne_bytes([image.read(at), image.read(at + 1)]),
        game,
    );
}

/// Generate explosions around Mother Brain body ($A9:B03E): count down the
/// interval timer; when it expires, restart it at `interval` and spawn `count`
/// explosions at the next row of offsets, each kind drawn from the three words
/// at `kinds`. Was `MotherBrain_GenerateExplosions`.
pub fn generate_explosions(interval: u16, kinds: u16, count: u16, game: &mut impl Lease) {
    let timer = session_slot!(game, Brain[BRAIN].explosion_timer).wrapping_sub(1);
    session_slot!(game, Brain[BRAIN].explosion_timer = timer);
    if (timer as i16) >= 0 {
        return;
    }
    session_slot!(game, Brain[BRAIN].explosion_timer = interval);
    let row = session_slot!(game, Brain[BRAIN].explosion_index).wrapping_sub(1);
    session_slot!(game, Brain[BRAIN].explosion_index = row);
    if (row as i16) < 0 {
        session_slot!(game, Brain[BRAIN].explosion_index = LAST_EXPLOSION_ROW);
    }
    let position =
        EXPLOSION_ROW_BYTES.wrapping_mul(session_slot!(game, Brain[BRAIN].explosion_index));
    game.foreign(|game| spawn_explosions(position, kinds, count, game));
    game.queue_sound(2, EXPLODING_SOUND, 3);
}

/// Spawns `count` explosions from offset row byte `position`, each kind
/// drawn from the three words at `kinds`.
fn spawn_explosions(
    mut position: u16,
    kinds: u16,
    count: u16,
    game: &mut (impl Host + crate::actors::host::ActorHost + ?Sized),
) {
    let mut remaining = count;
    loop {
        let current = remaining;
        let index = (position >> 1) as isize;
        let x = rom_table_word!(game, EXPLOSION_OFFSETS, index);
        let y = rom_table_word!(game, EXPLOSION_OFFSETS, index + 1);
        game.set_projectile_spawn_point(x, y);
        let table = native_record(EXPLOSION_BANK, kinds, game);
        let mut kind = table.word(0);
        let random = game.next_random();
        if random >= SECOND_KIND {
            kind = table.word(2);
            if random >= THIRD_KIND {
                kind = table.word(4);
            }
        }
        game.spawn_death_explosion(kind);
        position = position.wrapping_add(4);
        remaining = current.wrapping_sub(1);
        if current == 1 {
            break;
        }
    }
}
