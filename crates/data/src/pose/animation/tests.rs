use super::*;
use crate::rom::Rom;

#[test]
fn a_looping_list_decodes_every_offset() {
    // $91:B222's shape: a delay, a delay, go back one byte.
    let list = DelayList::decode(&[0x02, 0x10, 0xFE, 0x01, 0x55]).unwrap();
    assert_eq!(list.at(AnimationFrame(0)), Some(DelayItem::Delay(2)));
    assert_eq!(list.at(AnimationFrame(1)), Some(DelayItem::Delay(0x10)));
    assert_eq!(
        list.at(AnimationFrame(2)),
        Some(DelayItem::Instruction(Instruction::GoBack(1)))
    );
    // The operand byte, read as the original would read it.
    assert_eq!(list.at(AnimationFrame(3)), Some(DelayItem::Delay(1)));
    assert_eq!(list.at(AnimationFrame(4)), None);
    assert_eq!(list.byte(AnimationFrame(2)), Some(0xFE));
}

#[test]
fn the_first_nybble_is_ignored_and_operands_are_read() {
    let list = DelayList::decode(&[0x03, 0x8D, 0x02]).unwrap();
    assert_eq!(
        list.at(AnimationFrame(1)),
        Some(DelayItem::Instruction(Instruction::Transition(PoseId(2))))
    );
    let morph = DelayList::decode(&[0x01, 0xF9, 0x02, 0x00, 0x1D, 0x31, 0x79, 0x7D]).unwrap();
    assert_eq!(
        morph.at(AnimationFrame(1)),
        Some(DelayItem::Instruction(Instruction::ByItemsAndSpeed {
            items: ItemBits(2),
            without: BySpeed {
                still: PoseId(0x1D),
                moving: PoseId(0x31)
            },
            with: BySpeed {
                still: PoseId(0x79),
                moving: PoseId(0x7D)
            },
        }))
    );
    assert_eq!(
        DelayList::decode(&[0xF2, 0xFF])
            .unwrap()
            .at(AnimationFrame(0)),
        Some(DelayItem::Instruction(Instruction::NoOp(0xF2)))
    );
}

#[test]
fn a_wall_jump_reaches_past_its_first_end_and_a_list_without_one_fails() {
    let mut bytes = vec![0xFB, 0x05, 0xFF];
    bytes.resize(0x20, 0xFF);
    let list = DelayList::decode(&bytes).unwrap();
    // Its screw attack sequence's first frame, $15 bytes on, is the last it decodes.
    assert_eq!(
        list.at(AnimationFrame(0x15)),
        Some(DelayItem::Instruction(Instruction::Loop))
    );
    assert_eq!(list.at(AnimationFrame(0x16)), None);
    assert_eq!(DelayList::decode(&[0x01; 0x101]), None);
    assert_eq!(DelayList::decode(&[0x01, 0xFD]), None);
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
fn the_delay_lists_decode_as_the_bank_log_says() {
    let Some(cartridge) = rom() else {
        return;
    };
    let poses = Poses::load(cartridge.lorom()).unwrap();
    let delays = AnimationDelays::load(cartridge.lorom(), &poses).unwrap();
    let delay = |pose: u16, frame: u16| delays.of(PoseId(pose))?.at(AnimationFrame(frame));
    // $91:B298, poses 1 and 2: four 10-frame breaths, health check, four 8-frame, back 4.
    assert_eq!(delay(2, 0), Some(DelayItem::Delay(0x0A)));
    assert_eq!(
        delay(2, 4),
        Some(DelayItem::Instruction(Instruction::LoopIfHealthy))
    );
    assert_eq!(
        delay(2, 9),
        Some(DelayItem::Instruction(Instruction::GoBack(4)))
    );
    // Pose $25, turning left to right on the ground: ends by "enable auto-jump hack and
    // transition to pose 2" (bank $90 log, `$90:8370`).
    let turn = delays.of(PoseId(0x25)).unwrap();
    let last = (0..0x10)
        .filter_map(|frame| turn.at(AnimationFrame(frame)))
        .find(|item| matches!(item, DelayItem::Instruction(_)));
    assert_eq!(
        last,
        Some(DelayItem::Instruction(Instruction::AutoJumpTransition(
            PoseId(2)
        )))
    );
    // $91:B5D3: ten 2-frame delays, then back to the start.
    assert_eq!(
        delays.running().at(AnimationFrame(9)),
        Some(DelayItem::Delay(2))
    );
    assert_eq!(
        delays.running().at(AnimationFrame(10)),
        Some(DelayItem::Instruction(Instruction::Loop))
    );
}

#[test]
fn a_spin_jumps_list_runs_on_to_its_wall_jump_frame() {
    let Some(cartridge) = rom() else {
        return;
    };
    let poses = Poses::load(cartridge.lorom()).unwrap();
    let delays = AnimationDelays::load(cartridge.lorom(), &poses).unwrap();
    let delay = |pose: u16, frame: u16| delays.of(PoseId(pose))?.at(AnimationFrame(frame));
    // $91:B384, the spin jump's poses $19 and $1A, and $91:B391, the space jump's $1B and
    // $1C: back 8 bytes at frame 9, then the wall jump eligible frame's 8-frame delay
    // after the go-back's operand, which the wall jump check steps to.
    for pose in [0x19, 0x1A, 0x1B, 0x1C] {
        assert_eq!(
            delay(pose, 9),
            Some(DelayItem::Instruction(Instruction::GoBack(8)))
        );
        assert_eq!(delay(pose, 0x0B), Some(DelayItem::Delay(8)));
    }
    // $91:B39E, the screw attack's $81 and $82: back $18 bytes at frame $19, then its wall
    // jump eligible frame $1B.
    for pose in [0x81, 0x82] {
        assert_eq!(
            delay(pose, 0x19),
            Some(DelayItem::Instruction(Instruction::GoBack(0x18)))
        );
        assert_eq!(delay(pose, 0x1B), Some(DelayItem::Delay(8)));
    }
    assert_eq!(reached(&poses, PoseId(0x1A)), Some(WALL_JUMP_FRAME));
    assert_eq!(
        reached(&poses, PoseId(0x81)),
        Some(SCREW_ATTACK_WALL_JUMP_FRAME)
    );
    assert_eq!(reached(&poses, PoseId(1)), None);
}

#[test]
fn a_list_decodes_through_the_frame_reached_past_its_end() {
    let bytes = [0x04, 0xFE, 0x01, 0x08, 0xFF, 0x00];
    let list = DelayList::decode(&bytes).unwrap();
    assert_eq!(list.len(), 3);
    let through = DelayList::decode_through(&bytes, Some(AnimationFrame(3))).unwrap();
    assert_eq!(through.at(AnimationFrame(3)), Some(DelayItem::Delay(8)));
    assert_eq!(through.byte(AnimationFrame(3)), Some(8));
}
