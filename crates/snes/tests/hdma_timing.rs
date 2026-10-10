//! Regression for missing per-channel scanline overhead in the hardware core.
use rustysnes_core::{dma::Dma, dma_bus::DmaBus};

struct Memory([u8; 256]);
impl DmaBus for Memory {
    fn read_a(&mut self, addr: u32) -> u8 {
        self.0[addr as usize & 255]
    }
    fn write_a(&mut self, _: u32, _: u8) {}
    fn read_b(&mut self, _: u8) -> u8 {
        0
    }
    fn write_b(&mut self, _: u8, _: u8) {}
}

#[test]
fn hdma_charges_active_channel_overhead_even_on_non_transfer_lines() {
    // Direct, mode 0: one data byte, then hold for three scanlines.
    // 8 global + 8 per active channel + 8 per byte (SNESdev timing docs).
    let mut dma = Dma::default();
    let mut memory = Memory([0; 256]);
    memory.0[..3].copy_from_slice(&[3, 0x12, 0]);
    dma.hdma_enable = 1;
    dma.write_reg(0, 0, 0); // mode 0, direct A-to-B
    dma.write_reg(0, 2, 0);
    dma.write_reg(0, 3, 0);
    dma.write_reg(0, 4, 0);
    let _ = dma.hdma_setup(&mut memory);
    assert_eq!(dma.hdma_run(&mut memory), 24, "transfer line");
    assert_eq!(dma.hdma_run(&mut memory), 16, "active channel, held data");
    assert_eq!(dma.hdma_run(&mut memory), 16, "line-counter reload is included in channel overhead");
    assert_eq!(dma.hdma_run(&mut memory), 0, "all channels terminated: no HDMA overhead");
}

#[test]
fn terminated_enabled_channel_does_not_stop_another_active_channel() {
    let mut dma = Dma::default();
    let mut memory = Memory([0; 256]);
    memory.0[..3].copy_from_slice(&[1, 0x12, 0]);
    memory.0[16..19].copy_from_slice(&[3, 0x34, 0]);
    dma.hdma_enable = 3;
    for channel in 0..2 {
        dma.write_reg(channel, 0, 0);
        dma.write_reg(channel, 2, (channel * 16) as u8);
        dma.write_reg(channel, 3, 0);
        dma.write_reg(channel, 4, 0);
    }
    let _ = dma.hdma_setup(&mut memory);
    let _ = dma.hdma_run(&mut memory);
    assert!(dma.channels[0].hdma_completed);
    assert!(!dma.channels[1].hdma_completed);
    assert_eq!(dma.hdma_enable, 3, "termination leaves the enable bits set");
    assert_eq!(dma.hdma_run(&mut memory), 16, "the second channel still incurs global and channel overhead");
    let _ = dma.hdma_run(&mut memory);
    assert_eq!(dma.hdma_run(&mut memory), 0, "both enabled channels have now terminated");
}

#[test]
fn hdma_setup_and_visible_line_pauses_match_the_recording_core() {
    use rustysnes_core::{System, cart::Cart};
    // Independent cartridge built from public register documentation. Enable
    // HDMA during VBlank, then mix 8-clock ROM fetches and 6-clock internal
    // cycles. Golden rows cover setup, pause phases, reloads and frame rollover.
    let rom = hdma_cartridge();
    let mut system = System::new(0);
    system.bus.cart = Some(Cart::from_rom(&rom).unwrap());
    system.reset();
    for row in include_str!("hdma_oracle.csv").lines().filter(|row| !row.starts_with('#')) {
        let (clock, pc) = row.split_once(',').unwrap();
        let clock = clock.parse::<u64>().unwrap();
        let pc = u16::from_str_radix(pc, 16).unwrap();
        while system.bus.clock.master < clock {
            system.cpu.step(&mut system.bus);
        }
        assert_eq!((system.bus.clock.master, system.cpu.regs.pc), (clock, pc), "oracle row {row}");
    }
}

fn hdma_cartridge() -> Vec<u8> {
    let mut rom = vec![0; 0x10000];
    let code = [
        0x18, 0xfb, 0xe2, 0x30, 0xa9, 0, 0x8d, 0x70, 0x43, 0xa9, 0, 0x8d, 0x71, 0x43, 0xa9, 0, 0x8d, 0x72, 0x43, 0xa9,
        0x90, 0x8d, 0x73, 0x43, 0xa9, 0, 0x8d, 0x74, 0x43, 0xad, 0x12, 0x42, 0x10, 0xfb, 0xa9, 0x80, 0x8d, 0x0c, 0x42,
        0xea, 0xea, 0x80, 0xfc,
    ];
    rom[..code.len()].copy_from_slice(&code);
    rom[0x1000] = 0xff;
    rom[0x1001..0x1080].fill(0x80);
    rom[0x1080] = 0xff;
    rom[0x1081..0x1100].fill(0x80);
    rom[0x1100] = 0;
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM NMI ORACLE TEST   ");
    rom[0x7fd5] = 0x20;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    rom
}

mod hdma_emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/hdma_steps.rs"));
}

#[test]
fn generated_and_interpreted_implied_cycles_agree_across_hdma_pauses() {
    let rom = hdma_cartridge();
    let mut reference = snes::Reference::new(&rom, 0).unwrap();
    let mut subject = snes::Subject::with_steps(&rom, 0, hdma_emitted::STEP_TABLE).unwrap();
    for _ in 0..3 {
        reference.run_frame(0).unwrap();
        subject.run_frame(0).unwrap();
        assert_eq!(reference.state_bytes(), subject.state_bytes());
        assert_eq!(reference.wram(), subject.wram());
        assert_eq!(reference.framebuffer_bytes(), subject.framebuffer_bytes());
    }
    assert!(subject.generated_steps > 100);
    assert!(subject.interpreted_steps > 0);
}

#[test]
fn terminated_channels_do_not_charge_cpu_pause_alignment() {
    use rustysnes_core::{Bus, cart::Region, cpu::Bus as CpuBus};
    let mut bus = Bus::new(Region::Ntsc);
    while bus.ppu.scanline() != 1 || bus.ppu.dot() < 279 {
        bus.advance(1);
    }
    bus.dma.hdma_enable = 1;
    bus.dma.channels[0].hdma_completed = true;
    let before = bus.clock.master;
    assert!(!bus.halt_pending());
    bus.begin_cpu_cycle(6);
    assert_eq!(bus.clock.master, before, "no divider waits after termination");
    bus.advance(6);
    assert_eq!(bus.clock.master, before + 6);
}
