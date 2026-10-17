use super::*;

// Canonical offsets are independent of the implementation's compact word slots.
const LX: usize = 0x911;
const LXF: usize = 0x90f;
const LY: usize = 0x915;
const LYF: usize = 0x913;
const X: usize = 0xaf6;
const XF: usize = 0xaf8;
const Y: usize = 0xafa;
const YF: usize = 0xafc;
const PX: usize = 0xb10;
const PXF: usize = 0xb12;
const PY: usize = 0xb14;
const PYF: usize = 0xb16;
const DX: usize = 0xda2;
const DXF: usize = 0xda4;
const DY: usize = 0xda6;
const DYF: usize = 0xda8;

#[derive(Default)]
struct Probe {
    events: Vec<Observation>,
    stack: Vec<Routine>,
    fail_at: Option<usize>,
}

impl Observer for Probe {
    fn observe(&mut self, event: Observation) -> Result<(), String> {
        self.events.push(event);
        if self.fail_at == Some(self.events.len()) {
            return Err("authored observer failure".into());
        }
        match event {
            Observation::Enter { routine, operand } => {
                assert!(!self.stack.contains(&routine), "recursive camera routine");
                self.stack.push(routine);
                assert!(self.stack.len() <= 3);
                if routine != Routine::CallFinishedHook {
                    assert_eq!(operand, 0);
                }
            }
            Observation::Return { routine } => assert_eq!(self.stack.pop(), Some(routine)),
        }
        Ok(())
    }
}

