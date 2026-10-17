use crate::{Entry, Output, Terrain, execute, fixtures::*, memory::*};

struct Unavailable;
impl Terrain for Unavailable {
    fn available(&self) -> bool {
        false
    }
    fn move_vertical(&mut self, _: &mut [u8], _: i32) -> Result<i32, String> {
        panic!("unexpected terrain request")
    }
}

fn check(ram: &mut [u8], amount: i32) -> (u16, i32) {
    match execute(Entry::CheckSolidEnemy(amount), ram, &mut Unavailable).unwrap() {
        Some(Output::EnemyCollision { collision, amount }) => (collision, amount),
        other => panic!("unexpected checker output {other:?}"),
    }
}

#[test]
fn disabled_and_empty_lists_preserve_distinct_index_effects() {
    let mut ram = base();
    put(&mut ram, LIST, 3); // Disabled checking never reads an invalid list.
    let before = ram.clone();
    assert_eq!(check(&mut ram, i32::MIN), (0, i32::MIN));
    assert_eq!(ram, before);
    list(&mut ram, &[]);
    let mut expected = ram.clone();
    put(&mut expected, INDEX, 0);
    assert_eq!(check(&mut ram, -123), (0, -123));
    assert_eq!(ram, expected);
}

#[test]
fn eligibility_order_duplicates_and_last_slot_are_material() {
    let mut ram = base();
    put(&mut ram, DIRECTION, 3);
    enemy(&mut ram, 0x40, 40, 57, 0, 0); // Ignored despite intersecting.
    enemy(&mut ram, 0x7c0, 40, 58, 0, 1); // Frozen, no enemy_ptr needed.
    list(&mut ram, &[0x40, 0x40, 0x7c0]);
    assert_eq!(check(&mut ram, 0x40000), (0xffff, 0x20000));
    assert_eq!(word(&ram, INDEX), 0x7c0);
    assert_eq!(word(&ram, COLLIDING + 6), 0x7c0);
    enemy(&mut ram, 0x40, 40, 57, 0x8000, 0);
    list(&mut ram, &[0x7c0, 0x40]);
    assert_eq!(check(&mut ram, 0x40000), (0xffff, 0x20000), "first contact is not nearest contact");
    list(&mut ram, &[0x40, 0x7c0]);
    assert_eq!(check(&mut ram, 0x40000), (0xffff, 0x10000));
}

#[test]
fn exact_contact_clears_y_fraction_in_every_direction_and_preserves_other_bytes() {
    for direction in 0..4u16 {
        let mut ram = base();
        put(&mut ram, DIRECTION, direction | 0x80);
        put(&mut ram, YF, 0x3456);
        let coordinate = if direction & 1 == 0 { 24 } else { 56 };
        let (x, y) = if direction < 2 { (coordinate, 40) } else { (40, coordinate) };
        enemy(&mut ram, 0x40, x, y, 0x8000, 0);
        list(&mut ram, &[0x40]);
        let mut expected = ram.clone();
        put(&mut expected, YF, 0);
        put(&mut expected, INDEX, 0x40);
        put(&mut expected, COLLIDING + usize::from(direction) * 2, 0x40);
        assert_eq!(check(&mut ram, 0x20000), (0xffff, 0));
        assert_eq!(ram, expected, "direction{direction}");
    }
}

#[test]
fn promoted_zero_comparison_is_not_wrapped_zero_distance() {
    for case in body_fixtures().into_iter().filter(|case| case.name.contains("wrapped-zero-not-exact")) {
        let mut ram = case.wram;
        assert_eq!(check(&mut ram, 0x10000), (0xffff, 0), "{}", case.name);
        assert_eq!(word(&ram, YF), 0x1234, "{}", case.name);
    }
}

