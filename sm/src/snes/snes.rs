use crate::execution::DmaRomSource;
use crate::game_state::WorkRam;
use crate::initial_state::InitialState;
use crate::rtl::words;
use crate::snapshot_io::Block;
use crate::snes::apu::{apu_reset, apu_saveload};
use crate::snes::cart::{cart_reset, cart_saveload};
use crate::snes::cpu::cpu_saveload;
use crate::snes::ppu::{
    ppu_checkOverscan, ppu_handleVblank, ppu_read, ppu_reset, ppu_saveload, ppu_write,
};
use crate::types::{Apu, ApuRegisters, Cart, Cpu, Dma, DmaChannel, Dsp, Input, Ppu, Spc};
pub use crate::types::{Snes, size_t};
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
    input1: Cell<Input>,
    input2: Cell<Input>,
    // Keep WRAM alive until every device owner and its borrowed links drop.
    _ram: WorkRam,
}
#[forbid(unsafe_code)]
impl Backend {
    pub(crate) fn new(ram: WorkRam) -> Self {
        let snes = Rc::new(Cell::new(Snes::INITIAL));
        let cpu = Rc::new(Cell::new(Cpu::INITIAL));
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
        let dma = Rc::new(Cell::new(Dma { ..Dma::INITIAL }));
        let my_ppu = Rc::new(Cell::new(Ppu {
            snes: snes.as_ptr(),
            ..Ppu::INITIAL
        }));
        let snes_ppu = Rc::new(Cell::new(Ppu {
            snes: snes.as_ptr(),
            ..Ppu::INITIAL
        }));
        let cart = Rc::new(UnsafeCell::new(Cart {
            type_0: 0,
            rom: crate::rom::RomImage::default(),
            romSize: 0,
            ram: crate::save_ram::SaveRam::default(),
            ramSize: 0,
        }));
        let input1 = Cell::new(Input {
            type_0: 1,
            currentState: 0,
        });
        let input2 = Cell::new(Input {
            type_0: 1,
            currentState: 0,
        });
        snes.set(Snes {
            cpu: cpu.as_ptr(),
            apu: apu.get(),
            my_ppu: my_ppu.as_ptr(),
            snes_ppu: snes_ppu.as_ptr(),
            ppu: snes_ppu.as_ptr(),
            cart: cart.get(),
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
            _ram: ram,
        }
    }

    pub(crate) fn input1_mut(&mut self) -> &mut Input {
        self.input1.get_mut()
    }

    pub(crate) fn input2_mut(&mut self) -> &mut Input {
        self.input2.get_mut()
    }

    pub(crate) fn dma_channels(&self) -> [DmaChannel; 8] {
        self.dma.get().channel
    }

    pub(crate) fn dma_state(&self) -> Dma {
        self.dma.get()
    }

    pub(crate) fn set_dma_state(&self, state: Dma) {
        self.dma.set(state);
    }

    pub(crate) fn dma_busy(&self) -> bool {
        self.dma.get().dmaBusy
    }

    pub(crate) fn apu_view(&self) -> *mut Apu {
        self.apu.get()
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

fn SwapInputBits(input: u16) -> u16 {
    input.reverse_bits()
}

impl crate::execution::Execution<'_> {
    pub(crate) fn snes(&self) -> *mut Snes {
        self.backend.views().0
    }

    pub(crate) fn cart(&self) -> *mut Cart {
        self.backend.cart.get()
    }

    fn selected_ppu(&self) -> *mut Ppu {
        let selected = self.backend.snes.get().ppu;
        if selected == self.backend.snes_ppu.as_ptr() {
            return self.backend.snes_ppu.as_ptr();
        }
        assert_eq!(
            selected,
            self.backend.my_ppu.as_ptr(),
            "execution requires an owned PPU"
        );
        self.backend.my_ppu.as_ptr()
    }
}

// Owned operations recurse through this context and the same retained allocations.
impl crate::execution::Execution<'_> {
    pub(crate) unsafe fn save_load(&mut self, io: &mut crate::snapshot_io::SnapshotIo<'_>) {
        unsafe {
            let snes = self.snes();
            cpu_saveload(self.selected_cpu(), io);
            apu_saveload(self.backend.apu_view(), io);
            let result = {
                let mut dma = self.backend.dma_state();
                let result = io.state(&mut dma);
                if result.is_ok() && matches!(io, crate::snapshot_io::SnapshotIo::Load(_)) {
                    self.backend.set_dma_state(dma);
                }
                result
            };
            crate::rtl::snapshot_result(result);
            ppu_saveload(self.selected_ppu(), io);
            cart_saveload(self.cart(), io);
            crate::rtl::snapshot_result(io.state(&mut *snes));
            let ram = self.ram();
            let result = io.block(0x20000, |block| match block {
                Block::Save(bytes) => ram.copy_out(0, bytes),
                Block::Load(bytes) => ram.copy_in(0, bytes),
            });
            crate::rtl::snapshot_result(result);
            crate::rtl::snapshot_result(io.state(&mut (*snes).ramAdr));
            (*snes).runningWhichVersion = 0;
        }
    }

    /// # Safety
    /// The current SNES-selected CPU must be this Backend's live, exclusive CPU.
    /// All retained device links must target this Backend; native and device
    /// consumers must be quiescent.
    pub(crate) unsafe fn reset(&mut self, hard: bool) {
        unsafe {
            let snes = self.snes();
            cart_reset(self.cart());
            self.reset_cpu();
            apu_reset(self.backend.apu_view());
            self.reset_dma();
            ppu_reset(self.backend.my_ppu.as_ptr());
            ppu_reset(self.backend.snes_ppu.as_ptr());
            if hard {
                self.ram().fill(0..0x20000, 0);
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

    pub(crate) unsafe fn advance_beam(&mut self) {
        unsafe {
            let snes = self.snes();
            if (*snes).hPos == 0 {
                let mut startingVblank = false;
                if (*snes).vPos == 0 {
                    (*snes).inVblank = false;
                    (*snes).inNmi = false;
                    self.initialize_hdma();
                } else if (*snes).vPos == 225 {
                    startingVblank = !ppu_checkOverscan(self.selected_ppu());
                } else if (*snes).vPos == 240 && !(*snes).inVblank {
                    startingVblank = true;
                }
                if startingVblank {
                    ppu_handleVblank(self.selected_ppu());
                    (*snes).inVblank = true;
                    (*snes).inNmi = true;
                    (*self.selected_cpu()).nmiWanted = true;
                    if (*snes).autoJoyRead {
                        (*snes).autoJoyTimer = 0;
                    }
                }
            } else if (*snes).hPos == 512 {
                if !(*snes).inVblank && !(*snes).disableRender {
                    crate::snes::ppu::ppu_run_line(
                        self.selected_ppu(),
                        i32::from((*snes).vPos),
                        self.host.g_new_ppu,
                    );
                }
            } else if (*snes).hPos == 1024 && !(*snes).inVblank {
                self.step_hdma();
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

    pub(crate) unsafe fn catch_up_apu(&mut self) {
        unsafe {
            let snes = self.snes();
            if (*snes).apuCatchupCycles > 10000.0 {
                (*snes).apuCatchupCycles = 10000.0;
            }
            let cycles = (*snes).apuCatchupCycles as i32;
            for _ in 0..cycles {
                self.cycle_apu();
            }
            (*snes).apuCatchupCycles -= f64::from(cycles);
        }
    }

    pub(crate) unsafe fn read_bbus(&mut self, address: u8) -> u8 {
        unsafe {
            let snes = self.snes();
            match address {
                0x00..=0x3f => ppu_read(self.selected_ppu(), address),
                0x40..=0x7f => {
                    (*snes).apuCatchupCycles = 32.0;
                    self.catch_up_apu();
                    (*self.backend.apu_view()).registers.outPorts[usize::from(address & 3)]
                }
                0x80 => {
                    crate::update_place! { previous address = (*snes).ramAdr => (*snes).ramAdr.wrapping_add(1); }
                    let value = self.ram().read_byte(address as usize);
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

    pub(crate) unsafe fn write_bbus(&mut self, address: u8, value: u8) {
        unsafe {
            let snes = self.snes();
            match address {
                0x00..=0x3f => ppu_write(self.selected_ppu(), address, value),
                0x40..=0x7f => crate::rtl::apu_write(self, 0x2100 + u32::from(address), value),
                0x80 => {
                    crate::update_place! { previous address = (*snes).ramAdr => (*snes).ramAdr.wrapping_add(1); }
                    self.ram().write_byte(address as usize, value);
                    (*snes).ramAdr &= 0x1ffff;
                }
                0x81 => (*snes).ramAdr = ((*snes).ramAdr & 0x1ff00) | u32::from(value),
                0x82 => (*snes).ramAdr = ((*snes).ramAdr & 0x100ff) | (u32::from(value) << 8),
                0x83 => (*snes).ramAdr = ((*snes).ramAdr & 0xffff) | (u32::from(value & 1) << 16),
                _ => {}
            }
        }
    }

    unsafe fn read_register(&mut self, address: u16) -> u8 {
        unsafe {
            let snes = self.snes();
            match address {
                0x4210 => 2 | (u8::from((*snes).inNmi) << 7) | ((*snes).openBus & 0x70),
                0x4211 => {
                    let irq = u8::from((*snes).inIrq) << 7;
                    (*snes).inIrq = false;
                    (*self.selected_cpu()).irqWanted = false;
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
                0x4218 => SwapInputBits(self.backend.input1.get().currentState) as u8,
                0x4219 => (SwapInputBits(self.backend.input1.get().currentState) >> 8) as u8,
                0x421a..=0x421f => 0,
                _ => (*snes).openBus,
            }
        }
    }

    unsafe fn write_register(&mut self, adr: u16, val: u8, source: Option<&DmaRomSource<'_>>) {
        unsafe {
            let snes = self.snes();
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
                        (*self.selected_cpu()).irqWanted = false;
                    }
                }
                0x4201 => {
                    if val & 0x80 == 0 && (*snes).ppuLatch {
                        ppu_read(self.selected_ppu(), 0x37);
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
                        let t: u32 = (words!(
                            i32,
                            self.backend.dma_state().channel[1i32 as usize],
                            aBank,
                            aAdr
                        )) as u32;
                        let _data: i32 = self.read_with_dma_source(t, source) as i32
                            | (self.read_with_dma_source(t.wrapping_add(1), source) as i32) << 8;
                    }
                    self.start_dma(val, false_0 != 0);
                    while self.step_dma_with_source(source) {}
                }
                0x420c => self.start_dma(val, true_0 != 0),
                0x420d => (*snes).fastMem = val & 0x1 != 0,
                _ => {}
            };
        }
    }

    pub(crate) unsafe fn read(&mut self, address: u32) -> u8 {
        unsafe { self.read_with_dma_source(address, None) }
    }

    pub(crate) unsafe fn read_with_dma_source(
        &mut self,
        address: u32,
        source: Option<&DmaRomSource<'_>>,
    ) -> u8 {
        unsafe {
            let bank = (address >> 16) as u8;
            let offset = address as u16;
            if matches!(bank, 0x7e | 0x7f) {
                let index = (usize::from(bank & 1) << 16) | usize::from(offset);
                return self.ram().read_byte(index);
            }
            if matches!(bank, 0x00..=0x3f | 0x80..=0xbf) {
                match offset {
                    0x0000..=0x1fff => return self.ram().read_byte(usize::from(offset)),
                    0x2100..=0x21ff => return self.read_bbus(offset as u8),
                    0x4016 | 0x4017 => crate::native_assertion! {
                        @failure
                        b"0\0", b"src/snes/snes.c\0", 396, b"uint8_t snes_read(Snes *, uint32_t)\0"
                    },
                    0x4200..=0x421f => return self.read_register(offset),
                    0x4300..=0x437f => return self.read_dma_register(offset),
                    _ => {}
                }
            }
            match source {
                Some(source) => self.read_cart_with_dma_source(bank, offset, Some(source)),
                None => self.read_cart(bank, offset),
            }
        }
    }

    pub(crate) unsafe fn write(&mut self, address: u32, value: u8) {
        unsafe { self.write_with_dma_source(address, value, None) }
    }

    pub(crate) unsafe fn write_with_dma_source(
        &mut self,
        address: u32,
        value: u8,
        source: Option<&DmaRomSource<'_>>,
    ) {
        unsafe {
            let bank = (address >> 16) as u8;
            let offset = address as u16;
            if matches!(bank, 0x7e | 0x7f) {
                let index = (usize::from(bank & 1) << 16) | usize::from(offset);
                self.ram().write_byte(index, value);
            }
            if matches!(bank, 0x00..=0x3f | 0x80..=0xbf) {
                match offset {
                    0x0000..=0x1fff => self.ram().write_byte(usize::from(offset), value),
                    0x2100..=0x21ff => self.write_bbus(offset as u8, value),
                    0x4200..=0x421f => self.write_register(offset, value, source),
                    0x4300..=0x437f => self.backend.write_dma_register(offset, value),
                    _ => {}
                }
            }
            self.write_cart(bank, offset, value);
        }
    }

    pub(crate) unsafe fn cpu_read(&mut self, address: u32) -> u8 {
        unsafe {
            let snes = self.snes();
            (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
            (*snes).cpuCyclesLeft = (*snes).cpuCyclesLeft.wrapping_add(8);
            self.read(address)
        }
    }

    pub(crate) unsafe fn cpu_write(&mut self, address: u32, value: u8) {
        unsafe {
            let snes = self.snes();
            (*snes).cpuMemOps = (*snes).cpuMemOps.wrapping_add(1);
            (*snes).cpuCyclesLeft = (*snes).cpuCyclesLeft.wrapping_add(8);
            self.write(address, value);
        }
    }
}
