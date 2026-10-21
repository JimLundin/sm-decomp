use crate::c_library::{__assert_fail, free, malloc, memset, printf};
use crate::cpu_infra::{CpuOpcodeHook, HookedFunctionRts};
use crate::game_state::Game;
use crate::host::Die;
use crate::rtl::words;
use crate::set_fields;
use crate::snes::snes::{snes_cpuRead, snes_cpuWrite};
use crate::types::{Cpu, SaveLoadFunc, Snes};
use ::core::ffi::{c_char, c_void};
use ::core::mem::size_of;
static CYCLES_PER_OPCODE: [u8; 256] = [
    7, 6, 7, 4, 5, 3, 5, 6, 3, 2, 2, 4, 6, 4, 6, 5, 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 2, 2, 6, 4, 7, 5,
    6, 6, 8, 4, 3, 3, 5, 6, 4, 2, 2, 5, 4, 4, 6, 5, 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 2, 2, 4, 4, 7, 5,
    6, 6, 2, 4, 7, 3, 5, 6, 3, 2, 2, 3, 3, 4, 6, 5, 2, 5, 5, 7, 7, 4, 6, 6, 2, 4, 3, 2, 4, 4, 7, 5,
    6, 6, 6, 4, 3, 3, 5, 6, 4, 2, 2, 6, 5, 4, 6, 5, 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 4, 2, 6, 4, 7, 5,
    3, 6, 4, 4, 3, 3, 3, 6, 2, 2, 2, 3, 4, 4, 4, 5, 2, 6, 5, 7, 4, 4, 4, 6, 2, 5, 2, 2, 4, 5, 5, 5,
    2, 6, 2, 4, 3, 3, 3, 6, 2, 2, 2, 4, 4, 4, 4, 5, 2, 5, 5, 7, 4, 4, 4, 6, 2, 4, 2, 2, 4, 4, 4, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, 2, 5, 5, 7, 6, 4, 6, 6, 2, 4, 3, 3, 6, 4, 7, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5, 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 4, 2, 8, 4, 7, 5,
];

unsafe fn read_byte(cpu: *mut Cpu, adr: u32, game: &mut Game) -> u8 {
    snes_cpuRead((*cpu).mem as *mut Snes, adr, game)
}

unsafe fn write_byte(cpu: *mut Cpu, adr: u32, val: u8, game: &mut Game) {
    snes_cpuWrite((*cpu).mem as *mut Snes, adr, val, game);
}

pub unsafe fn cpu_init(mem: *mut c_void, memType: i32) -> *mut Cpu {
    let cpu = malloc(size_of::<Cpu>()) as *mut Cpu;
    memset(cpu as *mut c_void, 0, size_of::<Cpu>());
    (*cpu).mem = mem;
    (*cpu).memType = memType as u8;
    cpu
}

fn finish_reset(cpu: &mut Cpu) {
    set_fields! {[cpu], 0; dp, k, db}
    set_fields! {[cpu]; c = false, z = false, v = false, n = false, i = true, d = false,
    xf = true, mf = true, e = true, irqWanted = false, nmiWanted = false, waiting = false,
    stopped = false, cyclesUsed = 0, };
}

pub unsafe fn cpu_reset(cpu: *mut Cpu, game: &mut Game) {
    set_fields! {[(*cpu)], 0; a, x, y}
    (*cpu).sp = 0x100;
    (*cpu).pc = read_word(cpu, 0xfffc, 0xfffd, game);
    finish_reset(&mut *cpu);
}

pub unsafe fn cpu_saveload(
    cpu: *mut Cpu,
    func: Option<SaveLoadFunc>,
    ctx: *mut c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(ctx, &raw mut (*cpu).a as *mut c_void, 27, game);
    (*cpu).spBreakpoint = 0;
}

pub unsafe fn cpu_runOpcode(cpu: *mut Cpu, game: &mut Game) -> i32 {
    (*cpu).cyclesUsed = 0;
    if (*cpu).stopped {
        return 1;
    }
    if (*cpu).waiting {
        if !(*cpu).irqWanted && !(*cpu).nmiWanted {
            return 1;
        }
        // IRQ wakes WAI even when I prevents interrupt service.
        (*cpu).waiting = false;
    }
    if (!(*cpu).i && (*cpu).irqWanted) || (*cpu).nmiWanted {
        (*cpu).cyclesUsed = 7;
        if (*cpu).nmiWanted {
            (*cpu).nmiWanted = false;
            service_interrupt(cpu, false, game);
        } else {
            service_interrupt(cpu, true, game);
        }
    }
    let opcode = fetch_byte(cpu, game);
    (*cpu).cyclesUsed = (*cpu)
        .cyclesUsed
        .wrapping_add(CYCLES_PER_OPCODE[usize::from(opcode)]);
    dispatch_opcode(cpu, opcode, game);
    i32::from((*cpu).cyclesUsed)
}

