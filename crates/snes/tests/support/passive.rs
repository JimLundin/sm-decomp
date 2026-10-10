//! Passive actual hardware observations; expectations belong to each test.
use runtime::Bus as RuntimeBus;
use rustysnes_core::{Bus, cpu::Bus as CpuBus};
use std::{cell::RefCell, rc::Rc};
#[derive(Default)]
pub struct Observations {
    pub events: Vec<(String, u32, u64)>,
    pub waits: Vec<u64>,
}
pub struct Passive {
    pub bus: Rc<RefCell<Bus>>,
    pub observed: Rc<RefCell<Observations>>,
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
