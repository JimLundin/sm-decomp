pub use crate::game::constants::bank_86::*;
pub use crate::game::prelude::*;

use crate::game::sm_80::QueueSfx2_Max6;
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_a9::WriteColorsToPalette;

pub use crate::types::Consts_AD;

crate::game::constants::typed_constants! { Consts_AD;
    addr_kTurnOffLightsForShitroidDeath = 61999,
}
const RomPtr_AD: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xad>;
crate::game::constants::typed_constants! { i32;
    fnMotherBrainBody_FakeDeath_Descent_0_Pause = 0xa9881d,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_MotherBrain(o: i32) -> Enemy_MotherBrain = [0xf78, o as isize];
}
pub fn nullsub_341(game: &mut Game) {}
pub unsafe fn MotherBrain_SealWall(game: &mut Game) {
    game.eproj_spawn_pt = Point16U { x: 248, y: 72 };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
    game.eproj_spawn_pt = Point16U { x: 248, y: 152 };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0xf, 0x4, 0xb673 as VoidP), game);
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0xf, 0x9, 0xb673 as VoidP), game);
    (*Get_MotherBrain(0i32, game)).var_A = ((0xa9881di32) as u16);
}
pub unsafe fn MotherBrain_HealthBasedPaletteHandling(game: &mut Game) {
    if (*Get_MotherBrain(0i32, game)).var_1F >= 2 {
        let mut v0: u16 = 0;
        let health: u16 = (*Get_MotherBrain(0x40i32, game)).base.health;
        if health < 0x2328 {
            v0 = 2;
            if health < 0x1518 {
                v0 = 4;
                if health < 0x708 {
                    v0 = 6;
                }
            }
        }
        WriteColorsToPalette(
            0x82,
            0xad,
            crate::rtl::rom_table_word!(game, 0xade6a2, (v0 >> 1) as isize),
            0xf,
            game,
        );
        WriteColorsToPalette(
            0x122,
            0xad,
            crate::rtl::rom_table_word!(game, 0xade6a2, (v0 >> 1) as isize),
            0xf,
            game,
        );
        WriteColorsToPalette(
            0x162,
            0xad,
            crate::rtl::rom_table_word!(game, 0xade742, (v0 >> 1) as isize),
            0xf,
            game,
        );
    }
}
pub unsafe fn MotherBrain_FadePalToBlack(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xade9e8, v1 as isize) == 0 {
        return 1;
    }
    let j: u16 = crate::rtl::rom_table_word!(game, 0xade9e8, v1 as isize);
    WriteColorsToPalette(0x82, 0xad, j, 14, game);
    WriteColorsToPalette(0x122, 0xad, j, 14, game);
    WriteColorsToPalette(0x162, 0xad, (j).wrapping_add(28), 14, game);
    0
}
pub unsafe fn MotherBrain_FadeToGray_FakeDeath(a: u16, game: &mut Game) -> u8 {
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xaded8a, a as isize);
    if v1 != 0 {
        (MotherBrain_EEF6(v1, game)) & (1u8)
    } else {
        1
    }
}
pub unsafe fn MotherBrain_FadeFromGray_FakeDeath(a: u16, game: &mut Game) -> u8 {
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xaded9c, a as isize);
    if v1 != 0 {
        (MotherBrain_EEF6(v1, game)) & (1u8)
    } else {
        1
    }
}
pub unsafe fn MotherBrain_EEF6(a: u16, game: &mut Game) -> u8 {
    WriteColorsToPalette(0x122, 0xad, a, 3, game);
    0
}
pub unsafe fn MotherBrain_FadeFromGray_Drained(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xaded9c, v1 as isize) == 0 {
        return 1;
    }
    let j: u16 = crate::rtl::rom_table_word!(game, 0xaded9c, v1 as isize);
    WriteColorsToPalette(0x82, 0xad, j, 13, game);
    WriteColorsToPalette(0x122, 0xad, j, 13, game);
    WriteColorsToPalette(0x168, 0xad, (j).wrapping_add(26), 5, game);
    *(ram_ptr!(game, u16; 0x17c)) = *(RomPtr_AD((j).wrapping_add(36), game) as *mut u16);
    0
}
pub unsafe fn MotherBrain_FadeToGray_Drained(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xadef87, v1 as isize) == 0 {
        return 1;
    }
    let j: u16 = crate::rtl::rom_table_word!(game, 0xadef87, v1 as isize);
    WriteColorsToPalette(0x82, 0xad, j, 15, game);
    WriteColorsToPalette(0x122, 0xad, j, 15, game);
    WriteColorsToPalette(0x168, 0xad, (j).wrapping_add(30), 5, game);
    *(ram_ptr!(game, u16; 0x17c)) = *(RomPtr_AD((j).wrapping_add(40), game) as *mut u16);
    0
}
pub unsafe fn MotherBrain_FadeToGray_RealDeath(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xadf107, v1 as isize) == 0 {
        return 1;
    }
    WriteColorsToPalette(
        0x1e2,
        0xad,
        crate::rtl::rom_table_word!(game, 0xadf107, v1 as isize),
        0xf,
        game,
    );
    0
}
pub unsafe fn TurnOffLightsForShitroidDeath(game: &mut Game) {
    WriteColorsToPalette(
        0x62,
        0xad,
        addr_kTurnOffLightsForShitroidDeath as u16,
        14,
        game,
    );
    WriteColorsToPalette(
        0xa2,
        0xad,
        addr_kTurnOffLightsForShitroidDeath as u16,
        14,
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_TurnLightsBackOn(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xadf273, v1 as isize) == 0 {
        return 1;
    }
    let v3: u16 = crate::rtl::rom_table_word!(game, 0xadf273, v1 as isize);
    WriteColorsToPalette(0x62, 0xad, v3, 14, game);
    WriteColorsToPalette(0xa2, 0xad, (v3).wrapping_add(28), 14, game);
    0
}
pub unsafe fn EnableEarthquakeAframes(a: u16, game: &mut Game) {
    word! {game; earthquake_type = a; earthquake_timer = 20; }
    QueueSfx2_Max6(0x25, game);
}
pub unsafe fn HandleMotherBrainBodyFlickering(game: &mut Game) {
    let v0 = gEnemyData(0, game);
    if (*v0).frame_counter & 1 != 0 {
        crate::update_place! { unaligned_word fresh0 = &raw mut (*(ram_ptr!(game, u16; 0x69))) => (u16::from_le(fresh0.read_unaligned())) | (2u16); }
        (*v0).properties = ((*v0).properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    } else {
        crate::update_place! { unaligned_word fresh1 = &raw mut (*(ram_ptr!(game, u16; 0x69))) => (u16::from_le(fresh1.read_unaligned()) & !(2)); }
        (*v0).properties = ((*v0).properties) | ((kEnemyProps_Invisible) as u16);
    };
}