fn next_program_address(cpu: &mut Cpu) -> u32 {
    crate::update_place! { previous pc = cpu.pc => cpu.pc.wrapping_add(1); }
    u32::from(cpu.k) << 16 | u32::from(pc)
}

unsafe fn fetch_byte(cpu: *mut Cpu, game: &mut Game) -> u8 {
    let address = next_program_address(&mut *cpu);
    read_byte(cpu, address, game)
}

unsafe fn fetch_word(cpu: *mut Cpu, game: &mut Game) -> u16 {
    let low = fetch_byte(cpu, game);
    let high = fetch_byte(cpu, game);
    u16::from_le_bytes([low, high])
}

pub fn cpu_getFlags(cpu: &Cpu) -> u8 {
    (u8::from(cpu.n) << 7)
        | (u8::from(cpu.v) << 6)
        | (u8::from(cpu.mf) << 5)
        | (u8::from(cpu.xf) << 4)
        | (u8::from(cpu.d) << 3)
        | (u8::from(cpu.i) << 2)
        | (u8::from(cpu.z) << 1)
        | u8::from(cpu.c)
}

pub fn cpu_setFlags(val: u8, cpu: &mut Cpu) {
    set_fields! {[cpu]; n = val & 0x80 != 0, v = val & 0x40 != 0, mf = val & 0x20 != 0,
    xf = val & 0x10 != 0, d = val & 0x08 != 0, i = val & 0x04 != 0, z = val & 0x02 != 0,
    c = val & 0x01 != 0, };
    if cpu.e {
        cpu.mf = true;
        cpu.xf = true;
        cpu.sp = (cpu.sp & 0xff) | 0x100;
    }
    if cpu.xf {
        cpu.x &= 0xff;
        cpu.y &= 0xff;
    }
}

fn adjust_stack_pointer(delta: u16, cpu: &mut Cpu) {
    cpu.sp = cpu.sp.wrapping_add(delta);
    if cpu.e {
        cpu.sp = cpu.sp & 0xff | 0x100;
    }
}

unsafe fn pull_byte(cpu: *mut Cpu, game: &mut Game) -> u8 {
    adjust_stack_pointer(1, &mut *cpu);
    read_byte(cpu, u32::from((*cpu).sp), game)
}

unsafe fn push_byte(cpu: *mut Cpu, value: u8, game: &mut Game) {
    write_byte(cpu, u32::from((*cpu).sp), value, game);
    adjust_stack_pointer(u16::MAX, &mut *cpu);
}

unsafe fn pull_word(cpu: *mut Cpu, game: &mut Game) -> u16 {
    let low = pull_byte(cpu, game);
    let high = pull_byte(cpu, game);
    u16::from_le_bytes([low, high])
}

unsafe fn push_word(cpu: *mut Cpu, value: u16, game: &mut Game) {
    let [low, high] = value.to_le_bytes();
    push_byte(cpu, high, game);
    push_byte(cpu, low, game);
}

unsafe fn read_word(cpu: *mut Cpu, adrl: u32, adrh: u32, game: &mut Game) -> u16 {
    let low = read_byte(cpu, adrl, game);
    let high = read_byte(cpu, adrh, game);
    u16::from_le_bytes([low, high])
}

