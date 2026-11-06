use data::enemy::program::{EnemyPrograms, Spritemap, Target};
use data::rom::{Lorom, RomAddress};

use super::{label, process};
use crate::state::room::{AiHandler, EnemyBody, EnemyCommon, EnemyExtraProperty, EnemyProperty};
use crate::state::slots::{Frames, Occupancy, Slot};

/// Words written into bank $A8 of a blank image.
fn programs(lists: &[(u16, &[u16])], entries: &[u16]) -> EnemyPrograms {
    let mut image = vec![0; 0x15_0000];
    for &(offset, words) in lists {
        let start = (0x28 << 15) | usize::from(offset & 0x7fff);
        for (at, word) in (start..).step_by(2).zip(words) {
            image[at..][..2].copy_from_slice(&word.to_le_bytes());
        }
    }
    let entries: Vec<RomAddress> = entries
        .iter()
        .map(|&offset| RomAddress::new(0xa8, offset))
        .collect();
    EnemyPrograms::load(Lorom::new(&image), &entries, &[]).unwrap()
}

fn place(programs: &EnemyPrograms, offset: u16) -> Target {
    programs.target(RomAddress::new(0xa8, offset)).unwrap()
}

fn slot_at(at: Target, delay: u16) -> Slot<EnemyCommon, EnemyBody> {
    let mut slot: Slot<EnemyCommon, EnemyBody> = Slot::default();
    slot.common.cursor.at = label(at);
    slot.common.cursor.delay = Frames(delay);
    slot.occupancy = Occupancy::Live(EnemyBody::FaceBlock {
        variables: crate::state::room::AiVariables::default(),
    });
    slot
}

/// The elevator's shape: two frames of each of two spritemaps, round.
const ROUND: &[u16] = &[2, 0x962f, 2, 0x9645, 0x80ed, 0x9000];

#[test]
fn the_timer_counts_down_and_the_graphic_is_not_updated_until_it_runs_out() {
    let programs = programs(&[(0x9000, ROUND)], &[0x9000]);
    let mut slot = slot_at(place(&programs, 0x9008), 2);
    slot.common
        .extra_properties
        .insert(EnemyExtraProperty::GraphicUpdated);
    process(&mut slot, &programs);
    assert_eq!(slot.common.cursor.delay, Frames(1));
    assert!(
        !slot
            .common
            .extra_properties
            .contains(EnemyExtraProperty::GraphicUpdated)
    );
    // Run out: the next frame shows, past a go-to back to the start.
    process(&mut slot, &programs);
    assert_eq!(slot.common.cursor.delay, Frames(2));
    assert_eq!(slot.common.cursor.at, label(place(&programs, 0x9004)));
    assert!(
        slot.common
            .extra_properties
            .contains(EnemyExtraProperty::GraphicUpdated)
    );
    let first = programs
        .spritemap_id(Spritemap(RomAddress::new(0xa8, 0x962f)))
        .unwrap();
    assert_eq!(slot.common.spritemap, first);
}

#[test]
fn after_a_sleep_the_timer_wraps_below_zero_and_the_list_stays() {
    // One frame, then sleep: the face block's forward list.
    let programs = programs(&[(0x9000, &[1, 0x92a5, 0x812f])], &[0x9000]);
    let sleep = place(&programs, 0x9004);
    let mut slot = slot_at(place(&programs, 0x9000), 1);
    process(&mut slot, &programs);
    assert_eq!(slot.common.cursor.at, label(sleep));
    process(&mut slot, &programs);
    // The sleep keeps the cursor on itself; the timer is 0 now and wraps next.
    assert_eq!(slot.common.cursor.at, label(sleep));
    assert_eq!(slot.common.cursor.delay, Frames(0));
    assert!(
        slot.common
            .extra_properties
            .contains(EnemyExtraProperty::GraphicUpdated)
    );
    process(&mut slot, &programs);
    assert_eq!(slot.common.cursor.delay, Frames(0xffff));
    assert!(
        !slot
            .common
            .extra_properties
            .contains(EnemyExtraProperty::GraphicUpdated)
    );
}

#[test]
fn timers_waits_off_screen_and_delete_run_as_the_common_instructions_say() {
    // Timer 2; enable off-screen; frame; decrement and go back to the frame; wait 7; delete.
    let list: &[u16] = &[
        0x8123, 2, 0x8173, 3, 0x9300, 0x8110, 0x9006, 0x813a, 7, 0x807c,
    ];
    let programs = programs(&[(0x9000, list)], &[0x9000]);
    let mut slot = slot_at(place(&programs, 0x9000), 1);
    process(&mut slot, &programs);
    assert_eq!(slot.common.timer, 2);
    assert!(
        slot.common
            .properties
            .contains(EnemyProperty::ProcessedOffScreen)
    );
    assert_eq!(slot.common.cursor.delay, Frames(3));
    slot.common.cursor.delay = Frames(1);
    // 2 - 1 is 1, not zero: back to the frame.
    process(&mut slot, &programs);
    assert_eq!(slot.common.timer, 1);
    assert_eq!(slot.common.cursor.at, label(place(&programs, 0x900a)));
    slot.common.cursor.delay = Frames(1);
    // 1 - 1 is zero: on to the wait.
    process(&mut slot, &programs);
    assert_eq!(slot.common.cursor.delay, Frames(7));
    assert_eq!(slot.common.cursor.at, label(place(&programs, 0x9012)));
    slot.common.cursor.delay = Frames(1);
    process(&mut slot, &programs);
    assert!(slot.common.properties.contains(EnemyProperty::Deleted));
    assert!(matches!(slot.occupancy, Occupancy::Deleting(_)));
    assert_eq!(slot.common.cursor.at, label(place(&programs, 0x9012)));
}

#[test]
fn a_frozen_enemys_list_waits() {
    let programs = programs(&[(0x9000, ROUND)], &[0x9000]);
    let mut slot = slot_at(place(&programs, 0x9000), 1);
    slot.common.ai_handlers.insert(AiHandler::Frozen);
    process(&mut slot, &programs);
    assert_eq!(slot.common.cursor.delay, Frames(1));
    assert_eq!(slot.common.cursor.at, label(place(&programs, 0x9000)));
}
