//! Real emitted bodies against public SNES cycle lengths and DMA wait vectors.
use runtime::{Machine, TimedBus};
use rustysnes_core::{
    Bus,
    cart::{Cart, Region},
    cpu::Bus as CpuBus,
};
use std::{cell::RefCell, rc::Rc};

mod emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/dma_steps.rs"));
}
#[path = "support/passive.rs"]
mod passive;
use passive::{Observations, Passive};
fn machine() -> (Machine, Rc<RefCell<Bus>>, Rc<RefCell<Observations>>) {
    let mut rom = vec![0; 0x10000];
    rom[..7].copy_from_slice(&[0xa9, 1, 0x8d, 0x0b, 0x42, 0xea, 0xea]);
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
#[test]
fn emitted_fetch_and_sta_follow_slow_fast_and_twelve_clock_access_phases() {
    for (bank, fast, speed) in [(0, false, 8), (0x80, false, 8), (0x80, true, 6)] {
        let (mut m, bus, observed) = machine();
        bus.borrow_mut().write24(0x420d, u8::from(fast));
        let entry = emitted::STEP_TABLE.iter().find(|row| row.0 == 0x8000).unwrap();
        let next = runtime::step::execute_step(&mut m, (bank << 16) | 0x8000, entry.2, entry.3).unwrap();
        assert_eq!(next, (bank << 16) | 0x8002);
        assert_eq!(
            observed.borrow().events,
            vec![
                ("read".into(), (bank << 16) | 0x8000, speed - 4),
                ("read".into(), (bank << 16) | 0x8001, 2 * speed - 4)
            ]
        );
        assert_eq!(bus.borrow().clock.master, 2 * speed);
    }
    let (mut m, bus, observed) = machine();
    m.bus.read(0x4000); // public CPU I/O region: twelve clocks, read at end minus four
    assert_eq!(observed.borrow().events[0], ("read".into(), 0x4000, 8));
    assert_eq!(bus.borrow().clock.master, 12);
}
#[test]
fn emitted_sta_arms_dma_and_following_nop_matches_independent_pause_vectors() {
    for (phase, wait) in [(6, 48), (4, 48), (2, 48), (0, 54)] {
        let (mut m, bus, observed) = machine();
        {
            let mut b = bus.borrow_mut();
            b.clock.master = phase + 2;
            b.dma.write_reg(0, 0, 0);
            b.dma.write_reg(0, 1, 0x40);
            b.dma.write_reg(0, 2, 0);
            b.dma.write_reg(0, 3, 0);
            b.dma.write_reg(0, 4, 0x7e);
            b.dma.write_reg(0, 5, 3);
            b.dma.write_reg(0, 6, 0);
        }
        let sta = emitted::STEP_TABLE.iter().find(|row| row.0 == 0x8002).unwrap();
        let nop = emitted::STEP_TABLE.iter().find(|row| row.0 == 0x8005).unwrap();
        runtime::step::execute_step(&mut m, sta.0, sta.2, sta.3).unwrap();
        assert_eq!(bus.borrow().dma.channels[0].count_or_indirect, 3, "STA only arms DMA");
        runtime::step::execute_step(&mut m, nop.0, nop.2, nop.3).unwrap();
        assert_eq!(bus.borrow().dma.channels[0].count_or_indirect, 0);
        assert_eq!(*observed.borrow().waits.last().unwrap(), wait, "documented phase {phase}");
    }
}
#[test]
fn emitted_nop_crosses_documented_refresh_with_exact_forty_clock_pause() {
    let (mut m, bus, _) = machine();
    bus.borrow_mut().advance(530);
    let nop = emitted::STEP_TABLE.iter().find(|row| row.0 == 0x8005).unwrap();
    runtime::step::execute_step(&mut m, nop.0, nop.2, nop.3).unwrap();
    assert_eq!(bus.borrow().clock.master, 530 + 8 + 40 + 6);
}
