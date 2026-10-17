extern "C" {
    fn puts(__s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn spc_init(apu: *mut Apu) -> *mut Spc;
    fn spc_free(spc: *mut Spc);
    fn spc_reset(spc: *mut Spc);
    fn spc_runOpcode(spc: *mut Spc) -> ::core::ffi::c_int;
    fn spc_saveload(spc: *mut Spc, func: Option<SaveLoadFunc>, ctx: *mut ::core::ffi::c_void);
    fn dsp_init(ram: *mut uint8_t) -> *mut Dsp;
    fn dsp_free(dsp: *mut Dsp);
    fn dsp_reset(dsp: *mut Dsp);
    fn dsp_cycle(dsp: *mut Dsp);
    fn dsp_read(dsp: *mut Dsp, adr: uint8_t) -> uint8_t;
    fn dsp_write(dsp: *mut Dsp, adr: uint8_t, val: uint8_t);
    fn dsp_saveload(dsp: *mut Dsp, func: Option<SaveLoadFunc>, ctx: *mut ::core::ffi::c_void);
    fn getProcessorStateSpc(snes: *mut Apu, line: *mut ::core::ffi::c_char);
}
pub type size_t = usize;
pub type __int8_t = i8;
pub type __uint8_t = u8;
pub type __int16_t = i16;
pub type __uint16_t = u16;
pub type __uint32_t = u32;
pub type int8_t = __int8_t;
pub type int16_t = __int16_t;
pub type uint8_t = __uint8_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Apu {
    pub spc: *mut Spc,
    pub dsp: *mut Dsp,
    pub ram: [uint8_t; 65536],
    pub romReadable: bool,
    pub dspAdr: uint8_t,
    pub cycles: uint32_t,
    pub inPorts: [uint8_t; 6],
    pub outPorts: [uint8_t; 4],
    pub timer: [Timer; 3],
    pub cpuCyclesLeft: uint8_t,
    pub pad: [uint8_t; 6],
    pub c2rust_unnamed: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub hist: DspRegWriteHistory,
    pub padpad: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspRegWriteHistory {
    pub count: uint32_t,
    pub addr: [uint8_t; 256],
    pub val: [uint8_t; 256],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Timer {
    pub cycles: uint8_t,
    pub divider: uint8_t,
    pub target: uint8_t,
    pub counter: uint8_t,
    pub enabled: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dsp {
    pub apu_ram: *mut uint8_t,
    pub ram: [uint8_t; 128],
    pub channel: [DspChannel; 8],
    pub dirPage: uint16_t,
    pub evenCycle: bool,
    pub mute: bool,
    pub reset: bool,
    pub masterVolumeL: int8_t,
    pub masterVolumeR: int8_t,
    pub noiseSample: int16_t,
    pub noiseRate: uint16_t,
    pub noiseCounter: uint16_t,
    pub echoWrites: bool,
    pub echoVolumeL: int8_t,
    pub echoVolumeR: int8_t,
    pub feedbackVolume: int8_t,
    pub echoBufferAdr: uint16_t,
    pub echoDelay: uint16_t,
    pub echoRemain: uint16_t,
    pub echoBufferIndex: uint16_t,
    pub firBufferIndex: uint8_t,
    pub firValues: [int8_t; 8],
    pub firBufferL: [int16_t; 8],
    pub firBufferR: [int16_t; 8],
    pub sampleBuffer: [int16_t; 1068],
    pub sampleOffset: uint16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DspChannel {
    pub pitch: uint16_t,
    pub pitchCounter: uint16_t,
    pub pitchModulation: bool,
    pub decodeBuffer: [int16_t; 19],
    pub srcn: uint8_t,
    pub decodeOffset: uint16_t,
    pub previousFlags: uint8_t,
    pub old: int16_t,
    pub older: int16_t,
    pub useNoise: bool,
    pub adsrRates: [uint16_t; 4],
    pub rateCounter: uint16_t,
    pub adsrState: uint8_t,
    pub sustainLevel: uint16_t,
    pub useGain: bool,
    pub gainMode: uint8_t,
    pub directGain: bool,
    pub gainValue: uint16_t,
    pub gain: uint16_t,
    pub keyOn: bool,
    pub keyOff: bool,
    pub sampleOut: int16_t,
    pub volumeL: int8_t,
    pub volumeR: int8_t,
    pub echoEnable: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Spc {
    pub apu: *mut Apu,
    pub a: uint8_t,
    pub x: uint8_t,
    pub y: uint8_t,
    pub sp: uint8_t,
    pub pc: uint16_t,
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub h: bool,
    pub p: bool,
    pub b: bool,
    pub stopped: bool,
    pub cyclesUsed: uint8_t,
}
pub type SaveLoadFunc =
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, size_t) -> ();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut bootRom: [uint8_t; 64] = [
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
#[no_mangle]
pub unsafe extern "C" fn apu_init() -> *mut Apu {
    let mut apu: *mut Apu = malloc(::core::mem::size_of::<Apu>() as size_t) as *mut Apu;
    (*apu).spc = spc_init(apu as *mut Apu);
    (*apu).dsp = dsp_init(&raw mut (*apu).ram as *mut uint8_t);
    return apu;
}
#[no_mangle]
pub unsafe extern "C" fn apu_free(mut apu: *mut Apu) {
    spc_free((*apu).spc);
    dsp_free((*apu).dsp);
    free(apu as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn apu_reset(mut apu: *mut Apu) {
    (*apu).romReadable = true_0 != 0;
    spc_reset((*apu).spc);
    dsp_reset((*apu).dsp);
    memset(
        &raw mut (*apu).ram as *mut uint8_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint8_t; 65536]>() as size_t,
    );
    (*apu).dspAdr = 0 as uint8_t;
    (*apu).cycles = 0 as uint32_t;
    memset(
        &raw mut (*apu).inPorts as *mut uint8_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint8_t; 6]>() as size_t,
    );
    memset(
        &raw mut (*apu).outPorts as *mut uint8_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint8_t; 4]>() as size_t,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 3 as ::core::ffi::c_int {
        (*apu).timer[i as usize].cycles = 0 as uint8_t;
        (*apu).timer[i as usize].divider = 0 as uint8_t;
        (*apu).timer[i as usize].target = 0 as uint8_t;
        (*apu).timer[i as usize].counter = 0 as uint8_t;
        (*apu).timer[i as usize].enabled = false_0 != 0;
        i += 1;
    }
    (*apu).cpuCyclesLeft = 7 as uint8_t;
    (*apu).c2rust_unnamed.hist.count = 0 as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn apu_saveload(
    mut apu: *mut Apu,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*apu).ram as *mut uint8_t as *mut ::core::ffi::c_void,
        (65586 as size_t)
            .wrapping_add(6 as size_t)
            .wrapping_sub(16 as size_t),
    );
    dsp_saveload((*apu).dsp, func, ctx);
    spc_saveload((*apu).spc, func, ctx);
}
#[no_mangle]
pub static mut g_debug_apu_cycles: bool = false;
#[no_mangle]
pub unsafe extern "C" fn apu_cycle(mut apu: *mut Apu) {
    if (*apu).cpuCyclesLeft as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if g_debug_apu_cycles {
            let mut line: [::core::ffi::c_char; 80] = [0; 80];
            getProcessorStateSpc(apu, &raw mut line as *mut ::core::ffi::c_char);
            puts(&raw mut line as *mut ::core::ffi::c_char);
        }
        (*apu).cpuCyclesLeft = spc_runOpcode((*apu).spc) as uint8_t;
    }
    (*apu).cpuCyclesLeft = (*apu).cpuCyclesLeft.wrapping_sub(1);
    if (*apu).cycles & 0x1f as uint32_t == 0 as uint32_t {
        dsp_cycle((*apu).dsp);
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 3 as ::core::ffi::c_int {
        if (*apu).timer[i as usize].cycles as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*apu).timer[i as usize].cycles = (if i == 2 as ::core::ffi::c_int {
                16 as ::core::ffi::c_int
            } else {
                128 as ::core::ffi::c_int
            }) as uint8_t;
            if (*apu).timer[i as usize].enabled {
                (*apu).timer[i as usize].divider = (*apu).timer[i as usize].divider.wrapping_add(1);
                if (*apu).timer[i as usize].divider as ::core::ffi::c_int
                    == (*apu).timer[i as usize].target as ::core::ffi::c_int
                {
                    (*apu).timer[i as usize].divider = 0 as uint8_t;
                    (*apu).timer[i as usize].counter =
                        (*apu).timer[i as usize].counter.wrapping_add(1);
                    (*apu).timer[i as usize].counter =
                        ((*apu).timer[i as usize].counter as ::core::ffi::c_int
                            & 0xf as ::core::ffi::c_int) as uint8_t;
                }
            }
        }
        (*apu).timer[i as usize].cycles = (*apu).timer[i as usize].cycles.wrapping_sub(1);
        i += 1;
    }
    (*apu).cycles = (*apu).cycles.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn apu_cpuRead(mut apu: *mut Apu, mut adr: uint16_t) -> uint8_t {
    match adr as ::core::ffi::c_int {
        240 | 241 | 250 | 251 | 252 => return 0 as uint8_t,
        242 => return (*apu).dspAdr,
        243 => {
            return dsp_read(
                (*apu).dsp,
                ((*apu).dspAdr as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as uint8_t,
            );
        }
        244 | 245 | 246 | 247 | 248 | 249 => {
            return (*apu).inPorts
                [(adr as ::core::ffi::c_int - 0xf4 as ::core::ffi::c_int) as usize];
        }
        253 | 254 | 255 => {
            let mut ret: uint8_t = (*apu).timer
                [(adr as ::core::ffi::c_int - 0xfd as ::core::ffi::c_int) as usize]
                .counter;
            (*apu).timer[(adr as ::core::ffi::c_int - 0xfd as ::core::ffi::c_int) as usize]
                .counter = 0 as uint8_t;
            return ret;
        }
        _ => {}
    }
    if (*apu).romReadable as ::core::ffi::c_int != 0
        && adr as ::core::ffi::c_int >= 0xffc0 as ::core::ffi::c_int
    {
        return bootRom[(adr as ::core::ffi::c_int - 0xffc0 as ::core::ffi::c_int) as usize];
    }
    return (*apu).ram[adr as usize];
}
#[no_mangle]
pub unsafe extern "C" fn apu_cpuWrite(mut apu: *mut Apu, mut adr: uint16_t, mut val: uint8_t) {
    match adr as ::core::ffi::c_int {
        241 => {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 3 as ::core::ffi::c_int {
                if !(*apu).timer[i as usize].enabled
                    && val as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << i != 0
                {
                    (*apu).timer[i as usize].divider = 0 as uint8_t;
                    (*apu).timer[i as usize].counter = 0 as uint8_t;
                }
                (*apu).timer[i as usize].enabled =
                    val as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << i != 0;
                i += 1;
            }
            if val as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0 {
                (*apu).inPorts[0 as ::core::ffi::c_int as usize] = 0 as uint8_t;
                (*apu).inPorts[1 as ::core::ffi::c_int as usize] = 0 as uint8_t;
            }
            if val as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0 {
                (*apu).inPorts[2 as ::core::ffi::c_int as usize] = 0 as uint8_t;
                (*apu).inPorts[3 as ::core::ffi::c_int as usize] = 0 as uint8_t;
            }
            (*apu).romReadable = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
        }
        242 => {
            (*apu).dspAdr = val;
        }
        243 => {
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
        244 | 245 | 246 | 247 => {
            (*apu).outPorts[(adr as ::core::ffi::c_int - 0xf4 as ::core::ffi::c_int) as usize] =
                val;
        }
        248 | 249 => {
            (*apu).inPorts[(adr as ::core::ffi::c_int - 0xf4 as ::core::ffi::c_int) as usize] = val;
        }
        250 | 251 | 252 => {
            (*apu).timer[(adr as ::core::ffi::c_int - 0xfa as ::core::ffi::c_int) as usize]
                .target = val;
        }
        240 | _ => {}
    }
    (*apu).ram[adr as usize] = val;
}
