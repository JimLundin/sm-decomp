use crate::{Entry, Services, execute, fixtures::*, memory::*};

fn rom() -> Vec<u8> {
    let mut rom = vec![0; TABLE + TABLE_SIZE];
    // Independently transcribed rows from the frozen original-ROM slice.
    // Other rows are irrelevant to these unit expectations; native controls
    // use the complete bound runtime table rather than this model input.
    for (row, words) in [
        (0, [0, 0, 0, 0]),
        (1, [0, 0x1000, 0xffff, 0xf000]),
        (63, [3, 0xf000, 0xfffc, 0x1000]),
        (64, [4, 0, 0xfffc, 0]),
    ] {
        for (n, value) in words.into_iter().enumerate() {
            put(&mut rom, TABLE + row * 8 + n * 2, value);
        }
    }
    rom
}

struct NoServices;
impl Services for NoServices {
    fn available(&self) -> bool {
        panic!("initializer or rejected input must not consult service availability")
    }
    fn move_horizontal(&mut self, _: &mut [u8], _: u16, _: i32) -> Result<u8, String> {
        panic!("unexpected service request")
    }
}

struct Motion {
    available: bool,
    result: Result<u8, String>,
    writes: Vec<(usize, u16)>,
    calls: Vec<(u16, i32)>,
    input: Option<Vec<u8>>,
}
impl Motion {
    fn new(result: u8) -> Self {
        Self { available: true, result: Ok(result), writes: Vec::new(), calls: Vec::new(), input: None }
    }
}
impl Services for Motion {
    fn available(&self) -> bool {
        self.available
    }
    fn move_horizontal(&mut self, ram: &mut [u8], slot: u16, amount: i32) -> Result<u8, String> {
        self.calls.push((slot, amount));
        self.input = Some(ram.to_vec());
        for &(at, value) in &self.writes {
            put(ram, at, value);
        }
        self.result.clone()
    }
}

#[test]
fn all_init_variants_rows_and_direction_aliases_preserve_unowned_bytes() {
    let rom = rom();
    let cases = body_fixtures().into_iter().filter(|case| matches!(case.entry, Entry::JetInit | Entry::RipperInit));
    for mut case in cases {
        let b = ENEMIES + usize::from(word(&case.wram, CURRENT));
        let parameter = word(&case.wram, b + PARAMETER1);
        let row = match parameter {
            0 => 0,
            1 | 0x2001 => 1,
            63 => 63,
            64 => 64,
            _ => panic!("unlisted unit expectation"),
        };
        let right = word(&case.wram, b + PARAMETER2) != 0;
        let (integer, fraction) = match (row, right) {
            (0, _) => (0, 0),
            (1, true) => (0, 0x1000),
            (1, false) => (0xffff, 0xf000),
            (63, true) => (3, 0xf000),
            (63, false) => (0xfffc, 0x1000),
            (64, true) => (4, 0),
            (64, false) => (0xfffc, 0),
            _ => unreachable!(),
        };
        let instruction = match (case.entry, right) {
            (Entry::JetInit, false) => 0xe2e0,
            (Entry::JetInit, true) => 0xe2f4,
            (Entry::RipperInit, false) => 0xe48b,
            (Entry::RipperInit, true) => 0xe477,
            _ => unreachable!(),
        };
        let mut expected = case.wram.clone();
        for (offset, value) in [(E, row * 8), (D, integer), (C, fraction), (INSTRUCTION, instruction)] {
            put(&mut expected, b + offset, value);
        }
        assert_eq!(execute(case.entry, &mut case.wram, &rom, &mut NoServices).unwrap(), Some(()), "{}", case.name);
        assert_eq!(case.wram, expected, "{}", case.name);
    }
}

#[test]
fn initializer_uses_runtime_endpoint_bytes_instead_of_a_computed_speed() {
    let mut rom = rom();
    put(&mut rom, TABLE + 0x200, 7);
    put(&mut rom, TABLE + 0x202, 0x2468);
    let mut case = base("runtime-table", Entry::JetInit, 0x7c0);
    field(&mut case, PARAMETER1, 0xe040); // Wrapping multiplication aliases row64.
    field(&mut case, PARAMETER2, 2); // Nonzero, even direction is still right.
    assert_eq!(execute(case.entry, &mut case.wram, &rom, &mut NoServices).unwrap(), Some(()));
    let b = ENEMIES + 0x7c0;
    assert_eq!(word(&case.wram, b + E), 0x200);
    assert_eq!((word(&case.wram, b + D), word(&case.wram, b + C)), (7, 0x2468));
    assert_eq!(word(&case.wram, b + INSTRUCTION), 0xe2f4);
}

