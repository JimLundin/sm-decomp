use port::{Wram, ridley_subsystem as ridley};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Effect {
    Service(ridley::Service, [u16; 8]),
    Mode7(i16, u8),
    Audio(u8, u16, u8),
    Music(u16, u16),
    Projectile(u16, u16, Option<u16>),
}

#[derive(Default)]
struct RecordingHost {
    effects: Vec<Effect>,
    fail: Option<usize>,
}
impl RecordingHost {
    fn push(&mut self, effect: Effect) -> ridley::Result<()> {
        self.effects.push(effect);
        if self.fail == Some(self.effects.len()) { Err("host failure after acceptance") } else { Ok(()) }
    }
}
impl ridley::Host for RecordingHost {
    fn service(
        &mut self,
        _: &mut Wram<'_>,
        _: &[u8],
        projectile: &mut ridley::ProjectileHost,
        service: ridley::Service,
    ) -> ridley::Result<u32> {
        self.push(Effect::Service(service, projectile.words()))?;
        Ok(0)
    }
    fn mode7_multiply(&mut self, sine: i16, factor: u8) -> ridley::Result<u16> {
        self.push(Effect::Mode7(sine, factor))?;
        Ok(((i32::from(sine) * i32::from(factor as i8)) >> 8) as u16)
    }
    fn ordered_audio(&mut self, channel: u8, sound: u16, maximum: u8) -> ridley::Result<()> {
        self.push(Effect::Audio(channel, sound, maximum))
    }
    fn ordered_music(&mut self, track: u16, delay: u16) -> ridley::Result<()> {
        self.push(Effect::Music(track, delay))
    }
    fn ordered_projectile(&mut self, definition: u16, parameter: u16, slot: Option<u16>) -> ridley::Result<()> {
        self.push(Effect::Projectile(definition, parameter, slot))
    }
}

fn native_rom() -> Vec<u8> {
    let path = std::env::var_os("SM_RIDLEY_ROM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.sfc"));
    std::fs::read(path).expect("pinned native ROM is required for Ridley subsystem controls")
}

fn initial(slot: u16, header: u16) -> Vec<u8> {
    let mut bytes = vec![0; port::WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xe54, slot);
    ram.set_word(0xf78, 0xe13f);
    ram.set_byte(0xfa6, 0xa6);
    ram.set_word(0xf78 + usize::from(slot), header);
    ram.set_byte(0xfa6 + usize::from(slot), 0xa6);
    ram.set_word(0xf92 + usize::from(slot), 0xe73a);
    ram.set_word(0xf94 + usize::from(slot), 1);
    ram.set_word(0xf7a, 160);
    ram.set_word(0xf7e, 160);
    ram.set_word(0xf82, 16);
    ram.set_word(0xf84, 24);
    ram.set_word(0xf86, 0x2000);
    ram.set_word(0xf8c, 10000);
    ram.set_word(0xfa8, 0xa354);
    ram.set_word(0x79f, 6);
    ram.set_word(0x7a5, 32);
    ram.set_word(0x7a7, 32);
    ram.set_word(0xaf6, 128);
    ram.set_word(0xafa, 256);
    ram.set_word(0xafe, 8);
    ram.set_word(0xb00, 16);
    ram.set_word(0x9c2, 99);
    ram.set_word(0x5e5, 0x4abc);
    ram.set_word(0x7802, 1);
    ram.set_word(0x7804, 1);
    ram.set_word(0x780a, 0xda71);
    ram.set_word(0x780c, 1);
    ram.set_word(0x780e, 5);
    ram.set_word(0x7812, 32);
    ram.set_word(0x7818, 3584);
    ram.set_word(0x7838, 15);
    ram.set_word(0x7840, 0);
    ram.set_word(0x7842, 416);
    ram.set_word(0x7844, 32);
    ram.set_word(0x7846, 224);
    ram.set_word(0x8800, 0xbe9c);
    ram.set_word(0x2000, 1);
    ram.set_word(0x2012, 240);
    ram.set_word(0x2014, 1);
    ram.set_word(0x2016, 16368);
    ram.set_word(0x2018, 16448);
    ram.set_word(0x201a, 0xffff);
    ram.set_word(0x201c, 0xffff);
    ram.set_word(0x201e, 16);
    for index in (21..=81).step_by(10) {
        ram.set_word(0x2000 + index * 2, 0x4000);
    }
    bytes
}
fn projectile() -> ridley::ProjectileHost {
    ridley::ProjectileHost::from_words([0x5aa5, 0xa55a, 11, 22, 33, 44, 55, 66])
}
fn process(
    bytes: &mut [u8],
    rom: &[u8],
    host: &mut RecordingHost,
    limit: usize,
) -> ridley::Result<ridley::ScriptReport> {
    ridley::process_instructions(&mut Wram::new(bytes).unwrap(), rom, &mut projectile(), host, limit)
}
fn invoke(bytes: &mut [u8], rom: &[u8], symbol: &str, host: &mut RecordingHost) -> ridley::Result<u32> {
    ridley::dispatch_host(&mut Wram::new(bytes).unwrap(), rom, &mut projectile(), host, symbol)
}

