use super::*;

#[derive(Default)]
struct Disk {
    unavailable: bool,
    fail: bool,
    calls: Vec<(Vec<u8>, Vec<u8>)>,
}
impl Services for Disk {
    fn available(&self) -> bool {
        !self.unavailable
    }
    fn write_sram(&mut self, ram: &mut [u8], sram: &mut [u8]) -> Result<(), String> {
        self.calls.push((ram.to_vec(), sram.to_vec()));
        if self.fail { Err("disk service fault".into()) } else { Ok(()) }
    }
}
fn fixture(name: &str) -> BodyFixture {
    body_fixtures().into_iter().find(|f| f.name == name).unwrap()
}
fn run(f: &mut BodyFixture, disk: &mut Disk) -> Result<Option<Output>, String> {
    execute(f.entry, f.argument, &mut f.wram, &mut f.sram, &tables::fixture_rom(), disk)
}

#[test]
fn save_commits_complete_payload_before_one_disk_request() {
    let mut f = fixture("save-slot-1-area-4");
    let before = f.clone();
    let mut disk = Disk::default();
    assert_eq!(run(&mut f, &mut disk).unwrap(), Some(Output::Done));
    assert_eq!(disk.calls, vec![(f.wram.clone(), f.sram.clone())]);
    assert_eq!(&f.sram[0x66c..0xcc8], &f.wram[0xd7c0..0xde1c]);
    assert_eq!(&f.wram[0xd7c0..0xd820], &before.wram[0x9a2..0xa02]);
    assert_eq!(&f.wram[0xd152..0xd252], &before.wram[0x7f7..0x8f7]);
    assert_eq!(word(&f.wram, 0xd916), 0xcafe);
    assert_eq!(word(&f.wram, 0xd918), 4);
    let wide_sum: u32 =
        f.sram[0x66c..0xcc8].as_chunks::<2>().0.iter().map(|b| u32::from(u16::from_le_bytes([b[0], b[1]]))).sum();
    assert!(wide_sum > 0xffff);
    let sum = wide_sum as u16;
    for at in [2, 0x1ff2] {
        assert_eq!(word(&f.sram, at), sum);
    }
    for at in [10, 0x1ffa] {
        assert_eq!(word(&f.sram, at), !sum);
    }
    for i in 0..SRAM_SIZE {
        if !(0x66c..0xcc8).contains(&i) && ![2, 3, 10, 11, 0x1ff2, 0x1ff3, 0x1ffa, 0x1ffb].contains(&i) {
            assert_eq!(f.sram[i], before.sram[i], "SRAM {i:x}");
        }
    }
}

#[test]
fn either_checksum_pair_accepts_and_preserves_arbitrary_saved_location() {
    for mode in ["both", "lower", "upper"] {
        for slot in 0..3 {
            let mut f = fixture(&format!("load-slot-{slot}-{mode}"));
            let before = f.clone();
            let mut disk = Disk::default();
            assert_eq!(run(&mut f, &mut disk).unwrap(), Some(Output::Loaded(0)));
            assert_eq!(word(&f.wram, 0x078b), 0xcafe);
            assert_eq!(word(&f.wram, 0x079f), 0xfedc);
            assert_eq!(&f.wram[PLAYER..PLAYER + 96], &before.sram[SLOTS[slot]..SLOTS[slot] + 96]);
            assert_eq!(f.sram, before.sram);
            assert!(disk.calls.is_empty());
        }
    }
}

