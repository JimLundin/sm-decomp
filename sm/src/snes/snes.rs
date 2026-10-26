use crate::c_library::{__assert_fail, memset};
use crate::game_state::Game;
use crate::initial_state::InitialState;
use crate::rtl::RtlApuWrite;
use crate::rtl::words;
use crate::snes::apu::{apu_cycle, apu_reset, apu_saveload};
use crate::snes::cart::{cart_read, cart_reset, cart_saveload, cart_write};
use crate::snes::cpu::{cpu_reset, cpu_saveload};
use crate::snes::dma::{
    dma_cycle, dma_doHdma, dma_initHdma, dma_read, dma_reset, dma_saveload, dma_startDma, dma_write,
};
use crate::snes::input::input_reset;
use crate::snes::ppu::{
    ppu_checkOverscan, ppu_handleVblank, ppu_read, ppu_reset, ppu_runLine, ppu_saveload, ppu_write,
};
use crate::types::{Apu, ApuRegisters, Cart, Cpu, Dma, Dsp, Input, Ppu, Spc};
pub use crate::types::{Snes, size_t};
use ::core::ffi::{c_char, c_void};
use std::cell::{Cell, UnsafeCell};
use std::rc::Rc;
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

/// Own every device in one retained backend. Each allocation is stable and
/// separate; raw repr(C) links are borrowed views, never the owners. Rc keeps
/// reborrowing Game from imposing Box-style uniqueness on live device storage.
pub(crate) struct Backend {
    snes: Rc<Cell<Snes>>,
    cpu: Rc<Cell<Cpu>>,
    dma: Rc<Cell<Dma>>,
    my_ppu: Rc<Cell<Ppu>>,
    snes_ppu: Rc<Cell<Ppu>>,
    apu: Rc<UnsafeCell<Apu>>,
    spc: Rc<Cell<Spc>>,
    dsp: Rc<UnsafeCell<Dsp>>,
    cart: Rc<UnsafeCell<Cart>>,
    input1: Rc<Cell<Input>>,
    input2: Rc<Cell<Input>>,
}
#[forbid(unsafe_code)]
impl Backend {
    pub(crate) fn new(ram: *mut u8) -> Self {
        let snes = Rc::new(Cell::new(Snes::INITIAL));
        let cpu = Rc::new(Cell::new(Cpu {
            mem: snes.as_ptr().cast(),
            memType: 0,
            ..Cpu::INITIAL
        }));
        let apu_ram = crate::apu_ram::ApuRam::new();
        let spc = Rc::new(Cell::new(Spc::INITIAL));
        let dsp = Rc::new(UnsafeCell::new(Dsp::new(apu_ram.clone())));
        let apu = Rc::new(UnsafeCell::new(Apu {
            spc: spc.as_ptr(),
            dsp: dsp.get(),
            ram: apu_ram,
            registers: ApuRegisters::INITIAL,
        }));
        // Wire the backreference only after its APU destination is allocated.
        spc.set(Spc {
            apu: apu.get(),
            ..Spc::INITIAL
        });
        let dma = Rc::new(Cell::new(Dma {
            snes: snes.as_ptr(),
            ..Dma::INITIAL
        }));
        let my_ppu = Rc::new(Cell::new(Ppu {
            snes: snes.as_ptr(),
            ..Ppu::INITIAL
        }));
        let snes_ppu = Rc::new(Cell::new(Ppu {
            snes: snes.as_ptr(),
            ..Ppu::INITIAL
        }));
        let cart = Rc::new(UnsafeCell::new(Cart {
            snes: snes.as_ptr(),
            type_0: 0,
            rom: crate::rom::RomImage::default(),
            romSize: 0,
            ram: crate::save_ram::SaveRam::default(),
            ramSize: 0,
        }));
        let input1 = Rc::new(Cell::new(Input {
            snes: snes.as_ptr(),
            type_0: 1,
            currentState: 0,
        }));
        let input2 = Rc::new(Cell::new(Input {
            snes: snes.as_ptr(),
            type_0: 1,
            currentState: 0,
        }));
        snes.set(Snes {
            ram,
            cpu: cpu.as_ptr(),
            apu: apu.get(),
            dma: dma.as_ptr(),
            my_ppu: my_ppu.as_ptr(),
            snes_ppu: snes_ppu.as_ptr(),
            ppu: snes_ppu.as_ptr(),
            cart: cart.get(),
            input1: input1.as_ptr(),
            input2: input2.as_ptr(),
            ..Snes::INITIAL
        });
        Self {
            snes,
            cpu,
            dma,
            my_ppu,
            snes_ppu,
            apu,
            spc,
            dsp,
            cart,
            input1,
            input2,
        }
    }

