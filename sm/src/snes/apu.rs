use crate::c_library::puts;
use crate::execution::Execution;
use crate::initial_state::InitialState;
use crate::snes::dsp::{dsp_cycle, dsp_read, dsp_reset, dsp_saveload, dsp_write};
use crate::snes::spc::{spc_reset, spc_runOpcode, spc_saveload};
use crate::tracing::getProcessorStateSpc;
pub use crate::types::{Apu, ApuRegisters, DspRegWriteHistory, Timer, size_t};
use ::core::mem::size_of;
// Register serialization retains the forty bytes following legacy APU RAM.
const _: () = {
    assert!(core::mem::offset_of!(ApuRegisters, history) == 40);
    assert!(size_of::<ApuRegisters>() == 556);
};
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
static bootRom: [u8; 64] = [
    0xcd, 0xef, 0xbd, 0xe8, 0, 0xc6, 0x1d, 0xd0, 0xfc, 0x8f, 0xaa, 0xf4, 0x8f, 0xbb, 0xf5, 0x78,
    0xcc, 0xf4, 0xd0, 0xfb, 0x2f, 0x19, 0xeb, 0xf4, 0xd0, 0xfc, 0x7e, 0xf4, 0xd0, 0xb, 0xe4, 0xf5,
    0xcb, 0xf4, 0xd7, 0, 0xfc, 0xd0, 0xf3, 0xab, 0x1, 0x10, 0xef, 0x7e, 0xf4, 0x10, 0xeb, 0xba,
    0xf6, 0xda, 0, 0xba, 0xf4, 0xc4, 0xf4, 0xdd, 0x5d, 0xd0, 0xdb, 0x1f, 0, 0, 0xc0, 0xff,
];
pub unsafe fn apu_reset(apu: *mut Apu) {
    unsafe {
        (*apu).registers.begin_reset();
        spc_reset((*apu).spc);
        dsp_reset((*apu).dsp);
        (*apu).registers.finish_reset(&(*apu).ram);
    }
}
pub unsafe fn apu_saveload(apu: *mut Apu, io: &mut crate::snapshot_io::SnapshotIo<'_>) {
    unsafe {
        crate::rtl::snapshot_result(io.state(&mut *apu));
        dsp_saveload((*apu).dsp, io);
        spc_saveload((*apu).spc, io);
    }
}

pub(crate) unsafe fn apu_cycle_execution(game: &mut Execution<'_>) {
    unsafe { apu_cycle_parts(game.backend.apu_view(), game.host.g_debug_apu_cycles) }
}
unsafe fn apu_cycle_parts(apu: *mut Apu, debug_cycles: &bool) {
    unsafe {
        let opcode_cycles = if (*apu).registers.cpuCyclesLeft == 0 {
            if *debug_cycles {
                let mut line = [0u8; 80];
                getProcessorStateSpc(apu, &mut line);
                puts(line.as_ptr().cast());
            }
            Some(spc_runOpcode((*apu).spc) as u8)
        } else {
            None
        };
        // The native CPU and trace may reenter the bus; their register borrows ended.
        (*apu)
            .registers
            .cycle(opcode_cycles, || dsp_cycle((*apu).dsp));
    }
}
#[unsafe(export_name = "apu_cpuRead")]
pub unsafe extern "C-unwind" fn apu_cpuRead(apu: *mut Apu, adr: u16) -> u8 {
    unsafe {
        (*apu).registers.read(&(*apu).ram, adr, |address| {
            dsp_read(&(*(*apu).dsp).state, address)
        })
    }
}
#[forbid(unsafe_code)]
fn record_dsp_write(history: &mut DspRegWriteHistory, address: u8, value: u8) {
    let index = history.count as i32;
    if index != 256 {
        history.count = (index + 1) as u32;
        history.addr[index as usize] = address;
        history.val[index as usize] = value;
    }
}

#[unsafe(export_name = "apu_cpuWrite")]
pub unsafe extern "C-unwind" fn apu_cpuWrite(apu: *mut Apu, adr: u16, val: u8) {
    unsafe {
        (*apu)
            .registers
            .write(&(*apu).ram, adr, val, |address, value| {
                dsp_write((*apu).dsp, address, value);
            });
    }
}

