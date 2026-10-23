//! Candidate adapter calls actual safe core on the aggregate Runtime.
#![forbid(unsafe_code)]
use super::samus_defined_cases::{self, Operation};
use crate::{
    game::{
        samus_palette as core,
        session_access::SessionAccess,
        session_runtime::Runtime,
        visual_state::{PaletteBuffer, PaletteCarry},
    },
    game_state::Game,
};
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    samus_defined_cases::capture_case(id, run)
}
fn run(operation: Operation, game: &mut Game) -> u16 {
    let mut host = Runtime::new(game);
    let carry = match operation {
        Operation::Advance => {
            core::advance(&mut host);
            return 0;
        }
        Operation::Misc => {
            core::misc(&mut host);
            return 0;
        }
        Operation::Visor => return core::visor(&mut host).into(),
        Operation::ScrewSpeed => core::screw_speed(&mut host),
        Operation::Shine => core::shine(&mut host),
        Operation::Shinespark => core::shinespark(&mut host),
        Operation::Crystal => core::crystal_flash(&mut host),
        Operation::Xray => core::xray(&mut host),
        Operation::CopyCurrent => {
            core::copy_suit(&mut host, PaletteBuffer::Current, 0x9000);
            return 0;
        }
        Operation::CopyTarget => {
            core::copy_suit(&mut host, PaletteBuffer::Target, 0x9000);
            return 0;
        }
        Operation::Copy10 => {
            core::copy_crystal_first10(&mut host, 0x9000);
            return 0;
        }
        Operation::Copy6 => {
            core::copy_crystal_last6(&mut host, 0x9000);
            return 0;
        }
        Operation::LoadCurrent => {
            core::load_suit(&mut host, PaletteBuffer::Current);
            return 0;
        }
        Operation::LoadTarget => {
            core::load_suit(&mut host, PaletteBuffer::Target);
            return 0;
        }
        Operation::Cancel => {
            core::cancel_speed_boost(&mut host);
            return 0;
        }
        Operation::Beam => {
            let index = host.session().inventory.beams.active & 0xfff;
            core::write_beam_palette(&mut host, index);
            return 0;
        }
    };
    u16::from(carry == PaletteCarry::Set)
}