    pub(crate) fn views(&self) -> (*mut Snes, *mut Cpu) {
        (self.snes.as_ptr(), self.cpu.as_ptr())
    }

    /// Save inert copies, never borrow live devices or follow a supplied pointer.
    /// Callers hold Game exclusivity; raw backend borrows must end before callbacks.
    pub(crate) fn save_observation(
        &self,
        snes: *mut Snes,
        cpu: *mut Cpu,
        output: &mut crate::recording::RecordingBytes,
    ) {
        let mut io = crate::snapshot_io::SnapshotIo::Save(output);
        assert_eq!(
            snes,
            self.snes.as_ptr(),
            "observation requires the owned SNES"
        );
        io.state(&mut self.snes.get()).unwrap();
        assert_eq!(cpu, self.cpu.as_ptr(), "observation requires the owned CPU");
        io.state(&mut self.cpu.get()).unwrap();
        assert_eq!(
            self.snes.get().dma,
            self.dma.as_ptr(),
            "observation requires the owned DMA"
        );
        io.state(&mut self.dma.get()).unwrap();
        // Select at the original final-device point, after the first three saves.
        let selected = self.snes.get().ppu;
        let ppu = if selected == self.snes_ppu.as_ptr() {
            &self.snes_ppu
        } else {
            assert_eq!(
                selected,
                self.my_ppu.as_ptr(),
                "observation requires an owned PPU"
            );
            &self.my_ppu
        };
        io.state(&mut ppu.get()).unwrap();
    }
}

