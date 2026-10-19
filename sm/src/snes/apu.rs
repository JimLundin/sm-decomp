use crate::game_state::Game;
use crate::snes::dsp::{
    dsp_cycle, dsp_free, dsp_init, dsp_read, dsp_reset, dsp_saveload, dsp_write,
};
use crate::snes::spc::{spc_free, spc_init, spc_reset, spc_runOpcode, spc_saveload};
use crate::tracing::getProcessorStateSpc;
extern "C" {
    fn puts(__s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

}
pub use crate::types::{
    __int16_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16_t, int8_t, size_t, uint16_t,
    uint32_t, uint8_t, Apu, ApuHistory as C2RustUnnamed, Dsp, DspChannel, DspRegWriteHistory,
    SaveLoadFunc, Spc, Timer,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static bootRom: [uint8_t; 64] = [
    0xcd as ::core::ffi::c_int as uint8_t,
    0xef as ::core::ffi::c_int as uint8_t,
    0xbd as ::core::ffi::c_int as uint8_t,
    0xe8 as ::core::ffi::c_int as uint8_t,
    0 as ::core::ffi::c_int as uint8_t,
    0xc6 as ::core::ffi::c_int as uint8_t,
    0x1d as ::core::ffi::c_int as uint8_t,
    0xd0 as ::core::ffi::c_int as uint8_t,
    0xfc as ::core::ffi::c_int as uint8_t,
    0x8f as ::core::ffi::c_int as uint8_t,
    0xaa as ::core::ffi::c_int as uint8_t,
    0xf4 as ::core::ffi::c_int as uint8_t,
    0x8f as ::core::ffi::c_int as uint8_t,
    0xbb as ::core::ffi::c_int as uint8_t,
    0xf5 as ::core::ffi::c_int as uint8_t,
    0x78 as ::core::ffi::c_int as uint8_t,
    0xcc as ::core::ffi::c_int as uint8_t,
    0xf4 as ::core::ffi::c_int as uint8_t,
    0xd0 as ::core::ffi::c_int as uint8_t,
    0xfb as ::core::ffi::c_int as uint8_t,
    0x2f as ::core::ffi::c_int as uint8_t,
    0x19 as ::core::ffi::c_int as uint8_t,
    0xeb as ::core::ffi::c_int as uint8_t,
    0xf4 as ::core::ffi::c_int as uint8_t,
    0xd0 as ::core::ffi::c_int as uint8_t,
    0xfc as ::core::ffi::c_int as uint8_t,
    0x7e as ::core::ffi::c_int as uint8_t,
    0xf4 as ::core::ffi::c_int as uint8_t,
    0xd0 as ::core::ffi::c_int as uint8_t,
    0xb as ::core::ffi::c_int as uint8_t,
    0xe4 as ::core::ffi::c_int as uint8_t,
    0xf5 as ::core::ffi::c_int as uint8_t,
    0xcb as ::core::ffi::c_int as uint8_t,
    0xf4 as ::core::ffi::c_int as uint8_t,
    0xd7 as ::core::ffi::c_int as uint8_t,
    0 as ::core::ffi::c_int as uint8_t,
    0xfc as ::core::ffi::c_int as uint8_t,
    0xd0 as ::core::ffi::c_int as uint8_t,
    0xf3 as ::core::ffi::c_int as uint8_t,
    0xab as ::core::ffi::c_int as uint8_t,
    0x1 as ::core::ffi::c_int as uint8_t,
    0x10 as ::core::ffi::c_int as uint8_t,
    0xef as ::core::ffi::c_int as uint8_t,
    0x7e as ::core::ffi::c_int as uint8_t,
    0xf4 as ::core::ffi::c_int as uint8_t,
    0x10 as ::core::ffi::c_int as uint8_t,
    0xeb as ::core::ffi::c_int as uint8_t,
    0xba as ::core::ffi::c_int as uint8_t,
    0xf6 as ::core::ffi::c_int as uint8_t,
    0xda as ::core::ffi::c_int as uint8_t,
    0 as ::core::ffi::c_int as uint8_t,
    0xba as ::core::ffi::c_int as uint8_t,
    0xf4 as ::core::ffi::c_int as uint8_t,
    0xc4 as ::core::ffi::c_int as uint8_t,
    0xf4 as ::core::ffi::c_int as uint8_t,
    0xdd as ::core::ffi::c_int as uint8_t,
    0x5d as ::core::ffi::c_int as uint8_t,
    0xd0 as ::core::ffi::c_int as uint8_t,
    0xdb as ::core::ffi::c_int as uint8_t,
    0x1f as ::core::ffi::c_int as uint8_t,
    0 as ::core::ffi::c_int as uint8_t,
    0 as ::core::ffi::c_int as uint8_t,
    0xc0 as ::core::ffi::c_int as uint8_t,
    0xff as ::core::ffi::c_int as uint8_t,
];
pub unsafe fn apu_init() -> *mut Apu {
    let mut apu: *mut Apu = malloc(::core::mem::size_of::<Apu>() as size_t) as *mut Apu;
    core::ptr::write_bytes(apu, 0, 1);
    (*apu).spc = spc_init(apu as *mut Apu);
    (*apu).dsp = dsp_init(&raw mut (*apu).ram as *mut uint8_t);
    return apu;
}
pub unsafe fn apu_free(mut apu: *mut Apu) {
    spc_free((*apu).spc);
    dsp_free((*apu).dsp);
    free(apu as *mut ::core::ffi::c_void);
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
    mut apu: *mut Apu,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*apu).ram as *mut uint8_t as *mut ::core::ffi::c_void,
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
            let mut line: [::core::ffi::c_char; 80] = [0; 80];
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
            let mut i_0: ::core::ffi::c_int =
                (*apu).c2rust_unnamed.hist.count as ::core::ffi::c_int;
            if i_0 != 256 as ::core::ffi::c_int {
                (*apu).c2rust_unnamed.hist.count = (i_0 + 1 as ::core::ffi::c_int) as uint32_t;
                (*apu).c2rust_unnamed.hist.addr[i_0 as usize] = (*apu).dspAdr;
                (*apu).c2rust_unnamed.hist.val[i_0 as usize] = val;
            }
            if ((*apu).dspAdr as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
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