#[test]
fn lifecycle_selection_matches_native_headers_and_ignores_area_naming() {
    let rom = native_rom();
    for (header, accepted, refused) in [
        (0xe13f, "CeresRidley_Main", "Ridley_Main"),
        (0xe17f, "Ridley_Main", "CeresRidley_Main"),
        (0xe1bf, "RidleysExplosion_Main", "Ridley_Main"),
    ] {
        let mut bytes = initial(0, header);
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0x79f, 2);
        assert!(ridley::supports_call(&ram, &rom, accepted).unwrap(), "header={header:04x}");
        assert!(!ridley::supports_call(&ram, &rom, refused).unwrap());
        if header != 0xe1bf {
            assert!(ridley::supports_call(&ram, &rom, "CeresRidley_Init").unwrap());
        }
    }
}

#[test]
fn frame_refusal_preserves_every_wram_byte_all_host_words_and_effects() {
    let rom = native_rom();
    for fault in 0..11 {
        let mut bytes = initial(64, 0xe13f);
        let mut altered = rom.clone();
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0xfd2, 0xe746);
        match fault {
            0 => ram.set_word(0xfd2, 0xe74a),
            1 => ram.set_word(0xfd2, 0xe71c),
            2 => ram.set_word(0xfd2, 0xe91f),
            3 => ram.set_byte(0xfe6, 0xa5),
            4 => ram.set_word(0xfb8, 0xe1bf),
            5 => ram.set_word(0xf78, 0xdead),
            6 => ram.set_byte(0x647, 16),
            7 => ram.set_word(0xe54, 1),
            8 => altered[0x136752] ^= 1,
            9 => altered[0x133d36] ^= 1,
            _ => altered.truncate(0x1000),
        }
        let before = bytes.clone();
        let mut words = projectile();
        let before_words = words;
        let mut host = RecordingHost::default();
        let mut ram = Wram::new(&mut bytes).unwrap();
        assert!(!ridley::supports_call(&ram, &altered, "ProcessEnemyInstructions").unwrap(), "fault={fault}");
        assert!(ridley::process_instructions(&mut ram, &altered, &mut words, &mut host, 20).is_err());
        assert_eq!(bytes, before, "fault={fault}");
        assert_eq!(words, before_words);
        assert!(host.effects.is_empty());
    }
}

#[test]
fn all_actual_scripts_accept_odd_boundaries_and_preserve_callback_zero_aliases() {
    let rom = native_rom();
    for script in [0xe538, 0xe548, 0xe658, 0xe690, 0xe6c8, 0xe6f0, 0xe706, 0xe73a, 0xe91d] {
        for orientation in [0, 2] {
            let mut bytes = initial(64, 0xe13f);
            let mut ram = Wram::new(&mut bytes).unwrap();
            ram.set_word(0xfd2, script);
            ram.set_word(0x7820, orientation);
            assert!(ridley::supports_call(&ram, &rom, "ProcessEnemyInstructions").unwrap(), "script={script:04x}");
            let report = process(&mut bytes, &rom, &mut RecordingHost::default(), 20).unwrap();
            assert!(!report.opcodes.is_empty());
        }
    }
    let mut bytes = initial(64, 0xe13f);
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xfd2, 0xe548);
    ram.set_word(0x787a, 0x5a5a);
    let report = process(&mut bytes, &rom, &mut RecordingHost::default(), 20).unwrap();
    let ram = Wram::new(&mut bytes).unwrap();
    assert_eq!(report.opcodes[1].address, 0xa6e501);
    assert_eq!(ram.word(0x783a), 0);
    assert_eq!(ram.word(0x787a), 0x5a5a);
    assert_eq!(ram.word(0xb10), ram.word(0xaf6));
    assert_eq!((ram.word(0xfd2), ram.word(0xfd4), ram.word(0xfce)), (0xe554, 4, 0xe983));
    assert_eq!(ram.word(0xf92), 0);
}