pub unsafe fn snes_saveload(snes: *mut Snes, io: &mut crate::snapshot_io::SnapshotIo<'_>) {
    unsafe {
        cpu_saveload((*snes).cpu, io);
        apu_saveload((*snes).apu, io);
        dma_saveload((*snes).dma, io);
        ppu_saveload((*snes).ppu, io);
        cart_saveload((*snes).cart, io);
        crate::rtl::snapshot_result(io.state(&mut *snes));
        crate::rtl::snapshot_region(io, (*snes).ram as *mut u8, 0x20000);
        crate::rtl::snapshot_result(io.state(&mut (*snes).ramAdr));
        (*snes).runningWhichVersion = 0;
    }
}
pub unsafe fn snes_reset(snes: *mut Snes, hard: bool, game: &mut Game) {
    unsafe {
        cart_reset((*snes).cart);
        cpu_reset((*snes).cpu, game);
        apu_reset((*snes).apu);
        dma_reset((*snes).dma);
        ppu_reset((*snes).my_ppu);
        ppu_reset((*snes).snes_ppu);
        input_reset((*snes).input1);
        input_reset((*snes).input2);
        if hard {
            memset((*snes).ram as *mut c_void, 0, 0x20000i32 as size_t);
        }
        set_fields! {[(*snes)], 0; ramAdr, hPos, vPos, frames}
        (*snes).cpuCyclesLeft = 52;
        set_fields! {[(*snes)]; cpuMemOps = 0, apuCatchupCycles = 0.0f64, hIrqEnabled = false,
        vIrqEnabled = false, nmiEnabled = false, };
        set_fields! {[(*snes)], 0x1ff; hTimer, vTimer}
        (*snes).inNmi = false;
        set_fields! {[(*snes)]; inIrq = false, inVblank = false, portAutoReadX = [0; 4],
        autoJoyRead = false, autoJoyTimer = 0, ppuLatch = false, multiplyA = 0xff,
        multiplyResult = 0xfe01, divideA = 0xffff, divideResult = 0x101, fastMem = false,
        openBus = 0, };
    }
}
pub unsafe fn snes_handle_pos_stuff(snes: *mut Snes, game: &mut Game) {
    unsafe {
        if (*snes).hPos == 0 {
            let mut startingVblank = false;
            if (*snes).vPos == 0 {
                (*snes).inVblank = false;
                (*snes).inNmi = false;
                dma_initHdma((*snes).dma, game);
            } else if (*snes).vPos == 225 {
                startingVblank = !ppu_checkOverscan((*snes).ppu);
            } else if (*snes).vPos == 240 && !(*snes).inVblank {
                startingVblank = true;
            }
            if startingVblank {
                ppu_handleVblank((*snes).ppu);
                (*snes).inVblank = true;
                (*snes).inNmi = true;
                (*(*snes).cpu).nmiWanted = true;
                if (*snes).autoJoyRead {
                    (*snes).autoJoyTimer = 0;
                }
            }
        } else if (*snes).hPos == 512 {
            if !(*snes).inVblank && !(*snes).disableRender {
                ppu_runLine((*snes).ppu, i32::from((*snes).vPos), game);
            }
        } else if (*snes).hPos == 1024 && !(*snes).inVblank {
            dma_doHdma((*snes).dma, game);
        }
        (*snes).hPos = (*snes).hPos.wrapping_add(2);
        if (*snes).hPos == 1364 {
            (*snes).hPos = 0;
            (*snes).vPos = (*snes).vPos.wrapping_add(1);
            if (*snes).vPos == 262 {
                (*snes).vPos = 0;
                (*snes).frames = (*snes).frames.wrapping_add(1);
            }
        }
    }
}
pub unsafe fn snes_catchupApu(snes: *mut Snes, game: &mut Game) {
    unsafe {
        if (*snes).apuCatchupCycles > 10000.0 {
            (*snes).apuCatchupCycles = 10000.0;
        }
        let cycles = (*snes).apuCatchupCycles as i32;
        for _ in 0..cycles {
            apu_cycle((*snes).apu, game);
        }
        (*snes).apuCatchupCycles -= f64::from(cycles);
    }
}
pub unsafe fn snes_readBBus(snes: *mut Snes, address: u8, game: &mut Game) -> u8 {
    unsafe {
        match address {
            0x00..=0x3f => ppu_read((*snes).ppu, address),
            0x40..=0x7f => {
                (*snes).apuCatchupCycles = 32.0;
                snes_catchupApu(snes, game);
                (*(*snes).apu).registers.outPorts[usize::from(address & 3)]
            }
            0x80 => {
                crate::update_place! { previous address = (*snes).ramAdr => (*snes).ramAdr.wrapping_add(1); }
                let value = *(*snes).ram.offset(address as isize);
                (*snes).ramAdr &= 0x1ffff;
                value
            }
            _ => crate::native_assertion! {
                @failure
                b"0\0", b"src/snes/snes.c\0", 197, b"uint8_t snes_readBBus(Snes *, uint8_t)\0"
            },
        }
    }
}
pub unsafe fn snes_writeBBus(snes: *mut Snes, address: u8, value: u8, game: &mut Game) {
    unsafe {
        match address {
            0x00..=0x3f => ppu_write((*snes).ppu, address, value),
            0x40..=0x7f => RtlApuWrite(0x2100 + u32::from(address), value, game),
            0x80 => {
                crate::update_place! { previous address = (*snes).ramAdr => (*snes).ramAdr.wrapping_add(1); }
                *(*snes).ram.offset(address as isize) = value;
                (*snes).ramAdr &= 0x1ffff;
            }
            0x81 => (*snes).ramAdr = ((*snes).ramAdr & 0x1ff00) | u32::from(value),
            0x82 => (*snes).ramAdr = ((*snes).ramAdr & 0x100ff) | (u32::from(value) << 8),
            0x83 => (*snes).ramAdr = ((*snes).ramAdr & 0xffff) | (u32::from(value & 1) << 16),
            _ => {}
        }
    }
}
fn SwapInputBits(input: u16) -> u16 {
    input.reverse_bits()
}
unsafe fn snes_readReg(snes: *mut Snes, address: u16) -> u8 {
    unsafe {
        match address {
            0x4210 => 2 | (u8::from((*snes).inNmi) << 7) | ((*snes).openBus & 0x70),
            0x4211 => {
                let irq = u8::from((*snes).inIrq) << 7;
                (*snes).inIrq = false;
                (*(*snes).cpu).irqWanted = false;
                irq | ((*snes).openBus & 0x7f)
            }
            0x4212 => {
                u8::from((*snes).autoJoyTimer > 0)
                    | (u8::from((*snes).hPos >= 1024) << 6)
                    | (u8::from((*snes).inVblank) << 7)
                    | ((*snes).openBus & 0x3e)
            }
            0x4213 => u8::from((*snes).ppuLatch) << 7,
            0x4214 => (*snes).divideResult as u8,
            0x4215 => ((*snes).divideResult >> 8) as u8,
            0x4216 => (*snes).multiplyResult as u8,
            0x4217 => ((*snes).multiplyResult >> 8) as u8,
            0x4218 => SwapInputBits((*(*snes).input1).currentState) as u8,
            0x4219 => (SwapInputBits((*(*snes).input1).currentState) >> 8) as u8,
            0x421a..=0x421f => 0,
            _ => (*snes).openBus,
        }
    }
}
unsafe fn snes_writeReg(snes: *mut Snes, adr: u16, val: u8, game: &mut Game) {
    unsafe {
        match adr {
            0x4200 => {
                (*snes).autoJoyRead = val & 0x1 != 0;
                if !(*snes).autoJoyRead {
                    (*snes).autoJoyTimer = 0;
                }
                (*snes).hIrqEnabled = val & 0x10 != 0;
                (*snes).vIrqEnabled = val & 0x20 != 0;
                (*snes).nmiEnabled = val & 0x80 != 0;
                if !(*snes).hIrqEnabled && !(*snes).vIrqEnabled {
                    (*snes).inIrq = false;
                    (*(*snes).cpu).irqWanted = false;
                }
            }
            0x4201 => {
                if val & 0x80 == 0 && (*snes).ppuLatch {
                    ppu_read((*snes).ppu, 0x37);
                }
                (*snes).ppuLatch = val & 0x80 != 0;
            }
            0x4202 => (*snes).multiplyA = val,
            0x4203 => (*snes).multiplyResult = u16::from((*snes).multiplyA) * u16::from(val),
            0x4204 => (*snes).divideA = ((*snes).divideA & 0xff00) | u16::from(val),
            0x4205 => (*snes).divideA = ((*snes).divideA & 0xff) | (u16::from(val) << 8),
            0x4206 => {
                if val == 0 {
                    (*snes).divideResult = 0xffff;
                    (*snes).multiplyResult = (*snes).divideA;
                } else {
                    (*snes).divideResult = (*snes).divideA / u16::from(val);
                    (*snes).multiplyResult = (*snes).divideA % u16::from(val);
                }
            }
            0x4207 => (*snes).hTimer = ((*snes).hTimer & 0x100) | u16::from(val),
            0x4208 => (*snes).hTimer = ((*snes).hTimer & 0xff) | (u16::from(val & 1) << 8),
            0x4209 => (*snes).vTimer = ((*snes).vTimer & 0x100) | u16::from(val),
            0x420a => (*snes).vTimer = ((*snes).vTimer & 0xff) | (u16::from(val & 1) << 8),
            0x420b => {
                if val as i32 == 2 {
                    let t: u32 =
                        (words!(i32, (*(*snes).dma).channel[1i32 as usize], aBank, aAdr)) as u32;
                    let data: i32 = snes_read(snes, t, game) as i32
                        | (snes_read(snes, t.wrapping_add(1), game) as i32) << 8;
                }
                dma_startDma((*snes).dma, val, false_0 != 0);
                while dma_cycle((*snes).dma, game) {}
            }
            0x420c => dma_startDma((*snes).dma, val, true_0 != 0),
            0x420d => (*snes).fastMem = val & 0x1 != 0,
            _ => {}
        };
    }
}
pub unsafe fn snes_read(snes: *mut Snes, address: u32, game: &mut Game) -> u8 {
    unsafe {
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        if matches!(bank, 0x7e | 0x7f) {
            let index = (usize::from(bank & 1) << 16) | usize::from(offset);
            return *(*snes).ram.add(index);
        }
        if matches!(bank, 0x00..=0x3f | 0x80..=0xbf) {
            match offset {
                0x0000..=0x1fff => return *(*snes).ram.add(usize::from(offset)),
                0x2100..=0x21ff => return snes_readBBus(snes, offset as u8, game),
                0x4016 | 0x4017 => crate::native_assertion! {
                    @failure
                    b"0\0", b"src/snes/snes.c\0", 396, b"uint8_t snes_read(Snes *, uint32_t)\0"
                },
                0x4200..=0x421f => return snes_readReg(snes, offset),
                0x4300..=0x437f => return dma_read((*snes).dma, offset),
                _ => {}
            }
        }
        cart_read((*snes).cart, bank, offset, game)
    }
}
pub unsafe fn snes_write(snes: *mut Snes, address: u32, value: u8, game: &mut Game) {
    unsafe {
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        if matches!(bank, 0x7e | 0x7f) {
            let index = (usize::from(bank & 1) << 16) | usize::from(offset);
            *(*snes).ram.add(index) = value;
        }
        if matches!(bank, 0x00..=0x3f | 0x80..=0xbf) {
            match offset {
                0x0000..=0x1fff => *(*snes).ram.add(usize::from(offset)) = value,
                0x2100..=0x21ff => snes_writeBBus(snes, offset as u8, value, game),
                0x4200..=0x421f => snes_writeReg(snes, offset, value, game),
                0x4300..=0x437f => dma_write((*snes).dma, offset, value),
                _ => {}
            }
        }
        cart_write((*snes).cart, bank, offset, value);
    }
}
pub unsafe fn snes_cpuRead(snes: *mut Snes, address: u32, game: &mut Game) -> u8 {
    unsafe {
        (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
        (*snes).cpuCyclesLeft = (*snes).cpuCyclesLeft.wrapping_add(8);
        snes_read(snes, address, game)
    }
}
pub unsafe fn snes_cpuWrite(snes: *mut Snes, address: u32, value: u8, game: &mut Game) {
    unsafe {
        (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
        (*snes).cpuCyclesLeft = (*snes).cpuCyclesLeft.wrapping_add(8);
        snes_write(snes, address, value, game);
    }
}
