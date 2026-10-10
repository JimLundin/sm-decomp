//! Real emitted bodies against public SNES cycle lengths and DMA wait vectors.
use runtime::{Bus as RuntimeBus, Machine, TimedBus};
use rustysnes_core::{
    Bus,
    cart::{Cart, Region},
    cpu::Bus as CpuBus,
};
use std::{cell::RefCell, rc::Rc};

mod emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/prospective_lda.rs"));
}
#[derive(Default)]
struct Observations {
    events: Vec<(String, u32, u64)>,
    waits: Vec<u64>,
}
struct Passive {
    bus: Rc<RefCell<Bus>>,
    observed: Rc<RefCell<Observations>>,
}
impl RuntimeBus for Passive {
    fn read(&mut self, address: u32) -> u8 {
        let mut bus = self.bus.borrow_mut();
        let value = bus.read24(address);
        self.observed.borrow_mut().events.push(("read".into(), address, bus.clock.master));
        value
    }
    fn write(&mut self, address: u32, value: u8) {
        let mut bus = self.bus.borrow_mut();
        bus.write24(address, value);
        self.observed.borrow_mut().events.push(("write".into(), address, bus.clock.master));
    }
    fn tick(&mut self, clocks: u32) {
        self.bus.borrow_mut().advance(clocks);
    }
    fn access_cycles(&self, address: u32) -> u32 {
        self.bus.borrow().access_cycles(address)
    }
    fn begin_cpu_cycle(&mut self, clocks: u32) {
        let mut bus = self.bus.borrow_mut();
        let before = bus.clock.master;
        bus.begin_cpu_cycle(clocks);
        self.observed.borrow_mut().waits.push(bus.clock.master - before);
    }
    fn halt_pending(&self) -> bool {
        self.bus.borrow().halt_pending()
    }
    fn interrupt_pending(&self, masked: bool) -> bool {
        self.bus.borrow().interrupt_pending(masked)
    }
}
fn machine() -> (Machine, Rc<RefCell<Bus>>, Rc<RefCell<Observations>>) {
    let mut rom = vec![0; 0x10000];
    rom[..7].copy_from_slice(&[0xad, 0x10, 0, 0xea, 0xea, 0xea, 0xea]);
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM BUS TIMING TEST   ");
    rom[0x7fd5] = 0x30;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    let mut bus = Bus::new(Region::Ntsc);
    bus.cart = Some(Cart::from_rom(&rom).unwrap());
    let bus = Rc::new(RefCell::new(bus));
    let observed = Rc::new(RefCell::new(Observations::default()));
    let mut m =
        Machine::new(Box::new(TimedBus::new(Passive { bus: bus.clone(), observed: observed.clone() })), &[], None);
    m.cpu.p = 0x30;
    m.cpu.a = 1;
    (m, bus, observed)
}
// Independent rules: https://wiki.superfamicom.org/timing (CPU access,
// interrupts, refresh, DMA/HDMA). Expectations below are literal rules and
// phase goldens, never derived from shared-interpreter execution.
#[test]
fn prospective_lda_orders_mixed_speed_reads_and_wide_bank_boundary() {
    for (bank, fast, speed) in [(0, false, 8), (0x80, false, 8), (0x80, true, 6)] {
        let (mut m, bus, observed) = machine();
        bus.borrow_mut().write24(0x420d, u8::from(fast));
        bus.borrow_mut().write24(0x10, 0x80);
        m.cpu.a = 0xabcd;
        runtime::step::execute_step(&mut m, (bank << 16) | 0x8000, &[0xad, 0x10, 0], emitted::load8).unwrap();
        assert_eq!(
            observed.borrow().events,
            vec![
                ("read".into(), (bank << 16) | 0x8000, speed - 4),
                ("read".into(), (bank << 16) | 0x8001, 2 * speed - 4),
                ("read".into(), (bank << 16) | 0x8002, 3 * speed - 4),
                ("read".into(), 0x10, 3 * speed + 4)
            ]
        );
        assert_eq!(bus.borrow().clock.master, 3 * speed + 8);
        assert_eq!(m.cpu.a, 0xab80);
    }
    let (mut m, bus, observed) = machine();
    bus.borrow_mut().cart = None;
    bus.borrow_mut().write24(0, 0xad);
    bus.borrow_mut().write24(1, 0xff);
    bus.borrow_mut().write24(2, 0xff);
    bus.borrow_mut().write24(0x7effff, 0x34);
    bus.borrow_mut().write24(0x7f0000, 0x12);
    m.cpu.p = 0x10;
    m.cpu.db = 0x7e;
    runtime::step::execute_step(&mut m, 0, &[0xad, 0xff, 0xff], emitted::load16).unwrap();
    assert_eq!(m.cpu.a, 0x1234);
    assert_eq!(
        observed.borrow().events.iter().map(|row| (row.1, row.2)).collect::<Vec<_>>(),
        vec![(0, 4), (1, 12), (2, 20), (0x7effff, 28), (0x7f0000, 36)]
    );
    assert_eq!(bus.borrow().clock.master, 40);
}

