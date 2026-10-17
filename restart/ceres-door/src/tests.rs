use super::*;
use memory::{PALETTE, VAR_A, VAR_D, VAR_E, VAR_F};

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}
fn set(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn rom_bytes(rom: &mut [u8], at: u16, bytes: &[u8]) {
    let start = 0x130000 + usize::from(at & 0x7fff);
    rom[start..start + bytes.len()].copy_from_slice(bytes);
}
fn rom_word(rom: &mut [u8], at: u16, value: u16) {
    rom_bytes(rom, at, &value.to_le_bytes());
}
fn actor_ram(index: u16, parameter: u16) -> Vec<u8> {
    let mut ram = vec![0; WRAM_SIZE];
    let enemy = ENEMY + usize::from(index);
    set(&mut ram, enemy, 0xe23f);
    ram[enemy + 0x2e] = 0xa6;
    set(&mut ram, enemy + PARAMETER, parameter);
    set(&mut ram, 0xe54, index);
    ram
}
fn authored_rom() -> Vec<u8> {
    let mut rom = vec![0; 0x300000];
    for (parameter, main) in [0xf76b, 0xf770, 0xf7bd, 0xf7a5, 0xf850, 0xf76b, 0xf770].into_iter().enumerate() {
        rom_word(&mut rom, 0xf72b + 2 * parameter as u16, main);
        rom_word(&mut rom, 0xf52c + 2 * parameter as u16, 0xf53a + 8 * parameter as u16);
    }
    rom_bytes(&mut rom, 0xf4ee, &[0x31; 30]);
    rom_bytes(&mut rom, 0xf50e, &[0x62; 30]);
    rom_word(&mut rom, 0xf900, 0xe000);
    rom_word(&mut rom, 0xf902, 0xe020);
    rom_bytes(&mut rom, 0xe000, &[0x81, 1, 2, 3, 4, 5, 6, 7, 8, 0x40, 9, 10, 11, 12, 13, 14, 0]);
    rom_bytes(&mut rom, 0xe020, &[0x40, 21, 22, 23, 24, 25, 26, 0]);
    rom_bytes(&mut rom, 0xf871, &[0x73; 12]);
    rom_bytes(&mut rom, 0xf8e1, &[0x84; 12]);
    for (i, pair) in [[0xfff8, 0xfff8], [8, 0xfff8], [0xfff8, 8], [8, 8]].into_iter().enumerate() {
        rom_word(&mut rom, 0xf840 + 4 * i as u16, pair[0]);
        rom_word(&mut rom, 0xf842 + 4 * i as u16, pair[1]);
    }
    rom
}

#[derive(Default)]
struct HostProbe {
    requests: Vec<(u16, u16, [u16; 2], u16, u16, u16)>,
    fail: bool,
}
impl Host for HostProbe {
    fn spawn_eproj_with_room_gfx(
        &mut self,
        ram: &mut [u8],
        header: u16,
        parameter: u16,
        point: [u16; 2],
    ) -> Result<()> {
        self.requests.push((
            header,
            parameter,
            point,
            word(ram, 0x5e5),
            word(ram, ENEMY + VAR_E),
            word(ram, ENEMY + VAR_F),
        ));
        if self.fail {
            return Err("deliberate foreign failure".into());
        }
        // A copied-back native write must survive subsequent sound queue work.
        set(ram, 0x1993, 0xabcd);
        Ok(())
    }
}
fn invoke(entry: Entry, index: u16, instruction: u16, ram: &mut [u8], rom: &[u8]) -> Output {
    match run(entry, index, instruction, ram, rom, &mut HostProbe::default()).unwrap() {
        Outcome::Handled(output) => output,
        Outcome::Refused => panic!("fixture refused"),
    }
}

#[test]
fn instruction_sequence_changes_only_its_actor_and_preserves_other_properties() {
    let mut ram = actor_ram(0x40, 1);
    let enemy = ENEMY + 0x40;
    set(&mut ram, enemy + PROPERTIES, 0xa800);
    set(&mut ram, ENEMY + PROPERTIES, 0x1234);
    for (entry, properties, flag) in [
        (Entry::SetTangible, 0xac00, 0),
        (Entry::SetInvisible, 0xad00, 0),
        (Entry::Instr3, 0xad00, 1),
        (Entry::Instr1, 0xa900, 1),
        (Entry::Instr5, 0xa800, 0),
        (Entry::SetInvisible, 0xa900, 0),
        (Entry::Instr2, 0xa800, 0),
    ] {
        assert_eq!(invoke(entry, 0x40, 0xf57c, &mut ram, &[]), Output::InstructionOffset(0xf57c));
        assert_eq!(word(&ram, enemy + PROPERTIES), properties);
        assert_eq!(word(&ram, enemy + VAR_B), flag);
        assert_eq!(word(&ram, ENEMY + PROPERTIES), 0x1234);
    }
}

#[test]
fn proximity_branches_use_strict_48_pixel_box_and_wrapped_signed_distance() {
    let mut ram = actor_ram(0, 0);
    let mut rom = authored_rom();
    rom_word(&mut rom, 0xf5a2, 0xf580);
    set(&mut ram, ENEMY + 2, 10);
    set(&mut ram, ENEMY + 6, 20);
    for (x, y, next) in
        [(57, 67, 0xf5a4), (58, 20, 0xf580), (10, 68, 0xf580), (0xfffe, 20, 0xf5a4), (0x800a, 20, 0xf580)]
    {
        set(&mut ram, 0xaf6, x);
        set(&mut ram, 0xafa, y);
        assert_eq!(invoke(Entry::Instr6, 0, 0xf5a2, &mut ram, &rom), Output::InstructionOffset(next));
    }
}

#[test]
fn boss_and_escape_bytecode_branches_keep_their_distinct_conditions() {
    let mut ram = actor_ram(0, 0);
    let mut rom = authored_rom();
    rom_word(&mut rom, 0xf540, 0xf55a);
    set(&mut ram, 0x79f, 6);
    ram[0xd82f] = 1; // Neighboring area's bit is the high byte of the C read.
    assert_eq!(invoke(Entry::Instr4, 0, 0xf540, &mut ram, &rom), Output::InstructionOffset(0xf55a));
    ram[0xd82e] = 1;
    assert_eq!(invoke(Entry::Instr4, 0, 0xf540, &mut ram, &rom), Output::InstructionOffset(0xf542));
    assert_eq!(invoke(Entry::Instr8, 0, 0xf540, &mut ram, &rom), Output::InstructionOffset(0xf55a));
    set(&mut ram, 0x93f, 0x8000);
    assert_eq!(invoke(Entry::Instr8, 0, 0xf540, &mut ram, &rom), Output::InstructionOffset(0xf542));
}

#[test]
fn initialization_covers_all_parameters_and_palette_branches() {
    let rom = authored_rom();
    for parameter in 0..=6 {
        for escape in [0, 2] {
            let mut ram = actor_ram(0x40, parameter);
            let enemy = ENEMY + 0x40;
            set(&mut ram, 0x93f, escape);
            set(&mut ram, 0x330, 7);
            ram[0xd0..0xd7].fill(0x9a);
            assert_eq!(invoke(Entry::Init, 0x40, 0, &mut ram, &rom), Output::Done);
            assert_eq!(word(&ram, enemy + 0x16), 0xfac7);
            assert_eq!(word(&ram, enemy + 0x1c), 1);
            assert_eq!(word(&ram, enemy + 0x1a), 0xf53a + 8 * parameter);
            let special = escape == 0 && parameter == 3;
            assert_eq!(word(&ram, enemy + PALETTE), if special { 1024 } else { 3584 });
            let target = if special { 0xc342 } else { 0xc3e2 };
            assert_eq!(&ram[target..target + 30], &[if escape == 0 { 0x31 } else { 0x62 }; 30]);
            assert_eq!(&ram[0xd0..0xd7], &[0x9a; 7]);
            assert_eq!(word(&ram, 0x330), if parameter == 2 { 14 } else { 7 });
            if parameter == 2 {
                assert_eq!(&ram[0xd7..0xde], &[0, 4, 0, 0xc4, 0xb0, 0, 0x70]);
            }
        }
    }
}

#[test]
fn quake_states_do_not_advance_random_or_overwrite_active_quake() {
    for (state, normal, rare) in [(0xf76b, 0x14, 0x1a), (0xf770, 0x1d, 0x23)] {
        let mut ram = actor_ram(0, 0);
        set(&mut ram, ENEMY + VAR_A, state);
        set(&mut ram, 0x93f, 2);
        set(&mut ram, 0x5e5, 0x107f);
        invoke(Entry::Main, 0, 0, &mut ram, &[]);
        assert_eq!(word(&ram, 0x183e), rare);
        assert_eq!(word(&ram, 0x1840), 4);
        assert_eq!(word(&ram, 0x5e5), 0x107f);
        set(&mut ram, 0x5e5, 0x1080);
        invoke(Entry::Main, 0, 0, &mut ram, &[]);
        assert_eq!(word(&ram, 0x183e), rare);
        set(&mut ram, 0x1840, 0);
        invoke(Entry::Main, 0, 0, &mut ram, &[]);
        assert_eq!(word(&ram, 0x183e), normal);
        assert_eq!(word(&ram, 0x1840), 2);
    }
}

#[test]
fn visibility_main_uses_low_escape_bit() {
    let mut ram = actor_ram(0, 3);
    set(&mut ram, ENEMY + VAR_A, 0xf7a5);
    set(&mut ram, ENEMY + PROPERTIES, 0xac00);
    set(&mut ram, ENEMY + PALETTE, 1024);
    set(&mut ram, 0x93f, 2);
    invoke(Entry::Main, 0, 0, &mut ram, &[]);
    assert_eq!(word(&ram, ENEMY + PROPERTIES), 0xad00);
    assert_eq!(word(&ram, ENEMY + PALETTE), 1024);
    set(&mut ram, 0x93f, 3);
    invoke(Entry::Main, 0, 0, &mut ram, &[]);
    assert_eq!(word(&ram, ENEMY + PROPERTIES), 0xac00);
    assert_eq!(word(&ram, ENEMY + PALETTE), 3584);
}

#[test]
fn animation_appends_both_descriptor_widths_and_reads_wrapped_odd_palette_address() {
    let rom = authored_rom();
    let mut ram = actor_ram(0, 4);
    set(&mut ram, ENEMY + VAR_A, 0xf850);
    set(&mut ram, 0x334, 7);
    ram[0x2d0..0x2d7].fill(0x9a);
    ram[0x2e7] = 0xff;
    invoke(Entry::Main, 0, 0, &mut ram, &rom);
    assert_eq!(&ram[0x2d0..0x2d7], &[0x9a; 7]);
    assert_eq!(&ram[0x2d7..0x2e7], &[0x81, 1, 2, 3, 4, 5, 6, 7, 8, 0x40, 9, 10, 11, 12, 13, 14]);
    assert_eq!(ram[0x2e7], 0);
    assert_eq!(word(&ram, 0x334), 23);
    assert_eq!(&ram[0xc052..0xc05e], &[0x73; 12]);
    set(&mut ram, 0x334, 0);
    set(&mut ram, 0x5b6, 0x3a);
    invoke(Entry::Main, 0, 0, &mut ram, &rom);
    assert_eq!(&ram[0x2d0..0x2d8], &[0x40, 21, 22, 23, 24, 25, 26, 0]);
    assert_eq!(&ram[0xc052..0xc05e], &[0x84; 12]);
    set(&mut ram, 0x334, 0);
    set(&mut ram, 0x5b6, 0);
    set(&mut ram, 0xc400, 1);
    invoke(Entry::Main, 0, 0, &mut ram, &rom);
    assert_eq!(&ram[0xc052..0xc05e], &[0x84; 12]);
}

#[test]
fn escape_arms_then_runs_complete_countdown_using_slot_zero_counters() {
    let rom = authored_rom();
    let mut ram = actor_ram(0x40, 2);
    let enemy = ENEMY + 0x40;
    set(&mut ram, enemy + VAR_A, 0xf7bd);
    set(&mut ram, enemy + 2, 100);
    set(&mut ram, enemy + 6, 200);
    set(&mut ram, enemy + VAR_E, 0xbeef);
    set(&mut ram, enemy + VAR_F, 0xcafe);
    set(&mut ram, ENEMY + VAR_E, 99);
    set(&mut ram, ENEMY + VAR_F, 99);
    set(&mut ram, 0x93f, 2);
    invoke(Entry::Main, 0x40, 0, &mut ram, &rom);
    assert_eq!(word(&ram, enemy + VAR_A), 0xf7dc);
    assert_eq!(word(&ram, enemy + VAR_D), 48);
    assert_eq!(word(&ram, ENEMY + VAR_E), 0);
    assert_eq!(word(&ram, ENEMY + VAR_F), 0);
    let mut host = HostProbe::default();
    for tick in 1..=49 {
        assert_eq!(run(Entry::Main, 0x40, 0, &mut ram, &rom, &mut host).unwrap(), Outcome::Handled(Output::Done));
        if tick == 1 {
            assert_eq!(host.requests, [(0xe509, 12, [108, 208], 0x111, 4, 3)]);
            assert_eq!(word(&ram, 0x1993), 0xabcd);
            assert_eq!(ram[0x666], 0x25);
        }
    }
    assert_eq!(host.requests.len(), 10);
    assert_eq!(host.requests[1].2, [92, 208]);
    assert_eq!(word(&ram, enemy + VAR_D), 0xffff);
    assert_eq!(word(&ram, enemy + VAR_A), 0xf850);
    assert_eq!(word(&ram, enemy + PROPERTIES) & 0x100, 0x100);
    assert_eq!(word(&ram, 0x93f), 0x8000);
    assert_eq!(word(&ram, enemy + VAR_E), 0xbeef);
    assert_eq!(word(&ram, enemy + VAR_F), 0xcafe);
}

#[test]
fn foreign_failure_is_owned_and_never_becomes_refusal() {
    let rom = authored_rom();
    let mut ram = actor_ram(0, 2);
    set(&mut ram, ENEMY + VAR_A, 0xf7dc);
    set(&mut ram, ENEMY + VAR_D, 48);
    set(&mut ram, 0x5e5, 0x2000);
    let mut host = HostProbe { fail: true, ..HostProbe::default() };
    let error = run(Entry::Main, 0, 0, &mut ram, &rom, &mut host).unwrap_err();
    assert_eq!(error, "deliberate foreign failure");
    assert_eq!(host.requests.len(), 1);
    assert_eq!(host.requests[0].1, 3);
    assert_eq!(word(&ram, ENEMY + VAR_D), 47);
    assert_eq!(ram[0x666], 0); // Sound request follows successful foreign return.
}

#[test]
fn random_generator_preserves_byte_product_and_end_around_carry() {
    for (input, expected) in
        [(0, 0x111), (0x33, 0x210), (0x34, 0x215), (0xffff, 0x10d), (0x3333, 0x111), (0x6666, 0x110), (0x2000, 0xa111)]
    {
        assert_eq!(helpers::next_random(input), expected);
    }
}

#[test]
fn sound_ring_wraps_limits_and_keeps_disable_guards() {
    let mut ram = actor_ram(0, 0);
    ram[0x648] = 15;
    ram[0x645] = 14;
    ram[0x676] = 0xff;
    invoke(Entry::Instr7, 0, 0xf6bf, &mut ram, &[]);
    assert_eq!(ram[0x685], 0x2c);
    assert_eq!(ram[0x648], 0);
    assert_eq!(ram[0x676], 0);
    assert_eq!(ram[0x655], 6);
    ram[0x648] = 6;
    ram[0x645] = 0;
    let before = ram.clone();
    invoke(Entry::Instr7, 0, 0xf6bf, &mut ram, &[]);
    assert_eq!(ram, before);
    for guard in [0x5f5, 0x998, 0x592] {
        let mut ram = actor_ram(0, 0);
        set(&mut ram, guard, if guard == 0x998 { 40 } else { 0x8000 });
        invoke(Entry::Instr7, 0, 0xf6bf, &mut ram, &[]);
        assert_eq!(ram[0x648], 0);
        assert_eq!(ram[0x676], 0);
        assert_eq!(ram[0x655], 6);
    }
}

#[test]
fn solid_setup_changes_four_vertical_types_and_preserves_tiles_bts_and_neighbors() {
    let mut ram = vec![0x5a; WRAM_SIZE];
    set(&mut ram, 0x1c37 + 6, 0xba48);
    set(&mut ram, 0x1c87 + 6, 0x32);
    set(&mut ram, 0x7a5, 32);
    let offsets = [0x10034, 0x10074, 0x100b4, 0x100f4];
    for (i, at) in offsets.into_iter().enumerate() {
        set(&mut ram, at, 0xd123 + i as u16);
    }
    let mut expected = ram.clone();
    for (i, at) in offsets.into_iter().enumerate() {
        set(&mut expected, at, 0x8123 + i as u16);
    }
    assert_eq!(invoke(Entry::SolidSetup, 6, 0, &mut ram, &[]), Output::SetupStatus(0));
    assert_eq!(ram, expected);
}

#[test]
fn invalid_and_unadmitted_inputs_do_not_mutate_or_request_foreign_work() {
    let rom = authored_rom();
    let mut host = HostProbe::default();
    for wrong in [0, 1] {
        let mut ram = actor_ram(0, 0);
        set(&mut ram, if wrong == 0 { ENEMY } else { ENEMY + PARAMETER }, if wrong == 0 { 0xe200 } else { 7 });
        let before = ram.clone();
        assert_eq!(run(Entry::Init, 0, 0, &mut ram, &rom, &mut host).unwrap(), Outcome::Refused);
        assert_eq!(ram, before);
    }
    let mut cases = Vec::new();
    let mut init = actor_ram(0, 2);
    set(&mut init, 0x330, 0x1fa);
    cases.push((Entry::Init, init));
    let mut animation = actor_ram(0, 4);
    set(&mut animation, ENEMY + VAR_A, 0xf850);
    set(&mut animation, 0x334, 0x58);
    cases.push((Entry::Main, animation));
    let mut puff = actor_ram(0, 2);
    set(&mut puff, ENEMY + VAR_A, 0xf7dc);
    set(&mut puff, ENEMY + VAR_D, 48);
    set(&mut puff, ENEMY + VAR_F, 5);
    cases.push((Entry::Main, puff));
    let mut solid = vec![0; WRAM_SIZE];
    set(&mut solid, 0x1c37, 0xba48);
    set(&mut solid, 0x1c87, 0xfffe);
    cases.push((Entry::SolidSetup, solid));
    for (entry, mut ram) in cases {
        let before = ram.clone();
        assert!(run(entry, 0, 0, &mut ram, &rom, &mut host).is_err());
        assert_eq!(ram, before);
    }
    let mut ram = actor_ram(0, 0);
    let before = ram.clone();
    assert!(run(Entry::Init, 0, 0, &mut ram, &[], &mut host).is_err());
    assert_eq!(ram, before);
    assert!(host.requests.is_empty());
}