fn put(ram: &mut [u8], fields: &[(usize, u16)]) {
    for &(at, value) in fields {
        ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}

fn check(ram: &[u8], fields: &[(usize, u16)]) {
    for &(at, expected) in fields {
        assert_eq!(word(ram, at), expected, "WRAM {at:04x}");
    }
}

fn fixture() -> Vec<u8> {
    let mut ram = vec![0; WRAM_SIZE];
    put(
        &mut ram,
        &[
            (0x7a9, 16),
            (0x7ab, 16),
            (0x7ad, 112),
            (0x7af, 144),
            (0xa1e, 4),
            (LX, 256),
            (LY, 256),
            (X, 416),
            (Y, 368),
            (PX, 416),
            (PY, 368),
            (DX, 1),
            (DY, 1),
            (0x1fffe, 0x5aa5),
        ],
    );
    ram[0xcd20..0xce20].fill(2);
    ram
}

fn success(ram: &mut [u8]) -> Probe {
    let before = ram.to_vec();
    let mut probe = Probe::default();
    assert_eq!(run(ram, &mut probe).unwrap(), Outcome::Handled);
    assert!(probe.stack.is_empty());
    assert!(probe.events.len() <= 18);
    // The contract publishes only fourteen words, preserving all other state.
    let outputs = [LXF, LX, LYF, LY, 0xb0a, 0xb0e, PX, PXF, PY, PYF, DX, DXF, DY, DYF];
    for (at, (&old, &new)) in before.iter().zip(ram.iter()).enumerate() {
        if !outputs.iter().any(|&start| at == start || at == start + 1) {
            assert_eq!(new, old, "unexpected write at {at:05x}");
        }
    }
    probe
}

fn entered(routine: Routine) -> Observation {
    Observation::Enter { routine, operand: 0 }
}

fn returned(routine: Routine) -> Observation {
    Observation::Return { routine }
}

#[test]
fn frame_preserves_fractional_distance_and_exact_nested_order() {
    let mut ram = fixture();
    put(&mut ram, &[(X, 420), (XF, 0x1000), (PXF, 0xf000), (Y, 370), (YF, 0x8000)]);
    let probe = success(&mut ram);
    check(
        &ram,
        &[
            (DX, 4),
            (DXF, 0x2000),
            (DY, 3),
            (DYF, 0x8000),
            (LX, 260),
            (LXF, 0x2000),
            (LY, 258),
            (LYF, 0),
            (PX, 420),
            (PXF, 0x1000),
            (PY, 370),
            (PYF, 0x8000),
        ],
    );
    assert_eq!(
        probe.events,
        [
            entered(Routine::MainScrolling),
            entered(Routine::DistanceX),
            returned(Routine::DistanceX),
            entered(Routine::HandleX),
            entered(Routine::TriggerRight),
            returned(Routine::TriggerRight),
            returned(Routine::HandleX),
            entered(Routine::DistanceY),
            returned(Routine::DistanceY),
            entered(Routine::HandleY),
            entered(Routine::TriggerDown),
            returned(Routine::TriggerDown),
            returned(Routine::HandleY),
            returned(Routine::MainScrolling),
        ]
    );
}

#[test]
fn negative_axis_movement_borrows_fraction_and_publishes_all_previous_words() {
    let mut ram = fixture();
    put(
        &mut ram,
        &[
            (X, 400),
            (XF, 0x1000),
            (PX, 401),
            (PXF, 0x9000),
            (LXF, 0x4000),
            (Y, 350),
            (YF, 0x1000),
            (PY, 351),
            (PYF, 0x9000),
            (LYF, 0x4000),
        ],
    );
    let probe = success(&mut ram);
    check(
        &ram,
        &[
            (DX, 2),
            (DXF, 0x8000),
            (DY, 2),
            (DYF, 0x8000),
            (LX, 253),
            (LXF, 0xc000),
            (LY, 253),
            (LYF, 0xc000),
            (PX, 400),
            (PXF, 0x1000),
            (PY, 350),
            (PYF, 0x1000),
        ],
    );
    assert!(probe.events.contains(&entered(Routine::TriggerLeft)));
    assert!(probe.events.contains(&entered(Routine::TriggerUp)));
}

#[test]
fn distance_direction_uses_signed_integer_difference_and_wraps_pairs() {
    let mut ram = fixture();
    put(
        &mut ram,
        &[(X, 0), (XF, 0x1000), (PX, 0xffff), (PXF, 0xf000), (Y, 0), (YF, 0x1000), (PY, 0xffff), (PYF, 0xf000)],
    );
    success(&mut ram);
    check(&ram, &[(DX, 1), (DXF, 0x2000), (DY, 1), (DYF, 0x2000), (LX, 254), (LXF, 0xe000), (LY, 254), (LYF, 0xe000)]);

    // The exactly-half-range integer difference selects the negative branch.
    // Exact ideal coordinates avoid unrelated constraints on this authored state.
    let mut ram = fixture();
    put(&mut ram, &[(X, 0x8000), (PX, 0), (Y, 0x8000), (PY, 0), (LX, 0x7f60), (LY, 0x7f90)]);
    let probe = success(&mut ram);
    check(&ram, &[(DX, 0x8001), (DXF, 0), (DY, 0x8001), (DYF, 0), (LX, 0x7f60), (LY, 0x7f90)]);
    assert_eq!(probe.events.len(), 10);
}

#[test]
fn facing_xor_reads_byte_fields_and_accepts_all_even_and_odd_table_indices() {
    for (pose_and_movement, knockback, acceleration, distances) in [
        (0x0004, 0, 0, [160, 80, 32, 224]),
        (0x0008, 0, 0, [96, 64, 32, 224]),
        (0x1004, 0, 0, [96, 64, 32, 224]),
        (0x1008, 0, 0, [160, 80, 32, 224]),
        (0x0004, 1, 0, [96, 64, 32, 224]),
        (0x0008, 1, 0, [160, 80, 32, 224]),
        (0x0004, 0, 1, [96, 64, 32, 224]),
        (0x0008, 0, 1, [160, 80, 32, 224]),
        (0x1104, 0, 2, [160, 80, 32, 224]),
    ] {
        for index in 0..8u16 {
            let mut ram = fixture();
            put(
                &mut ram,
                &[
                    (X, 600),
                    (PX, 599),
                    (0xa1e, pose_and_movement),
                    (0xa52, knockback),
                    (0xb4a, acceleration),
                    (0x941, index),
                ],
            );
            success(&mut ram);
            check(&ram, &[(0xb0a, 600 - distances[usize::from(index >> 1)]), (LX, 258)]);
        }
    }
}

#[test]
fn fraction_only_axes_skip_unused_invalid_table_and_keep_ideals() {
    let mut ram = fixture();
    put(&mut ram, &[(0x941, 0xffff), (XF, 0), (PXF, 0xc000), (YF, 0x4000), (0xb0a, 0xabcd), (0xb0e, 0x1234)]);
    let probe = success(&mut ram);
    check(
        &ram,
        &[
            (LX, 256),
            (LY, 256),
            (0xb0a, 0xabcd),
            (0xb0e, 0x1234),
            (DX, 0),
            (DXF, 0x4000),
            (DY, 1),
            (DYF, 0x4000),
            (PXF, 0),
            (PYF, 0x4000),
        ],
    );
    assert!(probe.events.contains(&entered(Routine::AutoX)));
    assert!(probe.events.contains(&entered(Routine::AutoY)));
}

#[test]
fn slow_windows_are_unsigned_and_keep_fraction_and_distance_words() {
    for (x, y, expected_x, expected_y) in
        [(351, 367, 253, 253), (352, 368, 256, 256), (415, 399, 256, 256), (416, 400, 259, 259), (255, 255, 253, 253)]
    {
        let mut ram = fixture();
        put(
            &mut ram,
            &[
                (0xcf8, 1),
                (0xa78, 1),
                (0x941, 0xffff),
                (X, x),
                (Y, y),
                (LXF, 0x1234),
                (LYF, 0xabcd),
                (DX, 99),
                (DXF, 0x123),
                (DY, 77),
                (DYF, 0x456),
            ],
        );
        let probe = success(&mut ram);
        check(
            &ram,
            &[
                (LX, expected_x),
                (LY, expected_y),
                (LXF, 0x1234),
                (LYF, 0xabcd),
                (DX, 99),
                (DXF, 0x123),
                (DY, 77),
                (DYF, 0x456),
                (PX, x),
                (PY, y),
            ],
        );
        assert_eq!(
            probe.events,
            [
                entered(Routine::MainScrolling),
                entered(Routine::AutoX),
                returned(Routine::AutoX),
                entered(Routine::AutoY),
                returned(Routine::AutoY),
                returned(Routine::MainScrolling),
            ]
        );
    }
    let mut ram = fixture();
    put(&mut ram, &[(0xcf8, 1), (0xa78, 1), (LX, 1), (LY, 1), (X, 0), (Y, 0)]);
    success(&mut ram);
    check(&ram, &[(LX, 0xfffe), (LY, 0xfffe)]);
}

#[test]
fn signed_high_slow_x_skips_both_manual_axes_but_y_only_skips_y() {
    for (x, y, expected_x, expected_y) in [(0x8000, 400, 256, 256), (500, 0x8000, 259, 256)] {
        let mut ram = fixture();
        put(&mut ram, &[(0xcf8, 1), (0xa78, 1), (X, x), (Y, y)]);
        success(&mut ram);
        check(&ram, &[(LX, expected_x), (LY, expected_y), (PX, x), (PY, y)]);
    }
}

#[test]
fn frozen_guards_only_autoscrollers_and_does_not_skip_normal_frame() {
    let mut ram = fixture();
    put(&mut ram, &[(0xa78, 1), (LX, 0xffff), (LY, 0xffff), (XF, 0x4321), (YF, 0x8765)]);
    success(&mut ram);
    check(
        &ram,
        &[(LX, 0xffff), (LY, 0xffff), (DX, 1), (DXF, 0x4321), (DY, 1), (DYF, 0x8765), (PXF, 0x4321), (PYF, 0x8765)],
    );

    let mut ram = fixture();
    put(&mut ram, &[(0xa78, 1), (X, 420), (Y, 370)]);
    success(&mut ram);
    check(&ram, &[(LX, 260), (LY, 258), (DX, 5), (DY, 3), (PX, 420), (PY, 370)]);
}

#[test]
fn finish_hook_runs_after_axes_clamps_integer_only_and_publishes_tail() {
    for (initial_y, expected_y) in [(463, 464), (464, 464), (465, 465)] {
        let mut ram = fixture();
        put(&mut ram, &[(0xa78, 1), (0x7e9, 0x9589), (LY, initial_y), (LYF, 0xbeef), (XF, 0x1234), (YF, 0x5678)]);
        let probe = success(&mut ram);
        check(&ram, &[(LY, expected_y), (LYF, 0xbeef), (PX, 416), (PXF, 0x1234), (PY, 368), (PYF, 0x5678)]);
        assert_eq!(
            &probe.events[probe.events.len() - 5..],
            &[
                Observation::Enter { routine: Routine::CallFinishedHook, operand: 0x9589 },
                entered(Routine::SporeSpawnFinish),
                returned(Routine::SporeSpawnFinish),
                returned(Routine::CallFinishedHook),
                returned(Routine::MainScrolling),
            ]
        );
    }
}

#[test]
fn y_constraints_use_the_column_published_by_x_in_the_same_frame() {
    let mut ram = fixture();
    put(&mut ram, &[(0x7a9, 4), (0x7ab, 4), (LX, 383), (LY, 384), (X, 546), (PX, 545), (Y, 496), (PY, 496)]);
    // X crosses the x+128 column boundary; only the new Y cell is red.
    ram[0xcd20 + 6] = 0;
    success(&mut ram);
    check(&ram, &[(LX, 385), (LY, 387)]);
}

#[test]
fn vertical_ideal_selects_down_scroller_only_for_direction_one() {
    for (direction, ideal) in [(0, 488), (1, 456), (2, 488), (0xffff, 488)] {
        let mut ram = fixture();
        put(&mut ram, &[(Y, 600), (PY, 599), (0xb36, direction)]);
        success(&mut ram);
        check(&ram, &[(0xb0e, ideal), (LY, 258), (DY, 2)]);
    }
}

#[test]
fn red_scroll_trigger_constraints_and_blue_downward_margin_are_preserved() {
    for (x, y, previous_x, previous_y, blocked, expected_x, expected_y, direction) in [
        (600, 368, 599, 368, 18, 298, 350, 0),
        (400, 368, 401, 368, 17, 302, 350, 0),
        (416, 600, 416, 599, 33, 300, 348, 0),
        (416, 350, 416, 351, 17, 300, 352, 0),
    ] {
        let mut ram = fixture();
        put(
            &mut ram,
            &[(LX, 300), (LY, 350), (0xa78, 1), (X, x), (Y, y), (PX, previous_x), (PY, previous_y), (0xb36, direction)],
        );
        ram[0xcd20 + blocked] = 0;
        success(&mut ram);
        check(&ram, &[(LX, expected_x), (LY, expected_y)]);
    }
    // Blue has no extra 31-pixel bottom margin; green does.
    for (color, expected) in [(1, 256), (2, 258)] {
        let mut ram = fixture();
        put(&mut ram, &[(Y, 380), (PY, 379)]);
        ram[0xcd20 + 17] = color;
        ram[0xcd20 + 33] = 0;
        success(&mut ram);
        check(&ram, &[(LY, expected)]);
    }
}

#[test]
fn stationary_autoscrollers_move_away_from_red_cells_in_both_axes() {
    for (red_cells, expected_x, expected_y) in [
        (&[18][..], 256, 256), // Right red cell clamps at the current X boundary.
        (&[17][..], 259, 259), // Current red cell pushes right and down.
        (&[33][..], 256, 256), // Below red cell within the green 31-pixel margin.
    ] {
        let mut ram = fixture();
        for &index in red_cells {
            ram[0xcd20 + index] = 0;
        }
        success(&mut ram);
        check(&ram, &[(LX, expected_x), (LY, expected_y)]);
    }
    let mut ram = fixture();
    put(&mut ram, &[(LX, 300), (LY, 350)]);
    ram[0xcd20 + 18] = 0;
    ram[0xcd20 + 33] = 0;
    success(&mut ram);
    check(&ram, &[(LX, 297), (LY, 347)]);
}

#[test]
fn unsupported_accesses_refuse_atomically_even_after_private_fraction_reset() {
    for writes in [
        vec![(0x7e9, 0x9999)],
        vec![(X, 420), (0x941, 8)],
        vec![(0x7a9, 0)],
        vec![(0x7ab, 17)],
        vec![(0x7a9, 256)],
        vec![(X, 420), (LY, 0xff00), (LXF, 0xffff)],
        // X clamps successfully at the room maximum without reading scrolls;
        // Y then accesses a row beyond the bounded allocation.
        vec![(LX, 3839), (X, 4010), (PX, 4009), (Y, 0x1070), (PY, 0x106f), (LY, 0x0ffe), (LYF, 0xffff)],
    ] {
        let mut ram = fixture();
        put(&mut ram, &writes);
        let before = ram.clone();
        let mut probe = Probe::default();
        assert_eq!(run(&mut ram, &mut probe).unwrap(), Outcome::Refused, "writes={writes:?}");
        assert_eq!(ram, before, "refusal changed full WRAM");
        assert!(probe.events.is_empty(), "refusal emitted partial observations");
    }
}

#[test]
fn admitted_dimensions_and_unused_scroll_indices_do_not_add_eager_guards() {
    for (width, height) in [(1, 1), (1, 255), (255, 1), (16, 16)] {
        let mut ram = fixture();
        put(&mut ram, &[(0x7a9, width), (0x7ab, height), (0xa78, 1), (LX, 0xffff), (LY, 0xffff)]);
        success(&mut ram);
    }
    // A stationary frozen axis does not read its invalid camera table/scrolls.
    let mut ram = fixture();
    put(&mut ram, &[(0xa78, 1), (0x941, 0xffff), (LY, 0xff00)]);
    success(&mut ram);
}

#[test]
fn observer_failures_leave_output_uncommitted_and_are_owned_errors() {
    for fail_at in [1, 7, 14] {
        let mut ram = fixture();
        put(&mut ram, &[(X, 420), (Y, 370)]);
        let before = ram.clone();
        let mut probe = Probe { fail_at: Some(fail_at), ..Probe::default() };
        assert_eq!(run(&mut ram, &mut probe).unwrap_err(), "authored observer failure");
        assert_eq!(probe.events.len(), fail_at);
        assert_eq!(ram, before);
    }
}

#[test]
fn invalid_extent_errors_before_observation_or_state_change() {
    for size in [0, 4096, WRAM_SIZE - 1, WRAM_SIZE + 1] {
        let mut ram = vec![0xa5; size];
        let mut probe = Probe::default();
        assert_eq!(run(&mut ram, &mut probe).unwrap_err(), "camera requires complete canonical WRAM");
        assert!(ram.iter().all(|&byte| byte == 0xa5));
        assert!(probe.events.is_empty());
    }
}

#[test]
fn root_executes_independently_of_visible_game_state() {
    for game_state in [8, 9, 11, 0xffff] {
        let mut ram = fixture();
        put(&mut ram, &[(0x998, game_state), (X, 420), (Y, 370)]);
        success(&mut ram);
        check(&ram, &[(0x998, game_state), (LX, 260), (LY, 258), (PX, 420), (PY, 370)]);
    }
}