#[test]
fn fractional_rounding_changes_overlap_and_negative_separation_is_skipped() {
    for direction in [0u16, 2] {
        for (fraction, expected) in [(0, 0xffff), (0x1000, 0), (0x1001, 0)] {
            let mut ram = base();
            put(&mut ram, DIRECTION, direction);
            put(&mut ram, if direction == 0 { XF } else { YF }, fraction);
            let (x, y) = if direction == 0 { (23, 40) } else { (40, 23) };
            enemy(&mut ram, 0x40, x, y, 0x8000, 0);
            list(&mut ram, &[0x40]);
            assert_eq!(check(&mut ram, 0x1000).0, expected);
        }
    }
    for case in body_fixtures().into_iter().filter(|case| case.name.contains("frozen-gap-1")) {
        let mut ram = case.wram;
        assert_eq!(check(&mut ram, 0x20000), (0, 0x20000), "{}", case.name);
        assert_eq!(word(&ram, INDEX), 0x7c0);
        assert_eq!(word(&ram, YF), 0x1234);
    }
}

#[test]
fn abs8000_keeps_unsigned_magnitude_and_touch_is_not_overlap() {
    for case in body_fixtures().into_iter().filter(|case| case.name.contains("abs8000")) {
        let mut ram = case.wram;
        let expected = if case.name.ends_with("overlap") { (0xffff, 0) } else { (0, 0x10000) };
        assert_eq!(check(&mut ram, 0x10000), expected, "{}", case.name);
    }
}

#[test]
fn malformed_full_list_refuses_even_when_an_earlier_slot_would_collide() {
    for defect in ["misaligned", "outside-pool", "unterminated", "truncated", "oversized"] {
        let mut ram = base();
        enemy(&mut ram, 0x40, 40, 56, 0x8000, 0);
        list(&mut ram, &[0x40]);
        match defect {
            "misaligned" => put(&mut ram, LIST + 2, 3),
            "outside-pool" => put(&mut ram, LIST + 2, 0x800),
            "unterminated" => {
                for n in 0..32 {
                    put(&mut ram, LIST + 2 * n, 0x40);
                }
            }
            "truncated" => ram.truncate(SIZE - 1),
            "oversized" => ram.push(0),
            _ => unreachable!(),
        }
        for entry in [Entry::CheckSolidEnemy(0x20000), Entry::MoveUp(-0x20000), Entry::MoveDown(0x20000)] {
            let before = ram.clone();
            assert_eq!(execute(entry, &mut ram, &mut Unavailable).unwrap(), None, "{defect}");
            assert_eq!(ram, before, "{defect}");
        }
    }
}

#[test]
fn final_list_word_terminates_and_never_spills_into_collision_indexes() {
    let mut ram = base();
    enemy(&mut ram, 0x40, 40, 40, 0, 0);
    list(&mut ram, &[0x40; 31]);
    let mut expected = ram.clone();
    put(&mut expected, INDEX, 0x40);
    assert_eq!(check(&mut ram, 0x10000), (0, 0x10000));
    assert_eq!(ram, expected);
}

#[test]
fn no_collision_fractional_wrap_and_projectile_displacement_preserve_full_ram() {
    for (entry, position, fraction, new_position, new_fraction, displacement) in [
        (Entry::MoveUpNoColl(1), 0, 0, 0xffff, 0xffff, -1),
        (Entry::MoveDownNoColl(1), 0xffff, 0xffff, 0, 0, 1),
        (Entry::MoveUpNoColl(-0x18000), 40, 0x9000, 42, 0x1000, 0x18000),
        (Entry::MoveDownNoColl(-0x18000), 40, 0x1000, 38, 0x9000, -0x18000),
        (Entry::MoveDownNoColl(i32::MIN), 0, 0, 0x8000, 0, i32::MIN),
    ] {
        let mut ram = base();
        put(&mut ram, Y, position);
        put(&mut ram, YF, fraction);
        let mut expected = ram.clone();
        put(&mut expected, Y, new_position);
        put(&mut expected, YF, new_fraction);
        let at = if matches!(entry, Entry::MoveUpNoColl(_)) { UP } else { DOWN };
        put(&mut expected, at, (displacement >> 16) as u16);
        put(&mut expected, at + 2, displacement as u16);
        assert_eq!(execute(entry, &mut ram, &mut Unavailable).unwrap(), Some(Output::Done));
        assert_eq!(ram, expected);
    }
    for entry in [Entry::MoveUp(i32::MIN), Entry::MoveUpNoColl(i32::MIN)] {
        let mut ram = base();
        let before = ram.clone();
        assert_eq!(execute(entry, &mut ram, &mut Unavailable).unwrap(), None);
        assert_eq!(ram, before);
    }
}