#[forbid(unsafe_code)]
impl ApuRegisters {
    pub fn new() -> Self {
        Self::INITIAL
    }

    // No register borrow spans SPC reset: it reads the enabled boot ROM through the bus.
    pub fn begin_reset(&mut self) {
        self.romReadable = true;
    }

    pub fn finish_reset(&mut self, ram: &crate::apu_ram::ApuRam) {
        ram.fill(0);
        self.dspAdr = 0;
        self.cycles = 0;
        self.inPorts.fill(0);
        self.outPorts.fill(0);
        for timer in &mut self.timer {
            timer.cycles = 0;
            timer.divider = 0;
            timer.target = 0;
            timer.counter = 0;
            timer.enabled = false;
        }
        self.cpuCyclesLeft = 7;
        self.history.count = 0;
    }

    pub fn cycle(&mut self, opcode_cycles: Option<u8>, cycle_dsp: impl FnOnce()) {
        if let Some(cycles) = opcode_cycles {
            self.cpuCyclesLeft = cycles;
        }
        self.cpuCyclesLeft = self.cpuCyclesLeft.wrapping_sub(1);
        if self.cycles & 0x1f == 0 {
            cycle_dsp();
        }
        for (index, timer) in self.timer.iter_mut().enumerate() {
            if timer.cycles == 0 {
                timer.cycles = if index == 2 { 16 } else { 128 };
                if timer.enabled {
                    timer.divider = timer.divider.wrapping_add(1);
                    if timer.divider == timer.target {
                        timer.divider = 0;
                        timer.counter = timer.counter.wrapping_add(1) & 0xf;
                    }
                }
            }
            timer.cycles = timer.cycles.wrapping_sub(1);
        }
        self.cycles = self.cycles.wrapping_add(1);
    }

    pub fn read(
        &mut self,
        ram: &crate::apu_ram::ApuRam,
        address: u16,
        read_dsp: impl FnOnce(u8) -> u8,
    ) -> u8 {
        match address {
            0xf0 | 0xf1 | 0xfa..=0xfc => return 0,
            0xf2 => return self.dspAdr,
            0xf3 => return read_dsp(self.dspAdr & 0x7f),
            0xf4..=0xf9 => return self.inPorts[usize::from(address - 0xf4)],
            0xfd..=0xff => {
                let timer = &mut self.timer[usize::from(address - 0xfd)];
                let value = timer.counter;
                timer.counter = 0;
                return value;
            }
            _ => {}
        }
        if self.romReadable && address >= 0xffc0 {
            return bootRom[usize::from(address - 0xffc0)];
        }
        ram.read(address)
    }

    pub fn write(
        &mut self,
        ram: &crate::apu_ram::ApuRam,
        address: u16,
        value: u8,
        write_dsp: impl FnOnce(u8, u8),
    ) {
        match address {
            0xf1 => {
                for i in 0..3 {
                    let enabled = value & (1 << i) != 0;
                    let timer = &mut self.timer[i];
                    if !timer.enabled && enabled {
                        timer.divider = 0;
                        timer.counter = 0;
                    }
                    timer.enabled = enabled;
                }
                if value & 0x10 != 0 {
                    self.inPorts[0] = 0;
                    self.inPorts[1] = 0;
                }
                if value & 0x20 != 0 {
                    self.inPorts[2] = 0;
                    self.inPorts[3] = 0;
                }
                self.romReadable = value & 0x80 != 0;
            }
            0xf2 => self.dspAdr = value,
            0xf3 => {
                record_dsp_write(&mut self.history, self.dspAdr, value);
                if self.dspAdr < 0x80 {
                    write_dsp(self.dspAdr, value);
                }
            }
            0xf4..=0xf7 => self.outPorts[usize::from(address - 0xf4)] = value,
            0xf8..=0xf9 => self.inPorts[usize::from(address - 0xf4)] = value,
            0xfa..=0xfc => self.timer[usize::from(address - 0xfa)].target = value,
            _ => {}
        }
        ram.write(address, value);
    }
}