#[test]
fn frozen_timer_wrap_and_current_slot_sleep_keep_native_field_ownership() {
    let rom = native_rom();
    for (previous, frozen) in [(0_u16, false), (2, false), (0xffff, false), (1, true)] {
        let mut bytes = initial(64, 0xe13f);
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0xfd4, previous);
        ram.set_word(0xfc8, 0x9234);
        if frozen {
            ram.set_word(0xfca, 4);
        }
        let before = bytes.clone();
        let report = process(&mut bytes, &rom, &mut RecordingHost::default(), 20).unwrap();
        if frozen {
            assert!(report.frozen);
            assert_eq!(bytes, before);
        } else {
            let ram = Wram::new(&mut bytes).unwrap();
            assert!(report.timer_only);
            assert_eq!(ram.word(0xfd4), previous.wrapping_sub(1));
            assert_eq!(ram.word(0xfc8), 0x1234);
        }
    }
    let mut bytes = initial(64, 0xe13f);
    Wram::new(&mut bytes).unwrap().set_word(0xfd2, 0xe540);
    let report = process(&mut bytes, &rom, &mut RecordingHost::default(), 20).unwrap();
    let ram = Wram::new(&mut bytes).unwrap();
    assert_eq!(report.opcodes[0].pointer_kind, port::ridley_enemy_instructions::PointerKind::Stop);
    assert_eq!(ram.word(0xfd2), 0xe540);
    assert_eq!(ram.word(0xf92), 0);
}

#[test]
fn direct_callback_requires_its_actual_opcode_argument_and_branch_graph() {
    let rom = native_rom();
    let mut bytes = initial(64, 0xe13f);
    let ram = Wram::new(&mut bytes).unwrap();
    assert!(ridley::supports_callback(&ram, &rom, "Ridley_Instr_1", 64, 0xa6e73c).unwrap());
    for (name, pointer) in [("Ridley_Instr_2", 0xa6e73c), ("Ridley_Instr_1", 0xa6e73e), ("Ridley_Instr_1", 0xa5e73c)] {
        assert!(!ridley::supports_callback(&ram, &rom, name, 64, pointer).unwrap());
    }
    let mut changed = rom.clone();
    changed[0x13673c..0x13673e].copy_from_slice(&0xe74a_u16.to_le_bytes());
    assert!(!ridley::supports_callback(&ram, &changed, "Ridley_Instr_1", 64, 0xa6e73c).unwrap());
}

#[test]
fn accepted_instruction_host_error_propagates_once_after_audio_mutation() {
    let rom = native_rom();
    let mut bytes = initial(0, 0xe13f);
    Wram::new(&mut bytes).unwrap().set_word(0xf92, 0xe746);
    let mut host = RecordingHost { fail: Some(1), ..Default::default() };
    assert_eq!(process(&mut bytes, &rom, &mut host, 20), Err("host failure after acceptance"));
    let ram = Wram::new(&mut bytes).unwrap();
    assert_eq!(ram.word(0xf94), 0);
    assert_eq!(ram.word(0x781e), 89);
    assert_eq!(host.effects, vec![Effect::Audio(1, 0x59, 6)]);
}

#[test]
fn frame_execution_limit_refuses_before_timer_or_callback_changes() {
    let rom = native_rom();
    let mut bytes = initial(0, 0xe13f);
    let before = bytes.clone();
    let mut host = RecordingHost::default();
    assert!(process(&mut bytes, &rom, &mut host, 0).is_err());
    assert_eq!(bytes, before);
    assert!(host.effects.is_empty());
}

#[test]
fn explosion_initialization_rejects_bad_parameter_before_any_field_write() {
    let rom = native_rom();
    for parameter in [1, 23, 0xffff] {
        let mut bytes = initial(64, 0xe1bf);
        Wram::new(&mut bytes).unwrap().set_word(0xff4, parameter);
        let before = bytes.clone();
        let mut host = RecordingHost::default();
        assert!(invoke(&mut bytes, &rom, "RidleysExplosion_Init", &mut host).is_err());
        assert_eq!(bytes, before);
        assert!(host.effects.is_empty());
    }
}