#[test]
fn invalid_load_copies_payload_then_clears_only_selected_sram_payload() {
    for mode in ["neither", "split"] {
        for slot in 0..3 {
            let mut f = fixture(&format!("load-slot-{slot}-{mode}"));
            let before = f.clone();
            let mut disk = Disk::default();
            assert_eq!(run(&mut f, &mut disk).unwrap(), Some(Output::Loaded(1)));
            let mut expected_ram = before.wram;
            expected_ram[SAVED..PAYLOAD_END].copy_from_slice(&before.sram[SLOTS[slot]..SLOTS[slot] + PAYLOAD_SIZE]);
            expected_ram[0x78b..0x78d].fill(0);
            expected_ram[0x79f..0x7a1].fill(0);
            assert_eq!(f.wram, expected_ram);
            let mut expected_sram = before.sram;
            expected_sram[SLOTS[slot]..SLOTS[slot] + PAYLOAD_SIZE].fill(0);
            assert_eq!(f.sram, expected_sram);
            assert!(disk.calls.is_empty());
        }
    }
}

#[test]
fn checksum_addition_wraps_at_sixteen_bits() {
    let mut f = fixture("load-checksum-wrap");
    assert_eq!(word(&f.sram, 0), 0xfcd2); // 814 words of 0xffff.
    assert_eq!(run(&mut f, &mut Disk::default()).unwrap(), Some(Output::Loaded(0)));
    assert_eq!(word(&f.wram, 0x79f), 0xffff);
}

#[test]
fn masked_slot_high_bits_alias_exactly() {
    for slot in 0..3 {
        for entry in [Entry::Save, Entry::Load] {
            let mut a = fixture(if entry == Entry::Save { "save-slot-0-area-0" } else { "load-slot-0-both" });
            a.argument = slot;
            let mut b = a.clone();
            b.argument = 0xfffc | slot;
            let mut da = Disk::default();
            let mut db = Disk::default();
            assert_eq!(run(&mut a, &mut da).unwrap(), run(&mut b, &mut db).unwrap());
            assert_eq!(a.wram, b.wram);
            assert_eq!(a.sram, b.sram);
            assert_eq!(da.calls, db.calls);
        }
    }
}

#[test]
fn pack_copies_all_six_area_edges_and_preserves_other_bytes() {
    let mut f = fixture("pack-all-map-indices");
    let before = f.clone();
    assert_eq!(run(&mut f, &mut Disk::default()).unwrap(), Some(Output::Done));
    for (area, packed_first, packed_last, first, last) in [
        (0, 0, 73, 0x07, 0xba),
        (1, 74, 145, 0x05, 0xd9),
        (2, 146, 221, 0x05, 0xc4),
        (3, 222, 239, 0x2d, 0x52),
        (4, 240, 305, 0x07, 0xc4),
        (5, 306, 326, 0x26, 0x5a),
    ] {
        assert_eq!(f.wram[COMPRESSED + packed_first], before.wram[MAP + area * 256 + first]);
        assert_eq!(f.wram[COMPRESSED + packed_last], before.wram[MAP + area * 256 + last]);
    }
    assert_eq!(&f.wram[..COMPRESSED], &before.wram[..COMPRESSED]);
    assert_eq!(&f.wram[COMPRESSED + 327..], &before.wram[COMPRESSED + 327..]);
    assert_eq!(f.sram, before.sram);
}

#[test]
fn unpack_clears_exactly_1794_bytes_with_sparse_restoration() {
    let mut f = fixture("unpack-exact-clear-and-indices");
    let before = f.clone();
    run(&mut f, &mut Disk::default()).unwrap();
    assert_eq!(&f.wram[..MAP], &before.wram[..MAP]);
    assert_eq!(&f.wram[MAP + 0x702..], &before.wram[MAP + 0x702..]);
    assert!(f.wram[MAP + 0x600..MAP + 0x702].iter().all(|&byte| byte == 0));
    assert_eq!(f.wram[MAP], 0); // Unlisted first area byte.
    assert_eq!(f.wram[MAP + 7], before.wram[COMPRESSED]);
    assert_eq!(f.wram[MAP + 0x55a], before.wram[COMPRESSED + 326]);
    assert_eq!(f.sram, before.sram);
}