#[test]
fn prospective_lda_read_side_effect_and_final_cycle_interrupt_rule() {
    let (mut m, bus, _) = machine();
    // RDNMI is cleared by the actual data read. Interrupt observer only
    // samples; it must never clear the register itself.
    while bus.borrow().ppu.scanline() < 225 {
        bus.borrow_mut().advance(2);
    }
    bus.borrow_mut().cart = None;
    for (address, value) in [(0, 0xad), (1, 0x10), (2, 0x42)] {
        bus.borrow_mut().write24(address, value);
    }
    runtime::step::execute_step(&mut m, 0, &[0xad, 0x10, 0x42], emitted::io).unwrap();
    assert_eq!(m.cpu.a & 0x80, 0x80);
    assert_eq!(bus.borrow_mut().read24(0x4210) & 0x80, 0);
}

#[test]
fn prospective_lda_samples_irq_and_nmi_before_final_data_cycle() {
    for nmi in [false, true] {
        let (mut m, bus, _) = machine();
        if nmi {
            bus.borrow_mut().write24(0x4200, 0x80);
            while bus.borrow().ppu.scanline() < 224 || bus.borrow().ppu.dot() < 333 {
                bus.borrow_mut().begin_cpu_cycle(2);
                bus.borrow_mut().advance(2);
            }
        } else {
            // HV timer at H100: edge arrives during the fourth read cycle.
            bus.borrow_mut().write24(0x4207, 100);
            bus.borrow_mut().write24(0x4208, 0);
            bus.borrow_mut().write24(0x4200, 0x10);
            bus.borrow_mut().advance(372);
        }
        runtime::step::execute_step(&mut m, 0x8000, &[0xad, 0x10, 0], emitted::load8).unwrap();
        let sampled = if nmi { bus.borrow_mut().poll_nmi() } else { bus.borrow_mut().poll_irq() };
        assert!(!sampled, "edge within the final cycle waits for following instruction");
        runtime::step::execute_step(&mut m, 0x8000, &[0xad, 0x10, 0], emitted::load8).unwrap();
        let sampled = if nmi { bus.borrow_mut().poll_nmi() } else { bus.borrow_mut().poll_irq() };
        assert!(sampled, "following instruction samples the pending edge");
    }
}

