//! Mother Brain encounter palettes, wall sealing, earthquakes and body flicker.
use crate::game::constants::bank_86::*;
use crate::game::prelude::*;
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game::sm_80::QueueSfx2_Max6;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_a9::WriteColorsToPalette;
const RomPtr_AD: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xad>;
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_MotherBrain(o: i32) -> Enemy_MotherBrain = [0xf78, o as isize];
}
pub enum Fade {
    ToBlack,
    FakeDeathToGray,
    FakeDeathFromGray,
    DrainedFromGray,
    DrainedToGray,
    RealDeathToGray,
    LightsOn,
}

pub unsafe fn fade(kind: Fade, frame: u16, game: &mut Game) -> u8 {
    unsafe {
        let (table, colors) = match kind {
            Fade::ToBlack => (0xade9e8, 14),
            Fade::FakeDeathToGray => (0xaded8a, 3),
            Fade::FakeDeathFromGray => (0xaded9c, 3),
            Fade::DrainedFromGray => (0xaded9c, 13),
            Fade::DrainedToGray => (0xadef87, 15),
            Fade::RealDeathToGray => (0xadf107, 15),
            Fade::LightsOn => (0xadf273, 14),
        };
        let source = game
            .g_rom
            .offset(crate::rtl::lorom_offset(table))
            .cast::<u16>()
            .offset(frame as isize)
            .read_unaligned();
        if source == 0 {
            return 1;
        }
        match kind {
            Fade::FakeDeathToGray | Fade::FakeDeathFromGray => {
                WriteColorsToPalette(0x122, 0xad, source, colors, game);
            }
            Fade::RealDeathToGray => WriteColorsToPalette(0x1e2, 0xad, source, colors, game),
            Fade::LightsOn => {
                WriteColorsToPalette(0x62, 0xad, source, colors, game);
                WriteColorsToPalette(0xa2, 0xad, source.wrapping_add(28), colors, game);
            }
            _ => {
                WriteColorsToPalette(0x82, 0xad, source, colors, game);
                WriteColorsToPalette(0x122, 0xad, source, colors, game);
                let tail = source.wrapping_add(colors * 2);
                if matches!(kind, Fade::ToBlack) {
                    WriteColorsToPalette(0x162, 0xad, tail, colors, game);
                } else {
                    WriteColorsToPalette(0x168, 0xad, tail, 5, game);
                    // The original accent write targets $017c, outside the palette buffer.
                    *(ram_ptr!(game, u16; 0x17c)) =
                        *(RomPtr_AD(tail.wrapping_add(10), game) as *const u16);
                }
            }
        }
        0
    }
}

pub unsafe fn update_health_palette(game: &mut Game) {
    unsafe {
        if (*Get_MotherBrain(0, game)).var_1F < 2 {
            return;
        }
        let health = (*Get_MotherBrain(0x40, game)).base.health;
        let level = if health >= 9000 {
            0
        } else if health >= 5400 {
            1
        } else if health >= 1800 {
            2
        } else {
            3
        };
        for destination in [0x82, 0x122] {
            WriteColorsToPalette(
                destination,
                0xad,
                crate::rtl::rom_table_word!(game, 0xade6a2, level),
                15,
                game,
            );
        }
        WriteColorsToPalette(
            0x162,
            0xad,
            crate::rtl::rom_table_word!(game, 0xade742, level),
            15,
            game,
        );
    }
}

pub unsafe fn lights_off(game: &mut Game) {
    unsafe {
        for destination in [0x62, 0xa2] {
            WriteColorsToPalette(destination, 0xad, 61999, 14, game);
        }
    }
}

pub unsafe fn seal_wall(game: &mut Game) {
    unsafe {
        game.eproj_spawn_pt = Point16U { x: 248, y: 72 };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
        game.eproj_spawn_pt = Point16U { x: 248, y: 152 };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x4, 0xb673 as VoidP), game);
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x9, 0xb673 as VoidP), game);
        (*Get_MotherBrain(0i32, game)).var_A = ((0xa9881di32) as u16);
    }
}
pub unsafe fn start_earthquake(a: u16, game: &mut Game) {
    unsafe {
        word! {game; earthquake_type = a; earthquake_timer = 20; }
        QueueSfx2_Max6(0x25, game);
    }
}
pub unsafe fn flicker_body(game: &mut Game) {
    unsafe {
        let v0 = gEnemyData(0, game);
        if (*v0).frame_counter & 1 != 0 {
            crate::update_place! { unaligned_word fresh0 = &raw mut (*(ram_ptr!(game, u16; 0x69))) => (u16::from_le(fresh0.read_unaligned())) | (2u16); }
            (*v0).properties = ((*v0).properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
        } else {
            crate::update_place! { unaligned_word fresh1 = &raw mut (*(ram_ptr!(game, u16; 0x69))) => (u16::from_le(fresh1.read_unaligned()) & !(2)); }
            (*v0).properties = ((*v0).properties) | ((kEnemyProps_Invisible) as u16);
        };
    }
}
