use crate::c_library::{free, malloc, memset, puts};
use crate::game_state::Game;
use crate::snes::dsp::{
    dsp_cycle, dsp_free, dsp_init, dsp_read, dsp_reset, dsp_saveload, dsp_write,
};
use crate::snes::spc::{spc_free, spc_init, spc_reset, spc_runOpcode, spc_saveload};
use crate::tracing::getProcessorStateSpc;
pub use crate::types::{
    size_t, Apu, ApuHistory as C2RustUnnamed, Dsp, DspChannel, DspRegWriteHistory, SaveLoadFunc,
    Spc, Timer,
};
use ::core::ffi::{c_char, c_void};
use ::core::mem::size_of;
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
static bootRom: [u8; 64] = [
    0xcd, 0xef, 0xbd, 0xe8, 0, 0xc6, 0x1d, 0xd0, 0xfc, 0x8f, 0xaa, 0xf4, 0x8f, 0xbb, 0xf5, 0x78,
    0xcc, 0xf4, 0xd0, 0xfb, 0x2f, 0x19, 0xeb, 0xf4, 0xd0, 0xfc, 0x7e, 0xf4, 0xd0, 0xb, 0xe4, 0xf5,
    0xcb, 0xf4, 0xd7, 0, 0xfc, 0xd0, 0xf3, 0xab, 0x1, 0x10, 0xef, 0x7e, 0xf4, 0x10, 0xeb, 0xba,
    0xf6, 0xda, 0, 0xba, 0xf4, 0xc4, 0xf4, 0xdd, 0x5d, 0xd0, 0xdb, 0x1f, 0, 0, 0xc0, 0xff,
];
pub unsafe fn apu_init() -> *mut Apu {
    let apu: *mut Apu = malloc(size_of::<Apu>() as size_t) as *mut Apu;
    core::ptr::write_bytes(apu, 0, 1);
    (*apu).spc = spc_init(apu);
    (*apu).dsp = dsp_init(&raw mut (*apu).ram as *mut u8);
    apu
}
pub unsafe fn apu_free(apu: *mut Apu) {
    spc_free((*apu).spc);
    dsp_free((*apu).dsp);
    free(apu as *mut c_void);
}
pub unsafe fn apu_reset(apu: *mut Apu) {
    (*apu).romReadable = true;
    spc_reset((*apu).spc);
    dsp_reset((*apu).dsp);
    (*apu).ram.fill(0);
    (*apu).dspAdr = 0;
    (*apu).cycles = 0;
    (*apu).inPorts.fill(0);
    (*apu).outPorts.fill(0);
    for timer in &mut (*apu).timer {
        timer.cycles = 0;
        timer.divider = 0;
        timer.target = 0;
        timer.counter = 0;
        timer.enabled = false;
    }
    (*apu).cpuCyclesLeft = 7;
    (*apu).c2rust_unnamed.hist.count = 0;
}
pub unsafe fn apu_saveload(
    apu: *mut Apu,
    func: Option<SaveLoadFunc>,
    ctx: *mut c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*apu).ram as *mut u8 as *mut c_void,
        (65586 as size_t)
            .wrapping_add(6 as size_t)
            .wrapping_sub(16 as size_t),
        game,
    );
    dsp_saveload((*apu).dsp, func, ctx, game);
    spc_saveload((*apu).spc, func, ctx, game);
}

pub unsafe fn apu_cycle(apu: *mut Apu, game: &mut Game) {
    if (*apu).cpuCyclesLeft == 0 {
        if game.g_debug_apu_cycles {
            let mut line: [c_char; 80] = [0; 80];
            getProcessorStateSpc(apu, line.as_mut_ptr());
            puts(line.as_mut_ptr());
        }
        (*apu).cpuCyclesLeft = spc_runOpcode((*apu).spc) as u8;
    }
    (*apu).cpuCyclesLeft = (*apu).cpuCyclesLeft.wrapping_sub(1);
    if (*apu).cycles & 0x1f == 0 {
        dsp_cycle((*apu).dsp);
    }
    for i in 0..3 {
        let timer = &mut (*apu).timer[i];
        if timer.cycles == 0 {
            timer.cycles = if i == 2 { 16 } else { 128 };
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
    (*apu).cycles = (*apu).cycles.wrapping_add(1);
}
pub unsafe fn apu_cpuRead(apu: *mut Apu, adr: u16) -> u8 {
    match adr {
        0xf0 | 0xf1 | 0xfa..=0xfc => return 0,
        0xf2 => return (*apu).dspAdr,
        0xf3 => return dsp_read((*apu).dsp, (*apu).dspAdr & 0x7f),
        0xf4..=0xf9 => return (*apu).inPorts[usize::from(adr - 0xf4)],
        0xfd..=0xff => {
            let timer = &mut (*apu).timer[usize::from(adr - 0xfd)];
            let value = timer.counter;
            timer.counter = 0;
            return value;
        }
        _ => {}
    }
    if (*apu).romReadable && adr >= 0xffc0 {
        return bootRom[usize::from(adr - 0xffc0)];
    }
    (*apu).ram[usize::from(adr)]
}
fn record_dsp_write(history: &mut DspRegWriteHistory, address: u8, value: u8) {
    let index = history.count as i32;
    if index != 256 {
        history.count = (index + 1) as u32;
        history.addr[index as usize] = address;
        history.val[index as usize] = value;
    }
}

pub unsafe fn apu_cpuWrite(apu: *mut Apu, adr: u16, val: u8) {
    match adr {
        0xf1 => {
            for i in 0..3 {
                let enabled = val & (1 << i) != 0;
                let timer = &mut (*apu).timer[i];
                if !timer.enabled && enabled {
                    timer.divider = 0;
                    timer.counter = 0;
                }
                timer.enabled = enabled;
            }
            if val & 0x10 != 0 {
                (*apu).inPorts[0] = 0;
                (*apu).inPorts[1] = 0;
            }
            if val & 0x20 != 0 {
                (*apu).inPorts[2] = 0;
                (*apu).inPorts[3] = 0;
            }
            (*apu).romReadable = val & 0x80 != 0;
        }
        0xf2 => {
            (*apu).dspAdr = val;
        }
        0xf3 => {
            record_dsp_write(&mut (*apu).c2rust_unnamed.hist, (*apu).dspAdr, val);
            if ((*apu).dspAdr as i32) < 0x80i32 {
                dsp_write((*apu).dsp, (*apu).dspAdr, val);
            }
        }
        0xf4..=0xf7 => {
            (*apu).outPorts[usize::from(adr - 0xf4)] = val;
        }
        0xf8..=0xf9 => {
            (*apu).inPorts[usize::from(adr - 0xf4)] = val;
        }
        0xfa..=0xfc => {
            (*apu).timer[usize::from(adr - 0xfa)].target = val;
        }
        _ => {}
    }
    (*apu).ram[usize::from(adr)] = val;
}