#[test]
fn prospective_lda_pending_dma_and_refresh_use_independent_clock_totals() {
    for phase in [0, 2, 4, 6] {
        let (mut m, bus, observed) = machine();
        {
            let mut b = bus.borrow_mut();
            b.clock.master = phase;
            for (register, value) in [(0, 0), (1, 0x40), (2, 0), (3, 0), (4, 0x7e), (5, 3), (6, 0)] {
                b.dma.write_reg(0, register, value);
            }
            b.write24(0x420b, 1);
        }
        runtime::step::execute_step(&mut m, 0x8000, &[0xad, 0x10, 0], emitted::load8).unwrap();
        // 2/4/6/8 entry alignment + 40 transfer clocks; following 8-clock
        // access resumes on its divider. All phases total 48 clocks.
        let expected = if phase == 0 { 56 } else { 48 };
        assert_eq!(observed.borrow().waits.iter().sum::<u64>(), expected);
        assert_eq!(bus.borrow().clock.master, phase + 32 + expected);
        assert_eq!(bus.borrow().dma.channels[0].count_or_indirect, 0);
    }
    let (mut m, bus, _) = machine();
    bus.borrow_mut().advance(530);
    runtime::step::execute_step(&mut m, 0x8000, &[0xad, 0x10, 0], emitted::load8).unwrap();
    assert_eq!(bus.borrow().clock.master, 530 + 32 + 40);
}

#[test]
fn prospective_lda_hdma_pause_matches_documented_transfer_cost() {
    let (mut m, bus, observed) = machine();
    {
        let mut b = bus.borrow_mut();
        while b.ppu.scanline() != 1 || b.ppu.dot() < 280 {
            b.advance(1);
        }
        b.write24(0x7e0000, 0x55);
        b.dma.hdma_enable = 1;
        let channel = &mut b.dma.channels[0];
        channel.dmap = 0;
        channel.hdma_completed = false;
        channel.hdma_do_transfer = true;
        channel.hdma_addr = 0;
        channel.source_bank = 0x7e;
        channel.target = 0x40;
        channel.line_counter = 2;
    }
    let before = bus.borrow().clock.master;
    runtime::step::execute_step(&mut m, 0x8000, &[0xad, 0x10, 0], emitted::load8).unwrap();
    // One direct mode-0 channel: global 8 + active 8 + data 8 clocks,
    // plus phase alignment to the eight-clock CPU access divider.
    let wait = observed.borrow().waits.iter().sum::<u64>();
    assert_eq!(wait, 32, "phase 4: alignment 4 + transfer 24 + resume 4");
    assert_eq!(bus.borrow().clock.master, before + 32 + wait);
    assert_eq!(bus.borrow().dma.channels[0].hdma_addr, 1);
}

#[test]
fn prospective_lda_frame_rollover_observer_and_checkpoint_resume_are_passive() {
    let mut rom = vec![0; 0x10000];
    rom[..7].copy_from_slice(&[0x18, 0xfb, 0xad, 0x10, 0, 0x80, 0xfb]);
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM AD RESUME TEST    ");
    rom[0x7fd5] = 0x20;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    let mut inspected = snes::Subject::with_steps(&rom, 0, emitted::STEP_TABLE).unwrap();
    let mut plain = snes::Subject::with_steps(&rom, 0, emitted::STEP_TABLE).unwrap();
    let mut restored = snes::Subject::with_steps(&rom, 0, emitted::STEP_TABLE).unwrap();
    for pad in [0, 0x1000, 0x8000] {
        inspected.run_frame_inspected(pad, |_, _| {}).unwrap();
        plain.run_frame(pad).unwrap();
        assert_eq!(inspected.state_bytes(), plain.state_bytes());
        assert_eq!(inspected.wram(), plain.wram());
        assert_eq!(inspected.framebuffer_bytes(), plain.framebuffer_bytes());
        restored.restore_checkpoint_state(&inspected.checkpoint_state()).unwrap();
    }
    inspected.run_frame(0x200).unwrap();
    restored.run_frame(0x200).unwrap();
    assert_eq!(inspected.state_bytes(), restored.state_bytes());
    assert_eq!(inspected.wram(), restored.wram());
    assert_eq!(inspected.framebuffer_bytes(), restored.framebuffer_bytes());
    assert!(inspected.generated_steps > 100);
}
