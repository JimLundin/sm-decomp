use super::*;
use crate::rom::Rom;

/// A file offset in a `LoROM` image.
fn file(bank: u8, offset: u16) -> usize {
    (usize::from(bank & 0x7f) << 15) | usize::from(offset & 0x7fff)
}

fn put(image: &mut [u8], bank: u8, offset: u16, values: &[u16]) {
    let bytes: Vec<u8> = values.iter().flat_map(|word| word.to_le_bytes()).collect();
    image[file(bank, offset)..][..bytes.len()].copy_from_slice(&bytes);
}

fn at(offset: u16) -> RomAddress {
    RomAddress::new(0xa8, offset)
}

#[test]
fn a_list_entered_mid_way_is_one_program_with_one_place_per_step() {
    let mut image = vec![0; 0x15_0000];
    // $9000: show $9210 for 8, show $9217 for 5, sleep; $9010 goes into the first's second.
    put(&mut image, 0xa8, 0x9000, &[8, 0x9210, 5, 0x9217, SLEEP]);
    put(&mut image, 0xa8, 0x9010, &[SET_TIMER, 3, GOTO, 0x9004]);
    let programs = EnemyPrograms::load(
        Lorom::new(&image),
        &[at(0x9004), at(0x9010), at(0x9000)],
        &[],
    )
    .unwrap();
    assert_eq!(programs.len(), 2);
    let first = Target {
        program: ProgramId(0),
        step: 1,
    };
    assert_eq!(programs.target(at(0x9004)), Some(first));
    assert_eq!(programs.address(first), Some(at(0x9004)));
    let steps: Vec<Instruction> = programs.program(ProgramId(1)).unwrap().steps().collect();
    assert_eq!(steps, [Instruction::SetTimer(3), Instruction::Goto(first)]);
    let shown = programs.spritemap_id(Spritemap(at(0x9217))).unwrap();
    assert_eq!(
        programs.step(first),
        Some(Instruction::Show {
            delay: 5,
            spritemap: shown
        })
    );
    assert_eq!(programs.spritemap(shown), Some(Spritemap(at(0x9217))));
}

#[test]
fn steps_that_never_stop_and_unknown_routines_do_not_load() {
    let mut image = vec![0; 0x15_0000];
    put(&mut image, 0xa8, 0x9000, &[SET_TIMER, 1, GOTO, 0x9000]);
    assert_eq!(
        EnemyPrograms::load(Lorom::new(&image), &[at(0x9000)], &[]),
        Err(ProgramError::Spins(at(0x9000)))
    );
    put(&mut image, 0xa8, 0x9100, &[0x8999]);
    assert_eq!(
        EnemyPrograms::load(Lorom::new(&image), &[at(0x9100)], &[]),
        Err(ProgramError::UnknownInstruction(at(0x9100), 0x8999))
    );
    put(&mut image, 0xa8, 0x9200, &[CALL, 0x9300]);
    assert_eq!(
        EnemyPrograms::load(Lorom::new(&image), &[at(0x9200)], &[]),
        Err(ProgramError::UnknownFunction(at(0x9200)))
    );
}

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

#[test]
fn slice_zeros_lists_decode_as_the_bank_logs_say() {
    let Some(cartridge) = rom() else {
        return;
    };
    let programs = EnemyPrograms::slice_zero(cartridge.lorom()).unwrap();
    // The elevator's: two frames of each of its two spritemaps, round (`$A3:94D6`).
    let elevator = programs.target(RomAddress::new(0xa3, 0x94d6)).unwrap();
    assert_eq!(elevator.step, 0);
    let steps: Vec<Instruction> = programs
        .program(elevator.program)
        .unwrap()
        .steps()
        .collect();
    let spritemap = |offset| {
        programs
            .spritemap_id(Spritemap(RomAddress::new(0xa3, offset)))
            .unwrap()
    };
    assert_eq!(
        steps,
        [
            Instruction::Show {
                delay: 2,
                spritemap: spritemap(0x962f)
            },
            Instruction::Show {
                delay: 2,
                spritemap: spritemap(0x9645)
            },
            Instruction::Goto(elevator),
        ]
    );
    // The eye's closed list facing right is the tail of its deactivating one.
    let deactivating = programs.target(at(0x8ff0)).unwrap();
    let closed = programs.target(at(0x8ffc)).unwrap();
    assert_eq!(closed.program, deactivating.program);
    assert_eq!(closed.step, 3);
    assert_eq!(
        programs.step(Target { step: 4, ..closed }),
        Some(Instruction::Sleep)
    );
    // The face block's forward list: one frame, then sleep.
    let forward = programs.target(at(0xe828)).unwrap();
    assert_eq!(
        programs.step(Target {
            step: forward.step.wrapping_add(1),
            ..forward
        }),
        Some(Instruction::Sleep)
    );
    // Every list starts at a place, and every place round-trips.
    for list in SLICE_ZERO_LISTS {
        let target = programs.target(list).unwrap();
        assert_eq!(programs.address(target), Some(list));
    }
    for spritemap in SLICE_ZERO_SPRITEMAPS {
        assert!(programs.spritemap_id(spritemap).is_some());
    }
}

#[test]
fn the_eyes_lists_resolve_as_the_bank_log_says() {
    let Some(cartridge) = rom() else {
        return;
    };
    let programs = EnemyPrograms::slice_zero(cartridge.lorom()).unwrap();
    let lists = crate::enemy::eye::EyeLists::resolve(&programs).unwrap();
    // The active frames are one list's first sixteen steps, each ten frames, then back.
    let [first, ..] = lists.active;
    for (frame, step) in lists.active.iter().zip(0_u16..) {
        assert_eq!(*frame, Target { step, ..first });
        assert!(matches!(
            programs.step(*frame),
            Some(Instruction::Show { delay: 10, .. })
        ));
    }
    assert_eq!(
        programs.step(Target { step: 16, ..first }),
        Some(Instruction::Goto(first))
    );
    // Closing falls through into closed: facing right, closed is deactivating's fourth step.
    assert_eq!(
        lists.closed.right,
        Target {
            step: 3,
            ..lists.deactivating.right
        }
    );
    // Opening ends asleep, after four frames.
    assert_eq!(
        programs.step(Target {
            step: 4,
            ..lists.activating.left
        }),
        Some(Instruction::Sleep)
    );
}