#[test]
fn accepted_terminal_service_error_is_not_retried_or_followed_by_effects() {
    let rom = native_rom();
    let mut bytes = initial(64, 0xe1bf);
    Wram::new(&mut bytes).unwrap().set_word(0xff2, 0);
    let mut host = RecordingHost { fail: Some(1), ..Default::default() };
    let mut words = projectile();
    assert_eq!(
        ridley::dispatch_host(
            &mut Wram::new(&mut bytes).unwrap(),
            &rom,
            &mut words,
            &mut host,
            "RidleysExplosion_Main",
        ),
        Err("host failure after acceptance")
    );
    assert_eq!(
        host.effects,
        vec![Effect::Service(ridley::Service::EnemyDeath { slot: 64, animation: 0 }, projectile().words())]
    );
    assert_eq!(Wram::new(&mut bytes).unwrap().word(0xff2), 0xffff);
}

#[test]
fn samus_service_only_changes_reviewed_handlers_before_protected_graphics() {
    struct SamusHost;
    impl ridley::Host for SamusHost {
        fn service(
            &mut self,
            ram: &mut Wram<'_>,
            _: &[u8],
            _: &mut ridley::ProjectileHost,
            service: ridley::Service,
        ) -> ridley::Result<u32> {
            assert_eq!(service, ridley::Service::SamusCode { code: 1 });
            ram.set_word(0xa42, 0xe695);
            ram.set_word(0xa44, 0xe725);
            for at in [0xa28, 0xa2a, 0xa2c] {
                ram.set_word(at, 0xffff);
            }
            for at in [0xa2e, 0xa30, 0xa32] {
                ram.set_word(at, 0);
            }
            Ok(0xffff)
        }
        fn mode7_multiply(&mut self, sine: i16, factor: u8) -> ridley::Result<u16> {
            Ok(((i32::from(sine) * i32::from(factor as i8)) >> 8) as u16)
        }
        fn ordered_audio(&mut self, _: u8, _: u16, _: u8) -> ridley::Result<()> {
            Ok(())
        }
        fn ordered_music(&mut self, _: u16, _: u16) -> ridley::Result<()> {
            Ok(())
        }
        fn ordered_projectile(&mut self, _: u16, _: u16, _: Option<u16>) -> ridley::Result<()> {
            Ok(())
        }
    }
    let rom = native_rom();
    let mut bytes = initial(0, 0xe13f);
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xfa8, 0xbbf1);
    ram.set_word(0xfb2, 0);
    ram.set_word(0x7836, 1);
    ridley::dispatch_host(&mut ram, &rom, &mut projectile(), &mut SamusHost, "CeresRidley_Main").unwrap();
    assert_eq!(ram.word(0x7836), 0);
    assert_eq!(ram.word(0xa28), 0xffff);
    assert_eq!(ram.word(0xa42), 0xe695);
    assert_eq!(ram.word(0x330), 28);
}

#[test]
fn explosion_current_slot_keeps_boss_state_and_signed_velocity_rounding() {
    let rom = native_rom();
    let mut bytes = initial(64, 0xe1bf);
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xfba, 100);
    ram.set_word(0xfbe, 200);
    ram.set_word(0xfea, 0xff80);
    ram.set_word(0xfec, 0x00fe);
    ram.set_byte(0xfbd, 250);
    ram.set_byte(0xfc1, 250);
    ram.set_word(0xff2, 2);
    let boss = bytes[0xf78..0xfb8].to_vec();
    invoke(&mut bytes, &rom, "RidleysExplosion_Main", &mut RecordingHost::default()).unwrap();
    let ram = Wram::new(&mut bytes).unwrap();
    assert_eq!((ram.word(0xfba), ram.word(0xfbe)), (100, 201));
    assert_eq!((ram.word(0xfea), ram.word(0xfec)), (0xff84, 0x0102));
    assert_eq!(ram.byte(0xfbd), 126);
    assert_eq!(ram.byte(0xfc1), 252);
    assert_eq!(&bytes[0xf78..0xfb8], boss.as_slice());
}