#[test]
fn all_feedback_entries_preserve_facing_alias_and_low5_continuation() {
    // Independent full output words transcribed from the two pinned C tables.
    let falling = [
        0x002, 0x002, 0x402, 0x402, 0x102, 0x002, 0x402, 0x202, 0x402, 0x402, 0x002, 0x002, 0x002, 0x002, 0x402, 0x402,
        0x002, 0x302, 0x402, 0x402, 0x402, 0x002, 0x402, 0x402, 0x402, 0x402, 0x402, 0x402,
    ];
    let landing = [
        0x401, 0x401, 0x001, 0x001, 0x401, 0x401, 0x001, 0x401, 0x101, 0x201, 0x001, 0x401, 0x401, 0x001, 0x401, 0x401,
        0x401, 0x401, 0x301, 0x301, 0x001, 0x401, 0x401, 0x401, 0x401, 0x001, 0x401, 0x401,
    ];
    for mode in 0..28 {
        for collision in [0, 1, 0xffff] {
            let mut ram = base();
            ram[MOVEMENT] = mode as u8;
            put(&mut ram, COLLISION, collision);
            let mut expected = ram.clone();
            put(&mut expected, FEEDBACK, if collision == 0 { falling[mode] } else { landing[mode] });
            assert_eq!(execute(Entry::DownFeedback, &mut ram, &mut Unavailable).unwrap(), Some(Output::Done));
            assert_eq!(ram, expected);
        }
    }
    let mut ram = base();
    put(&mut ram, FEEDBACK, 0xab05);
    put(&mut ram, COLLISION, 0);
    ram[MOVEMENT] = 0xff;
    let before = ram.clone();
    assert_eq!(execute(Entry::DownFeedback, &mut ram, &mut Unavailable).unwrap(), Some(Output::Done));
    assert_eq!(ram, before, "unreachable feedback table does not require an index");
    put(&mut ram, COLLISION, 1);
    let before = ram.clone();
    assert_eq!(execute(Entry::DownFeedback, &mut ram, &mut Unavailable).unwrap(), None);
    assert_eq!(ram, before);
    for collision in [0, 1, 0xffff] {
        put(&mut ram, COLLISION, collision);
        let mut expected = ram.clone();
        put(&mut expected, FEEDBACK, if collision == 0 { 0 } else { 4 });
        assert_eq!(execute(Entry::UpFeedback, &mut ram, &mut Unavailable).unwrap(), Some(Output::Done));
        assert_eq!(ram, expected);
    }
}

struct Copyback {
    direction: u16,
    amount: i32,
    collision: u16,
    calls: usize,
    fail: bool,
}
impl Terrain for Copyback {
    fn available(&self) -> bool {
        true
    }
    fn move_vertical(&mut self, ram: &mut [u8], amount: i32) -> Result<i32, String> {
        self.calls += 1;
        assert_eq!(ram.len(), SIZE);
        assert_eq!(amount, self.amount);
        assert_eq!(word(ram, DIRECTION), self.direction);
        assert_eq!(word(ram, COLLISION), 0);
        assert_eq!(word(ram, FEEDBACK), 0xa55a, "feedback must follow the synchronous service");
        put(ram, COLLISION, self.collision);
        put(ram, Y, 77);
        put(ram, YF, 0x1234);
        put(ram, 0x1c37, 0xd044); // Model a legitimate PLM field copyback.
        ram[MOVEMENT] = 9;
        if self.fail { Err("accepted terrain failure".into()) } else { Ok(-0x12345) }
    }
}

