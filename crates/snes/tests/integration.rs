use runtime::{Bus, Machine};
use snes::{InterpreterFallback, Reference, Subject};

mod emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/test_steps.rs"));
}

mod nmi_emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/nmi_steps.rs"));
}

// A synthetic cartridge, independent of Super Metroid and its ROM-derived output.
fn cartridge(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0; 0x10000];
    rom[..program.len()].copy_from_slice(program);
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM RECOMP TEST       ");
    rom[0x7fd5] = 0x20; // LoROM
    rom[0x7fd9] = 1; // NTSC
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    rom
}

#[test]
fn fallback_subject_matches_reference_through_boot_calls_and_frames() {
    let mut rom = cartridge(&[
        0x18, 0xfb, // CLC, XCE: native mode
        0xc2, 0x30, // REP: 16-bit registers
        0x20, 0x20, 0x80, // JSR $8020
        0x80, 0xfb, // repeat the JSR
    ]);
    rom[0x20..0x24].copy_from_slice(&[0xee, 0x10, 0x00, 0x60]); // INC $10, RTS
    let mut reference = Reference::new(&rom, 0).unwrap();
    let mut subject = Subject::fallback_only(&rom, 0).unwrap();
    for pad in [0, 0x1000, 0x8000] {
        reference.run_frame(pad).unwrap();
        subject.run_frame(pad).unwrap();
        assert_eq!(reference.wram(), subject.wram());
    }
    assert!(subject.misses_csv().contains("008020,m0x0,"));
    assert_ne!(&subject.wram()[0x10..0x12], &[0, 0]);
}

#[test]
fn compiled_call_to_fallback_shares_hardware_widths_and_balances_stack() {
    // Native fallback mutates 16-bit RAM, nests a call in its bank, then returns long.
    let mut rom = cartridge(&[0x80, 0xfe]);
    rom[0x20..0x27].copy_from_slice(&[0xee, 0x10, 0x00, 0x20, 0x30, 0x80, 0x6b]);
    rom[0x30..0x37].copy_from_slice(&[0xa9, 0x78, 0x56, 0x8d, 0x14, 0x00, 0x60]);
    let subject = Subject::fallback_only(&rom, 0).unwrap();
    fn compiled(m: &mut Machine) {
        m.cpu.a = 0x1234;
        m.write(0x10, m.cpu.a, true);
        m.push8(m.cpu.pb);
        m.push16(0x8005);
        m.call(0x808020);
        m.assume_mx(0, 0x808000);
        m.write(0x12, m.cpu.a, true);
        m.pull16();
    }
    let mut m = Machine::new(Box::new(subject.bus()), &[(0x808000, 0, compiled)], Some(Box::new(InterpreterFallback)));
    m.cpu.s = 0x1fff;
    m.cpu.set_p(0);
    m.push16(0xffff);
    m.call(0x808000);
    assert_eq!(m.read(0x10, true), 0x1235);
    assert_eq!(m.read(0x12, true), 0x5678);
    assert_eq!(m.read(0x14, true), 0x5678);
    assert_eq!(m.cpu.s, 0x1fff);
    assert_eq!(m.cpu.pb, 0);
    assert_eq!(m.generated_calls, 1);
    assert_eq!(m.misses.len(), 1);
}

#[test]
fn runtime_bus_routes_rom_wram_and_hardware_registers() {
    let rom = cartridge(&[0x80, 0xfe]);
    let subject = Subject::fallback_only(&rom, 0).unwrap();
    let mut bus = subject.bus();
    assert_eq!(bus.read(0x808000), 0x80);
    bus.write(0x7e0010, 0xab);
    assert_eq!(bus.read(0x10), 0xab); // WRAM mirror
    bus.write(0x2181, 0x20); // WRAM data-port address
    bus.write(0x2182, 0);
    bus.write(0x2183, 0);
    bus.write(0x2180, 0xcd);
    assert_eq!(bus.read(0x7e0020), 0xcd);
}

#[test]
fn replay_refuses_another_rom() {
    let movie = movie::Movie { frames: vec![0], rom_sha256: Some("0".repeat(64)) };
    assert!(snes::verify_movie_rom(&movie, &cartridge(&[0x80, 0xfe])).is_err());
}