#[test]
fn every_preflight_refusal_is_unchanged_and_requests_no_service() {
    let rom = rom();
    for entry in Entry::ALL {
        for defect in
            ["short-wram", "long-wram", "slot-alignment", "slot-range", "row-range", "row-alignment", "short-table"]
        {
            let mut case = base("invalid", entry, 0x40);
            let mut input_rom = rom.clone();
            let init = matches!(entry, Entry::JetInit | Entry::RipperInit);
            match defect {
                "short-wram" => case.wram.truncate(SIZE - 1),
                "long-wram" => case.wram.push(0),
                "slot-alignment" => put(&mut case.wram, CURRENT, 1),
                "slot-range" => put(&mut case.wram, CURRENT, 0x800),
                "row-range" => field(&mut case, if init { PARAMETER1 } else { E }, if init { 65 } else { 0x208 }),
                "row-alignment" if !init => field(&mut case, E, 2),
                "row-alignment" => field(&mut case, PARAMETER1, 0x1fff),
                "short-table" => {
                    input_rom.pop();
                }
                _ => unreachable!(),
            }
            let before = case.wram.clone();
            assert_eq!(
                execute(entry, &mut case.wram, &input_rom, &mut NoServices).unwrap(),
                None,
                "{entry:?} {defect}"
            );
            assert_eq!(case.wram, before, "{entry:?} {defect}");
        }
    }
}

#[test]
fn no_collision_keeps_all_native_copyback_without_resetting_animation_or_timers() {
    let rom = rom();
    for entry in [Entry::JetMain, Entry::RipperMain] {
        let mut case = base("no-collision", entry, 0x40);
        let b = ENEMIES + 0x40;
        field(&mut case, C, 0x1234);
        field(&mut case, D, 0xffff);
        field(&mut case, PARAMETER1, 0xffff); // Main uses stored E, not parameter1.
        let before = case.wram.clone();
        let mut service = Motion::new(0);
        service.writes = vec![(b + 2, 63), (b + 4, 0xabcd), (0xdc4, 52), (0x1c37, 0xd094)];
        let mut expected = before.clone();
        for &(at, value) in &service.writes {
            put(&mut expected, at, value);
        }
        assert_eq!(execute(entry, &mut case.wram, &rom, &mut service).unwrap(), Some(()));
        assert_eq!(service.calls, [(0x40, -0xedcc)], "amount comes from C/D, not neighboring A/B");
        assert_eq!(service.input.as_ref(), Some(&before), "pre-service actor state is unchanged");
        assert_eq!(case.wram, expected);
    }
}

#[test]
fn collision_reloads_the_stored_row_including_zero_integer_and_mismatched_velocity() {
    let rom = rom();
    for entry in [Entry::JetMain, Entry::RipperMain] {
        for (integer, fraction, expected_integer, expected_fraction, right) in
            [(0, 0x8000, 0xffff, 0xf000, false), (1, 0x8123, 0xffff, 0xf000, false), (0xfffe, 0x8123, 0, 0x1000, true)]
        {
            let mut case = base("collision", entry, 0x40);
            let b = ENEMIES + 0x40;
            field(&mut case, D, integer);
            field(&mut case, C, fraction);
            let mut expected = case.wram.clone();
            let instruction = match (entry, right) {
                (Entry::JetMain, false) => 0xe2e0,
                (Entry::JetMain, true) => 0xe2f4,
                (Entry::RipperMain, false) => 0xe48b,
                (Entry::RipperMain, true) => 0xe477,
                _ => unreachable!(),
            };
            for (offset, value) in [
                (D, expected_integer),
                (C, expected_fraction),
                (INSTRUCTION, instruction),
                (INSTRUCTION_TIMER, 1),
                (TIMER, 0),
            ] {
                put(&mut expected, b + offset, value);
            }
            let mut service = Motion::new(1);
            assert_eq!(execute(entry, &mut case.wram, &rom, &mut service).unwrap(), Some(()));
            assert_eq!(service.calls.len(), 1);
            assert_eq!(case.wram, expected);
        }
    }
}

#[test]
fn collision_reloads_zero_and_endpoint_rows_and_still_resets_timers() {
    let rom = rom();
    for entry in [Entry::JetMain, Entry::RipperMain] {
        for (row, integer, expected_integer) in [(0, 1, 0), (0x200, 1, 0xfffc), (0x200, 0xffff, 4)] {
            let mut case = base("endpoint-collision", entry, 0x40);
            field(&mut case, E, row);
            field(&mut case, D, integer);
            field(&mut case, C, 0x1234);
            let mut service = Motion::new(1);
            assert_eq!(execute(entry, &mut case.wram, &rom, &mut service).unwrap(), Some(()));
            let b = ENEMIES + 0x40;
            assert_eq!((word(&case.wram, b + D), word(&case.wram, b + C)), (expected_integer, 0));
            assert_eq!((word(&case.wram, b + TIMER), word(&case.wram, b + INSTRUCTION_TIMER)), (0, 1));
        }
    }
}

