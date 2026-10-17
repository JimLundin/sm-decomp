use std::collections::VecDeque;

use super::{memory::*, *};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Service {
    Layer2X,
    Layer2Y,
    Blocks,
    Scroll,
}

#[derive(Debug)]
struct Call {
    service: Service,
    before: Vec<(usize, u16)>,
    writes: Vec<(usize, u16)>,
    result: u8,
    fail: bool,
}

fn call(service: Service, before: &[(usize, u16)], writes: &[(usize, u16)]) -> Call {
    Call { service, before: before.to_vec(), writes: writes.to_vec(), result: 0xfe, fail: false }
}

#[derive(Default)]
struct Probe {
    calls: VecDeque<Call>,
    services: Vec<(Service, u16)>,
    observations: Vec<Observation>,
    stack: Vec<Routine>,
    fail_observation: Option<usize>,
}

impl Probe {
    fn new(calls: impl IntoIterator<Item = Call>) -> Self {
        Self { calls: calls.into_iter().collect(), ..Self::default() }
    }

    fn service(&mut self, service: Service, ram: &mut [u8]) -> Result<u8> {
        assert_eq!(ram.len(), WRAM_SIZE);
        self.services.push((service, word(ram, COUNTER)));
        let expected = self.calls.pop_front().expect("unexpected service");
        assert_eq!(service, expected.service);
        check(ram, &expected.before);
        put(ram, &expected.writes);
        if expected.fail { Err("authored service failure".into()) } else { Ok(expected.result) }
    }

    fn finish(&self) {
        assert!(self.calls.is_empty(), "unconsumed service calls: {:?}", self.calls);
        assert!(self.stack.is_empty(), "unbalanced routine observations");
    }
}

impl Host for Probe {
    fn calculate_layer2_x(&mut self, ram: &mut [u8]) -> Result<u8> {
        self.service(Service::Layer2X, ram)
    }

    fn calculate_layer2_y(&mut self, ram: &mut [u8]) -> Result<u8> {
        self.service(Service::Layer2Y, ram)
    }

    fn calculate_blocks(&mut self, ram: &mut [u8]) -> Result<()> {
        self.service(Service::Blocks, ram).map(|_| ())
    }

    fn update_scroll_vars(&mut self, ram: &mut [u8]) -> Result<()> {
        self.service(Service::Scroll, ram).map(|_| ())
    }

    fn observe(&mut self, observation: Observation) -> Result<()> {
        self.observations.push(observation);
        if self.fail_observation == Some(self.observations.len()) {
            return Err("authored observation failure".into());
        }
        match observation {
            Observation::Enter { routine, .. } => {
                assert!(!self.stack.contains(&routine), "recursive routine");
                self.stack.push(routine);
                assert!(self.stack.len() <= 4, "unexpected private nesting depth");
            }
            Observation::Return { routine, result, .. } => {
                assert_eq!(self.stack.pop(), Some(routine));
                if (routine as u32) < Routine::MoveRight as u32 {
                    assert_eq!(result, 0, "void routine returned movement completion");
                }
            }
        }
        Ok(())
    }
}

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}