#[test]
fn save_load_chain_restores_player_and_selected_location() {
    let mut f = fixture("save-slot-2-area-5");
    let player = f.wram[PLAYER..PLAYER + 96].to_vec();
    let mut disk = Disk::default();
    run(&mut f, &mut disk).unwrap();
    f.wram[PLAYER..PLAYER + 96].fill(0);
    put(&mut f.wram, 0x78b, 0);
    put(&mut f.wram, 0x79f, 0);
    f.entry = Entry::Load;
    assert_eq!(run(&mut f, &mut disk).unwrap(), Some(Output::Loaded(0)));
    assert_eq!(&f.wram[PLAYER..PLAYER + 96], player);
    assert_eq!(word(&f.wram, 0x78b), 0xcafe);
    assert_eq!(word(&f.wram, 0x79f), 5);
    assert_eq!(disk.calls.len(), 1);
}

#[test]
fn domain_refusals_leave_both_buffers_and_service_untouched() {
    for entry in [Entry::Save, Entry::Load] {
        for argument in [3, 0x1237, 0xffff] {
            let mut f = fixture("save-slot-0-area-0");
            f.entry = entry;
            f.argument = argument;
            let before = f.clone();
            let mut disk = Disk::default();
            assert_eq!(run(&mut f, &mut disk).unwrap(), None);
            assert_eq!(f.wram, before.wram);
            assert_eq!(f.sram, before.sram);
            assert!(disk.calls.is_empty());
        }
    }
    for area in [6, 7, 0xffff] {
        let mut f = fixture("save-slot-0-area-0");
        put(&mut f.wram, 0x79f, area);
        let before = f.clone();
        let mut disk = Disk::default();
        assert_eq!(run(&mut f, &mut disk).unwrap(), None);
        assert_eq!(f.wram, before.wram);
        assert_eq!(f.sram, before.sram);
        assert!(disk.calls.is_empty());
    }
}

#[test]
fn malformed_buffers_tables_and_unavailable_service_fail_before_mutation() {
    for kind in 0..7 {
        let mut f = fixture("save-slot-0-area-0");
        let mut rom = tables::fixture_rom();
        let mut disk = Disk { unavailable: kind == 6, ..Disk::default() };
        match kind {
            0 => {
                f.wram.pop();
            }
            1 => {
                f.sram.pop();
            }
            2 => {
                rom.truncate(0x812b);
            }
            3 => {
                rom[0x812b] ^= 1;
            }
            4 => {
                rom[0x82d6] ^= 1;
            }
            5 => {
                rom[0x8146] ^= 1;
            }
            _ => {}
        }
        let before = f.clone();
        assert!(execute(f.entry, f.argument, &mut f.wram, &mut f.sram, &rom, &mut disk).is_err());
        assert_eq!(f.wram, before.wram);
        assert_eq!(f.sram, before.sram);
        assert!(disk.calls.is_empty());
    }
}

#[test]
fn service_failure_keeps_mutated_state_and_cannot_be_a_refusal() {
    let mut f = fixture("save-slot-0-area-0");
    let before = f.clone();
    let mut disk = Disk { fail: true, ..Disk::default() };
    assert_eq!(run(&mut f, &mut disk), Err("disk service fault".into()));
    assert_ne!(f.sram, before.sram);
    assert_eq!(disk.calls, vec![(f.wram, f.sram)]);
}

#[test]
fn helpers_need_no_disk_service_but_require_zero_argument() {
    for entry in [Entry::Pack, Entry::Unpack] {
        let mut f = fixture("pack-all-map-indices");
        f.entry = entry;
        let mut disk = Disk { unavailable: true, ..Disk::default() };
        assert_eq!(run(&mut f, &mut disk).unwrap(), Some(Output::Done));
        f.argument = 1;
        let before = f.clone();
        assert!(run(&mut f, &mut disk).is_err());
        assert_eq!(f.wram, before.wram);
        assert_eq!(f.sram, before.sram);
        assert!(disk.calls.is_empty());
    }
}
