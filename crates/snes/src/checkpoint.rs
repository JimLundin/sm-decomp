//! Game-state observations at the entry to RunOneFrameOfGame ($82:8948).
//! This software point avoids comparing different emulator video-edge phases.
pub const CHECKPOINT_PC: u32 = 0x828948;
pub const CSV_HEADER: &str =
    "frame,event,game_state,room,x,y,health,items,time_frames,time_seconds,time_minutes,time_hours";
pub const OFFSETS: [usize; 10] = [0x998, 0x79b, 0xaf6, 0xafa, 0x9c2, 0x9a4, 0x9da, 0x9dc, 0x9de, 0x9e0];

/// rr2-beta23's NTSC frame callback occurs at V=241. Checkpoints past that
/// boundary belong to its next movie frame, while RustySNES wraps at V=0.
/// This changes labels only; it does not change input or execution timing.
pub fn lsnes_frame(frame: usize, scanline: u16) -> usize {
    frame + usize::from(scanline >= 241)
}

pub fn csv_row(frame: usize, event: usize, wram: &[u8]) -> String {
    let values = OFFSETS.map(|offset| u16::from_le_bytes([wram[offset], wram[offset + 1]]));
    format!("{frame},{event},{}\n", values.map(|word| format!("{word:04X}")).join(","))
}

pub(crate) struct ObservedBus<'a, F> {
    pub bus: &'a mut rustysnes_core::Bus,
    pub pc: u32,
    pub observe: F,
    pub armed: bool,
}

impl<F: FnMut(&[u8], u16)> rustysnes_core::cpu::Bus for ObservedBus<'_, F> {
    fn begin_interrupt(&mut self) {
        self.armed = false;
    }
    fn begin_cpu_cycle(&mut self, clocks: u32) {
        self.bus.begin_cpu_cycle(clocks);
    }
    fn halt_pending(&self) -> bool {
        self.bus.halt_pending()
    }
    fn interrupt_pending(&self, irq_masked: bool) -> bool {
        self.bus.interrupt_pending(irq_masked)
    }
    fn read24(&mut self, addr: u32) -> u8 {
        if self.armed && addr == self.pc {
            (self.observe)(self.bus.wram(), self.bus.ppu.scanline());
            self.armed = false;
        }
        self.bus.read24(addr)
    }
    fn write24(&mut self, addr: u32, value: u8) {
        self.bus.write24(addr, value);
    }
    fn access_cycles(&self, addr: u32) -> u32 {
        self.bus.access_cycles(addr)
    }
    fn advance(&mut self, clocks: u32) {
        self.bus.advance(clocks);
    }
    fn poll_nmi(&mut self) -> bool {
        self.bus.poll_nmi()
    }
    fn poll_irq(&mut self) -> bool {
        self.bus.poll_irq()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustysnes_core::{System, cpu::Bus as CpuBus};

    #[test]
    fn discarded_interrupt_fetch_does_not_count_as_a_checkpoint_visit() {
        let mut system = System::new(0);
        system.cpu.regs.pbr = 0x82;
        system.cpu.regs.pc = 0x8948;
        system.bus.write24(0x4200, 0x80);
        while system.bus.ppu.scanline() < 225 {
            system.bus.advance(8);
        }
        system.bus.advance(8);
        system.bus.begin_cpu_cycle(8);
        let mut visits = 0;
        system.cpu.step(&mut ObservedBus {
            bus: &mut system.bus,
            pc: CHECKPOINT_PC,
            observe: |_: &[u8], _: u16| visits += 1,
            armed: true,
        });
        assert_eq!(system.cpu.interrupts_taken, 1);
        assert_eq!(visits, 0);
        let mut bus = ObservedBus {
            bus: &mut system.bus,
            pc: CHECKPOINT_PC,
            observe: |_: &[u8], _: u16| visits += 1,
            armed: true,
        };
        bus.read24(CHECKPOINT_PC);
        bus.read24(CHECKPOINT_PC);
        assert_eq!(visits, 1, "a real opcode fetch counts exactly once");
    }
}
