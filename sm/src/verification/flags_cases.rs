//! Frozen synthetic gameplay-flags scenarios; actual implementation supplied by adapter.
use crate::{game_state::Game, rom::RomImage, save_ram::SaveRam};
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) const CASES: u32 = 71;
#[derive(Clone, Copy)]
pub(crate) enum Operation {
    EventQuery(u16),
    EventSet(u16),
    EventClear(u16),
    Prepare(u16),
    BossQuery(u16),
    BossSet(u16),
    Plm(u32),
    Animated(u32),
    PalettePre,
    NewSave,
    Demo,
    DemoBoss,
    Load(u16),
    SporeDeath,
}
#[derive(Clone, Copy)]
pub(crate) struct Returned {
    pub kind: u8,
    pub value: u64,
}
impl Returned {
    pub(crate) fn word(value: u16) -> Self {
        Self {
            kind: 1,
            value: u64::from(value),
        }
    }
}
pub(crate) fn put(g: &mut Game, at: usize, value: u16) {
    let _ = &[(); 0x20000][at..at + 2];
    g.ram().copy_in(at, &value.to_le_bytes());
}
pub(crate) fn word(g: &Game, at: usize) -> u16 {
    {
        let _ = &[(); 0x20000][at..at + 2];
        let mut bytes = [0; 2];
        g.ram().copy_out(at, &mut bytes);
        u16::from_le_bytes(bytes)
    }
}
pub(crate) fn rom_word(image: &RomImage, address: u32, value: u16) {
    let at = crate::rtl::lorom_offset(address) as usize;
    for (i, b) in value.to_ne_bytes().into_iter().enumerate() {
        image.write(at + i, b);
    }
}
fn seed(g: &mut Game, image: &RomImage) {
    for i in 0..g.ram().len() {
        let value = (i.wrapping_mul(37) ^ (i >> 8) ^ 0x5a) as u8;
        g.ram().write_byte(i, value);
    }
    g.g_sram = SaveRam::zeroed(8192);
    for i in 0..8192 {
        g.g_sram.write(i, (i.wrapping_mul(19) ^ 0xa7) as u8);
    }
    g.loaded_rom = Some(image.clone());
    g.register_native_rom(image.clone());
    g.VERIFY_ACTIVE = true;
    g.g_fail = false;
    put(g, 0x5e7, 0xa534);
    put(g, 0x79f, 0);
    put(g, 0xe54, 0);
    put(g, 0x1c27, 0);
    put(g, 0x1dc7, 0);
    put(g, 0x1d77, 0);
    rom_word(image, 0x849100, 0x9234);
    rom_word(image, 0x849102, 0x9256);
    rom_word(image, 0x879100, 1);
    rom_word(image, 0x879102, 0x9234);
}
fn observe(out: &mut Vec<u8>, phase: u8, returned: Returned, g: &Game) {
    out.push(phase);
    out.push(returned.kind);
    out.extend_from_slice(&returned.value.to_le_bytes());
    out.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
    out.extend_from_slice(&g.g_sram.bytes(0..8192));
    out.push(u8::from(g.g_fail));
}
/// Seed actual LoadFromSram without invoking SaveToSram. Safe also for Runtime guards.
/// 63 primary valid;64 backup valid;65 both sums bad;66 primary complement bad.
pub(crate) fn seed_load(id: u32, g: &mut Game, image: &RomImage) -> u16 {
    assert!((63..=66).contains(&id));
    let slot = if id & 1 == 0 { 2u16 } else { 0u16 };
    let start = if slot == 0 { 0x10 } else { 0xcc8 };
    for (i, v) in [0x10, 0x66c, 0xcc8].into_iter().enumerate() {
        rom_word(image, 0x81812b + i as u32 * 2, v);
    }
    // Positive map counts avoid do-until zero-count paths. One real byte perarea.
    for area in 0..6 {
        image.write(crate::rtl::lorom_offset(0x818131) as usize + area, 1);
        rom_word(image, 0x818138 + area as u32 * 2, area as u16);
        rom_word(image, 0x8182d6 + area as u32 * 2, 0x9000 + area as u16);
        image.write(
            crate::rtl::lorom_offset(0x819000) as usize + area,
            (7 + area) as u8,
        );
    }
    for i in 0..0x65c {
        g.g_sram.write(start + i, (i.wrapping_mul(13) ^ 0xc3) as u8);
    }
    // Save payload offsets are literal SNES format, not productioncodec output.
    g.g_sram.set_word(start + 0xd916 - 0xd7c0, 3);
    g.g_sram.set_word(start + 0xd918 - 0xd7c0, 2);
    let checksum = (0..0x65c)
        .step_by(2)
        .fold(0u16, |sum, i| sum.wrapping_add(g.g_sram.word(start + i)));
    let index = usize::from(slot) * 2;
    g.g_sram.set_word(
        index,
        if id == 63 || id == 66 {
            checksum
        } else {
            checksum ^ 1
        },
    );
    g.g_sram
        .set_word(index + 8, if id == 66 { !checksum ^ 1 } else { !checksum });
    g.g_sram.set_word(
        index + 0x1ff0,
        if id == 64 { checksum } else { checksum ^ 1 },
    );
    g.g_sram.set_word(index + 0x1ff8, !checksum);
    put(g, 0x79f, 5);
    put(g, 0x78b, 7);
    put(g, 0x5e7, 0xa534);
    slot
}
fn seed_demo(g: &mut Game, image: &RomImage) {
    put(g, crate::ram_map::words::DEMO_SET, 0);
    put(g, crate::ram_map::words::DEMO_SCENE, 0);
    rom_word(image, 0x82876c, 0x9000);
    for (i, v) in [0x9100, 0x9200, 0, 0x100, 0x200, 0x30, 0x40, 0x500, 0]
        .into_iter()
        .enumerate()
    {
        rom_word(image, 0x829000 + i as u32 * 2, v);
    }
    image.write(crate::rtl::lorom_offset(0x8f9100) as usize + 1, 2);
}
pub(crate) fn capture_case(
    id: u32,
    run: fn(Operation, &RomImage, &mut Game) -> Returned,
) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let image = RomImage::zeroed(0x200000);
        let mut g = Game::new();
        seed(&mut g, &image);
        let mut operations = Vec::new();
        match id {
            0..=5 => {
                let bit = [0, 7, 63, 64, 511, 65535][id as usize];
                let value = 0;
                g.ram().write_byte(0xd820 + usize::from(bit >> 3), value);
                operations.extend([
                    Operation::EventQuery(bit),
                    Operation::EventSet(bit),
                    Operation::EventQuery(bit),
                    Operation::EventClear(bit),
                    Operation::EventQuery(bit),
                ]);
            }
            6..=25 => {
                let n = (id - 6) as usize;
                let area = [0, 7, 8, 65534, 65535][n / 4];
                let mask = [0x0102, 0x0080, 0x0100, 0xffff][n % 4];
                put(&mut g, 0x79f, area);
                let value = 0x40;
                g.ram().write_byte(0xd828 + usize::from(area), value);
                operations.extend([
                    Operation::BossQuery(mask),
                    Operation::BossSet(mask),
                    Operation::BossQuery(mask),
                ]);
            }
            26..=31 => operations.push(Operation::Prepare(
                [0, 7, 8, 511, 65535, 0x8000][(id - 26) as usize],
            )),
            32..=39 => {
                let n = (id - 32) as usize;
                let item = n >= 4;
                let bit = [0, 511, 512, 65535][n % 4];
                put(&mut g, 0x1dc7, bit);
                let base = if item { 0xd870 } else { 0xd830 };
                let value = 0;
                g.ram().write_byte(base + usize::from(bit >> 3), value);
                let query = if item { 8685692 } else { 8685640 };
                let mark = if item { 8685721 } else { 8685669 };
                operations.extend([
                    Operation::Plm(query),
                    Operation::Plm(mark),
                    Operation::Plm(query),
                ]);
            }
            40..=43 => {
                let bit = [0, 511, 512, 32767][(id - 40) as usize];
                put(&mut g, 0x1dc7, bit);
                let at = 0xd8b0 + usize::from(bit >> 3);
                let value = 0;
                g.ram().write_byte(at, value);
                let value = 0x5a;
                g.ram().write_byte(at + 1, value);
                image.write(crate::rtl::lorom_offset(0x849100) as usize, 1);
                rom_word(&image, 0x849101, 0x9234);
                operations.push(Operation::Plm(8686225));
            }
            44..=49 => {
                let bit = [0, 7, 63, 64, 511, 65535][(id - 44) as usize];
                let value = 0;
                g.ram().write_byte(0xd820 + usize::from(bit >> 3), value);
                rom_word(&image, 0x849100, bit);
                rom_word(&image, 0x879100, bit);
                operations.extend([Operation::Plm(8685630), Operation::Animated(8880447)]);
            }
            50..=59 => {
                let n = (id - 50) as usize;
                let area = [0, 1, 7, 65534, 65535][n / 2];
                put(&mut g, 0x79f, area);
                put(
                    &mut g,
                    0xd828 + usize::from(area),
                    if n & 1 == 0 { 0x8000 } else { 0x8002 },
                );
                put(&mut g, 0x1e7d, 0x8123);
                operations.push(Operation::PalettePre);
            }
            60 => operations.push(Operation::NewSave),
            61 => {
                seed_demo(&mut g, &image);
                operations.push(Operation::Demo);
            }
            62 => operations.push(Operation::DemoBoss),
            63..=66 => {
                let slot = seed_load(id, &mut g, &image);
                operations.push(Operation::Load(slot));
            }
            67..=70 => {
                let area = [1, 7, 65534, 65535][(id - 67) as usize];
                put(&mut g, 0x79f, area);
                put(&mut g, 0xf8c, 0);
                put(&mut g, 0xd828 + usize::from(area), 0xa500);
                for i in 0..40 {
                    put(&mut g, 0x1c37 + i * 2, 1);
                } // Exhaust PLM allocation; actual death flag write still executes.
                operations.push(Operation::SporeDeath);
            }
            _ => unreachable!(),
        }
        let initial = (0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<_>>();
        let initial_sram = g.g_sram.bytes(0..8192);
        let mut out = b"FLGC1".to_vec();
        out.extend_from_slice(&id.to_le_bytes());
        out.push(operations.len() as u8);
        observe(&mut out, 0, Returned { kind: 0, value: 0 }, &g);
        for (phase, operation) in operations.into_iter().enumerate() {
            let returned = run(operation, &image, &mut g);
            assert!(!g.g_fail, "flags.{id}.diagnostic");
            if (6..=25).contains(&id) {
                assert_eq!(g.ram().read_byte(0x5e8), 0xa5, "flags.boss.scratch_high");
            }
            if (50..=70).contains(&id) {
                assert_eq!(
                    (0x5e7..0x5e9)
                        .map(|at| g.ram().read_byte(at))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &initial[0x5e7..0x5e9],
                    "flags.raw_reset_load.scratch"
                );
            }
            if id == 60 {
                assert_eq!(
                    (0xd820..0xd830)
                        .map(|at| g.ram().read_byte(at))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &initial[0xd820..0xd830],
                    "flags.new_save.events_bosses_retained"
                );
            }
            if id == 60 {
                assert!(
                    (0xd830..0xd8f0).all(|at| g.ram().read_byte(at) == 0),
                    "flags.new_save.collections_clear"
                );
            }
            if id == 61 {
                assert!(
                    (0xd820..0xd830).all(|at| g.ram().read_byte(at) == 0),
                    "flags.demo.events_bosses_clear"
                );
                assert!(
                    (0xd830..0xd8b0).all(|at| g.ram().read_byte(at) == 0xff),
                    "flags.demo.objects_items_set"
                );
                assert!(
                    (0xd8b0..0xd8f0).all(|at| g.ram().read_byte(at) == 0),
                    "flags.demo.doors_clear"
                );
            }
            if id == 62 {
                let mut expected = initial.clone();
                expected[0xd829] = 1;
                assert_eq!(
                    (0..g.ram().len())
                        .map(|at| g.ram().read_byte(at))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &expected[..],
                    "flags.demo_boss.byte_only"
                );
            }
            if id == 63 || id == 64 {
                let start = if id == 63 { 0x10 } else { 0xcc8 };
                assert_eq!(returned.value, 0, "flags.load.success_status");
                assert_eq!(
                    (0xd820..0xd8f0)
                        .map(|at| g.ram().read_byte(at))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &initial_sram[start + 0x60..start + 0x130],
                    "flags.load.success_flags"
                );
                assert_eq!(
                    g.g_sram.bytes(0..8192),
                    initial_sram,
                    "flags.load.success_preserves_sram"
                );
                assert_eq!(
                    (word(&g, 0x79f), word(&g, 0x78b)),
                    (2, 3),
                    "flags.load.success_area_station"
                );
            }
            if id >= 67 {
                let area = [1u16, 7, 65534, 65535][(id - 67) as usize];
                assert_eq!(
                    word(&g, 0xd828 + usize::from(area)),
                    0xa502,
                    "flags.spore.le_word_alias"
                );
            }
            if id == 65 || id == 66 {
                let slot = if id & 1 == 0 { 2usize } else { 0 };
                let start = if slot == 0 { 0x10 } else { 0xcc8 };
                assert_eq!(returned.value, 1, "flags.load.failure_status");
                assert_eq!(
                    (0xd7c0..0xde1c)
                        .map(|at| g.ram().read_byte(at))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &initial_sram[start..start + 0x65c],
                    "flags.load.failure_keeps_copied_payload"
                );
                assert!(
                    g.g_sram.bytes(start..start + 0x65c).iter().all(|&b| b == 0),
                    "flags.load.failure_clears_sram"
                );
                assert_eq!(
                    (word(&g, 0x79f), word(&g, 0x78b)),
                    (0, 0),
                    "flags.load.failure_area_station"
                );
                assert_eq!(
                    (0x9a2..0xa02)
                        .map(|at| g.ram().read_byte(at))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &initial[0x9a2..0xa02],
                    "flags.load.failure_retains_player"
                );
                assert_eq!(
                    (0xcd52..0xd454)
                        .map(|at| g.ram().read_byte(at))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &initial[0xcd52..0xd454],
                    "flags.load.failure_retains_map"
                );
                let mut expected_sram = initial_sram.clone();
                expected_sram[start..start + 0x65c].fill(0);
                assert_eq!(
                    g.g_sram.bytes(0..8192),
                    expected_sram,
                    "flags.load.failure_only_selected_payload"
                );
            }
            observe(&mut out, (phase + 1) as u8, returned, &g);
        }
        out
    }));
    match result {
        Ok(bytes) => Some(bytes),
        Err(_) => {
            eprintln!("flags case {id} failed");
            None
        }
    }
}
