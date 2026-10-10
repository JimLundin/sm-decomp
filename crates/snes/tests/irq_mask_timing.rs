//! Black-box IRQ mask tests; public specification: Super Famicom Dev, Timing.
use rustysnes_core::{System, cart::Cart};

fn cartridge(case: usize) -> Vec<u8> {
    let mut rom = vec![0; 0x10000];
    let code = [
        0x18, 0xfb, 0xe2, 0x30, 0xa9, 1, 0x8d, 0x0d, 0x42, 0x8d, 0x10, 0, 0xa9, 0x98, 0x8d, 7, 0x42, 0xa9, 0, 0x8d, 8,
        0x42, 0x8d, 0x0a, 0x42, 0xa9, 0, 0x8d, 9, 0x42, 0xa9, 0x30, 0x8d, 0, 0x42, 0x5c, 0x60, 0x80, 0x80,
    ];
    rom[..code.len()].copy_from_slice(&code);
    rom[0x60..0x7e].fill(0xea);
    let unmask: &[u8] = match case {
        0 => &[0x58],                   // CLI: next NOP completes before IRQ
        1 => &[0xc2, 4],                // REP: likewise
        2 => &[0xa9, 0x30, 0x48, 0x28], // PLP pulls I=0
        3 => &[0x58, 0x78],             // SEI uses old I=0, then pushes new I=1
        4 => &[0x58, 0xe2, 4],          // SEP uses old I=0
        _ => unreachable!(),
    };
    rom[0x7e..0x7e + unmask.len()].copy_from_slice(unmask);
    rom[0x7e + unmask.len()..0x81 + unmask.len()].copy_from_slice(&[0xea, 0x80, 0xfe]);
    rom[0x1000..0x1004].copy_from_slice(&[0xad, 0x11, 0x42, 0x40]);
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM IRQ DELAY TEST    ");
    rom[0x7fd5] = 0x30;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7fee..0x7ff0].copy_from_slice(&0x9000u16.to_le_bytes());
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    rom
}

#[test]
fn irq_uses_the_mask_before_cli_rep_plp_sei_and_sep() {
    // rr2-beta23 / bsnes085Compatibility captures. Handler time, stacked
    // return address and stacked I distinguish mask sampling from latency.
    for (case, clock, return_pc, masked) in [
        (0, 1044, 0x8080, false),
        (1, 1050, 0x8081, false),
        (2, 1090, 0x8083, false),
        (3, 1044, 0x8080, true),
        (4, 1050, 0x8081, true),
    ] {
        let mut s = System::new(0);
        s.bus.cart = Some(Cart::from_rom(&cartridge(case)).unwrap());
        s.reset();
        for _ in 0..200 {
            if s.cpu.regs.pc == 0x9000 {
                break;
            }
            s.cpu.step(&mut s.bus);
        }
        assert_eq!((s.cpu.regs.pc, s.bus.clock.master), (0x9000, clock), "case {case}");
        let stacked_pc = u16::from_le_bytes([s.bus.peek_wram(0x1fd), s.bus.peek_wram(0x1fe)]);
        assert_eq!(stacked_pc, return_pc, "case {case}");
        assert_eq!(s.bus.peek_wram(0x1fc) & 4 != 0, masked, "case {case}");
    }
}

#[test]
fn save_restore_retains_the_old_mask_after_cli() {
    let mut s = System::new(0);
    s.bus.cart = Some(Cart::from_rom(&cartridge(0)).unwrap());
    s.reset();
    while s.cpu.regs.pc != 0x807f {
        s.cpu.step(&mut s.bus);
    }
    let mut restored = System::new(0);
    restored.bus.cart = Some(Cart::from_rom(&cartridge(0)).unwrap());
    restored.load_state(&s.save_state()).unwrap();
    for _ in 0..2 {
        s.cpu.step(&mut s.bus);
        restored.cpu.step(&mut restored.bus);
        assert_eq!(s.save_state(), restored.save_state());
    }
    assert_eq!((restored.cpu.regs.pc, restored.bus.clock.master), (0x9000, 1044));
}

mod emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/irq_steps.rs"));
}

mod emitted_rep {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/irq_rep_steps.rs"));
}

mod emitted_sep {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/irq_sep_steps.rs"));
}

#[test]
fn generated_rep_and_sep_preserve_the_old_mask() {
    for (case, steps) in [(1, emitted_rep::STEP_TABLE), (4, emitted_sep::STEP_TABLE)] {
        let rom = cartridge(case);
        let mut reference = snes::Reference::new(&rom, 0).unwrap();
        let mut subject = snes::Subject::with_steps(&rom, 0, steps).unwrap();
        reference.run_frame(0).unwrap();
        subject.run_frame(0).unwrap();
        assert_eq!(reference.state_bytes(), subject.state_bytes(), "case {case}");
        assert_eq!(reference.wram(), subject.wram(), "case {case}");
        assert!(subject.generated_steps > 0);
    }
}

#[test]
fn generated_nop_after_cli_consumes_the_same_mask_sample() {
    let rom = cartridge(0);
    let mut reference = snes::Reference::new(&rom, 0).unwrap();
    let mut subject = snes::Subject::with_steps(&rom, 0, emitted::STEP_TABLE).unwrap();
    reference.run_frame(0).unwrap();
    subject.run_frame(0).unwrap();
    assert_eq!(reference.state_bytes(), subject.state_bytes());
    assert_eq!(reference.wram(), subject.wram());
    assert!(subject.generated_steps > 0);
}