unsafe fn service_interrupt(cpu: *mut Cpu, irq: bool, game: &mut Game) {
    if !(*cpu).e {
        push_byte(cpu, (*cpu).k, game);
        (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
    }
    push_word(cpu, (*cpu).pc, game);
    let flags = cpu_getFlags(&*cpu);
    // Bit 4 is the cleared B flag on an emulation-mode hardware interrupt.
    push_byte(cpu, if (*cpu).e { flags & !0x10 } else { flags }, game);
    (*cpu).i = true;
    (*cpu).d = false;
    (*cpu).k = 0;
    let vector = match ((*cpu).e, irq) {
        (false, false) => 0xffea,
        (false, true) => 0xffee,
        (true, false) => 0xfffa,
        (true, true) => 0xfffe,
    };
    (*cpu).pc = read_word(cpu, vector, vector + 1, game);
}

unsafe fn fetch_long_address(cpu: *mut Cpu, game: &mut Game) -> u32 {
    let offset = fetch_word(cpu, game);
    let bank = fetch_byte(cpu, game);
    u32::from(bank) << 16 | u32::from(offset)
}

unsafe fn address_absolute_indexed_indirect(cpu: *mut Cpu, game: &mut Game) -> u16 {
    let adr = fetch_word(cpu, game);
    let (low, high) = indexed_jump_pointer(adr, &*cpu);
    read_word(cpu, low, high, game)
}

fn indexed_jump_pointer(offset: u16, cpu: &Cpu) -> (u32, u32) {
    let pointer = offset.wrapping_add(cpu.x);
    let bank = u32::from(cpu.k) << 16;
    (
        bank | u32::from(pointer),
        bank | u32::from(pointer.wrapping_add(1)),
    )
}

pub unsafe fn DumpCpuHistory(game: &mut Game) {
    for i in 0i32..8 {
        printf(
            c"PC history: 0x%x\n".as_ptr(),
            game.pc_hist[(game.pc_hist_ctr.wrapping_add(i as u32) & 7_) as usize],
        );
    }
}

fn record_opcode_history(bank: u8, pc: u16, game: &mut Game) {
    // Preserve the inherited history value of 0xffffffff when PC is zero.
    let cur_pc = (i32::from(bank) << 16 | (i32::from(pc) - 1)) as u32;
    game.pc_hist[game.pc_hist_ctr as usize] = cur_pc;
    game.pc_hist_ctr = game.pc_hist_ctr.wrapping_add(1) & 7;
}

// Public Cpu layout from unmodified dinkc64/LakeSnes 048a0d72568668f74e0cdf371dcd9e8efd614965.
crate::typed_struct! {
    #[repr(C)]
    struct LakeCpu {
        mem: *mut c_void;
        read: Option<unsafe extern "C" fn(*mut c_void, u32) -> u8>;
        write: Option<unsafe extern "C" fn(*mut c_void, u32, u8)>;
        idle: Option<unsafe extern "C" fn(*mut c_void, bool)>;
        a, x, y, sp, pc, dp: u16;
        k, db: u8;
        c, z, v, n, i, d, xf, mf, e, waiting, stopped: bool;
        irqWanted, nmiWanted, intWanted, intDelay, resetWanted: bool;
    }
}
#[link(name = "lakesnes_cpu", kind = "static", modifiers = "-bundle")]
extern "C" {
    #[link_name = "cpu_runOpcode"]
    fn lake_run_opcode(cpu: *mut LakeCpu);
}

// Copy the public register values without imposing canonical E/M/X widths.
// Interrupt latches, WAI/STP and debugger state remain in the current CPU.
macro_rules! transfer_registers {
    ($destination:expr, $source:expr) => {{
        let destination = $destination;
        let source = $source;
        set_fields! {[destination]; a = source.a, x = source.x, y = source.y,
        sp = source.sp, pc = source.pc, dp = source.dp, k = source.k, db = source.db,
        c = source.c, z = source.z, v = source.v, n = source.n, i = source.i,
        d = source.d, xf = source.xf, mf = source.mf, e = source.e, };
    }};
}
struct LakeBus {
    cpu: *mut Cpu,
    core: *mut LakeCpu,
    game: *mut Game,
    opcode: Option<u8>,
    cycles: u8,
    accesses: u8,
    indexed_at: u8,
    indexed_write: bool,
    index: u16,
}
impl LakeBus {
    unsafe fn access(&mut self, address: u32) {
        self.cycles = self.cycles.wrapping_add(1);
        self.accesses += 1;
        if self.accesses == self.indexed_at {
            let core = &*self.core;
            let offset = address
                .wrapping_sub(u32::from(core.db) << 16)
                .wrapping_sub(u32::from(self.index)) as u16;
            let surcharge = u8::from(
                !core.xf
                    || u32::from(offset) >> 8 != (u32::from(offset) + u32::from(self.index)) >> 8,
            );
            // Legacy indexed reads never add this surcharge; writes add it on
            // top of their table's fixed cost, which already includes an idle.
            self.cycles = if self.indexed_write {
                self.cycles.wrapping_add(surcharge)
            } else {
                self.cycles.wrapping_sub(surcharge)
            };
        }
    }
}
unsafe extern "C" fn lake_read(context: *mut c_void, address: u32) -> u8 {
    let bus = &mut *context.cast::<LakeBus>();
    bus.access(address);
    if let Some(opcode) = bus.opcode.take() {
        return opcode;
    }
    transfer_registers!(&mut *bus.cpu, &*bus.core);
    let value = read_byte(bus.cpu, address, &mut *bus.game);
    transfer_registers!(&mut *bus.core, &*bus.cpu);
    value
}
unsafe extern "C" fn lake_write(context: *mut c_void, address: u32, value: u8) {
    let bus = &mut *context.cast::<LakeBus>();
    bus.access(address);
    transfer_registers!(&mut *bus.cpu, &*bus.core);
    write_byte(bus.cpu, address, value, &mut *bus.game);
    transfer_registers!(&mut *bus.core, &*bus.cpu);
}
unsafe extern "C" fn lake_idle(context: *mut c_void, _waiting: bool) {
    let bus = &mut *context.cast::<LakeBus>();
    bus.cycles = bus.cycles.wrapping_add(1);
}
unsafe fn execute_upstream(cpu: *mut Cpu, opcode: u8, game: &mut Game) {
    // No foreign state persists across instruction boundaries or snapshots.
    let mut core: LakeCpu = core::mem::zeroed();
    transfer_registers!(&mut core, &*cpu);
    core.pc = core.pc.wrapping_sub(1);
    core.read = Some(lake_read);
    core.write = Some(lake_write);
    core.idle = Some(lake_idle);
    let mode = opcode & 0x1f;
    let indexed = matches!(mode, 0x11 | 0x19 | 0x1d)
        || matches!(
            opcode,
            0x3c | 0xbc | 0xbe | 0x9e | 0x1e | 0x3e | 0x5e | 0x7e | 0xde | 0xfe
        );
    let index = if matches!(mode, 0x11 | 0x19) || opcode == 0xbe {
        core.y
    } else {
        core.x
    };
    let mut bus = LakeBus {
        cpu,
        core: &mut core,
        game,
        opcode: Some(opcode),
        cycles: 0,
        accesses: 0,
        indexed_at: if indexed {
            if mode == 0x11 {
                5
            } else {
                4
            }
        } else {
            0
        },
        indexed_write: opcode != 0xbe && (opcode >> 5 == 4 || mode == 0x1e),
        index,
    };
    core.mem = (&mut bus as *mut LakeBus).cast();
    // The existing wrapper already fetched the opcode and serviced interrupts.
    // Its base cost belongs to the original byte, even when a hook replaced it.
    lake_run_opcode(&mut core);
    transfer_registers!(&mut *cpu, &core);
    (*cpu).cyclesUsed = (*cpu)
        .cyclesUsed
        .wrapping_add(bus.cycles.wrapping_sub(CYCLES_PER_OPCODE[opcode as usize]));
}

unsafe fn dispatch_opcode(cpu: *mut Cpu, mut opcode: u8, game: &mut Game) {
    record_opcode_history((*cpu).k, (*cpu).pc, game);
    if opcode == 0 {
        let addr = (u32::from((*cpu).k) << 16) | u32::from((*cpu).pc) | 0x800000;
        opcode = CpuOpcodeHook(addr.wrapping_sub(1), game) as u8;
        match opcode {
            0x00 => return,
            0x01 => {
                (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
                return;
            }
            0x02 => {
                (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
                (*cpu).k = pull_byte(cpu, game);
                return;
            }
            0xe5 | 0xe9 | 0xed => (*cpu).c = true,
            0x65 | 0x6d | 0x69 | 0x7d | 0x7f => (*cpu).c = false,
            _ => {}
        }
    }
    match opcode {
        0x02 => {
            fetch_byte(cpu, game);
            push_byte(cpu, (*cpu).k, game);
            push_word(cpu, (*cpu).pc, game);
            push_byte(cpu, cpu_getFlags(&*cpu), game);
            set_fields! {[(*cpu)]; cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1), i = true,
            d = false, k = 0_, pc = read_word(cpu, 0xffe4_, 0xffe5_, game), };
        }
        0x1b => ((*cpu).sp = (*cpu).a),
        0x22 => {
            let address = fetch_long_address(cpu, game);
            let target = address as u16;
            let newK = (address >> 16) as u8;
            push_byte(cpu, (*cpu).k, game);
            push_word(cpu, (*cpu).pc.wrapping_sub(1), game);
            (*cpu).pc = target;
            (*cpu).k = newK;
        }
        0x40 => {
            cpu_setFlags(pull_byte(cpu, game), &mut *cpu);
            (*cpu).pc = pull_word(cpu, game);
            if !(*cpu).e {
                (*cpu).cyclesUsed = (*cpu).cyclesUsed.wrapping_add(1);
                (*cpu).k = pull_byte(cpu, game);
            }
        }
        0x42 => {
            fetch_byte(cpu, game);
        }
        0x44 => {
            let dest = fetch_byte(cpu, game);
            let src = fetch_byte(cpu, game);
            (*cpu).db = dest;
            write_byte(
                cpu,
                u32::from(dest) << 16 | u32::from((*cpu).y),
                read_byte(cpu, u32::from(src) << 16 | u32::from((*cpu).x), game),
                game,
            );
            (*cpu).a = (*cpu).a.wrapping_sub(1);
            (*cpu).x = (*cpu).x.wrapping_sub(1);
            (*cpu).y = (*cpu).y.wrapping_sub(1);
            if (*cpu).a != 0xffff {
                (*cpu).pc = (*cpu).pc.wrapping_sub(3);
            }
            if (*cpu).xf {
                (*cpu).x &= 0xff;
                (*cpu).y &= 0xff;
            }
        }
        0x54 => {
            let dest_0 = fetch_byte(cpu, game);
            let src_0 = fetch_byte(cpu, game);
            (*cpu).db = dest_0;
            write_byte(
                cpu,
                u32::from(dest_0) << 16 | u32::from((*cpu).y),
                read_byte(cpu, u32::from(src_0) << 16 | u32::from((*cpu).x), game),
                game,
            );
            (*cpu).a = (*cpu).a.wrapping_sub(1);
            (*cpu).x = (*cpu).x.wrapping_add(1);
            (*cpu).y = (*cpu).y.wrapping_add(1);
            if (*cpu).a != 0xffff {
                (*cpu).pc = (*cpu).pc.wrapping_sub(3);
            }
            if (*cpu).xf {
                (*cpu).x &= 0xff;
                (*cpu).y &= 0xff;
            }
        }
        0x5c => {
            let address = fetch_long_address(cpu, game);
            let target = address as u16;
            let target_bank = (address >> 16) as u8;
            if target_bank == 0x80 && target == 0x8573 {
                printf(
                    b"Current PC = 0x%x\n\0" as *const u8 as *const c_char,
                    words!(i32, (*cpu), k, pc),
                );
                DumpCpuHistory(game);
                Die(b"The game has crashed!\n\0" as *const u8 as *const c_char);
            }
            (*cpu).k = target_bank;
            (*cpu).pc = target;
        }
        0x60 => {
            if (*cpu).sp >= (*cpu).spBreakpoint && (*cpu).spBreakpoint != 0 {
                let delta = (*cpu).sp - (*cpu).spBreakpoint;
                if delta != 0 && delta != 2 {
                    crate::native_assertion! {
                        @failure
                        b"delta == 0 || delta == 2\0", b"src/snes/cpu.c\0", 1386,
                        b"void cpu_doOpcode(Cpu *, uint8_t)\0"
                    };
                }
                (*cpu).spBreakpoint = 0;
                if HookedFunctionRts(i32::from(delta == 2), game) {
                    return;
                }
            }
            (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
        }
        0x6b => {
            if (*cpu).sp >= (*cpu).spBreakpoint && (*cpu).spBreakpoint != 0 {
                if (*cpu).sp != (*cpu).spBreakpoint {
                    crate::native_assertion! {
                        @failure
                        b"cpu->sp == cpu->spBreakpoint\0", b"src/snes/cpu.c\0", 1464,
                        b"void cpu_doOpcode(Cpu *, uint8_t)\0"
                    };
                }
                (*cpu).spBreakpoint = 0;
                if HookedFunctionRts(0, game) {
                    return;
                }
            }
            (*cpu).pc = pull_word(cpu, game).wrapping_add(1);
            (*cpu).k = pull_byte(cpu, game);
        }
        0x9a => ((*cpu).sp = (*cpu).x),
        0xcb => ((*cpu).waiting = true),
        0xdb => ((*cpu).stopped = true),
        0xf4 => {
            let w: u16 = fetch_word(cpu, game);
            if w == 0xc2ae {
                (*cpu).y = 0;
            }
            push_word(cpu, w, game);
        }
        0xfc => {
            let target = address_absolute_indexed_indirect(cpu, game);
            push_word(cpu, (*cpu).pc.wrapping_sub(1), game);
            (*cpu).pc = target;
        }
        _ => execute_upstream(cpu, opcode, game),
    }
}