#[test]
fn ordered_tail_mode7_transactions_precede_following_graphics_effects() {
    let rom = native_rom();
    let mut bytes = initial(0, 0xe13f);
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x2000, 1);
    ram.set_word(0x2002, 0);
    ram.set_word(0x2004, 0);
    ram.set_word(0x2010, 0);
    ram.set_word(0x202a, 0x4000);
    for segment in 0..7 {
        ram.set_word(0x2000 + (16 + segment * 10) * 2, 0x8000);
        ram.set_word(0x2000 + (17 + segment * 10) * 2, 0xffff);
        ram.set_word(0x2000 + (19 + segment * 10) * 2, 0x1800);
        ram.set_word(0x2000 + (21 + segment * 10) * 2, 0);
    }
    let mut host = RecordingHost::default();
    invoke(&mut bytes, &rom, "Ridley_Func_70", &mut host).unwrap();
    assert_eq!(host.effects.len(), 14);
    for (index, pair) in host.effects.as_chunks::<2>().0.iter().enumerate() {
        let angle = if index == 0 { 1_usize } else { 2 };
        let native_sine = |phase: usize| {
            let p = 0x1033c3 + (angle + phase) * 2;
            i16::from_le_bytes([rom[p], rom[p + 1]])
        };
        assert_eq!(pair, &[Effect::Mode7(native_sine(64), 24), Effect::Mode7(native_sine(128), 24)]);
    }
    let ram = Wram::new(&mut bytes).unwrap();
    assert_eq!((ram.word(0x2030), ram.word(0x2032)), (0, 23));
}

#[test]
fn services_preserve_exact_population_order_and_stop_on_accepted_error() {
    let rom = native_rom();
    for failing in [false, true] {
        let mut bytes = initial(0, 0xe13f);
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0xfa8, 0xa9a0);
        ram.set_word(0xfb2, 0);
        ram.set_word(0x93f, 1);
        let mut host = RecordingHost { fail: failing.then_some(1), ..Default::default() };
        let result = ridley::dispatch_fixture(
            &mut Wram::new(&mut bytes).unwrap(),
            &rom,
            &mut projectile(),
            &mut host,
            "CeresRidley_Main",
            0,
            0,
        );
        let expected =
            Effect::Service(ridley::Service::SpawnEnemy { bank: 0xa6, population: 0xaa2f }, projectile().words());
        assert_eq!(host.effects[0], expected);
        if failing {
            assert_eq!(result, Err("host failure after acceptance"));
            assert_eq!(host.effects.len(), 1);
        } else {
            result.unwrap();
            assert_eq!(
                host.effects[1],
                Effect::Service(ridley::Service::SpawnEnemy { bank: 0xa6, population: 0xaa3f }, projectile().words())
            );
            assert_eq!(host.effects.len(), 2);
        }
    }
}

#[test]
fn controlled_raw_queue_overlap_keeps_word_store_order_and_cursor_wrap() {
    let rom = native_rom();
    let mut bytes = initial(0, 0xe13f);
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x330, 0xfffc);
    ram.set_word(0x780c, 1);
    ram.set_word(0x780a, 0xda71);
    ridley::dispatch_fixture(
        &mut ram,
        &rom,
        &mut projectile(),
        &mut RecordingHost::default(),
        "CeresRidley_Main",
        0,
        0,
    )
    .unwrap();
    assert_eq!(ram.word(0x330), 10);
    assert_eq!(ram.word(0x100cc), 64);
    assert_eq!(ram.word(0x100ce), 0x9840);
    assert_eq!(ram.byte(0x100d0), 0xb0);
}

#[test]
fn lifecycle_refuses_foreign_allocation_and_corrupt_domains_immutably() {
    let rom = native_rom();
    for fault in 0..12 {
        let mut bytes = initial(0, 0xe13f);
        let mut ram = Wram::new(&mut bytes).unwrap();
        match fault {
            0 => ram.set_word(0xfa8, 0xa9a0),
            1 => ram.set_word(0xfa8, 0xc5da),
            2 => ram.set_word(0xfa8, 0xdead),
            3 => ram.set_word(0x7820, 3),
            4 => ram.set_word(0x2000, 9),
            5 => ram.set_word(0x780a, 0xda73),
            6 => ram.set_word(0x330, 0x25b),
            7 => ram.set_word(0x334, 0xffff),
            8 => ram.set_word(0x590, 0x300),
            9 => ram.set_word(0x783a, 3),
            10 => ram.set_word(0x780e, 10),
            _ => ram.set_word(0x8800, 0xdead),
        }
        let before = bytes.clone();
        let mut words = projectile();
        let before_words = words;
        let mut host = RecordingHost::default();
        assert!(ridley::dispatch_host(
            &mut Wram::new(&mut bytes).unwrap(), &rom, &mut words, &mut host, "CeresRidley_Main",
        ).is_err(), "fault={fault}");
        assert_eq!(bytes, before);
        assert_eq!(words, before_words);
        assert!(host.effects.is_empty());
    }
}

