//! Actual candidate entry adapter; identical operations and fixture carrier. No SaveToSram call exists here.
use super::flags_cases::{self, Operation, Returned};
use crate::{
    game::{enemy_script::EnemyScript, gameplay_flag_runtime as flags},
    game_state::Game,
    rom::RomImage,
};
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    flags_cases::capture_case(id, run)
}
fn run(operation: Operation, image: &RomImage, g: &mut Game) -> Returned {
    unsafe {
        match operation {
            Operation::EventQuery(bit) => return Returned::word(flags::CheckEventHappened(bit, g)),
            Operation::EventSet(bit) => flags::SetEventHappened(bit, g),
            Operation::EventClear(bit) => flags::ClearEventHappened(bit, g),
            Operation::Prepare(bit) => return Returned::word(flags::PrepareBitAccess(bit, g)),
            Operation::BossQuery(mask) => {
                return Returned::word(u16::from(flags::CheckBossBitForCurArea(mask, g)));
            }
            Operation::BossSet(mask) => flags::SetBossBitForCurArea(mask, g),
            Operation::Plm(address) => {
                let script =
                    EnemyScript::at(image.clone(), crate::rtl::lorom_offset(0x849100) as usize);
                return match crate::game::room_mechanisms::checks::command(address, script, 0, g) {
                    EnemyScript::Stop => Returned { kind: 2, value: 0 },
                    EnemyScript::Jump(v) => Returned {
                        kind: 3,
                        value: u64::from(v),
                    },
                    EnemyScript::At { position, .. } => Returned {
                        kind: 4,
                        value: position as u64,
                    },
                };
            }
            Operation::Animated(address) => {
                return Returned::word(crate::game::animated_tiles::checks::command(
                    address, 0, 0x9100, g,
                ));
            }
            Operation::PalettePre => crate::game::palette_effects::checks::pre(9301701, 0, g),
            Operation::NewSave => crate::game::save_slots::initialize(g),
            Operation::Demo => crate::game::demo_rooms::load(g),
            Operation::DemoBoss => crate::game::demo_rooms::run_room_hook(0x828932, g),
            Operation::Load(slot) => {
                return Returned::word(u16::from(crate::game::save_slots::load(slot, g)));
            }
            Operation::SporeDeath => crate::game::sm_a5::SporeSpawn_Func_6(g),
        }
    }
    Returned { kind: 0, value: 0 }
}
