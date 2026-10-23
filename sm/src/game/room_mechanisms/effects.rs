//! Existing external implementation calls; callers release RoomMechanisms first.
use crate::game_state::Game;
pub(super) fn missing_displacement(game: &mut Game) {
    unsafe {
        crate::c_library::printf(c"ERROR: PlmSetup_QuicksandSurfaceB is broken!!\n".as_ptr());
        crate::rtl::Unreachable(game);
    }
}

pub(super) fn sound(
    channel: super::fields::SoundChannel,
    maximum: u8,
    sound: u16,
    game: &mut Game,
) {
    use super::fields::SoundChannel::*;
    use crate::game::sm_80::*;
    let call: unsafe fn(u16, &mut Game) = match (channel, maximum) {
        (One, 1) => QueueSfx1_Max1,
        (One, 3) => QueueSfx1_Max3,
        (One, 6) => QueueSfx1_Max6,
        (One, 9) => QueueSfx1_Max9,
        (One, 15) => QueueSfx1_Max15,
        (Two, 1) => QueueSfx2_Max1,
        (Two, 3) => QueueSfx2_Max3,
        (Two, 6) => QueueSfx2_Max6,
        (Two, 9) => QueueSfx2_Max9,
        (Two, 15) => QueueSfx2_Max15,
        (Three, 1) => QueueSfx3_Max1,
        (Three, 3) => QueueSfx3_Max3,
        (Three, 6) => QueueSfx3_Max6,
        (Three, 9) => QueueSfx3_Max9,
        (Three, 15) => QueueSfx3_Max15,
        _ => unreachable!("mechanism sound limit"),
    };
    unsafe { call(sound, game) };
}

pub(super) fn eproj(definition: u16, parameter: u16, game: &mut Game) {
    unsafe {
        crate::game::enemy_projectiles::SpawnEprojWithRoomGfx(definition, parameter, game);
    }
}
pub(super) fn player_control(operation: u16, game: &mut Game) {
    unsafe {
        crate::game::sm_90::CallSomeSamusCode(operation, game);
    }
}

pub(super) fn record<const BANK: u8>(address: u16, game: &mut Game) -> crate::rom::RomRecord {
    unsafe { crate::rtl::rom_bank_record::<BANK>(address, game) }
}

pub(super) fn unsupported_b3e3(game: &mut Game) -> u8 {
    use crate::c_library::__assert_fail;
    use core::ffi::c_char;
    unsafe {
        crate::native_assertion! { '_c2rust_label; b"0\0", b"src/sm_84.c\0", 1370, b"uint8 UNUSED_sub_84B3E3(uint16)\0" };
    }
    0
}
pub(super) fn unreachable(game: &mut Game) -> u8 {
    unsafe { crate::rtl::Unreachable(game) as u8 }
}

pub(super) fn instruction_function(ea: u32, game: &mut Game) {
    unsafe {
        match ea {
            9557220 => crate::game::sm_91::VariaSuitPickup(game),
            9557434 => crate::game::sm_91::GravitySuitPickup(game),
            _ => {
                crate::rtl::Unreachable(game);
            }
        }
    }
}
pub(super) fn face_forward(game: &mut Game) {
    unsafe { crate::game::sm_91::MakeSamusFaceForward(game) }
}
pub(super) fn save(slot: u16, game: &mut Game) {
    unsafe {
        crate::game::sm_81::SaveToSram(slot, game);
    }
}
pub(super) fn pickup_visual(visual: crate::game::item_pickups::VisualUpdate, game: &mut Game) {
    use crate::game::item_pickups::VisualUpdate::*;
    use crate::game::sm_80::*;
    unsafe {
        match visual {
            None => {}
            Beams => crate::game::sm_90::UpdateBeamTilesAndPalette(game),
            Grapple => AddGrappleToHudTilemap(game),
            Xray => AddXrayToHudTilemap(game),
            Missiles => AddMissilesToHudTilemap(game),
            SuperMissiles => AddSuperMissilesToHudTilemap(game),
            PowerBombs => AddPowerBombsToHudTilemap(game),
        }
    }
}