fn put(ram: &mut [u8], fields: &[(usize, u16)]) {
    for &(at, value) in fields {
        ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}

fn check(ram: &[u8], fields: &[(usize, u16)]) {
    for &(at, value) in fields {
        assert_eq!(word(ram, at), value, "WRAM {at:04x}");
    }
}

fn ram(fields: &[(usize, u16)]) -> Vec<u8> {
    let mut ram = vec![0; WRAM_SIZE];
    put(&mut ram, fields);
    ram
}

fn run_ok(entry: Entry, ram: &mut [u8], host: &mut Probe) {
    assert_eq!(run(entry, ram, host).unwrap(), Outcome::Handled);
    host.finish();
}

fn enter(routine: Routine, counter: u16) -> Observation {
    Observation::Enter { routine, counter }
}

fn returned(routine: Routine, result: u8, counter: u16) -> Observation {
    Observation::Return { routine, result, counter }
}

#[test]
fn horizontal_setups_preserve_completed_flag_and_execute_first_movement() {
    for direction in [0, 1] {
        let (shift_x, shift_l2, final_x, final_l2, prev1, prev2, samus, setup, movement) = if direction == 0 {
            (0xff20, 0xff80, 0xff24, 0xff84, 0xffff, 0xfffe, 0x102, Routine::SetupRight, Routine::MoveRight)
        } else {
            (0x120, 0x180, 0x11c, 0x17c, 1, 0, 0xff, Routine::SetupLeft, Routine::MoveLeft)
        };
        let mut ram = ram(&[
            (DIRECTION, 0xfffc | direction),
            (DEST_X, 0x20),
            (DEST_Y, 0xfff0),
            (BG1_H, 0x1000),
            (BG1_V, 0x10),
            (BG2_H, 0x3000),
            (BG2_V, 0x20),
            (COMPLETION, 0x8123),
            (SAMUS_X, 0x100),
            (SAMUS_SUB_X, 0x9000),
            (SPEED, 1),
            (SUBSPEED, 0x8000),
        ]);
        let mut host = Probe::new([
            call(Service::Layer2X, &[(L1_X, 0x20), (L1_Y, 0xfff0)], &[(L2_X, 0x80)]),
            call(Service::Layer2Y, &[(L2_X, shift_l2), (L1_X, 0x20)], &[(L2_Y, 0x7000)]),
            call(
                Service::Blocks,
                &[
                    (L1_X, shift_x),
                    (L2_Y, 0x7000),
                    (BG1_X_OFFSET, 0x1000u16.wrapping_sub(shift_x)),
                    (BG1_Y_OFFSET, 0x20),
                    (BG2_X_OFFSET, 0x3000u16.wrapping_sub(shift_x)),
                    (BG2_Y_OFFSET, 0x30),
                ],
                &[(L1_X_BLOCK, 0), (L2_X_BLOCK, 0xffff), (L1_Y_BLOCK, 2), (L2_Y_BLOCK, 3)],
            ),
            call(
                Service::Scroll,
                &[
                    (COUNTER, 0),
                    (L1_X, final_x),
                    (L2_X, final_l2),
                    (PREV_L1_X_BLOCK, prev1),
                    (PREV_L2_X_BLOCK, prev2),
                    (PREV_L1_Y_BLOCK, 2),
                    (PREV_L2_Y_BLOCK, 3),
                    (SAMUS_X, samus),
                    (SAMUS_PREV_X, samus),
                    (SAMUS_SUB_X, 0x1000),
                ],
                &[(COUNTER, 0x7777), (0x1ffe0, 0x4321)],
            ),
        ]);
        run_ok(Entry::ScrollingSetup, &mut ram, &mut host);
        check(&ram, &[(COUNTER, 1), (COMPLETION, 0x8123), (0x1ffe0, 0x4321), (L1_Y, 0xfff0)]);
        assert_eq!(
            host.observations,
            [
                enter(Routine::ScrollingSetup, 0),
                enter(setup, 0),
                enter(Routine::UpdateBgOffsets, 0),
                returned(Routine::UpdateBgOffsets, 0, 0),
                enter(Routine::UpdatePreviousBlocks, 0),
                returned(Routine::UpdatePreviousBlocks, 0, 0),
                enter(movement, 0),
                returned(movement, 0, 1),
                returned(setup, 0, 1),
                returned(Routine::ScrollingSetup, 0, 1),
            ]
        );
    }
}

#[test]
fn downward_setup_performs_transient_first_movement_and_keeps_child_copyback() {
    let mut ram = ram(&[
        (DIRECTION, 2),
        (DEST_X, 0x120),
        (DEST_Y, 0x80),
        (BG1_V, 0x30),
        (BG2_V, 0x40),
        (SAMUS_Y, 0x1234),
        (SAMUS_SUB_Y, 0x5678),
    ]);
    let mut host = Probe::new([
        call(Service::Layer2X, &[(L1_X, 0x120), (L1_Y, 0x80)], &[(L2_X, 0x220)]),
        call(Service::Layer2Y, &[(L1_Y, 0x80)], &[(L2_Y, 0x200)]),
        call(
            Service::Blocks,
            &[(L1_Y, 0xffa0), (L2_Y, 0x120), (BG1_Y_OFFSET, 0x90), (BG2_Y_OFFSET, 0xa0)],
            &[(L1_Y_BLOCK, 0), (L2_Y_BLOCK, 0xffff)],
        ),
        call(
            Service::Blocks,
            &[(L1_Y, 0xff91), (L2_Y, 0x111), (PREV_L1_Y_BLOCK, 0xffff), (PREV_L2_Y_BLOCK, 0xfffe)],
            &[(L1_X_BLOCK, 4), (L2_X_BLOCK, 5), (L1_Y_BLOCK, 0), (L2_Y_BLOCK, 0xffff)],
        ),
        call(
            Service::Scroll,
            &[
                (COUNTER, 0),
                (L1_Y, 0xff91),
                (L2_Y, 0x111),
                (PREV_L1_X_BLOCK, 4),
                (PREV_L2_X_BLOCK, 5),
                (PREV_L1_Y_BLOCK, 0xffff),
                (PREV_L2_Y_BLOCK, 0xfffe),
            ],
            &[
                (L1_Y, 0x1111),
                (L2_Y, 0x2222),
                (BG1_V, 0x3333),
                (BG2_V, 0x4444),
                (BG1_H, 0x5555),
                (BG2_H, 0x6666),
                (L1_X, 0x7777),
                (COUNTER, 0x8888),
                (PREV_L1_Y_BLOCK, 0x9999),
                (0x1ffe0, 0xabcd),
            ],
        ),
    ]);
    run_ok(Entry::ScrollingSetup, &mut ram, &mut host);
    check(
        &ram,
        &[
            (COUNTER, 1),
            (L1_Y, 0xffa0),
            (L2_Y, 0x120),
            (BG1_V, 0x30),
            (BG2_V, 0x40),
            (BG1_H, 0x5555),
            (BG2_H, 0x6666),
            (L1_X, 0x7777),
            (PREV_L1_Y_BLOCK, 0x9999),
            (0x1ffe0, 0xabcd),
            (SAMUS_Y, 0x1234),
            (SAMUS_SUB_Y, 0x5678),
        ],
    );
}

#[test]
fn upward_prepass_restores_only_saved_fields_and_preserves_counter_into_setup() {
    let mut ram = ram(&[
        (COUNTER, 0x4444),
        (DIRECTION, 3),
        (L1_Y, 8),
        (L2_Y, 10),
        (BG1_V, 0x100),
        (BG2_V, 0x200),
        (SAMUS_Y, 0x300),
        (SAMUS_SUB_Y, 0x1000),
        (SPEED, 1),
        (SUBSPEED, 0x2000),
        (DEST_X, 0x80),
        (DEST_Y, 0xfff0),
    ]);
    let mut host = Probe::new([
        call(Service::Blocks, &[(COUNTER, 0), (L1_Y, 8), (L2_Y, 10)], &[(L1_Y_BLOCK, 0xffff), (L2_Y_BLOCK, 2)]),
        call(
            Service::Blocks,
            &[(COUNTER, 0), (L1_Y, 0xfff8), (L2_Y, 0xfffa), (PREV_L1_Y_BLOCK, 0), (PREV_L2_Y_BLOCK, 3)],
            &[(L1_X_BLOCK, 7), (L2_X_BLOCK, 8), (L1_Y_BLOCK, 0xffff), (L2_Y_BLOCK, 0)],
        ),
        call(
            Service::Scroll,
            &[(L1_Y, 0xfff8), (L2_Y, 0xfffa), (PREV_L1_Y_BLOCK, 0), (PREV_L2_Y_BLOCK, 1)],
            &[
                (L1_Y, 0xaaaa),
                (L2_Y, 0xbbbb),
                (BG1_V, 0xcccc),
                (BG2_V, 0xdddd),
                (BG1_H, 0x400),
                (BG2_H, 0x800),
                (PREV_L2_Y_BLOCK, 0x9876),
                (COUNTER, 0xeeee),
                (0x1ffe0, 0xabcd),
            ],
        ),
    ]);
    run_ok(Entry::FixDoorsMovingUp, &mut ram, &mut host);
    check(
        &ram,
        &[
            (COUNTER, 1),
            (L1_Y, 8),
            (L2_Y, 10),
            (BG1_V, 0x100),
            (BG2_V, 0x200),
            (BG1_H, 0x400),
            (BG2_H, 0x800),
            (PREV_L2_Y_BLOCK, 0x9876),
            (0x1ffe0, 0xabcd),
            (SAMUS_Y, 0x300),
            (SAMUS_SUB_Y, 0x1000),
        ],
    );
    assert_eq!(host.observations.first(), Some(&enter(Routine::FixDoorsMovingUp, 0x4444)));
    assert_eq!(host.observations.last(), Some(&returned(Routine::FixDoorsMovingUp, 0, 1)));

    // The coordinator calls setup later without clearing the prepass counter.
    // X service may change layer1Y; Y service may then overwrite it, but the
    // source's saved local was captured after X and must survive Y copyback.
    let mut host = Probe::new([
        call(Service::Layer2X, &[(COUNTER, 1), (L1_X, 0x80), (L1_Y, 0xfff0)], &[(L1_Y, 0xffe0), (L2_X, 0x180)]),
        call(Service::Layer2Y, &[(COUNTER, 1), (L1_Y, 0xffff)], &[(L1_Y, 0x7777), (L2_Y, 0xff80)]),
        call(
            Service::Blocks,
            &[
                (COUNTER, 1),
                (L1_Y, 0xe0),
                (L2_Y, 0x60),
                (DEST_Y, 0x10),
                (BG1_X_OFFSET, 0x380),
                (BG2_X_OFFSET, 0x780),
                (BG1_Y_OFFSET, 0x20),
                (BG2_Y_OFFSET, 0x120),
            ],
            &[(L1_Y_BLOCK, 0xffff), (L2_Y_BLOCK, 0), (L1_X_BLOCK, 7), (L2_X_BLOCK, 8)],
        ),
    ]);
    run_ok(Entry::ScrollingSetup, &mut ram, &mut host);
    check(
        &ram,
        &[
            (COUNTER, 2),
            (L1_Y, 0xdb),
            (L2_Y, 0x5c),
            (DEST_Y, 0x10),
            (PREV_L1_Y_BLOCK, 0),
            (PREV_L2_Y_BLOCK, 1),
            (PREV_L1_X_BLOCK, 7),
            (PREV_L2_X_BLOCK, 8),
            (BG1_H, 0x400),
            (BG2_H, 0x900),
            (BG1_V, 0xfb),
            (BG2_V, 0x17c),
            (SAMUS_Y, 0x2fe),
            (SAMUS_SUB_Y, 0xf000),
            (SAMUS_PREV_Y, 0x2fe),
        ],
    );
    assert!(host.observations.contains(&enter(Routine::MoveUp, 1)));
}

#[test]
fn fixed_point_carry_borrow_and_layer_wrap_apply_in_each_direction() {
    for (direction, position, subposition, expected_position, expected_subposition, expected_layer) in [
        (0, 0xffff, 0x9000, 1, 0x1000, 2),
        (1, 0, 0x1000, 0xfffe, 0x9000, 0xfffd),
        (2, 0xffff, 0x9000, 1, 0x1000, 2),
        (3, 0, 0x1000, 0xfffe, 0x9000, 0xfffd),
    ] {
        let (pos, sub, prev, layer1, layer2) = if direction < 2 {
            (SAMUS_X, SAMUS_SUB_X, SAMUS_PREV_X, L1_X, L2_X)
        } else {
            (SAMUS_Y, SAMUS_SUB_Y, SAMUS_PREV_Y, L1_Y, L2_Y)
        };
        let initial_layer = if direction & 1 == 0 { 0xfffe } else { 1 };
        let mut ram = ram(&[
            (DIRECTION, direction),
            (COUNTER, 5),
            (pos, position),
            (sub, subposition),
            (layer1, initial_layer),
            (layer2, initial_layer),
            (SPEED, 1),
            (SUBSPEED, 0x8000),
            (prev + 2, 0x1357),
            (COMPLETION, 0x8004),
        ]);
        let mut host = Probe::new([call(
            Service::Scroll,
            &[
                (COUNTER, 5),
                (pos, expected_position),
                (sub, expected_subposition),
                (prev, expected_position),
                (layer1, expected_layer),
                (layer2, expected_layer),
            ],
            &[(COUNTER, 0xeeee)],
        )]);
        run_ok(Entry::FollowTransition, &mut ram, &mut host);
        check(&ram, &[(COUNTER, 6), (prev + 2, 0x1357), (COMPLETION, 0x8004)]);
    }
}

#[test]
fn upward_direct_shadows_switch_to_child_at_five() {
    let mut ram = ram(&[
        (DIRECTION, 3),
        (COUNTER, 4),
        (L1_X, 0xfff0),
        (L1_Y, 3),
        (L2_X, 0x200),
        (L2_Y, 0x300),
        (BG1_X_OFFSET, 0x20),
        (BG1_Y_OFFSET, 5),
        (BG2_X_OFFSET, 0xff00),
        (BG2_Y_OFFSET, 0xfd00),
    ]);
    run_ok(Entry::FollowTransition, &mut ram, &mut Probe::default());
    check(
        &ram,
        &[(COUNTER, 5), (L1_Y, 0xffff), (L2_Y, 0x2fc), (BG1_H, 0x10), (BG1_V, 4), (BG2_H, 0x100), (BG2_V, 0xfffc)],
    );
    let mut host = Probe::new([call(
        Service::Scroll,
        &[(COUNTER, 5), (L1_Y, 0xfffb), (L2_Y, 0x2f8), (BG1_H, 0x10), (BG1_V, 4), (BG2_H, 0x100), (BG2_V, 0xfffc)],
        &[(BG1_H, 0x1234)],
    )]);
    run_ok(Entry::FollowTransition, &mut ram, &mut host);
    check(&ram, &[(COUNTER, 6), (BG1_H, 0x1234)]);
}

#[test]
fn right_terminal_calls_are_separate_and_snap_only_layer1_after_second_copyback() {
    let mut ram = ram(&[
        (COUNTER, 63),
        (DIRECTION, 0),
        (L1_X, 0x100),
        (L1_Y, 0x200),
        (L2_X, 0x300),
        (L2_Y, 0x400),
        (DEST_X, 0x500),
        (DEST_Y, 0x600),
        (COMPLETION, 0x45),
    ]);
    let mut host = Probe::new([
        call(
            Service::Scroll,
            &[(COUNTER, 63), (L1_X, 0x104), (L2_X, 0x304)],
            &[(COUNTER, 0xfffe), (L1_X, 0x1111), (L2_X, 0x2222), (0x1ffe0, 0xabcd)],
        ),
        call(
            Service::Scroll,
            &[(COUNTER, 64), (L1_X, 0x1111), (L2_X, 0x2222), (0x1ffe0, 0xabcd)],
            &[
                (COUNTER, 0x1234),
                (L1_X, 0x3333),
                (L1_Y, 0x4444),
                (L2_X, 0x5555),
                (L2_Y, 0x6666),
                (BG1_H, 0x7777),
                (BG1_V, 0x8888),
                (BG2_H, 0x9999),
                (BG2_V, 0xaaaa),
                (DEST_X, 0xbbbb),
                (DEST_Y, 0xcccc),
                (COMPLETION, 0x2345),
            ],
        ),
    ]);
    run_ok(Entry::FollowTransition, &mut ram, &mut host);
    check(
        &ram,
        &[
            (COUNTER, 0x1234),
            (L1_X, 0xbbbb),
            (L1_Y, 0xcccc),
            (L2_X, 0x5555),
            (L2_Y, 0x6666),
            (BG1_H, 0x7777),
            (BG1_V, 0x8888),
            (BG2_H, 0x9999),
            (BG2_V, 0xaaaa),
            (COMPLETION, 0xa345),
        ],
    );
    assert_eq!(host.services, [(Service::Scroll, 63), (Service::Scroll, 64)]);
    assert_eq!(
        host.observations,
        [
            enter(Routine::FollowTransition, 63),
            enter(Routine::MoveRight, 63),
            returned(Routine::MoveRight, 1, 0x1234),
            returned(Routine::FollowTransition, 0, 0x1234),
        ]
    );
}

#[test]
fn terminal_equality_and_downward_inequality_are_distinct() {
    for (direction, initial_counter, children, done, moved) in [
        (0, 62, 1, false, true),
        (0, 63, 2, true, true),
        (0, 64, 1, false, true),
        (1, 62, 1, false, true),
        (1, 63, 1, true, true),
        (1, 64, 1, false, true),
        (2, 55, 1, false, true),
        (2, 56, 2, true, true),
        (2, 57, 1, true, false),
        (2, 0xfffe, 1, true, false),
        (2, 0xffff, 0, false, false),
        (3, 55, 1, false, true),
        (3, 56, 1, true, true),
        (3, 57, 1, false, true),
    ] {
        let mut ram = ram(&[
            (DIRECTION, direction),
            (COUNTER, initial_counter),
            (COMPLETION, 0x123),
            (DEST_X, 0x800),
            (DEST_Y, 0x900),
            (L1_X, 0x100),
            (L1_Y, 0x200),
            (L2_X, 0x300),
            (L2_Y, 0x400),
            (SAMUS_X, 0x500),
            (SAMUS_Y, 0x600),
            (SPEED, 1),
            (BG1_H, 0xaaa1),
            (BG1_V, 0xaaa2),
            (BG2_H, 0xaaa3),
            (BG2_V, 0xaaa4),
        ]);
        let mut host = Probe::new((0..children).map(|_| call(Service::Scroll, &[], &[])));
        run_ok(Entry::FollowTransition, &mut ram, &mut host);
        check(
            &ram,
            &[
                (COUNTER, initial_counter.wrapping_add(1)),
                (COMPLETION, if done { 0x8123 } else { 0x123 }),
                (BG1_H, 0xaaa1),
                (BG1_V, 0xaaa2),
                (BG2_H, 0xaaa3),
                (BG2_V, 0xaaa4),
            ],
        );
        if done {
            check(&ram, &[(L1_X, 0x800), (L1_Y, 0x900)]);
        }
        let delta = if !moved {
            0
        } else if direction & 1 == 0 {
            4
        } else {
            0xfffc
        };
        if direction < 2 {
            check(&ram, &[(L2_X, 0x300u16.wrapping_add(delta)), (L2_Y, 0x400)]);
        } else {
            check(&ram, &[(L2_X, 0x300), (L2_Y, 0x400u16.wrapping_add(delta))]);
        }
        let movement = match direction {
            0 => Routine::MoveRight,
            1 => Routine::MoveLeft,
            2 => Routine::MoveDown,
            _ => Routine::MoveUp,
        };
        assert!(host.observations.contains(&returned(movement, u8::from(done), initial_counter.wrapping_add(1))));
    }
}

#[test]
fn counter_wrap_keeps_old_local_even_when_child_overwrites_counter() {
    for direction in [0, 1, 3] {
        let mut ram = ram(&[(DIRECTION, direction), (COUNTER, 0xffff), (COMPLETION, 7)]);
        let mut host = Probe::new([call(Service::Scroll, &[(COUNTER, 0xffff)], &[(COUNTER, 56)])]);
        run_ok(Entry::FollowTransition, &mut ram, &mut host);
        check(&ram, &[(COUNTER, 0), (COMPLETION, 7)]);
    }
}

#[test]
fn down_terminal_second_child_observes_increment_then_owns_final_counter() {
    let mut ram = ram(&[(DIRECTION, 2), (COUNTER, 56)]);
    let mut host = Probe::new([
        call(Service::Scroll, &[(COUNTER, 56), (L1_Y, 4), (L2_Y, 4)], &[(COUNTER, 0xffff)]),
        call(Service::Scroll, &[(COUNTER, 57), (L1_Y, 4), (L2_Y, 4)], &[(COUNTER, 0x7777)]),
    ]);
    run_ok(Entry::FollowTransition, &mut ram, &mut host);
    check(&ram, &[(COUNTER, 0x7777), (COMPLETION, 0x8000)]);
    assert!(host.observations.contains(&returned(Routine::MoveDown, 1, 0x7777)));
}

#[test]
fn setup_ignores_private_movement_completion() {
    let mut ram = ram(&[(DIRECTION, 1), (COUNTER, 63), (DEST_X, 0x100), (DEST_Y, 0x200), (COMPLETION, 0x45)]);
    let mut host = Probe::new([
        call(Service::Layer2X, &[], &[(L2_X, 0x300)]),
        call(Service::Layer2Y, &[], &[(L2_Y, 0x400)]),
        call(Service::Blocks, &[], &[]),
        call(Service::Scroll, &[(COUNTER, 63)], &[]),
    ]);
    run_ok(Entry::ScrollingSetup, &mut ram, &mut host);
    check(&ram, &[(COUNTER, 64), (L1_X, 0x1fc), (L1_Y, 0x200), (COMPLETION, 0x45)]);
    assert!(host.observations.contains(&returned(Routine::MoveLeft, 1, 64)));
    assert_eq!(host.observations.last(), Some(&returned(Routine::ScrollingSetup, 0, 64)));
}

#[test]
fn invalid_wram_extent_fails_before_observation_or_mutation() {
    for entry in [Entry::FixDoorsMovingUp, Entry::ScrollingSetup, Entry::FollowTransition] {
        for size in [0, 0x1000, WRAM_SIZE - 1, WRAM_SIZE + 1] {
            let mut ram = vec![0xa5; size];
            let mut host = Probe::default();
            assert_eq!(run(entry, &mut ram, &mut host).unwrap_err(), "door scrolling requires complete canonical WRAM");
            assert!(ram.iter().all(|&b| b == 0xa5));
            assert!(host.observations.is_empty());
            assert!(host.services.is_empty());
        }
    }
}

#[test]
fn service_errors_are_owned_and_stop_before_counter_increment_or_retry() {
    for failing in [Service::Layer2X, Service::Layer2Y, Service::Blocks, Service::Scroll] {
        let sequence = [Service::Layer2X, Service::Layer2Y, Service::Blocks, Service::Scroll];
        let last = sequence.iter().position(|&s| s == failing).unwrap();
        let mut calls: Vec<_> = sequence[..=last].iter().map(|&s| call(s, &[], &[])).collect();
        calls.last_mut().unwrap().fail = true;
        calls.last_mut().unwrap().writes.push((0x1ffe0, 0xf00d));
        let mut host = Probe::new(calls);
        let mut ram = ram(&[(DEST_X, 0x1234), (DEST_Y, 0x5678), (COUNTER, 9)]);
        assert_eq!(run(Entry::ScrollingSetup, &mut ram, &mut host).unwrap_err(), "authored service failure");
        check(&ram, &[(COUNTER, 9), (0x1ffe0, 0xf00d)]);
        assert_eq!(host.services.len(), last + 1);
        assert!(host.calls.is_empty());
        assert!(
            !host
                .observations
                .iter()
                .any(|o| matches!(o, Observation::Return { routine: Routine::ScrollingSetup, .. }))
        );
    }
}

#[test]
fn passive_observation_failure_propagates_without_gameplay_retry() {
    let mut ram = ram(&[(L1_X, 0x1234), (DEST_X, 0x5678)]);
    let original = ram.clone();
    let mut host = Probe { fail_observation: Some(1), ..Probe::default() };
    assert_eq!(run(Entry::ScrollingSetup, &mut ram, &mut host).unwrap_err(), "authored observation failure");
    assert_eq!(ram, original);
    assert!(host.services.is_empty());

    let mut host = Probe { fail_observation: Some(2), ..Probe::default() };
    assert_eq!(run(Entry::ScrollingSetup, &mut ram, &mut host).unwrap_err(), "authored observation failure");
    check(&ram, &[(L1_X, 0x5678)]);
    assert!(host.services.is_empty());
}