#[test]
fn emitted_steps_match_cpu_clocks_ram_and_video_across_frames_and_nmi() {
    let mut rom = cartridge(&[
        0x18, 0xfb, 0xc2, 0x30, 0xa9, 0x34, 0x12, 0x8d, 0x10, 0x00, 0xe2, 0x20, 0xa9, 0x80, 0x8d, 0x00, 0x42, 0xa2,
        0x03, 0x00, 0xca, 0xd0, 0xfd, 0xa9, 0xab, 0x8d, 0x12, 0x00, 0x80, 0xf3,
    ]);
    rom[0x100..0x104].copy_from_slice(&[0xee, 0x14, 0x00, 0x40]); // INC, RTI
    rom[0x7fea..0x7fec].copy_from_slice(&0x8100u16.to_le_bytes());
    let mut reference = Reference::new(&rom, 0).unwrap();
    let mut subject = Subject::with_steps(&rom, 0, emitted::STEP_TABLE).unwrap();
    for _ in 0..3 {
        reference.run_frame(0).unwrap();
        subject.run_frame(0).unwrap();
        assert_eq!(reference.state_bytes(), subject.state_bytes());
        assert_eq!(reference.wram(), subject.wram());
        assert_eq!(reference.framebuffer_bytes(), subject.framebuffer_bytes());
    }
    assert!(subject.generated_steps > 100);
    assert!(subject.interpreted_steps > 0);
    assert_eq!(subject.wram()[0x10..0x13], [0x34, 0x12, 0xab]);
    assert!(subject.wram()[0x14] >= 3, "NMI fired and returned each frame");
}

#[test]
fn generated_implied_cycle_preserves_interrupt_dependent_read_timing() {
    let mut rom = cartridge(&[0x18, 0xfb, 0xe2, 0x30, 0xa9, 0x80, 0x8d, 0, 0x42, 0xea, 0xea, 0xea, 0x80, 0xfc]);
    rom[0x1000] = 0x40;
    rom[0x7fea..0x7fec].copy_from_slice(&0x9000u16.to_le_bytes());
    let mut reference = Reference::new(&rom, 0).unwrap();
    let mut subject = Subject::with_steps(&rom, 0, nmi_emitted::STEP_TABLE).unwrap();
    for _ in 0..3 {
        reference.run_frame(0).unwrap();
        subject.run_frame(0).unwrap();
        assert_eq!(reference.state_bytes(), subject.state_bytes());
        assert_eq!(reference.wram(), subject.wram());
    }
    assert!(subject.generated_steps > 100);
}

#[test]
fn wrong_generated_body_is_visible_in_frame_comparison() {
    fn wrong(m: &mut Machine) {
        m.cpu.a = 0xdead;
    }
    let rom = cartridge(&[0x18, 0xfb, 0xa9, 0x12, 0x8d, 0x10, 0x00, 0x80, 0xf9]);
    let mut reference = Reference::new(&rom, 0).unwrap();
    let mut subject = Subject::with_steps(&rom, 0, &[(0x8002, 3, &[0xa9, 0x12], wrong)]).unwrap();
    reference.run_frame(0).unwrap();
    subject.run_frame(0).unwrap();
    assert_ne!(reference.wram()[0x10], subject.wram()[0x10]);
    assert!(subject.generated_steps > 0);
}

#[test]
fn compiled_bytes_from_another_revision_are_left_interpreted() {
    fn wrong(_: &mut Machine) {
        panic!("stale body must never execute");
    }
    let rom = cartridge(&[0x18, 0xfb, 0xa9, 0x12, 0x8d, 0x10, 0x00, 0x80, 0xf9]);
    let mut reference = Reference::new(&rom, 0).unwrap();
    let mut subject = Subject::with_steps(&rom, 0, &[(0x8002, 3, &[0xa9, 0xff], wrong)]).unwrap();
    reference.run_frame(0).unwrap();
    subject.run_frame(0).unwrap();
    assert_eq!(reference.state_bytes(), subject.state_bytes());
    assert_eq!(reference.wram(), subject.wram());
    assert_eq!(subject.generated_steps, 0);
}
