//! Public DMA timing vectors: Super Famicom Dev, "Timing", DMA section.
use rustysnes_core::{Bus, cart::Region, cpu::Bus as CpuBus};

fn bus_at(phase: u64) -> Bus {
    let mut bus = Bus::new(Region::Ntsc);
    bus.clock.master = phase;
    // One direct channel, three bytes to an APU port. No frame/refresh edge.
    bus.dma.write_reg(0, 0, 0);
    bus.dma.write_reg(0, 1, 0x40);
    bus.dma.write_reg(0, 2, 0);
    bus.dma.write_reg(0, 3, 0);
    bus.dma.write_reg(0, 4, 0x7e);
    bus.dma.write_reg(0, 5, 3);
    bus.dma.write_reg(0, 6, 0);
    bus
}

#[test]
fn dma_waits_one_cycle_then_aligns_entry_and_exit() {
    // Documentation's STA $420B : NOP example: at pause phases 6,4,2,0,
    // align by 2,4,6,8 clocks, then 40 transfer clocks, then 6,4,2,6.
    for (pause_phase, expected) in [(6, 48), (4, 48), (2, 48), (0, 54)] {
        let mut bus = bus_at(pause_phase);
        bus.write24(0x420b, 1);
        assert_eq!(bus.dma.channels[0].count_or_indirect, 3, "register write arms DMA without transferring");
        let before = bus.clock.master;
        bus.begin_cpu_cycle(8); // opcode fetch: one more cycle before pause
        bus.advance(8);
        assert_eq!(bus.clock.master, before + 8);
        assert_eq!(bus.dma.channels[0].count_or_indirect, 3);
        let pause = bus.clock.master;
        bus.begin_cpu_cycle(6); // NOP internal cycle determines resume speed
        assert_eq!(bus.clock.master - pause, expected);
        assert_eq!(bus.dma.channels[0].count_or_indirect, 0);
    }
}

#[test]
fn dma_resume_alignment_uses_the_following_cycle_speed() {
    for (speed, expected) in [(6, 48), (8, 48), (12, 48)] {
        let mut bus = bus_at(6);
        bus.write24(0x420b, 1);
        bus.begin_cpu_cycle(8);
        bus.advance(8);
        let pause = bus.clock.master;
        bus.begin_cpu_cycle(speed);
        assert_eq!(bus.clock.master - pause, expected);
        assert_eq!((bus.clock.master - pause) % u64::from(speed), 0);
    }
}

#[test]
fn save_restore_preserves_dma_armed_between_cpu_cycles() {
    use rustysnes_core::System;
    let mut system = System::new(0);
    system.bus = bus_at(6);
    system.bus.write24(0x420b, 1);
    system.bus.begin_cpu_cycle(8);
    system.bus.advance(8);
    let saved = system.save_state();
    system.bus.begin_cpu_cycle(6);
    let completed = system.save_state();
    let mut restored = System::new(0);
    restored.load_state(&saved).unwrap();
    restored.bus.begin_cpu_cycle(6);
    assert_eq!(restored.save_state(), completed);
}

#[test]
fn recording_profile_starts_at_the_oracle_reset_phase() {
    use rustysnes_core::{System, cart::Cart};
    let mut rom = vec![0; 0x10000];
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM DMA ORACLE TEST   ");
    rom[0x7fd5] = 0x20;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    let mut system = System::new(0);
    system.bus.cart = Some(Cart::from_rom(&rom).unwrap());
    system.reset();
    assert_eq!(system.cpu.regs.pc, 0x8000);
    assert_eq!(system.bus.clock.master, 186, "synthetic black-box recording-core first opcode");
    system.reset();
    assert_eq!(system.bus.clock.master, 202, "soft reset preserves the running clock phase");
}