#[test]
fn complete_controlled_attack_chains_are_strictly_admitted_for_both_orientations() {
    let rom = native_rom();
    for orientation in [0, 2] {
        let mut bytes = initial(0, 0xe13f);
        Wram::new(&mut bytes).unwrap().set_word(0x7820, orientation);
        let mut host = RecordingHost::default();
        let mut words = projectile();
        let before_words = words;
        let mut slept = false;
        for _ in 0..240 {
            let report =
                ridley::process_instructions(&mut Wram::new(&mut bytes).unwrap(), &rom, &mut words, &mut host, 20)
                    .unwrap();
            if report
                .opcodes
                .iter()
                .any(|opcode| opcode.pointer_kind == port::ridley_enemy_instructions::PointerKind::Stop)
            {
                slept = true;
                break;
            }
        }
        assert!(slept);
        assert_eq!(words, before_words);
        assert_eq!(
            host.effects.iter().filter(|effect| matches!(effect, Effect::Projectile(0x9642, _, Some(_)))).count(),
            8
        );
        assert_eq!(host.effects.iter().filter(|effect| matches!(effect, Effect::Audio(1, 0x59, 6))).count(), 2);
    }
}

/// Existing immutable actual-C captures are optional for an ordinary Cargo
/// checkout, but the subsystem verification gate supplies this manifest.
#[test]
fn retained_native_frames_and_callbacks_pass_strict_admission_and_full_state_parity() {
    let Some(manifest) = std::env::var_os("SM_RIDLEY_CAPTURE_MANIFEST") else { return };
    let manifest = std::fs::read_to_string(manifest).unwrap();
    let rom = native_rom();
    let quoted = |line: &str| line.split_once('=').unwrap().1.trim().trim_matches('"').to_owned();
    let integer = |block: &str, key: &str| -> Option<u32> {
        block
            .lines()
            .find(|line| line.starts_with(&format!("{key} = ")))
            .map(|line| line.split_once('=').unwrap().1.trim().parse().unwrap())
    };
    let path =
        |block: &str, key: &str| quoted(block.lines().find(|line| line.starts_with(&format!("{key} = "))).unwrap());
    let mut captures = 0;
    for routine in manifest.split("[[routines]]").skip(1) {
        let symbol = quoted(routine.lines().find(|line| line.starts_with("symbol = ")).unwrap());
        for block in routine.split("[[routines.cases]]").skip(1) {
            let name = path(block, "name");
            let mut bytes = std::fs::read(path(block, "before_file")).unwrap();
            let expected = std::fs::read(path(block, "expected_after_file")).unwrap();
            let host_text = block.split_once("host_words = [").unwrap().1.split_once(']').unwrap().0;
            let values = host_text
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.parse().unwrap())
                .collect::<Vec<u16>>();
            let mut words = ridley::ProjectileHost::from_words(values.try_into().unwrap());
            let before_words = words;
            let mut host = RecordingHost::default();
            let result = if symbol == "ProcessEnemyInstructions" {
                ridley::process_instructions(&mut Wram::new(&mut bytes).unwrap(), &rom, &mut words, &mut host, 20)
                    .map(|_| 0)
            } else {
                ridley::dispatch_callback(
                    &mut Wram::new(&mut bytes).unwrap(),
                    &rom,
                    &mut words,
                    &mut host,
                    &symbol,
                    integer(block, "a").unwrap() as i32,
                    integer(block, "b").unwrap() as i32,
                )
            };
            assert_eq!(result.unwrap(), integer(block, "expected_return").unwrap_or(0), "{name}");
            assert_eq!(bytes, expected, "{name}");
            assert_eq!(words, before_words, "{name}");
            captures += 1;
        }
    }
    assert_eq!(captures, 90);
}
