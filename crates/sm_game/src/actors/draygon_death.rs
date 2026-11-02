//! Draygon's hurt palettes, reactions and death sequence (bank
//! $A5:9185-$A5:9701).
//! Boss words remain native compatibility state; player and sprites use their owners.
use crate::actors::boss_lease::boss_ram;
use crate::actors::draygon_host::Lease;
use crate::actors::draygon_host::{BANK, BODY, Draygon};
use crate::actors::session_slots::session_slot;
use crate::enemy_script::EnemyScript;

/// Colours 9-12 of BG1/2 palette 5, as palette words and in RAM at $7E:C000.
const HEALTH_COLOURS: core::ops::Range<usize> = 89..93;
const PALETTE_RAM: usize = 0xc000;
/// The hurt AI's palettes: BG1/2 palette 5 and sprite palette 7, which flash
/// to $A5:A297.
const BG_PALETTE_5: u16 = 0xa277;
const SPRITE_PALETTE_7: u16 = 0xa1f7;
const FLASH_PALETTE: u16 = 0xa297;
const BG_PALETTE_5_RAM: usize = 0xc0a0;
const SPRITE_PALETTE_7_RAM: usize = 0xc1e0;
const PALETTE_BYTES: usize = 32;
/// Flash timer bit 1 flashes.
const FLASHING: u16 = 2;
/// The grapple beam drains 100h health every 8 frames.
const GRAPPLE_FLAG: u16 = 1;
const GRAPPLE_DRAIN: i16 = 0x100;
const GRAPPLE_FRAMES: u16 = 7;
/// Shots raise the swoop Y acceleration by 8, up to 98h.
const ACCELERATION_STEP: u16 = 8;
const MAX_ACCELERATION: u16 = 0x98;
/// The death spot, (100h, 1E0h), reached within 4 px; the evirs bury Draygon
/// for 1A0h frames, until Y 240h.
const DEATH_X: u16 = 0x100;
const DEATH_Y: u16 = 0x1e0;
const DEATH_SPOT_RANGE: u16 = 4;
const EVIR_FRAMES: u16 = 0x1a0;
const BURIED_Y: u16 = 0x240;
const FOAM_FRAMES: u16 = 0xf;
/// The elevator music track, and boss bit 0: Draygon is dead.
const ELEVATOR_MUSIC: u16 = 3;
const DRAYGON_DEAD: u16 = 1;
/// Angles are measured from the top, clockwise; 40h less is the maths angle.
const QUARTER_TURN: u8 = 0x40;

/// Write the health-based colours: natively into palette RAM, or into the
/// palette buffer. Was `damage_palette`.
pub fn write_health_palette<L: Lease>(native: bool, game: &mut L) {
    let mut cursor = session_slot!(game, Draygon[BODY].health_palette).wrapping_mul(4);
    for index in HEALTH_COLOURS {
        let value =
            crate::actor_tables::word(game, |t| &t.draygon_health_palettes, (cursor >> 1) as isize);
        if native {
            let bytes = value.to_ne_bytes();
            let at = PALETTE_RAM + index * 2;
            game.set_slot_byte(at, bytes[0]);
            game.set_slot_byte(at + 1, bytes[1]);
        } else {
            boss_ram::set_at(game, boss_ram::Array::PaletteBuffer, index, value);
        }
        cursor = cursor.wrapping_add(2);
    }
}
fn copy_hurt_palette<L: Lease>(default: u16, target: usize, game: &mut L) {
    let k = game.current_enemy() as i32;
    let palette = if session_slot!(game, Draygon[k].base.flash_timer) & FLASHING != 0 {
        FLASH_PALETTE
    } else {
        default
    };
    let record = crate::cartridge::native_record(BANK, palette, game);
    for offset in 0..PALETTE_BYTES {
        let byte = record.byte(offset);
        game.set_slot_byte(target + offset, byte);
    }
}
/// Hurt AI - Draygon ($A5:954D): flash the palettes and, while grappled,
/// drain health. Was `hurt`.
pub fn apply_hurt_palettes<L: Lease>(game: &mut L) {
    copy_hurt_palette(BG_PALETTE_5, BG_PALETTE_5_RAM, game);
    let k = game.current_enemy() as i32;
    if session_slot!(game, Draygon[k].base.flash_timer) & FLASHING == 0 {
        write_health_palette(false, game);
    }
    copy_hurt_palette(SPRITE_PALETTE_7, SPRITE_PALETTE_7_RAM, game);
}

pub fn hurt_ai<L: Lease>(game: &mut L) {
    apply_hurt_palettes(game);
    game.foreign(|game| {
        owned::hurt_grapple_drain(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    });
}
/// Draygon health-based palette handling ($A5:9701).
pub fn update_health_palette<L: Lease>(game: &mut L) {
    let mut cursor: i32 = 0;
    while (session_slot!(game, Draygon[BODY].base.health) as i16).wrapping_sub(
        crate::actor_tables::word(
            game,
            |t| &t.draygon_health_thresholds,
            (cursor >> 1) as isize,
        ) as i16,
    ) < 0
    {
        cursor += 2;
    }
    if cursor != i32::from(session_slot!(game, Draygon[BODY].health_palette)) {
        session_slot!(game, Draygon[BODY].health_palette = cursor as u16);
        write_health_palette(true, game);
    }
}
/// Enemy shot - Draygon ($A5:95F0).
pub fn shot<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::shot(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
/// Enemy touch - Draygon ($A5:95EA).
pub fn touch<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::touch(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
/// Power bomb reaction - Draygon ($A5:9607).
pub fn power_bomb<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::power_bomb(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
/// Draygon reaction ($A5:960D): update the health palette, or on death drop
/// the grapple, let go of Samus and drift to the death spot. Was `after_damage`.
pub fn reaction<L: Lease>(game: &mut L) {
    game.foreign(|game| owned::reaction(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}
/// Draygon body function - death sequence - drift to death spot ($A5:9185).
/// Was `approach`.
pub fn drift_to_death_spot<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        owned::drift_to_death_spot(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}
/// Handle dying Draygon smoke ($A5:92EA): every 8 frames, a big dust cloud low in the room.

/// Instruction - spawn dying Draygon sprite object `kind` within 40h px ($A5:973F-$A5:9778).
pub fn spawn_dying_sprite<H: owned::Host>(
    kind: u16,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    owned::spawn_dying_sprite(kind, script, game)
}
/// Instruction - spawn Draygon goop - leftwards ($A5:9F7C) or rightwards ($A5:9FAE).
pub fn spawn_goop<H: owned::Host>(right: bool, script: EnemyScript, game: &mut H) -> EnemyScript {
    owned::spawn_goop(right, script, game)
}
/// Draygon body function - death sequence - wait for evirs ($A5:9294). Was
/// `ring`.
pub fn wait_for_evirs<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        owned::wait_for_evirs(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}
/// Draygon body function - death sequence - buried by evirs ($A5:92AB): sink
/// until buried, then delete Draygon, set its boss bit and drop its rewards.
/// Was `sink`.
pub fn buried_by_evirs<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        owned::buried_by_evirs(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

#[path = "draygon_death_owned.rs"]
pub mod owned;
