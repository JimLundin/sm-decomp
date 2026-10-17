//! Interrupt sampling before the final CPU cycle (Super Famicom Dev, Timing).
use rustysnes_core::{Bus, cart::Region, cpu::Bus as CpuBus};

#[test]
fn nmi_edge_during_final_cycle_waits_until_next_instruction() {
    let mut bus = Bus::new(Region::Ntsc);
    bus.write24(0x4200, 0x80);
    // Finish advancing through complete cycles just before VBlank.
    while bus.ppu.scanline() < 224 || bus.ppu.dot() < 339 {
        bus.begin_cpu_cycle(2);
        bus.advance(2);
    }
    assert!(!bus.poll_nmi());
    bus.begin_cpu_cycle(8); // final cycle begins before the NMI edge
    bus.advance(8);
    assert_eq!(bus.ppu.scanline(), 225);
    assert!(!bus.poll_nmi(), "edge after the final-cycle sample must not preempt the next instruction");
    bus.begin_cpu_cycle(8); // following instruction observes the pending edge
    bus.advance(8);
    bus.begin_cpu_cycle(6); // sample before that instruction's final cycle
    bus.advance(6);
    assert!(bus.poll_nmi());
    assert!(!bus.poll_nmi(), "the sampled NMI edge is consumed exactly once");
}

#[test]
fn save_restore_preserves_a_sampled_interrupt_before_it_is_consumed() {
    use rustysnes_core::System;
    let mut system = System::new(0);
    system.bus.write24(0x4200, 0x80);
    while system.bus.ppu.scanline() < 225 {
        system.bus.advance(8);
    }
    system.bus.advance(8);
    system.bus.begin_cpu_cycle(8);
    system.bus.advance(8);
    let saved = system.save_state();
    let mut restored = System::new(0);
    restored.load_state(&saved).unwrap();
    assert!(restored.bus.poll_nmi());
    assert!(!restored.bus.poll_nmi());
    assert_eq!(restored.bus.clock.master, system.bus.clock.master);
}

#[test]
fn synthetic_nmi_handler_matches_recording_core_across_instruction_phases() {
    use rustysnes_core::{System, cart::Cart};
    // Original rr2-beta23 black-box captures of eight start phases. The
    // handler and returned PC distinguish a delayed edge from a slower cycle.
    for (phase, handler, return_pc) in [
        (0, 94, 0x8009),
        (1, 88, 0x800c),
        (2, 88, 0x800c),
        (3, 86, 0x800c),
        (4, 80, 0x800f),
        (5, 80, 0x800f),
        (6, 78, 0x800f),
        (7, 92, 0x8010),
    ] {
        let mut rom = vec![0; 0x10000];
        let mut code = vec![0x18, 0xfb, 0xe2, 0x30, 0xa9, 0x80, 0x8d, 0, 0x42];
        code.extend(std::iter::repeat_n(0xea, phase));
        code.extend([0xea, 0xea, 0x80, 0xfc]);
        rom[..code.len()].copy_from_slice(&code);
        rom[0x1000] = 0x40;
        rom[0x7fc0..0x7fd5].copy_from_slice(b"SM NMI ORACLE TEST   ");
        rom[0x7fd5] = 0x20;
        rom[0x7fd9] = 1;
        rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
        rom[0x7fea..0x7fec].copy_from_slice(&0x9000u16.to_le_bytes());
        rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        let mut system = System::new(0);
        system.bus.cart = Some(Cart::from_rom(&rom).unwrap());
        system.reset();
        for _ in 0..30_000 {
            if system.cpu.regs.pc == 0x9000 {
                break;
            }
            system.cpu.step(&mut system.bus);
        }
        assert_eq!(system.cpu.regs.pc, 0x9000);
        assert_eq!(system.bus.clock.master, 225 * 1364 + handler, "phase {phase}");
        system.cpu.step(&mut system.bus);
        assert_eq!(system.cpu.regs.pc, return_pc, "phase {phase}");
        assert_eq!(system.bus.clock.master, 225 * 1364 + handler + 52, "phase {phase}");
    }
}

#[test]
fn refresh_pause_follows_the_eight_clock_phase_from_reset() {
    // Super Famicom Dev Timing: first refresh H538, later starts use the
    // same eight-clock divider and land closest to H536 (H534/H538).
    let mut bus = Bus::new(Region::Ntsc);
    bus.advance(537);
    assert_eq!(bus.clock.master, 537);
    bus.advance(1);
    assert_eq!(bus.clock.master, 578);
    bus.advance(1364 + 533 - 578);
    assert_eq!(bus.clock.master, 1364 + 533);
    bus.advance(1);
    assert_eq!(bus.clock.master, 1364 + 574);
}

macro_rules! emitted_phase {
    ($name:ident,$file:literal) => {
        mod $name {
            #![allow(unused_imports)]
            include!(concat!(env!("OUT_DIR"), $file));
        }
    };
}
emitted_phase!(phase0, "/nmi_phase_0.rs");
emitted_phase!(phase1, "/nmi_phase_1.rs");
emitted_phase!(phase2, "/nmi_phase_2.rs");
emitted_phase!(phase3, "/nmi_phase_3.rs");
emitted_phase!(phase4, "/nmi_phase_4.rs");
emitted_phase!(phase5, "/nmi_phase_5.rs");
emitted_phase!(phase6, "/nmi_phase_6.rs");
emitted_phase!(phase7, "/nmi_phase_7.rs");

#[test]
fn emitted_patterns_match_all_independent_nmi_phase_oracles() {
    for (phase, steps, handler, return_pc) in [
        (0, phase0::STEP_TABLE, 94, 0x8009),
        (1, phase1::STEP_TABLE, 88, 0x800c),
        (2, phase2::STEP_TABLE, 88, 0x800c),
        (3, phase3::STEP_TABLE, 86, 0x800c),
        (4, phase4::STEP_TABLE, 80, 0x800f),
        (5, phase5::STEP_TABLE, 80, 0x800f),
        (6, phase6::STEP_TABLE, 78, 0x800f),
        (7, phase7::STEP_TABLE, 92, 0x8010),
    ] {
        let mut rom = vec![0; 0x10000];
        let mut code = vec![0x18, 0xfb, 0xe2, 0x30, 0xa9, 0x80, 0x8d, 0, 0x42];
        code.extend(std::iter::repeat_n(0xea, phase));
        code.extend([0xea, 0xea, 0x80, 0xfc]);
        rom[..code.len()].copy_from_slice(&code);
        rom[0x1000] = 0x40;
        rom[0x7fc0..0x7fd5].copy_from_slice(b"SM NMI ORACLE TEST   ");
        rom[0x7fd5] = 0x20;
        rom[0x7fd9] = 1;
        rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
        rom[0x7fea..0x7fec].copy_from_slice(&0x9000u16.to_le_bytes());
        rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        let mut subject = snes::Subject::with_steps(&rom, 0, steps).unwrap();
        let mut observed = None;
        let mut returned = None;
        for _ in 0..2 {
            subject
                .run_frame_inspected(0, |boundary, _| {
                    if boundary.pc == 0x9000 && observed.is_none() {
                        observed = Some(boundary.master_clock);
                    } else if observed.is_some() && returned.is_none() {
                        returned = Some((boundary.master_clock, boundary.pc));
                    }
                })
                .unwrap();
        }
        assert_eq!(observed, Some(225 * 1364 + handler), "phase {phase}");
        assert_eq!(returned, Some((225 * 1364 + handler + 52, return_pc)), "phase {phase}");
        assert!(subject.generated_steps > 0);
    }
}
