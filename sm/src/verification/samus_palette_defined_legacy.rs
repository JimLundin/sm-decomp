//! Reference-only adapter: never install alongside retired bank implementations.
use super::samus_defined_cases::{self, Operation};
use crate::{
    game::{sm_90, sm_91 as old},
    game_state::Game,
};
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    samus_defined_cases::capture_case(id, run)
}
fn run(operation: Operation, game: &mut Game) -> u16 {
    unsafe {
        match operation {
            Operation::Advance => old::Samus_HandlePalette(game),
            Operation::Misc => old::HandleMiscSamusPalette(game),
            Operation::Visor => return old::HandleVisorPalette(game).into(),
            Operation::ScrewSpeed => {
                return old::Samus_HandleScrewAttackSpeedBoostingPals(game).into();
            }
            Operation::Shine => return old::Samus_SpeedBoosterShinePals(game).into(),
            Operation::Shinespark => return old::Samus_HandleShinesparkingPals(game).into(),
            Operation::Crystal => return old::Samus_HandleCrystalFlashPals(game).into(),
            Operation::Xray => return old::Samus_HandleXrayPals(game).into(),
            Operation::CopyCurrent => old::CopyToSamusSuitPalette(0x9000, game),
            Operation::CopyTarget => old::CopyToSamusSuitTargetPalette(0x9000, game),
            Operation::Copy10 => old::Samus_Copy10PalColors(0x9000, game),
            Operation::Copy6 => old::Samus_Copy6PalColors(0x9000, game),
            Operation::LoadCurrent => old::Samus_LoadSuitPalette(game),
            Operation::LoadTarget => old::Samus_LoadSuitTargetPalette(game),
            Operation::Cancel => old::Samus_CancelSpeedBoost(game),
            Operation::Beam => sm_90::WriteBeamPalette_A(
                u16::from_le_bytes(game.g_ram[0x9a6..0x9a8].try_into().unwrap()),
                game,
            ),
        }
    }
    0
}
