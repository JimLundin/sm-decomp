use snes::Subject;

fn cartridge(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0; 0x10000];
    rom[..program.len()].copy_from_slice(program);
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM RECOMP TEST       ");
    rom[0x7fd5] = 0x20;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    rom
}

#[test]
fn compiled_inventory_names_executed_and_unexecuted_keys() {
    fn nop(m: &mut runtime::Machine) {
        m.bus.implied_cycle(0x8003, m.cpu.flag(runtime::flag::I));
    }
    let rom = cartridge(&[0x18, 0xfb, 0xea, 0x80, 0xfd]);
    let mut subject = Subject::with_steps(&rom, 0, &[(0x8002, 3, &[0xea], nop), (0x8020, 3, &[0xea], nop)]).unwrap();
    subject.run_frame(0).unwrap();
    let report = subject.coverage();
    assert!(report.compiled.iter().find(|entry| entry.address == 0x8002).unwrap().count > 0);
    assert_eq!(report.compiled.iter().find(|entry| entry.address == 0x8020).unwrap().count, 0);
    assert_eq!(report.generated_instructions, subject.generated_steps);
    assert!(report.interpreted.iter().any(|entry| entry.address == 0x8003 && entry.opcode == 0x80));
}

#[test]
fn interrupt_takeover_does_not_count_the_discarded_opcode_as_generated() {
    fn nop(m: &mut runtime::Machine) {
        m.bus.implied_cycle(0x800b, m.cpu.flag(runtime::flag::I));
    }
    let mut rom = cartridge(&[0x18, 0xfb, 0xe2, 0x30, 0xa9, 0x80, 0x8d, 0x00, 0x42, 0xcb, 0xea, 0x80, 0xfc]);
    rom[0x100..0x103].copy_from_slice(&[0xee, 0x10, 0x00]);
    rom[0x103] = 0x40;
    rom[0x7fea..0x7fec].copy_from_slice(&0x8100u16.to_le_bytes());
    let mut subject = Subject::with_steps(&rom, 0, &[(0x800a, 3, &[0xea], nop)]).unwrap();
    for _ in 0..3 {
        subject.run_frame(0).unwrap();
    }
    let report = subject.coverage();
    assert!(report.noninstruction_dispatches > 0);
    assert_eq!(report.generated_instructions, subject.generated_steps);
    assert!(report.compiled[0].count > 0);
    assert!(report.interpreted.iter().any(|entry| entry.emulation));
}