#[test]
fn collision_reads_post_service_row_and_direction_from_the_original_record() {
    let rom = rom();
    let mut case = base("copyback-row", Entry::RipperMain, 0x40);
    let b = ENEMIES + 0x40;
    let mut service = Motion::new(1);
    service.writes = vec![(b + E, 0x200), (b + D, 0xffff), (CURRENT, 0x80), (0x1c37, 0xd094)];
    let mut expected = case.wram.clone();
    for &(at, value) in &service.writes {
        put(&mut expected, at, value);
    }
    for (offset, value) in [(D, 4), (C, 0), (INSTRUCTION, 0xe477), (INSTRUCTION_TIMER, 1), (TIMER, 0)] {
        put(&mut expected, b + offset, value);
    }
    assert_eq!(execute(case.entry, &mut case.wram, &rom, &mut service).unwrap(), Some(()));
    assert_eq!(case.wram, expected, "C retains its original record pointer across the native call");
}

#[test]
fn unavailable_service_refuses_without_mutation_and_accepted_errors_never_refuse() {
    let rom = rom();
    let mut case = base("unavailable", Entry::JetMain, 0x40);
    let before = case.wram.clone();
    let mut service = Motion::new(0);
    service.available = false;
    assert_eq!(execute(case.entry, &mut case.wram, &rom, &mut service).unwrap(), None);
    assert!(service.calls.is_empty());
    assert_eq!(case.wram, before);
    for failure in ["service", "collision", "row"] {
        let mut case = base(failure, Entry::JetMain, 0x40);
        let b = ENEMIES + 0x40;
        let mut service = Motion::new(1);
        service.writes.push((0x1c37, 0xd094));
        match failure {
            "service" => service.result = Err("native failure".into()),
            "collision" => service.result = Ok(2),
            "row" => service.writes.push((b + E, 0x208)),
            _ => unreachable!(),
        }
        let mut expected = case.wram.clone();
        for &(at, value) in &service.writes {
            put(&mut expected, at, value);
        }
        assert!(execute(case.entry, &mut case.wram, &rom, &mut service).is_err());
        assert_eq!(service.calls.len(), 1);
        assert_eq!(case.wram, expected, "accepted effects retained without actor replay after {failure}");
    }
}

#[test]
fn inherited_init_collision_and_clear_patrol_use_the_previous_result() {
    let rom = rom();
    for (init, main) in [(Entry::JetInit, Entry::JetMain), (Entry::RipperInit, Entry::RipperMain)] {
        let mut case = base("chain", init, 0x40);
        let b = ENEMIES + 0x40;
        field(&mut case, PARAMETER1, 64);
        field(&mut case, PARAMETER2, 0x8000);
        field(&mut case, 2, 56);
        assert_eq!(execute(init, &mut case.wram, &rom, &mut NoServices).unwrap(), Some(()));
        let mut service = Motion::new(1);
        service.writes = vec![(b + 4, 0xffff)];
        assert_eq!(execute(main, &mut case.wram, &rom, &mut service).unwrap(), Some(()));
        assert_eq!((word(&case.wram, b + D), word(&case.wram, b + C)), (0xfffc, 0));
        let animation = word(&case.wram, b + INSTRUCTION);
        service.result = Ok(0);
        service.writes = vec![(b + 2, 52)];
        assert_eq!(execute(main, &mut case.wram, &rom, &mut service).unwrap(), Some(()));
        assert_eq!(service.calls, [(0x40, 0x40000), (0x40, -0x40000)]);
        assert_eq!(word(&case.wram, b + INSTRUCTION), animation);
        assert_eq!((word(&case.wram, b + TIMER), word(&case.wram, b + INSTRUCTION_TIMER)), (0, 1));
    }
}

#[test]
fn shared_matrix_is_unique_finite_full_wram_and_contains_only_the_four_roots() {
    let cases = body_fixtures();
    assert_eq!(cases.len(), 46);
    let names: std::collections::BTreeSet<_> = cases.iter().map(|case| &case.name).collect();
    assert_eq!(names.len(), cases.len());
    let mut counts = [0; 4];
    for case in cases {
        counts[case.entry as usize] += 1;
        assert_eq!(case.wram.len(), SIZE);
        assert!(valid_slot(word(&case.wram, CURRENT)));
    }
    assert_eq!(counts, [10, 13, 10, 13]);
    for (entry, address) in Entry::ALL.into_iter().zip([0xa2e318, 0xa2e353, 0xa2e49f, 0xa2e4da]) {
        assert_eq!(Entry::try_from(entry as u32).unwrap(), entry);
        assert_eq!(entry.address(), address);
    }
    assert!(Entry::try_from(4).is_err());
}