#[test]
fn composed_roots_observe_synchronous_copyback_before_displacement_and_feedback() {
    for upward in [false, true] {
        for collision in [0, 1] {
            let mut ram = base();
            let mut terrain =
                Copyback { direction: if upward { 2 } else { 3 }, amount: -0x18000, collision, calls: 0, fail: false };
            let entry = if upward { Entry::MoveUp(-0x18000) } else { Entry::MoveDown(-0x18000) };
            let mut expected = ram.clone();
            put(&mut expected, DIRECTION, terrain.direction);
            put(&mut expected, COLLISION, collision);
            put(&mut expected, Y, 77);
            put(&mut expected, YF, 0x1234);
            put(&mut expected, 0x1c37, 0xd044);
            expected[MOVEMENT] = 9;
            let displacement = if upward { UP } else { DOWN };
            put(&mut expected, displacement, 0xfffe);
            put(&mut expected, displacement + 2, 0xdcbb);
            put(
                &mut expected,
                FEEDBACK,
                if upward {
                    if collision == 0 { 0 } else { 4 }
                } else if collision == 0 {
                    0x402
                } else {
                    0x201
                },
            );
            assert_eq!(execute(entry, &mut ram, &mut terrain).unwrap(), Some(Output::Done));
            assert_eq!(terrain.calls, 1);
            assert_eq!(ram, expected);
        }
    }
}

#[test]
fn enemy_hit_completes_without_terrain_and_unavailable_no_hit_refuses_unchanged() {
    for upward in [false, true] {
        let mut ram = base();
        put(&mut ram, YF, 0x1234);
        enemy(&mut ram, 0x40, 40, if upward { 23 } else { 57 }, 0, 1);
        list(&mut ram, &[0x40]);
        let entry = if upward { Entry::MoveUp(-0x20000) } else { Entry::MoveDown(0x20000) };
        assert_eq!(execute(entry, &mut ram, &mut Unavailable).unwrap(), Some(Output::Done));
        assert_eq!(word(&ram, COLLISION), 0xffff);
        assert_eq!(word(&ram, Y), if upward { 39 } else { 41 });
        assert_eq!(word(&ram, YF), 0x1234);
        assert_eq!(word(&ram, FEEDBACK), if upward { 4 } else { 1 });
        assert_eq!(word(&ram, if upward { UP } else { DOWN }), if upward { 0xffff } else { 1 });
        let mut ram = base();
        let before = ram.clone();
        assert_eq!(execute(entry, &mut ram, &mut Unavailable).unwrap(), None);
        assert_eq!(ram, before);
    }
}

#[test]
fn invalid_reachable_feedback_refuses_before_terrain_and_service_error_is_owned_failure() {
    let mut ram = base();
    ram[MOVEMENT] = 28;
    let before = ram.clone();
    let mut terrain = Copyback { direction: 3, amount: 0x10000, collision: 1, calls: 0, fail: false };
    assert_eq!(execute(Entry::MoveDown(0x10000), &mut ram, &mut terrain).unwrap(), None);
    assert_eq!(ram, before);
    assert_eq!(terrain.calls, 0);
    ram = base();
    terrain.fail = true;
    assert_eq!(execute(Entry::MoveDown(0x10000), &mut ram, &mut terrain), Err("accepted terrain failure".into()));
    assert_eq!(terrain.calls, 1);
    assert_eq!(word(&ram, DIRECTION), 3);
    assert_eq!(word(&ram, 0x1c37), 0xd044);
    assert_eq!(word(&ram, FEEDBACK), 0xa55a, "no feedback after accepted failure");
}

#[test]
fn shared_body_matrix_is_finite_unique_and_all_entries_use_full_wram() {
    let cases = body_fixtures();
    assert_eq!(cases.len(), 192);
    let names: std::collections::BTreeSet<_> = cases.iter().map(|case| &case.name).collect();
    assert_eq!(names.len(), cases.len());
    for case in cases {
        assert_eq!(case.wram.len(), SIZE, "{}", case.name);
        assert!(!matches!(case.entry, Entry::MoveUp(i32::MIN) | Entry::MoveUpNoColl(i32::MIN)));
    }
}
