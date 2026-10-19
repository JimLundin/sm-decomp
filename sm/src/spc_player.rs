use crate::snes::dsp::{dsp_cycle, dsp_init, dsp_reset, dsp_write};
extern "C" {
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

}
pub use crate::types::{
    __int16_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16_t, int8_t, size_t, uint16,
    uint16_t, uint32_t, uint8, uint8_t, Channel, Dsp, DspChannel, DspRegWriteHistory, SfxChan,
    SfxRegs, SpcPlayer,
};
pub const EVOLL: DspReg = 44;
pub const EVOLR: DspReg = 60;
pub const EFB: DspReg = 13;
pub const EON: DspReg = 77;
pub const FLG: DspReg = 108;
pub const KON: DspReg = 76;
pub const KOF: DspReg = 92;
pub const NON: DspReg = 61;
pub const PMON: DspReg = 45;
pub const V0PITCHH: DspReg = 3;
pub const V0PITCHL: DspReg = 2;
pub const V0VOLL: DspReg = 0;
pub const V0GAIN: DspReg = 7;
pub const V0ADSR2: DspReg = 6;
pub const V0ADSR1: DspReg = 5;
pub const V0SRCN: DspReg = 4;
pub const FIR0: DspReg = 15;
pub const ESA: DspReg = 109;
pub const EDL: DspReg = 125;
pub const DIR: DspReg = 93;
pub const MVOLR: DspReg = 28;
pub const MVOLL: DspReg = 12;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemMapSized {
    pub off: uint16,
    pub org_off: uint16,
    pub size: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemMap {
    pub off: uint16,
    pub org_off: uint16,
}
pub use crate::types::DspReg;
pub const FIR7: DspReg = 127;
pub const ENDX: DspReg = 124;
pub const V7OUTX: DspReg = 121;
pub const V7ENVX: DspReg = 120;
pub const V7GAIN: DspReg = 119;
pub const V7ADSR2: DspReg = 118;
pub const V7ADSR1: DspReg = 117;
pub const V7SRCN: DspReg = 116;
pub const V7PH: DspReg = 115;
pub const V7PL: DspReg = 114;
pub const V7VOLR: DspReg = 113;
pub const V7VOLL: DspReg = 112;
pub const FIR6: DspReg = 111;
pub const V6OUTX: DspReg = 105;
pub const V6ENVX: DspReg = 104;
pub const V6GAIN: DspReg = 103;
pub const V6ADSR2: DspReg = 102;
pub const V6ADSR1: DspReg = 101;
pub const V6SRCN: DspReg = 100;
pub const V6PH: DspReg = 99;
pub const V6PL: DspReg = 98;
pub const V6VOLR: DspReg = 97;
pub const V6VOLL: DspReg = 96;
pub const FIR5: DspReg = 95;
pub const V5OUTX: DspReg = 89;
pub const V5ENVX: DspReg = 88;
pub const V5GAIN: DspReg = 87;
pub const V5ADSR2: DspReg = 86;
pub const V5ADSR1: DspReg = 85;
pub const V5SRCN: DspReg = 84;
pub const V5PH: DspReg = 83;
pub const V5PL: DspReg = 82;
pub const V5VOLR: DspReg = 81;
pub const V5VOLL: DspReg = 80;
pub const FIR4: DspReg = 79;
pub const V4OUTX: DspReg = 73;
pub const V4ENVX: DspReg = 72;
pub const V4GAIN: DspReg = 71;
pub const V4ADSR2: DspReg = 70;
pub const V4ADSR1: DspReg = 69;
pub const V4SRCN: DspReg = 68;
pub const V4PH: DspReg = 67;
pub const V4PL: DspReg = 66;
pub const V4VOLR: DspReg = 65;
pub const V4VOLL: DspReg = 64;
pub const FIR3: DspReg = 63;
pub const V3OUTX: DspReg = 57;
pub const V3ENVX: DspReg = 56;
pub const V3GAIN: DspReg = 55;
pub const V3ADSR2: DspReg = 54;
pub const V3ADSR1: DspReg = 53;
pub const V3SRCN: DspReg = 52;
pub const V3PH: DspReg = 51;
pub const V3PL: DspReg = 50;
pub const V3VOLR: DspReg = 49;
pub const V3VOLL: DspReg = 48;
pub const FIR2: DspReg = 47;
pub const V2OUTX: DspReg = 41;
pub const V2ENVX: DspReg = 40;
pub const V2GAIN: DspReg = 39;
pub const V2ADSR2: DspReg = 38;
pub const V2ADSR1: DspReg = 37;
pub const V2SRCN: DspReg = 36;
pub const V2PH: DspReg = 35;
pub const V2PL: DspReg = 34;
pub const V2VOLR: DspReg = 33;
pub const V2VOLL: DspReg = 32;
pub const FIR1: DspReg = 31;
pub const V1OUTX: DspReg = 25;
pub const V1ENVX: DspReg = 24;
pub const V1GAIN: DspReg = 23;
pub const V1ADSR2: DspReg = 22;
pub const V1ADSR1: DspReg = 21;
pub const V1SRCN: DspReg = 20;
pub const V1PH: DspReg = 19;
pub const V1PL: DspReg = 18;
pub const V1VOLR: DspReg = 17;
pub const V1VOLL: DspReg = 16;
pub const V0OUTX: DspReg = 9;
pub const V0ENVX: DspReg = 8;
pub const V0VOLR: DspReg = 1;
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
static kChannel_Maps: [MemMap; 47] = [
    MemMap {
        off: 0 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8030 as uint16,
    },
    MemMap {
        off: 3 as ::core::ffi::c_ulong as uint16,
        org_off: 0x70 as uint16,
    },
    MemMap {
        off: 4 as ::core::ffi::c_ulong as uint16,
        org_off: 0x71 as uint16,
    },
    MemMap {
        off: 5 as ::core::ffi::c_ulong as uint16,
        org_off: 0x80 as uint16,
    },
    MemMap {
        off: 6 as ::core::ffi::c_ulong as uint16,
        org_off: 0x90 as uint16,
    },
    MemMap {
        off: 7 as ::core::ffi::c_ulong as uint16,
        org_off: 0x91 as uint16,
    },
    MemMap {
        off: 8 as ::core::ffi::c_ulong as uint16,
        org_off: 0xa0 as uint16,
    },
    MemMap {
        off: 9 as ::core::ffi::c_ulong as uint16,
        org_off: 0xa1 as uint16,
    },
    MemMap {
        off: 10 as ::core::ffi::c_ulong as uint16,
        org_off: 0xb0 as uint16,
    },
    MemMap {
        off: 11 as ::core::ffi::c_ulong as uint16,
        org_off: 0xb1 as uint16,
    },
    MemMap {
        off: 12 as ::core::ffi::c_ulong as uint16,
        org_off: 0xc0 as uint16,
    },
    MemMap {
        off: 13 as ::core::ffi::c_ulong as uint16,
        org_off: 0xc1 as uint16,
    },
    MemMap {
        off: 14 as ::core::ffi::c_ulong as uint16,
        org_off: 0x100 as uint16,
    },
    MemMap {
        off: 15 as ::core::ffi::c_ulong as uint16,
        org_off: 0x200 as uint16,
    },
    MemMap {
        off: 16 as ::core::ffi::c_ulong as uint16,
        org_off: 0x201 as uint16,
    },
    MemMap {
        off: 17 as ::core::ffi::c_ulong as uint16,
        org_off: 0x210 as uint16,
    },
    MemMap {
        off: 18 as ::core::ffi::c_ulong as uint16,
        org_off: 0x211 as uint16,
    },
    MemMap {
        off: 20 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8220 as uint16,
    },
    MemMap {
        off: 22 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8230 as uint16,
    },
    MemMap {
        off: 24 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8240 as uint16,
    },
    MemMap {
        off: 26 as ::core::ffi::c_ulong as uint16,
        org_off: 0x280 as uint16,
    },
    MemMap {
        off: 27 as ::core::ffi::c_ulong as uint16,
        org_off: 0x281 as uint16,
    },
    MemMap {
        off: 28 as ::core::ffi::c_ulong as uint16,
        org_off: 0x290 as uint16,
    },
    MemMap {
        off: 29 as ::core::ffi::c_ulong as uint16,
        org_off: 0x291 as uint16,
    },
    MemMap {
        off: 30 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2a0 as uint16,
    },
    MemMap {
        off: 31 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2a1 as uint16,
    },
    MemMap {
        off: 32 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2b0 as uint16,
    },
    MemMap {
        off: 33 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2b1 as uint16,
    },
    MemMap {
        off: 34 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2c0 as uint16,
    },
    MemMap {
        off: 35 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2c1 as uint16,
    },
    MemMap {
        off: 36 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2d0 as uint16,
    },
    MemMap {
        off: 37 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2d1 as uint16,
    },
    MemMap {
        off: 38 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2e0 as uint16,
    },
    MemMap {
        off: 39 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2f0 as uint16,
    },
    MemMap {
        off: 40 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8300 as uint16,
    },
    MemMap {
        off: 42 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8310 as uint16,
    },
    MemMap {
        off: 44 as ::core::ffi::c_ulong as uint16,
        org_off: 0x320 as uint16,
    },
    MemMap {
        off: 45 as ::core::ffi::c_ulong as uint16,
        org_off: 0x321 as uint16,
    },
    MemMap {
        off: 46 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8330 as uint16,
    },
    MemMap {
        off: 48 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8340 as uint16,
    },
    MemMap {
        off: 50 as ::core::ffi::c_ulong as uint16,
        org_off: 0x350 as uint16,
    },
    MemMap {
        off: 51 as ::core::ffi::c_ulong as uint16,
        org_off: 0x351 as uint16,
    },
    MemMap {
        off: 52 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8360 as uint16,
    },
    MemMap {
        off: 54 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8370 as uint16,
    },
    MemMap {
        off: 56 as ::core::ffi::c_ulong as uint16,
        org_off: 0x380 as uint16,
    },
    MemMap {
        off: 57 as ::core::ffi::c_ulong as uint16,
        org_off: 0x381 as uint16,
    },
    MemMap {
        off: 58 as ::core::ffi::c_ulong as uint16,
        org_off: 0x400 as uint16,
    },
];
static kSpcPlayer_Maps: [MemMapSized; 283] = [
    MemMapSized {
        off: 855 as ::core::ffi::c_ulong as uint16,
        org_off: 0 as uint16,
        size: 4 as uint16,
    },
    MemMapSized {
        off: 24 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4 as uint16,
        size: 4 as uint16,
    },
    MemMapSized {
        off: 28 as ::core::ffi::c_ulong as uint16,
        org_off: 0x8 as uint16,
        size: 4 as uint16,
    },
    MemMapSized {
        off: 32 as ::core::ffi::c_ulong as uint16,
        org_off: 0xc as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 34 as ::core::ffi::c_ulong as uint16,
        org_off: 0xe as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 36 as ::core::ffi::c_ulong as uint16,
        org_off: 0x10 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 38 as ::core::ffi::c_ulong as uint16,
        org_off: 0x12 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 39 as ::core::ffi::c_ulong as uint16,
        org_off: 0x13 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 40 as ::core::ffi::c_ulong as uint16,
        org_off: 0x14 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 42 as ::core::ffi::c_ulong as uint16,
        org_off: 0x16 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 44 as ::core::ffi::c_ulong as uint16,
        org_off: 0x18 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 46 as ::core::ffi::c_ulong as uint16,
        org_off: 0x1a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 47 as ::core::ffi::c_ulong as uint16,
        org_off: 0x1b as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 8 as ::core::ffi::c_ulong as uint16,
        org_off: 0x1c as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 48 as ::core::ffi::c_ulong as uint16,
        org_off: 0x20 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 592 as ::core::ffi::c_ulong as uint16,
        org_off: 0x22 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 582 as ::core::ffi::c_ulong as uint16,
        org_off: 0x24 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 586 as ::core::ffi::c_ulong as uint16,
        org_off: 0x26 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 588 as ::core::ffi::c_ulong as uint16,
        org_off: 0x28 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 658 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2a as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 684 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2c as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 710 as ::core::ffi::c_ulong as uint16,
        org_off: 0x2e as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 50 as ::core::ffi::c_ulong as uint16,
        org_off: 0x40 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 52 as ::core::ffi::c_ulong as uint16,
        org_off: 0x42 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 53 as ::core::ffi::c_ulong as uint16,
        org_off: 0x43 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 54 as ::core::ffi::c_ulong as uint16,
        org_off: 0x44 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 55 as ::core::ffi::c_ulong as uint16,
        org_off: 0x45 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 56 as ::core::ffi::c_ulong as uint16,
        org_off: 0x46 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 57 as ::core::ffi::c_ulong as uint16,
        org_off: 0x47 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 58 as ::core::ffi::c_ulong as uint16,
        org_off: 0x48 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 59 as ::core::ffi::c_ulong as uint16,
        org_off: 0x49 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 60 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 61 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4c as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 62 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4d as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 63 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4e as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 64 as ::core::ffi::c_ulong as uint16,
        org_off: 0x50 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 65 as ::core::ffi::c_ulong as uint16,
        org_off: 0x51 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 66 as ::core::ffi::c_ulong as uint16,
        org_off: 0x52 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 68 as ::core::ffi::c_ulong as uint16,
        org_off: 0x54 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 69 as ::core::ffi::c_ulong as uint16,
        org_off: 0x55 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 70 as ::core::ffi::c_ulong as uint16,
        org_off: 0x56 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 72 as ::core::ffi::c_ulong as uint16,
        org_off: 0x58 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 74 as ::core::ffi::c_ulong as uint16,
        org_off: 0x5a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 75 as ::core::ffi::c_ulong as uint16,
        org_off: 0x5b as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 76 as ::core::ffi::c_ulong as uint16,
        org_off: 0x5c as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 78 as ::core::ffi::c_ulong as uint16,
        org_off: 0x5e as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 79 as ::core::ffi::c_ulong as uint16,
        org_off: 0x5f as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 80 as ::core::ffi::c_ulong as uint16,
        org_off: 0x60 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 82 as ::core::ffi::c_ulong as uint16,
        org_off: 0x62 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 84 as ::core::ffi::c_ulong as uint16,
        org_off: 0x64 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 86 as ::core::ffi::c_ulong as uint16,
        org_off: 0x66 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 88 as ::core::ffi::c_ulong as uint16,
        org_off: 0x68 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 89 as ::core::ffi::c_ulong as uint16,
        org_off: 0x69 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 90 as ::core::ffi::c_ulong as uint16,
        org_off: 0x6a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 736 as ::core::ffi::c_ulong as uint16,
        org_off: 0xd0 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 92 as ::core::ffi::c_ulong as uint16,
        org_off: 0xd2 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 614 as ::core::ffi::c_ulong as uint16,
        org_off: 0xd4 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 604 as ::core::ffi::c_ulong as uint16,
        org_off: 0xd6 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 608 as ::core::ffi::c_ulong as uint16,
        org_off: 0xd8 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 610 as ::core::ffi::c_ulong as uint16,
        org_off: 0xda as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 762 as ::core::ffi::c_ulong as uint16,
        org_off: 0xdc as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 788 as ::core::ffi::c_ulong as uint16,
        org_off: 0xde as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 636 as ::core::ffi::c_ulong as uint16,
        org_off: 0xe0 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 626 as ::core::ffi::c_ulong as uint16,
        org_off: 0xe2 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 630 as ::core::ffi::c_ulong as uint16,
        org_off: 0xe4 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 632 as ::core::ffi::c_ulong as uint16,
        org_off: 0xe6 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 814 as ::core::ffi::c_ulong as uint16,
        org_off: 0xe8 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 840 as ::core::ffi::c_ulong as uint16,
        org_off: 0xea as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 94 as ::core::ffi::c_ulong as uint16,
        org_off: 0xee as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 96 as ::core::ffi::c_ulong as uint16,
        org_off: 0x390 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 584 as ::core::ffi::c_ulong as uint16,
        org_off: 0x392 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 598 as ::core::ffi::c_ulong as uint16,
        org_off: 0x393 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 651 as ::core::ffi::c_ulong as uint16,
        org_off: 0x394 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 677 as ::core::ffi::c_ulong as uint16,
        org_off: 0x395 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 703 as ::core::ffi::c_ulong as uint16,
        org_off: 0x396 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 729 as ::core::ffi::c_ulong as uint16,
        org_off: 0x397 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 647 as ::core::ffi::c_ulong as uint16,
        org_off: 0x398 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 673 as ::core::ffi::c_ulong as uint16,
        org_off: 0x399 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 699 as ::core::ffi::c_ulong as uint16,
        org_off: 0x39a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 725 as ::core::ffi::c_ulong as uint16,
        org_off: 0x39b as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 668 as ::core::ffi::c_ulong as uint16,
        org_off: 0x39c as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 694 as ::core::ffi::c_ulong as uint16,
        org_off: 0x39d as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 720 as ::core::ffi::c_ulong as uint16,
        org_off: 0x39e as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 746 as ::core::ffi::c_ulong as uint16,
        org_off: 0x39f as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 596 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a0 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 595 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a1 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 601 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a2 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 597 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a3 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 600 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a4 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 599 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a5 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 646 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a6 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 672 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a7 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 698 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a8 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 724 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3a9 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 666 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3aa as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 692 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ab as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 718 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ac as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 744 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ad as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 581 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ae as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 652 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3af as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 678 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b0 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 704 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b1 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 730 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b2 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 594 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b3 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 653 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b4 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 679 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b5 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 705 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b6 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 731 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b7 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 655 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b8 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 657 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3b9 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 681 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ba as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 683 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3bb as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 707 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3bc as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 709 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3bd as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 733 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3be as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 735 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3bf as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 662 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c0 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 654 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c1 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 688 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c2 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 680 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c3 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 714 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c4 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 706 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c5 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 740 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c6 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 732 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c7 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 660 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c8 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 686 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3c9 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 712 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ca as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 738 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3cb as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 649 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3cc as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 675 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3cd as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 701 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ce as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 727 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3cf as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 664 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3d0 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 690 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3d2 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 716 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3d4 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 742 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3d6 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 670 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3d8 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 696 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3d9 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 722 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3da as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 748 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3db as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 650 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3dc as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 648 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3dd as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 669 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3de as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 656 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3df as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 661 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e0 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 667 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e1 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 663 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e2 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 676 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e3 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 674 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e4 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 695 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e5 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 682 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e6 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 687 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e7 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 693 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e8 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 689 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3e9 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 702 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ea as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 700 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3eb as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 721 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ec as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 708 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ed as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 713 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ee as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 719 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ef as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 715 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f0 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 728 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f1 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 726 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f2 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 747 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f3 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 734 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f4 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 739 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f5 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 745 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f6 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 741 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f7 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 606 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f8 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 620 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3f9 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 755 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3fa as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 781 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3fb as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 751 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3fc as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 777 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3fd as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 772 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3fe as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 798 as ::core::ffi::c_ulong as uint16,
        org_off: 0x3ff as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 618 as ::core::ffi::c_ulong as uint16,
        org_off: 0x440 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 617 as ::core::ffi::c_ulong as uint16,
        org_off: 0x441 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 623 as ::core::ffi::c_ulong as uint16,
        org_off: 0x442 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 619 as ::core::ffi::c_ulong as uint16,
        org_off: 0x443 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 622 as ::core::ffi::c_ulong as uint16,
        org_off: 0x444 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 621 as ::core::ffi::c_ulong as uint16,
        org_off: 0x445 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 750 as ::core::ffi::c_ulong as uint16,
        org_off: 0x446 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 776 as ::core::ffi::c_ulong as uint16,
        org_off: 0x447 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 770 as ::core::ffi::c_ulong as uint16,
        org_off: 0x448 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 796 as ::core::ffi::c_ulong as uint16,
        org_off: 0x449 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 603 as ::core::ffi::c_ulong as uint16,
        org_off: 0x44a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 756 as ::core::ffi::c_ulong as uint16,
        org_off: 0x44b as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 782 as ::core::ffi::c_ulong as uint16,
        org_off: 0x44c as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 616 as ::core::ffi::c_ulong as uint16,
        org_off: 0x44d as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 757 as ::core::ffi::c_ulong as uint16,
        org_off: 0x44e as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 783 as ::core::ffi::c_ulong as uint16,
        org_off: 0x44f as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 759 as ::core::ffi::c_ulong as uint16,
        org_off: 0x450 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 761 as ::core::ffi::c_ulong as uint16,
        org_off: 0x451 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 785 as ::core::ffi::c_ulong as uint16,
        org_off: 0x452 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 787 as ::core::ffi::c_ulong as uint16,
        org_off: 0x453 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 766 as ::core::ffi::c_ulong as uint16,
        org_off: 0x454 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 758 as ::core::ffi::c_ulong as uint16,
        org_off: 0x455 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 792 as ::core::ffi::c_ulong as uint16,
        org_off: 0x456 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 784 as ::core::ffi::c_ulong as uint16,
        org_off: 0x457 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 764 as ::core::ffi::c_ulong as uint16,
        org_off: 0x458 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 790 as ::core::ffi::c_ulong as uint16,
        org_off: 0x459 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 753 as ::core::ffi::c_ulong as uint16,
        org_off: 0x45a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 779 as ::core::ffi::c_ulong as uint16,
        org_off: 0x45b as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 768 as ::core::ffi::c_ulong as uint16,
        org_off: 0x45c as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 794 as ::core::ffi::c_ulong as uint16,
        org_off: 0x45e as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 774 as ::core::ffi::c_ulong as uint16,
        org_off: 0x460 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 800 as ::core::ffi::c_ulong as uint16,
        org_off: 0x461 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 754 as ::core::ffi::c_ulong as uint16,
        org_off: 0x462 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 752 as ::core::ffi::c_ulong as uint16,
        org_off: 0x463 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 773 as ::core::ffi::c_ulong as uint16,
        org_off: 0x464 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 760 as ::core::ffi::c_ulong as uint16,
        org_off: 0x465 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 765 as ::core::ffi::c_ulong as uint16,
        org_off: 0x466 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 771 as ::core::ffi::c_ulong as uint16,
        org_off: 0x467 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 767 as ::core::ffi::c_ulong as uint16,
        org_off: 0x468 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 780 as ::core::ffi::c_ulong as uint16,
        org_off: 0x469 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 778 as ::core::ffi::c_ulong as uint16,
        org_off: 0x46a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 799 as ::core::ffi::c_ulong as uint16,
        org_off: 0x46b as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 786 as ::core::ffi::c_ulong as uint16,
        org_off: 0x46c as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 791 as ::core::ffi::c_ulong as uint16,
        org_off: 0x46d as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 797 as ::core::ffi::c_ulong as uint16,
        org_off: 0x46e as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 793 as ::core::ffi::c_ulong as uint16,
        org_off: 0x46f as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 628 as ::core::ffi::c_ulong as uint16,
        org_off: 0x470 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 642 as ::core::ffi::c_ulong as uint16,
        org_off: 0x471 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 807 as ::core::ffi::c_ulong as uint16,
        org_off: 0x472 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 833 as ::core::ffi::c_ulong as uint16,
        org_off: 0x473 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 803 as ::core::ffi::c_ulong as uint16,
        org_off: 0x474 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 829 as ::core::ffi::c_ulong as uint16,
        org_off: 0x475 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 824 as ::core::ffi::c_ulong as uint16,
        org_off: 0x476 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 850 as ::core::ffi::c_ulong as uint16,
        org_off: 0x477 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 640 as ::core::ffi::c_ulong as uint16,
        org_off: 0x478 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 639 as ::core::ffi::c_ulong as uint16,
        org_off: 0x479 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 645 as ::core::ffi::c_ulong as uint16,
        org_off: 0x47a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 641 as ::core::ffi::c_ulong as uint16,
        org_off: 0x47b as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 644 as ::core::ffi::c_ulong as uint16,
        org_off: 0x47c as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 643 as ::core::ffi::c_ulong as uint16,
        org_off: 0x47d as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 802 as ::core::ffi::c_ulong as uint16,
        org_off: 0x47e as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 828 as ::core::ffi::c_ulong as uint16,
        org_off: 0x47f as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 822 as ::core::ffi::c_ulong as uint16,
        org_off: 0x480 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 848 as ::core::ffi::c_ulong as uint16,
        org_off: 0x481 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 625 as ::core::ffi::c_ulong as uint16,
        org_off: 0x482 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 808 as ::core::ffi::c_ulong as uint16,
        org_off: 0x483 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 834 as ::core::ffi::c_ulong as uint16,
        org_off: 0x484 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 638 as ::core::ffi::c_ulong as uint16,
        org_off: 0x485 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 809 as ::core::ffi::c_ulong as uint16,
        org_off: 0x486 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 835 as ::core::ffi::c_ulong as uint16,
        org_off: 0x487 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 811 as ::core::ffi::c_ulong as uint16,
        org_off: 0x488 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 813 as ::core::ffi::c_ulong as uint16,
        org_off: 0x489 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 837 as ::core::ffi::c_ulong as uint16,
        org_off: 0x48a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 839 as ::core::ffi::c_ulong as uint16,
        org_off: 0x48b as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 818 as ::core::ffi::c_ulong as uint16,
        org_off: 0x48c as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 810 as ::core::ffi::c_ulong as uint16,
        org_off: 0x48d as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 844 as ::core::ffi::c_ulong as uint16,
        org_off: 0x48e as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 836 as ::core::ffi::c_ulong as uint16,
        org_off: 0x48f as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 816 as ::core::ffi::c_ulong as uint16,
        org_off: 0x490 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 842 as ::core::ffi::c_ulong as uint16,
        org_off: 0x491 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 805 as ::core::ffi::c_ulong as uint16,
        org_off: 0x492 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 831 as ::core::ffi::c_ulong as uint16,
        org_off: 0x493 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 820 as ::core::ffi::c_ulong as uint16,
        org_off: 0x494 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 846 as ::core::ffi::c_ulong as uint16,
        org_off: 0x496 as uint16,
        size: 2 as uint16,
    },
    MemMapSized {
        off: 826 as ::core::ffi::c_ulong as uint16,
        org_off: 0x498 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 852 as ::core::ffi::c_ulong as uint16,
        org_off: 0x499 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 806 as ::core::ffi::c_ulong as uint16,
        org_off: 0x49a as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 804 as ::core::ffi::c_ulong as uint16,
        org_off: 0x49b as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 825 as ::core::ffi::c_ulong as uint16,
        org_off: 0x49c as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 812 as ::core::ffi::c_ulong as uint16,
        org_off: 0x49d as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 817 as ::core::ffi::c_ulong as uint16,
        org_off: 0x49e as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 823 as ::core::ffi::c_ulong as uint16,
        org_off: 0x49f as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 819 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a0 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 832 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a1 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 830 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a2 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 851 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a3 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 838 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a4 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 843 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a5 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 849 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a6 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 845 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a7 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 97 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4a9 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 98 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4b1 as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 634 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4ba as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 580 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4bb as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 602 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4bc as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 624 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4bd as uint16,
        size: 1 as uint16,
    },
    MemMapSized {
        off: 854 as ::core::ffi::c_ulong as uint16,
        org_off: 0x4be as uint16,
        size: 1 as uint16,
    },
];
unsafe fn Dsp_Write(mut p: *mut SpcPlayer, mut reg: uint8_t, mut value: uint8) {
    let mut hist: *mut DspRegWriteHistory = (*p).reg_write_history;
    if !hist.is_null() {
        if (*hist).count < 256 as uint32_t {
            if (*hist).count == 9 as uint32_t
                && reg as ::core::ffi::c_int == 0x5d as ::core::ffi::c_int
            {
                reg = (reg as ::core::ffi::c_int + 0 as ::core::ffi::c_int) as uint8_t;
            }
            (*hist).addr[(*hist).count as usize] = reg;
            (*hist).val[(*hist).count as usize] = value as uint8_t;
            (*hist).count = (*hist).count.wrapping_add(1);
        }
    }
    if !(*p).dsp.is_null() {
        dsp_write((*p).dsp, reg, value as uint8_t);
    }
}
unsafe fn Not_Implemented() {
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/spc_player.c\0" as *const u8 as *const ::core::ffi::c_char,
            383 as ::core::ffi::c_uint,
            b"void Not_Implemented(void)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    printf(b"Not Implemented\n\0" as *const u8 as *const ::core::ffi::c_char);
}
unsafe fn SpcDivHelper(mut a: ::core::ffi::c_int, mut b: uint8) -> uint16 {
    let mut org_a: ::core::ffi::c_int = a;
    if a & 0x100 as ::core::ffi::c_int != 0 {
        a = -a;
    }
    let mut q: ::core::ffi::c_int = if b as ::core::ffi::c_int != 0 {
        (a & 0xff as ::core::ffi::c_int) / b as ::core::ffi::c_int
    } else {
        0xff as ::core::ffi::c_int
    };
    let mut r: ::core::ffi::c_int = if b as ::core::ffi::c_int != 0 {
        (a & 0xff as ::core::ffi::c_int) % b as ::core::ffi::c_int
    } else {
        a & 0xff as ::core::ffi::c_int
    };
    let mut t: ::core::ffi::c_int = (q << 8 as ::core::ffi::c_int)
        + (if b as ::core::ffi::c_int != 0 {
            (r << 8 as ::core::ffi::c_int) / b as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
        } else {
            0xff as ::core::ffi::c_int
        });
    return (if org_a & 0x100 as ::core::ffi::c_int != 0 {
        -t
    } else {
        t
    }) as uint16;
}
unsafe fn SetupEchoParameter_EDL(mut p: *mut SpcPlayer, mut a: uint8) {
    (*p).echo_parameter_EDL = a;
    if a as ::core::ffi::c_int != (*p).last_written_edl as ::core::ffi::c_int {
        a = ((*p).last_written_edl as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
            ^ 0xff as ::core::ffi::c_int) as uint8;
        if (*p).echo_stored_time as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
            a = (a as ::core::ffi::c_int + (*p).echo_stored_time as ::core::ffi::c_int) as uint8;
        }
        (*p).echo_stored_time = a;
        Dsp_Write(p, EON as ::core::ffi::c_int as uint8_t, 0 as uint8);
        Dsp_Write(p, EFB as ::core::ffi::c_int as uint8_t, 0 as uint8);
        Dsp_Write(p, EVOLR as ::core::ffi::c_int as uint8_t, 0 as uint8);
        Dsp_Write(p, EVOLL as ::core::ffi::c_int as uint8_t, 0 as uint8);
        Dsp_Write(
            p,
            FLG as ::core::ffi::c_int as uint8_t,
            ((*p).reg_FLG as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint8,
        );
        (*p).last_written_edl = (*p).echo_parameter_EDL;
        Dsp_Write(
            p,
            EDL as ::core::ffi::c_int as uint8_t,
            (*p).echo_parameter_EDL,
        );
    }
    Dsp_Write(
        p,
        ESA as ::core::ffi::c_int as uint8_t,
        (((*p).echo_parameter_EDL as ::core::ffi::c_int * 8 as ::core::ffi::c_int
            ^ 0xff as ::core::ffi::c_int)
            + 0x16 as ::core::ffi::c_int) as uint8,
    );
}
unsafe fn WriteVolumeToDsp(mut p: *mut SpcPlayer, mut c: *mut Channel, mut volume: uint16) {
    static kVolumeTable: [uint8; 22] = [
        0 as ::core::ffi::c_int as uint8,
        1 as ::core::ffi::c_int as uint8,
        3 as ::core::ffi::c_int as uint8,
        7 as ::core::ffi::c_int as uint8,
        13 as ::core::ffi::c_int as uint8,
        21 as ::core::ffi::c_int as uint8,
        30 as ::core::ffi::c_int as uint8,
        41 as ::core::ffi::c_int as uint8,
        52 as ::core::ffi::c_int as uint8,
        66 as ::core::ffi::c_int as uint8,
        81 as ::core::ffi::c_int as uint8,
        94 as ::core::ffi::c_int as uint8,
        103 as ::core::ffi::c_int as uint8,
        110 as ::core::ffi::c_int as uint8,
        115 as ::core::ffi::c_int as uint8,
        119 as ::core::ffi::c_int as uint8,
        122 as ::core::ffi::c_int as uint8,
        124 as ::core::ffi::c_int as uint8,
        125 as ::core::ffi::c_int as uint8,
        126 as ::core::ffi::c_int as uint8,
        127 as ::core::ffi::c_int as uint8,
        127 as ::core::ffi::c_int as uint8,
    ];
    if (*p).is_chan_on as ::core::ffi::c_int & (*p).cur_chan_bit as ::core::ffi::c_int != 0 {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 2 as ::core::ffi::c_int {
        let mut j: ::core::ffi::c_int = volume as ::core::ffi::c_int >> 8 as ::core::ffi::c_int;
        let mut t: uint8 = 0;
        if j >= 21 as ::core::ffi::c_int {
            t = ((*p).ram[(j + 0x1e1d as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                + (((*p).ram[(j + 0x1e1d as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    - (*p).ram[(j + 0x1e1d as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
                    * volume as uint8 as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int)) as uint8;
        } else {
            t = (kVolumeTable[j as usize] as ::core::ffi::c_int
                + ((kVolumeTable[(j + 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                    - kVolumeTable[j as usize] as ::core::ffi::c_int)
                    * volume as uint8 as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int)) as uint8;
        }
        t = (t as ::core::ffi::c_int * (*c).final_volume as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
        if ((*c).pan_flag_with_phase_invert as ::core::ffi::c_int) << i & 0x80 as ::core::ffi::c_int
            != 0
        {
            t = -(t as ::core::ffi::c_int) as uint8;
        }
        Dsp_Write(
            p,
            (V0VOLL as ::core::ffi::c_int
                + i
                + (*c).index as ::core::ffi::c_int * 16 as ::core::ffi::c_int)
                as uint8_t,
            t,
        );
        volume = (0x1400 as ::core::ffi::c_int - volume as ::core::ffi::c_int) as uint16;
        i += 1;
    }
}
unsafe fn WritePitchInner(mut p: *mut SpcPlayer, mut c: *mut Channel, mut pitch: uint16) {
    static kBaseNoteFreqs: [uint16; 13] = [
        2143 as ::core::ffi::c_int as uint16,
        2270 as ::core::ffi::c_int as uint16,
        2405 as ::core::ffi::c_int as uint16,
        2548 as ::core::ffi::c_int as uint16,
        2700 as ::core::ffi::c_int as uint16,
        2860 as ::core::ffi::c_int as uint16,
        3030 as ::core::ffi::c_int as uint16,
        3211 as ::core::ffi::c_int as uint16,
        3402 as ::core::ffi::c_int as uint16,
        3604 as ::core::ffi::c_int as uint16,
        3818 as ::core::ffi::c_int as uint16,
        4045 as ::core::ffi::c_int as uint16,
        4286 as ::core::ffi::c_int as uint16,
    ];
    let mut pp: uint8 = (pitch as ::core::ffi::c_int >> 8 as ::core::ffi::c_int
        & 0x7f as ::core::ffi::c_int) as uint8;
    let mut q: uint8 = (pp as ::core::ffi::c_int / 12 as ::core::ffi::c_int) as uint8;
    let mut r: uint8 = (pp as ::core::ffi::c_int % 12 as ::core::ffi::c_int) as uint8;
    let mut t: uint16 = (kBaseNoteFreqs[r as usize] as ::core::ffi::c_int
        + ((kBaseNoteFreqs[(r as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int
            - kBaseNoteFreqs[r as usize] as ::core::ffi::c_int) as uint8
            as ::core::ffi::c_int
            * pitch as uint8 as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int)) as uint16;
    t = (t as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint16;
    while q as ::core::ffi::c_int != 6 as ::core::ffi::c_int {
        t = (t as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        q = q.wrapping_add(1);
    }
    t = ((*c).instrument_pitch_base as ::core::ffi::c_int * t as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
    if (*p).cur_chan_bit as ::core::ffi::c_int & (*p).is_chan_on as ::core::ffi::c_int == 0 {
        let mut reg: uint8 = ((*c).index as ::core::ffi::c_int * 16 as ::core::ffi::c_int) as uint8;
        Dsp_Write(
            p,
            (reg as ::core::ffi::c_int + V0PITCHL as ::core::ffi::c_int) as uint8_t,
            (t as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8,
        );
        Dsp_Write(
            p,
            (reg as ::core::ffi::c_int + V0PITCHH as ::core::ffi::c_int) as uint8_t,
            (t as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8,
        );
    }
}
unsafe fn WritePitch(mut p: *mut SpcPlayer, mut c: *mut Channel, mut pitch: uint16) {
    if pitch as ::core::ffi::c_int >> 8 as ::core::ffi::c_int >= 0x34 as ::core::ffi::c_int {
        pitch = (pitch as ::core::ffi::c_int
            + ((pitch as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                - 0x34 as ::core::ffi::c_int)) as uint16;
    } else if (pitch as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) < 0x13 as ::core::ffi::c_int
    {
        pitch = (pitch as ::core::ffi::c_int
            + ((((pitch as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                - 0x13 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int
                - 256 as ::core::ffi::c_int)) as uint16;
    }
    WritePitchInner(p, c, pitch);
}
unsafe fn Music_ResetChan(mut p: *mut SpcPlayer) {
    let mut c: *mut Channel = (&raw mut (*p).channel as *mut Channel).offset(7) as *mut Channel;
    (*p).cur_chan_bit = 0x80 as uint8;
    loop {
        *(&raw mut (*c).channel_volume as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = 0xff as uint8;
        (*c).pan_flag_with_phase_invert = 10 as uint8;
        (*c).pan_value = ((10 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
        (*c).instrument_id = 0 as uint8;
        (*c).fine_tune = 0 as uint8;
        (*c).channel_transposition = 0 as uint8;
        (*c).pitch_envelope_num_ticks = 0 as uint8;
        (*c).cutk = 0 as uint8;
        (*c).vib_depth = 0 as uint8;
        (*c).tremolo_depth = 0 as uint8;
        c = c.offset(-1);
        (*p).cur_chan_bit =
            ((*p).cur_chan_bit as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8;
        if !((*p).cur_chan_bit != 0) {
            break;
        }
    }
    (*p).master_volume_fade_ticks = 0 as uint8;
    (*p).echo_volume_fade_ticks = 0 as uint8;
    (*p).tempo_fade_num_ticks = 0 as uint8;
    (*p).global_transposition = 0 as uint8;
    (*p).block_count = 0 as uint8;
    (*p).percussion_base_id = 0 as uint8;
    *(&raw mut (*p).master_volume as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = 0xc0 as uint8;
    *(&raw mut (*p).tempo as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = 0x20 as uint8;
}
unsafe fn Channel_SetInstrument_NoSetId(
    mut p: *mut SpcPlayer,
    mut c: *mut Channel,
    mut instrument: uint8,
) {
    if instrument as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
        instrument = (instrument as ::core::ffi::c_int
            + 54 as ::core::ffi::c_int
            + (*p).percussion_base_id as ::core::ffi::c_int) as uint8;
    }
    let mut ip: *const uint8 = (&raw mut (*p).ram as *mut uint8)
        .offset((instrument as ::core::ffi::c_int * 6 as ::core::ffi::c_int) as isize)
        .offset(0x6c00);
    if (*p).is_chan_on as ::core::ffi::c_int & (*p).cur_chan_bit as ::core::ffi::c_int != 0 {
        return;
    }
    let mut reg: uint8 = ((*c).index as ::core::ffi::c_int * 16 as ::core::ffi::c_int) as uint8;
    if *ip.offset(0) as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
        (*p).reg_FLG = ((*p).reg_FLG as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int
            | *ip.offset(0) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
            as uint8;
        (*p).reg_NON =
            ((*p).reg_NON as ::core::ffi::c_int | (*p).cur_chan_bit as ::core::ffi::c_int) as uint8;
        Dsp_Write(
            p,
            (reg as ::core::ffi::c_int + V0SRCN as ::core::ffi::c_int) as uint8_t,
            0 as uint8,
        );
    } else {
        Dsp_Write(
            p,
            (reg as ::core::ffi::c_int + V0SRCN as ::core::ffi::c_int) as uint8_t,
            *ip.offset(0),
        );
    }
    Dsp_Write(
        p,
        (reg as ::core::ffi::c_int + V0ADSR1 as ::core::ffi::c_int) as uint8_t,
        *ip.offset(1),
    );
    Dsp_Write(
        p,
        (reg as ::core::ffi::c_int + V0ADSR2 as ::core::ffi::c_int) as uint8_t,
        *ip.offset(2),
    );
    Dsp_Write(
        p,
        (reg as ::core::ffi::c_int + V0GAIN as ::core::ffi::c_int) as uint8_t,
        *ip.offset(3),
    );
    (*c).instrument_pitch_base = ((*ip.offset(4) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
        | *ip.offset(5) as ::core::ffi::c_int) as uint16;
}
unsafe fn Channel_SetInstrument(mut p: *mut SpcPlayer, mut c: *mut Channel, mut instrument: uint8) {
    (*c).instrument_id = instrument;
    Channel_SetInstrument_NoSetId(p, c, instrument);
}
unsafe fn ComputePitchAdd(mut c: *mut Channel, mut pitch: uint8) {
    (*c).pitch_target = (pitch as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as uint8;
    (*c).pitch_add_per_tick = SpcDivHelper(
        (*c).pitch_target as ::core::ffi::c_int
            - ((*c).pitch as ::core::ffi::c_int >> 8 as ::core::ffi::c_int),
        (*c).pitch_slide_length,
    );
}
unsafe fn PitchSlideToNote_Check(mut p: *mut SpcPlayer, mut c: *mut Channel) {
    if (*c).pitch_slide_length as ::core::ffi::c_int != 0
        || (*p).ram[(*c).pattern_order_ptr_for_chan as usize] as ::core::ffi::c_int
            != 0xf9 as ::core::ffi::c_int
    {
        return;
    }
    (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
    let fresh3 = (*c).pattern_order_ptr_for_chan;
    (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
    (*c).pitch_slide_delay_left = (*p).ram[fresh3 as usize];
    let fresh4 = (*c).pattern_order_ptr_for_chan;
    (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
    (*c).pitch_slide_length = (*p).ram[fresh4 as usize];
    let fresh5 = (*c).pattern_order_ptr_for_chan;
    (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
    ComputePitchAdd(
        c,
        ((*p).ram[fresh5 as usize] as ::core::ffi::c_int
            + (*p).global_transposition as ::core::ffi::c_int
            + (*c).channel_transposition as ::core::ffi::c_int) as uint8,
    );
}
static kEffectByteLength: [uint8; 31] = [
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
];
unsafe fn HandleEffect(mut p: *mut SpcPlayer, mut c: *mut Channel, mut effect: uint8) {
    let mut arg: uint8 = (if kEffectByteLength
        [(effect as ::core::ffi::c_int - 0xe0 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
        != 0
    {
        let fresh9 = (*c).pattern_order_ptr_for_chan;
        (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
        (*p).ram[fresh9 as usize] as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as uint8;
    match effect as ::core::ffi::c_int {
        224 => {
            Channel_SetInstrument(p, c, arg);
        }
        225 => {
            (*c).pan_flag_with_phase_invert = arg;
            (*c).pan_value = ((arg as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as uint16;
        }
        226 => {
            (*c).pan_num_ticks = arg;
            let fresh10 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pan_target_value = (*p).ram[fresh10 as usize];
            (*c).pan_add_per_tick = SpcDivHelper(
                (*c).pan_target_value as ::core::ffi::c_int
                    - ((*c).pan_value as ::core::ffi::c_int >> 8 as ::core::ffi::c_int),
                arg,
            );
        }
        227 => {
            (*c).vibrato_delay_ticks = arg;
            let fresh11 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).vibrato_rate = (*p).ram[fresh11 as usize];
            let fresh12 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).vib_depth = (*p).ram[fresh12 as usize];
            (*c).vibrato_depth_target = (*c).vib_depth;
            (*c).vibrato_fade_num_ticks = 0 as uint8;
        }
        228 => {
            (*c).vib_depth = 0 as uint8;
            (*c).vibrato_depth_target = (*c).vib_depth;
            (*c).vibrato_fade_num_ticks = 0 as uint8;
        }
        229 => {
            (*p).master_volume = ((arg as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
        }
        230 => {
            (*p).master_volume_fade_ticks = arg;
            let fresh13 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).master_volume_fade_target = (*p).ram[fresh13 as usize];
            (*p).master_volume_fade_add_per_tick = SpcDivHelper(
                (*p).master_volume_fade_target as ::core::ffi::c_int
                    - ((*p).master_volume as ::core::ffi::c_int >> 8 as ::core::ffi::c_int),
                arg,
            );
        }
        231 => {
            (*p).tempo = ((arg as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
        }
        232 => {
            (*p).tempo_fade_num_ticks = arg;
            let fresh14 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).tempo_fade_final = (*p).ram[fresh14 as usize];
            (*p).tempo_fade_add = SpcDivHelper(
                (*p).tempo_fade_final as ::core::ffi::c_int
                    - ((*p).tempo as ::core::ffi::c_int >> 8 as ::core::ffi::c_int),
                arg,
            );
        }
        233 => {
            (*p).global_transposition = arg;
        }
        234 => {
            (*c).channel_transposition = arg;
        }
        235 => {
            (*c).tremolo_delay_ticks = arg;
            let fresh15 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).tremolo_rate = (*p).ram[fresh15 as usize];
            let fresh16 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).tremolo_depth = (*p).ram[fresh16 as usize];
        }
        236 => {
            (*c).tremolo_depth = 0 as uint8;
        }
        237 => {
            (*c).channel_volume =
                ((arg as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
        }
        238 => {
            (*c).volume_fade_ticks = arg;
            let fresh17 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).volume_fade_target = (*p).ram[fresh17 as usize];
            (*c).volume_fade_addpertick = SpcDivHelper(
                (*c).volume_fade_target as ::core::ffi::c_int
                    - ((*c).channel_volume as ::core::ffi::c_int >> 8 as ::core::ffi::c_int),
                arg,
            );
        }
        239 => {
            let fresh18 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pattern_start_ptr = (((*p).ram[fresh18 as usize] as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int
                | arg as ::core::ffi::c_int) as uint16;
            let fresh19 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).subroutine_num_loops = (*p).ram[fresh19 as usize];
            (*c).saved_pattern_ptr = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_start_ptr;
        }
        240 => {
            (*c).vibrato_fade_num_ticks = arg;
            (*c).vibrato_fade_add_per_tick = (if arg as ::core::ffi::c_int != 0 {
                (*c).vib_depth as ::core::ffi::c_int / arg as ::core::ffi::c_int
            } else {
                0xff as ::core::ffi::c_int
            }) as uint8;
        }
        241 | 242 => {
            (*c).pitch_envelope_direction = (effect as ::core::ffi::c_int
                == 0xf1 as ::core::ffi::c_int)
                as ::core::ffi::c_int as uint8;
            (*c).pitch_envelope_delay = arg;
            let fresh20 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pitch_envelope_num_ticks = (*p).ram[fresh20 as usize];
            let fresh21 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pitch_envelope_slide_value = (*p).ram[fresh21 as usize];
        }
        243 => {
            (*c).pitch_envelope_num_ticks = 0 as uint8;
        }
        244 => {
            (*c).fine_tune = arg;
        }
        245 => {
            (*p).reg_EON = arg;
            let fresh22 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).echo_volume_left = (((*p).ram[fresh22 as usize] as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as uint16;
            let fresh23 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).echo_volume_right = (((*p).ram[fresh23 as usize] as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as uint16;
            (*p).reg_FLG =
                ((*p).reg_FLG as ::core::ffi::c_int & !(0x20 as ::core::ffi::c_int)) as uint8;
        }
        246 => {
            (*p).echo_volume_left = 0 as uint16;
            (*p).echo_volume_right = 0 as uint16;
            (*p).reg_FLG =
                ((*p).reg_FLG as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint8;
        }
        247 => {
            static kEchoFirParameters: [int8_t; 32] = [
                127 as ::core::ffi::c_int as int8_t,
                0 as ::core::ffi::c_int as int8_t,
                0 as ::core::ffi::c_int as int8_t,
                0 as ::core::ffi::c_int as int8_t,
                0 as ::core::ffi::c_int as int8_t,
                0 as ::core::ffi::c_int as int8_t,
                0 as ::core::ffi::c_int as int8_t,
                0 as ::core::ffi::c_int as int8_t,
                88 as ::core::ffi::c_int as int8_t,
                -(65 as ::core::ffi::c_int) as int8_t,
                -(37 as ::core::ffi::c_int) as int8_t,
                -(16 as ::core::ffi::c_int) as int8_t,
                -(2 as ::core::ffi::c_int) as int8_t,
                7 as ::core::ffi::c_int as int8_t,
                12 as ::core::ffi::c_int as int8_t,
                12 as ::core::ffi::c_int as int8_t,
                12 as ::core::ffi::c_int as int8_t,
                33 as ::core::ffi::c_int as int8_t,
                43 as ::core::ffi::c_int as int8_t,
                43 as ::core::ffi::c_int as int8_t,
                19 as ::core::ffi::c_int as int8_t,
                -(2 as ::core::ffi::c_int) as int8_t,
                -(13 as ::core::ffi::c_int) as int8_t,
                -(7 as ::core::ffi::c_int) as int8_t,
                52 as ::core::ffi::c_int as int8_t,
                51 as ::core::ffi::c_int as int8_t,
                0 as ::core::ffi::c_int as int8_t,
                -(39 as ::core::ffi::c_int) as int8_t,
                -(27 as ::core::ffi::c_int) as int8_t,
                1 as ::core::ffi::c_int as int8_t,
                -(4 as ::core::ffi::c_int) as int8_t,
                -(21 as ::core::ffi::c_int) as int8_t,
            ];
            SetupEchoParameter_EDL(p, arg);
            let fresh24 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).reg_EFB = (*p).ram[fresh24 as usize];
            let fresh25 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            let mut ep: *const int8_t = (&raw const kEchoFirParameters as *const int8_t).offset(
                ((*p).ram[fresh25 as usize] as ::core::ffi::c_int * 8 as ::core::ffi::c_int)
                    as isize,
            );
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 8 as ::core::ffi::c_int {
                let fresh26 = ep;
                ep = ep.offset(1);
                Dsp_Write(
                    p,
                    (FIR0 as ::core::ffi::c_int + i * 16 as ::core::ffi::c_int) as uint8_t,
                    *fresh26 as uint8,
                );
                i += 1;
            }
        }
        248 => {
            (*p).echo_volume_fade_ticks = arg;
            let fresh27 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).echo_volume_fade_target_left = (*p).ram[fresh27 as usize];
            let fresh28 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).echo_volume_fade_target_right = (*p).ram[fresh28 as usize];
            (*p).echo_volume_fade_add_left = SpcDivHelper(
                (*p).echo_volume_fade_target_left as ::core::ffi::c_int
                    - ((*p).echo_volume_left as ::core::ffi::c_int >> 8 as ::core::ffi::c_int),
                arg,
            );
            (*p).echo_volume_fade_add_right = SpcDivHelper(
                (*p).echo_volume_fade_target_right as ::core::ffi::c_int
                    - ((*p).echo_volume_right as ::core::ffi::c_int >> 8 as ::core::ffi::c_int),
                arg,
            );
        }
        249 => {
            (*c).pitch_slide_delay_left = arg;
            let fresh29 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pitch_slide_length = (*p).ram[fresh29 as usize];
            let fresh30 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            ComputePitchAdd(
                c,
                ((*p).ram[fresh30 as usize] as ::core::ffi::c_int
                    + (*p).global_transposition as ::core::ffi::c_int
                    + (*c).channel_transposition as ::core::ffi::c_int) as uint8,
            );
        }
        250 => {
            (*p).percussion_base_id = arg;
        }
        251 => {
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
        }
        252 => {
            (*c).cutk = (arg as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8;
        }
        253 => {
            (*p).fast_forward = (arg as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8;
            (*p).key_OFF = ((*p).key_OFF as ::core::ffi::c_int
                | !((*p).is_chan_on as ::core::ffi::c_int)) as uint8;
        }
        254 => {
            (*p).fast_forward = arg;
            (*p).key_OFF = ((*p).key_OFF as ::core::ffi::c_int
                | !((*p).is_chan_on as ::core::ffi::c_int)) as uint8;
        }
        _ => {
            Not_Implemented();
        }
    };
}
unsafe fn WantWriteKof(mut p: *mut SpcPlayer, mut c: *mut Channel) -> bool {
    let mut loops: ::core::ffi::c_int = (*c).subroutine_num_loops as ::core::ffi::c_int;
    let mut ptr: ::core::ffi::c_int = (*c).pattern_order_ptr_for_chan as ::core::ffi::c_int;
    loop {
        let fresh7 = ptr;
        ptr = ptr + 1;
        let mut cmd: uint8 = (*p).ram[fresh7 as usize];
        if cmd as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            if loops == 0 as ::core::ffi::c_int {
                return true_0 != 0;
            }
            loops -= 1;
            ptr = if loops == 0 as ::core::ffi::c_int {
                (*c).saved_pattern_ptr as ::core::ffi::c_int
            } else {
                (*c).pattern_start_ptr as ::core::ffi::c_int
            };
        } else {
            while cmd as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
                let fresh8 = ptr;
                ptr = ptr + 1;
                cmd = (*p).ram[fresh8 as usize];
            }
            if cmd as ::core::ffi::c_int == 0xc8 as ::core::ffi::c_int {
                return false_0 != 0;
            }
            if cmd as ::core::ffi::c_int == 0xef as ::core::ffi::c_int {
                ptr = (*p).ram[(ptr + 0 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                    | ((*p).ram[(ptr + 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int;
            } else if cmd as ::core::ffi::c_int >= 0xe0 as ::core::ffi::c_int {
                ptr += kEffectByteLength
                    [(cmd as ::core::ffi::c_int - 0xe0 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int;
            } else {
                return true_0 != 0;
            }
        }
    }
}
pub unsafe fn CalcFinalVolume(mut p: *mut SpcPlayer, mut c: *mut Channel, mut vol: uint8) {
    let mut t: ::core::ffi::c_int = ((*p).master_volume as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int)
        * vol as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int;
    t = t * (*c).channel_volume_master as ::core::ffi::c_int >> 8 as ::core::ffi::c_int;
    t = t * ((*c).channel_volume as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int;
    (*c).final_volume = (t * t >> 8 as ::core::ffi::c_int) as uint8;
}
unsafe fn CalcTremolo(mut p: *mut SpcPlayer, mut c: *mut Channel, mut a: uint8) {
    if a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
        a = (a as ::core::ffi::c_int * 2 as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int)
            as uint8;
    } else {
        a = (a as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as uint8;
    }
    a = (a as ::core::ffi::c_int * (*c).tremolo_depth as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint8;
    CalcFinalVolume(
        p,
        c,
        (a as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8,
    );
}
unsafe fn HandleTremolo(mut p: *mut SpcPlayer, mut c: *mut Channel) {
    (*p).did_affect_volumepitch_flag = ((*p).did_affect_volumepitch_flag as ::core::ffi::c_int
        | 0x80 as ::core::ffi::c_int) as uint8;
    CalcTremolo(
        p,
        c,
        ((*c).tremolo_rate as ::core::ffi::c_int * (*p).main_tempo_accum as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8,
    );
}
unsafe fn CalcVibratoAddPitch(
    mut p: *mut SpcPlayer,
    mut c: *mut Channel,
    mut pitch: uint16,
    mut value: uint8,
) {
    let mut t: ::core::ffi::c_int = (value as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
    t ^= if t & 0x100 as ::core::ffi::c_int != 0 {
        0xff as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut r: ::core::ffi::c_int =
        if (*c).vib_depth as ::core::ffi::c_int >= 0xf1 as ::core::ffi::c_int {
            t as uint8 as ::core::ffi::c_int
                * ((*c).vib_depth as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        } else {
            t as uint8 as ::core::ffi::c_int * (*c).vib_depth as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int
        };
    WritePitch(
        p,
        c,
        (pitch as ::core::ffi::c_int
            + (if value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
                -r
            } else {
                r
            })) as uint16,
    );
}
unsafe fn HandlePanAndSweep(mut p: *mut SpcPlayer, mut c: *mut Channel) {
    (*p).did_affect_volumepitch_flag = 0 as uint8;
    if (*c).tremolo_depth as ::core::ffi::c_int != 0
        && (*c).tremolo_hold_count as ::core::ffi::c_int
            == (*c).tremolo_delay_ticks as ::core::ffi::c_int
    {
        HandleTremolo(p, c);
    }
    let mut volume: uint16 = (*c).pan_value;
    if (*c).pan_num_ticks != 0 {
        (*p).did_affect_volumepitch_flag = 0x80 as uint8;
        volume = (volume as ::core::ffi::c_int
            + (*p).main_tempo_accum as ::core::ffi::c_int
                * (*c).pan_add_per_tick as int16_t as ::core::ffi::c_int
                / 256 as ::core::ffi::c_int) as uint16;
    }
    if (*p).did_affect_volumepitch_flag != 0 {
        WriteVolumeToDsp(p, c, volume);
    }
    (*p).did_affect_volumepitch_flag = 0 as uint8;
    let mut pitch: uint16 = (*c).pitch;
    if (*c).pitch_slide_length as ::core::ffi::c_int != 0 && (*c).pitch_slide_delay_left == 0 {
        (*p).did_affect_volumepitch_flag = ((*p).did_affect_volumepitch_flag as ::core::ffi::c_int
            | 0x80 as ::core::ffi::c_int) as uint8;
        pitch = (pitch as ::core::ffi::c_int
            + (*p).main_tempo_accum as ::core::ffi::c_int
                * (*c).pitch_add_per_tick as int16_t as ::core::ffi::c_int
                / 256 as ::core::ffi::c_int) as uint16;
    }
    if (*c).vib_depth as ::core::ffi::c_int != 0
        && (*c).vibrato_delay_ticks as ::core::ffi::c_int
            == (*c).vibrato_hold_count as ::core::ffi::c_int
    {
        CalcVibratoAddPitch(
            p,
            c,
            pitch,
            (((*p).main_tempo_accum as ::core::ffi::c_int
                * (*c).vibrato_rate as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)
                + (*c).vibrato_count as ::core::ffi::c_int) as uint8,
        );
        return;
    }
    if (*p).did_affect_volumepitch_flag != 0 {
        WritePitch(p, c, pitch);
    }
}
unsafe fn HandleNoteTick(mut p: *mut SpcPlayer, mut c: *mut Channel) {
    if (*c).note_keyoff_ticks_left as ::core::ffi::c_int != 0 as ::core::ffi::c_int && {
        (*c).note_keyoff_ticks_left = (*c).note_keyoff_ticks_left.wrapping_sub(1);
        (*c).note_keyoff_ticks_left as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            || (*c).note_ticks_left as ::core::ffi::c_int == 2 as ::core::ffi::c_int
    } {
        if WantWriteKof(p, c) as ::core::ffi::c_int != 0
            && (*p).cur_chan_bit as ::core::ffi::c_int & (*p).is_chan_on as ::core::ffi::c_int == 0
        {
            Dsp_Write(p, KOF as ::core::ffi::c_int as uint8_t, (*p).cur_chan_bit);
        }
    }
    (*p).did_affect_volumepitch_flag = 0 as uint8;
    if (*c).pitch_slide_length != 0 {
        if (*c).pitch_slide_delay_left != 0 {
            (*c).pitch_slide_delay_left = (*c).pitch_slide_delay_left.wrapping_sub(1);
        } else {
            (*p).did_affect_volumepitch_flag = 0x80 as uint8;
            (*c).pitch_slide_length = (*c).pitch_slide_length.wrapping_sub(1);
            if (*c).pitch_slide_length as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                (*c).pitch = (((*c).pitch_target as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                    | (*c).fine_tune as ::core::ffi::c_int) as uint16;
            } else {
                (*c).pitch = ((*c).pitch as ::core::ffi::c_int
                    + (*c).pitch_add_per_tick as ::core::ffi::c_int)
                    as uint16;
            }
        }
    }
    let mut pitch: uint16 = (*c).pitch;
    if (*c).vib_depth != 0 {
        if (*c).vibrato_delay_ticks as ::core::ffi::c_int
            == (*c).vibrato_hold_count as ::core::ffi::c_int
        {
            if (*c).vibrato_change_count as ::core::ffi::c_int
                == (*c).vibrato_fade_num_ticks as ::core::ffi::c_int
            {
                (*c).vib_depth = (*c).vibrato_depth_target;
            } else {
                let fresh6 = (*c).vibrato_change_count;
                (*c).vibrato_change_count = (*c).vibrato_change_count.wrapping_add(1);
                (*c).vib_depth = ((if fresh6 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    0 as ::core::ffi::c_int
                } else {
                    (*c).vib_depth as ::core::ffi::c_int
                }) + (*c).vibrato_fade_add_per_tick as ::core::ffi::c_int)
                    as uint8;
            }
            (*c).vibrato_count = ((*c).vibrato_count as ::core::ffi::c_int
                + (*c).vibrato_rate as ::core::ffi::c_int)
                as uint8;
            CalcVibratoAddPitch(p, c, pitch, (*c).vibrato_count);
            return;
        }
        (*c).vibrato_hold_count = (*c).vibrato_hold_count.wrapping_add(1);
    }
    if (*p).did_affect_volumepitch_flag != 0 {
        WritePitch(p, c, pitch);
    }
}
unsafe fn Chan_HandleTick(mut p: *mut SpcPlayer, mut c: *mut Channel) {
    if (*c).volume_fade_ticks != 0 {
        (*c).volume_fade_ticks = (*c).volume_fade_ticks.wrapping_sub(1);
        if (*c).volume_fade_ticks as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*c).channel_volume = (((*c).volume_fade_target as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as uint16;
        } else {
            (*c).channel_volume = ((*c).channel_volume as ::core::ffi::c_int
                + (*c).volume_fade_addpertick as ::core::ffi::c_int)
                as uint16;
        }
        (*p).vol_dirty = ((*p).vol_dirty as ::core::ffi::c_int
            | (*p).cur_chan_bit as ::core::ffi::c_int) as uint8;
    }
    if (*c).tremolo_depth != 0 {
        if (*c).tremolo_delay_ticks as ::core::ffi::c_int
            == (*c).tremolo_hold_count as ::core::ffi::c_int
        {
            (*p).vol_dirty = ((*p).vol_dirty as ::core::ffi::c_int
                | (*p).cur_chan_bit as ::core::ffi::c_int) as uint8;
            if (*c).tremolo_count as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0
                && (*c).tremolo_depth as ::core::ffi::c_int == 0xff as ::core::ffi::c_int
            {
                (*c).tremolo_count = 0x80 as uint8;
            } else {
                (*c).tremolo_count = ((*c).tremolo_count as ::core::ffi::c_int
                    + (*c).tremolo_rate as ::core::ffi::c_int)
                    as uint8;
            }
            CalcTremolo(p, c, (*c).tremolo_count);
        } else {
            (*c).tremolo_hold_count = (*c).tremolo_hold_count.wrapping_add(1);
            CalcFinalVolume(p, c, 0xff as uint8);
        }
    } else {
        CalcFinalVolume(p, c, 0xff as uint8);
    }
    if (*c).pan_num_ticks != 0 {
        (*c).pan_num_ticks = (*c).pan_num_ticks.wrapping_sub(1);
        if (*c).pan_num_ticks as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*c).pan_value = (((*c).pan_target_value as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int) as uint16;
        } else {
            (*c).pan_value = ((*c).pan_value as ::core::ffi::c_int
                + (*c).pan_add_per_tick as ::core::ffi::c_int)
                as uint16;
        }
        (*p).vol_dirty = ((*p).vol_dirty as ::core::ffi::c_int
            | (*p).cur_chan_bit as ::core::ffi::c_int) as uint8;
    }
    if (*p).vol_dirty as ::core::ffi::c_int & (*p).cur_chan_bit as ::core::ffi::c_int != 0 {
        WriteVolumeToDsp(p, c, (*c).pan_value);
    }
}
unsafe fn Music_HandleCmdFromSnes(mut p: *mut SpcPlayer) {
    let mut c: *mut Channel = ::core::ptr::null_mut::<Channel>();
    let mut t: ::core::ffi::c_int = 0;
    let mut a: uint8 = (*p).input_ports[0 as ::core::ffi::c_int as usize];
    (*p).input_ports[0 as ::core::ffi::c_int as usize] = 255 as uint8;
    if a as ::core::ffi::c_int == 0xf0 as ::core::ffi::c_int {
        (*p).key_OFF = ((*p).key_OFF as ::core::ffi::c_int
            | !((*p).is_chan_on as ::core::ffi::c_int)) as uint8;
    } else {
        let mut current_block_83: u64;
        if !(a as ::core::ffi::c_int != 0xf1 as ::core::ffi::c_int
            && a as ::core::ffi::c_int != 255 as ::core::ffi::c_int
            && a as ::core::ffi::c_int
                != (*p).port_to_snes[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
        {
            if (*p).port_to_snes[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                return;
            }
            if (*p).counter_sf0c as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                current_block_83 = 7829208624863839112;
            } else {
                (*p).counter_sf0c = (*p).counter_sf0c.wrapping_sub(1);
                if (*p).counter_sf0c as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                    Music_ResetChan(p);
                    return;
                }
                current_block_83 = 12349973810996921269;
            }
            '_label_a: loop {
                match current_block_83 {
                    12349973810996921269 => {
                        t = *((&raw mut (*p).ram as *mut uint8)
                            .offset((*p).music_ptr_toplevel as isize)
                            as *mut uint8 as *mut uint16)
                            as ::core::ffi::c_int;
                        (*p).music_ptr_toplevel = ((*p).music_ptr_toplevel as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int)
                            as uint16;
                        if t >> 8 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i < 8 as ::core::ffi::c_int {
                                (*p).channel[i as usize].pattern_order_ptr_for_chan =
                                    *((&raw mut (*p).ram as *mut uint8).offset(t as isize)
                                        as *mut uint8
                                        as *mut uint16);
                                t += 2 as ::core::ffi::c_int;
                                i += 1;
                            }
                            c = &raw mut (*p).channel as *mut Channel;
                            (*p).cur_chan_bit = 1 as uint8;
                            loop {
                                if *(&raw mut (*c).pattern_order_ptr_for_chan as *mut uint8).offset(
                                    (::core::mem::size_of::<uint16>() as usize)
                                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                        .wrapping_sub(1 as usize)
                                        as isize,
                                ) as ::core::ffi::c_int
                                    != 0
                                    && (*c).instrument_id as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                {
                                    Channel_SetInstrument(p, c, 0 as uint8);
                                }
                                (*c).subroutine_num_loops = 0 as uint8;
                                (*c).volume_fade_ticks = 0 as uint8;
                                (*c).pan_num_ticks = 0 as uint8;
                                (*c).note_ticks_left = 1 as uint8;
                                c = c.offset(1);
                                (*p).cur_chan_bit = (((*p).cur_chan_bit as ::core::ffi::c_int)
                                    << 1 as ::core::ffi::c_int)
                                    as uint8;
                                if !((*p).cur_chan_bit != 0) {
                                    break;
                                }
                            }
                            current_block_83 = 7829208624863839112;
                        } else if t == 0 as ::core::ffi::c_int {
                            a = 0 as uint8;
                            break;
                        } else {
                            if t == 0x80 as ::core::ffi::c_int {
                                (*p).fast_forward = 0x80 as uint8;
                            } else if t == 0x81 as ::core::ffi::c_int {
                                (*p).fast_forward = 0 as uint8;
                            } else {
                                (*p).block_count = (*p).block_count.wrapping_sub(1);
                                if (*p).block_count as ::core::ffi::c_int
                                    & 0x80 as ::core::ffi::c_int
                                    != 0
                                {
                                    (*p).block_count = t as uint8;
                                }
                                t = *((&raw mut (*p).ram as *mut uint8)
                                    .offset((*p).music_ptr_toplevel as isize)
                                    as *mut uint8
                                    as *mut uint16)
                                    as ::core::ffi::c_int;
                                (*p).music_ptr_toplevel = ((*p).music_ptr_toplevel
                                    as ::core::ffi::c_int
                                    + 2 as ::core::ffi::c_int)
                                    as uint16;
                                if (*p).block_count as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                                {
                                    (*p).music_ptr_toplevel = t as uint16;
                                }
                            }
                            current_block_83 = 12349973810996921269;
                        }
                    }
                    _ => {
                        (*p).vol_dirty = 0 as uint8;
                        c = &raw mut (*p).channel as *mut Channel;
                        (*p).cur_chan_bit = 1 as uint8;
                        loop {
                            if !(*(&raw mut (*c).pattern_order_ptr_for_chan as *mut uint8).offset(
                                (::core::mem::size_of::<uint16>() as usize)
                                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            ) == 0)
                            {
                                (*c).note_ticks_left = (*c).note_ticks_left.wrapping_sub(1);
                                if (*c).note_ticks_left == 0 {
                                    loop {
                                        let fresh0 = (*c).pattern_order_ptr_for_chan;
                                        (*c).pattern_order_ptr_for_chan =
                                            (*c).pattern_order_ptr_for_chan.wrapping_add(1);
                                        let mut cmd: uint8 = (*p).ram[fresh0 as usize];
                                        if cmd as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                                            if (*c).subroutine_num_loops == 0 {
                                                current_block_83 = 12349973810996921269;
                                                continue '_label_a;
                                            }
                                            (*c).subroutine_num_loops =
                                                (*c).subroutine_num_loops.wrapping_sub(1);
                                            (*c).pattern_order_ptr_for_chan =
                                                (if (*c).subroutine_num_loops as ::core::ffi::c_int
                                                    == 0 as ::core::ffi::c_int
                                                {
                                                    (*c).saved_pattern_ptr as ::core::ffi::c_int
                                                } else {
                                                    (*c).pattern_start_ptr as ::core::ffi::c_int
                                                })
                                                    as uint16;
                                        } else {
                                            if cmd as ::core::ffi::c_int
                                                & 0x80 as ::core::ffi::c_int
                                                == 0
                                            {
                                                static kNoteVol: [uint8; 16] = [
                                                    25 as ::core::ffi::c_int as uint8,
                                                    50 as ::core::ffi::c_int as uint8,
                                                    76 as ::core::ffi::c_int as uint8,
                                                    101 as ::core::ffi::c_int as uint8,
                                                    114 as ::core::ffi::c_int as uint8,
                                                    127 as ::core::ffi::c_int as uint8,
                                                    140 as ::core::ffi::c_int as uint8,
                                                    152 as ::core::ffi::c_int as uint8,
                                                    165 as ::core::ffi::c_int as uint8,
                                                    178 as ::core::ffi::c_int as uint8,
                                                    191 as ::core::ffi::c_int as uint8,
                                                    203 as ::core::ffi::c_int as uint8,
                                                    216 as ::core::ffi::c_int as uint8,
                                                    229 as ::core::ffi::c_int as uint8,
                                                    242 as ::core::ffi::c_int as uint8,
                                                    252 as ::core::ffi::c_int as uint8,
                                                ];
                                                static kNoteGateOffPct: [uint8; 8] = [
                                                    50 as ::core::ffi::c_int as uint8,
                                                    101 as ::core::ffi::c_int as uint8,
                                                    127 as ::core::ffi::c_int as uint8,
                                                    152 as ::core::ffi::c_int as uint8,
                                                    178 as ::core::ffi::c_int as uint8,
                                                    203 as ::core::ffi::c_int as uint8,
                                                    229 as ::core::ffi::c_int as uint8,
                                                    252 as ::core::ffi::c_int as uint8,
                                                ];
                                                (*c).note_length = cmd;
                                                let fresh1 = (*c).pattern_order_ptr_for_chan;
                                                (*c).pattern_order_ptr_for_chan =
                                                    (*c).pattern_order_ptr_for_chan.wrapping_add(1);
                                                cmd = (*p).ram[fresh1 as usize];
                                                if cmd as ::core::ffi::c_int
                                                    & 0x80 as ::core::ffi::c_int
                                                    == 0
                                                {
                                                    (*c).note_gate_off_fixedpt = kNoteGateOffPct
                                                        [(cmd as ::core::ffi::c_int
                                                            >> 4 as ::core::ffi::c_int
                                                            & 7 as ::core::ffi::c_int)
                                                            as usize];
                                                    (*c).channel_volume_master = kNoteVol[(cmd
                                                        as ::core::ffi::c_int
                                                        & 0xf as ::core::ffi::c_int)
                                                        as usize];
                                                    let fresh2 = (*c).pattern_order_ptr_for_chan;
                                                    (*c).pattern_order_ptr_for_chan = (*c)
                                                        .pattern_order_ptr_for_chan
                                                        .wrapping_add(1);
                                                    cmd = (*p).ram[fresh2 as usize];
                                                }
                                            }
                                            if cmd as ::core::ffi::c_int
                                                >= 0xe0 as ::core::ffi::c_int
                                            {
                                                HandleEffect(p, c, cmd);
                                            } else {
                                                if (*c).cutk as ::core::ffi::c_int
                                                    | (*p).fast_forward as ::core::ffi::c_int
                                                    == 0 as ::core::ffi::c_int
                                                {
                                                    PlayNote(p, c, cmd);
                                                }
                                                (*c).note_ticks_left = (*c).note_length;
                                                t = (*c).note_ticks_left as ::core::ffi::c_int
                                                    * (*c).note_gate_off_fixedpt
                                                        as ::core::ffi::c_int
                                                    >> 8 as ::core::ffi::c_int;
                                                (*c).note_keyoff_ticks_left =
                                                    (if t != 0 as ::core::ffi::c_int {
                                                        t
                                                    } else {
                                                        1 as ::core::ffi::c_int
                                                    })
                                                        as uint8;
                                                PitchSlideToNote_Check(p, c);
                                                break;
                                            }
                                        }
                                    }
                                } else if (*p).fast_forward == 0 {
                                    HandleNoteTick(p, c);
                                    PitchSlideToNote_Check(p, c);
                                }
                            }
                            c = c.offset(1);
                            (*p).cur_chan_bit = (((*p).cur_chan_bit as ::core::ffi::c_int)
                                << 1 as ::core::ffi::c_int)
                                as uint8;
                            if !((*p).cur_chan_bit != 0) {
                                break;
                            }
                        }
                        if (*p).tempo_fade_num_ticks != 0 {
                            (*p).tempo_fade_num_ticks = (*p).tempo_fade_num_ticks.wrapping_sub(1);
                            (*p).tempo = (if (*p).tempo_fade_num_ticks as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                ((*p).tempo_fade_final as ::core::ffi::c_int)
                                    << 8 as ::core::ffi::c_int
                            } else {
                                (*p).tempo as ::core::ffi::c_int
                                    + (*p).tempo_fade_add as ::core::ffi::c_int
                            }) as uint16;
                        }
                        if (*p).echo_volume_fade_ticks != 0 {
                            (*p).echo_volume_left = ((*p).echo_volume_left as ::core::ffi::c_int
                                + (*p).echo_volume_fade_add_left as ::core::ffi::c_int)
                                as uint16;
                            (*p).echo_volume_right = ((*p).echo_volume_right as ::core::ffi::c_int
                                + (*p).echo_volume_fade_add_right as ::core::ffi::c_int)
                                as uint16;
                            (*p).echo_volume_fade_ticks =
                                (*p).echo_volume_fade_ticks.wrapping_sub(1);
                            if (*p).echo_volume_fade_ticks as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                (*p).echo_volume_left = (((*p).echo_volume_fade_target_left
                                    as ::core::ffi::c_int)
                                    << 8 as ::core::ffi::c_int)
                                    as uint16;
                                (*p).echo_volume_right = (((*p).echo_volume_fade_target_right
                                    as ::core::ffi::c_int)
                                    << 8 as ::core::ffi::c_int)
                                    as uint16;
                            }
                        }
                        if (*p).master_volume_fade_ticks != 0 {
                            (*p).master_volume_fade_ticks =
                                (*p).master_volume_fade_ticks.wrapping_sub(1);
                            (*p).master_volume = (if (*p).master_volume_fade_ticks
                                as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                ((*p).master_volume_fade_target as ::core::ffi::c_int)
                                    << 8 as ::core::ffi::c_int
                            } else {
                                (*p).master_volume as ::core::ffi::c_int
                                    + (*p).master_volume_fade_add_per_tick as ::core::ffi::c_int
                            }) as uint16;
                            (*p).vol_dirty = 0xff as uint8;
                        }
                        c = &raw mut (*p).channel as *mut Channel;
                        (*p).cur_chan_bit = 1 as uint8;
                        loop {
                            if *(&raw mut (*c).pattern_order_ptr_for_chan as *mut uint8).offset(
                                (::core::mem::size_of::<uint16>() as usize)
                                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                    .wrapping_sub(1 as usize)
                                    as isize,
                            ) != 0
                            {
                                Chan_HandleTick(p, c);
                            }
                            c = c.offset(1);
                            (*p).cur_chan_bit = (((*p).cur_chan_bit as ::core::ffi::c_int)
                                << 1 as ::core::ffi::c_int)
                                as uint8;
                            if !((*p).cur_chan_bit != 0) {
                                break;
                            }
                        }
                        return;
                    }
                }
            }
        }
        (*p).port_to_snes[0 as ::core::ffi::c_int as usize] = a;
        (*p).music_ptr_toplevel = *((&raw mut (*p).ram as *mut uint8).offset(
            (0x5820 as ::core::ffi::c_int
                + (a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) * 2 as ::core::ffi::c_int)
                as isize,
        ) as *mut uint8 as *mut uint16);
        (*p).counter_sf0c = 2 as uint16;
        (*p).key_OFF = ((*p).key_OFF as ::core::ffi::c_int
            | !((*p).is_chan_on as ::core::ffi::c_int)) as uint8;
        return;
    };
}
unsafe fn PlayNote(mut p: *mut SpcPlayer, mut c: *mut Channel, mut note: uint8) {
    if note as ::core::ffi::c_int >= 0xca as ::core::ffi::c_int {
        Channel_SetInstrument(p, c, note);
        note = 0xa4 as uint8;
    }
    if note as ::core::ffi::c_int >= 0xc8 as ::core::ffi::c_int
        || (*p).is_chan_on as ::core::ffi::c_int & (*p).cur_chan_bit as ::core::ffi::c_int != 0
    {
        return;
    }
    (*c).pitch = (((note as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int)
        + (*p).global_transposition as ::core::ffi::c_int
        + (*c).channel_transposition as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int
        | (*c).fine_tune as ::core::ffi::c_int) as uint16;
    (*c).vibrato_count =
        (((*c).vibrato_fade_num_ticks as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as uint8;
    (*c).vibrato_hold_count = 0 as uint8;
    (*c).vibrato_change_count = 0 as uint8;
    (*c).tremolo_count = 0 as uint8;
    (*c).tremolo_hold_count = 0 as uint8;
    (*p).vol_dirty =
        ((*p).vol_dirty as ::core::ffi::c_int | (*p).cur_chan_bit as ::core::ffi::c_int) as uint8;
    (*p).key_ON =
        ((*p).key_ON as ::core::ffi::c_int | (*p).cur_chan_bit as ::core::ffi::c_int) as uint8;
    (*c).pitch_slide_length = (*c).pitch_envelope_num_ticks;
    if (*c).pitch_slide_length != 0 {
        (*c).pitch_slide_delay_left = (*c).pitch_envelope_delay;
        if (*c).pitch_envelope_direction == 0 {
            (*c).pitch = ((*c).pitch as ::core::ffi::c_int
                - (((*c).pitch_envelope_slide_value as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int)) as uint16;
        }
        ComputePitchAdd(
            c,
            (((*c).pitch as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
                + (*c).pitch_envelope_slide_value as ::core::ffi::c_int) as uint8,
        );
    }
    WritePitch(p, c, (*c).pitch);
}
unsafe fn Spc_Loop_Part1(mut p: *mut SpcPlayer) {
    Dsp_Write(p, KOF as ::core::ffi::c_int as uint8_t, (*p).key_OFF);
    Dsp_Write(p, PMON as ::core::ffi::c_int as uint8_t, 0 as uint8);
    Dsp_Write(p, NON as ::core::ffi::c_int as uint8_t, (*p).reg_NON);
    Dsp_Write(p, KOF as ::core::ffi::c_int as uint8_t, 0 as uint8);
    Dsp_Write(p, KON as ::core::ffi::c_int as uint8_t, (*p).key_ON);
    if (*p).echo_stored_time as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
        Dsp_Write(p, FLG as ::core::ffi::c_int as uint8_t, (*p).reg_FLG);
        if (*p).echo_stored_time as ::core::ffi::c_int
            == (*p).echo_parameter_EDL as ::core::ffi::c_int
        {
            Dsp_Write(p, EON as ::core::ffi::c_int as uint8_t, (*p).reg_EON);
            Dsp_Write(p, EFB as ::core::ffi::c_int as uint8_t, (*p).reg_EFB);
            Dsp_Write(
                p,
                EVOLR as ::core::ffi::c_int as uint8_t,
                ((*p).echo_volume_right as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8,
            );
            Dsp_Write(
                p,
                EVOLL as ::core::ffi::c_int as uint8_t,
                ((*p).echo_volume_left as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8,
            );
        }
    }
    (*p).key_ON = 0 as uint8;
    (*p).key_OFF = (*p).key_ON;
}
unsafe fn Spc_Loop_Part2(mut p: *mut SpcPlayer, mut ticks: uint8) {
    let mut t: ::core::ffi::c_int = (*p).sfx_timer_accum as ::core::ffi::c_int
        + (ticks as ::core::ffi::c_int * 0x20 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int;
    (*p).sfx_timer_accum = t as uint8;
    if t >= 0x100 as ::core::ffi::c_int {
        Sfx1_HandleCmdFromSnes(p);
        Sfx2_HandleCmdFromSnes(p);
        Sfx3_HandleCmdFromSnes(p);
        if (*p).echo_stored_time as ::core::ffi::c_int
            != (*p).echo_parameter_EDL as ::core::ffi::c_int
        {
            (*p).echo_stored_time = (*p).echo_stored_time.wrapping_add(1);
        }
    }
    t = (*p).main_tempo_accum as ::core::ffi::c_int
        + (ticks as ::core::ffi::c_int
            * *(&raw mut (*p).tempo as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int;
    (*p).main_tempo_accum = t as uint8;
    if t >= 256 as ::core::ffi::c_int {
        loop {
            Music_HandleCmdFromSnes(p);
            if !((*p).fast_forward != 0) {
                break;
            }
        }
    } else if (*p).port_to_snes[0 as ::core::ffi::c_int as usize] != 0 {
        let mut c: *mut Channel = &raw mut (*p).channel as *mut Channel;
        (*p).cur_chan_bit = 1 as uint8;
        while (*p).cur_chan_bit as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if *(&raw mut (*c).pattern_order_ptr_for_chan as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) != 0
            {
                HandlePanAndSweep(p, c);
            }
            (*p).cur_chan_bit =
                (((*p).cur_chan_bit as ::core::ffi::c_int) << 1 as ::core::ffi::c_int) as uint8;
            c = c.offset(1);
        }
    }
}
unsafe fn Vector_Reset_Spc(mut p: *mut SpcPlayer) {
    dsp_reset((*p).dsp);
    memset(
        (&raw mut (*p).ram as *mut uint8).offset(0x500) as *mut uint8 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        0x1000 as size_t,
    );
    memset(
        (&raw mut (*p).ram as *mut uint8).offset(0x20) as *mut uint8 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        0xf as size_t,
    );
    memset(
        (&raw mut (*p).ram as *mut uint8).offset(0xd0) as *mut uint8 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        0x1f as size_t,
    );
    memset(
        (&raw mut (*p).ram as *mut uint8).offset(0x391) as *mut uint8 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        0x6f as size_t,
    );
    memset(
        (&raw mut (*p).ram as *mut uint8).offset(0x440) as *mut uint8 as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        0x7f as size_t,
    );
    SpcPlayer_CopyVariablesFromRam(p);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        (*p).channel[i as usize].index = i as uint8;
        i += 1;
    }
    SetupEchoParameter_EDL(p, 1 as uint8);
    (*p).reg_FLG = ((*p).reg_FLG as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as uint8;
    Dsp_Write(p, MVOLL as ::core::ffi::c_int as uint8_t, 0x60 as uint8);
    Dsp_Write(p, MVOLR as ::core::ffi::c_int as uint8_t, 0x60 as uint8);
    Dsp_Write(p, DIR as ::core::ffi::c_int as uint8_t, 0x6d as uint8);
    *(&raw mut (*p).tempo as *mut uint8).offset(
        (::core::mem::size_of::<uint16>() as usize)
            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
            .wrapping_sub(1 as usize) as isize,
    ) = 16 as uint8;
    (*p).timer_cycles = 0 as uint8;
}
pub unsafe fn Sfx_Chan_Reset(mut p: *mut SpcPlayer, mut sr: *mut SfxRegs, mut s: *mut SfxChan) {
    (*s).disable = 0xff as uint8;
    (*s).update_adsr = 0 as uint8;
    (*sr).enabled_voices = ((*sr).enabled_voices as ::core::ffi::c_int
        & (*s).channel_mask as ::core::ffi::c_int) as uint8;
    (*p).is_chan_on =
        ((*p).is_chan_on as ::core::ffi::c_int & (*s).channel_mask as ::core::ffi::c_int) as uint8;
    (*p).cur_chan_bit = ((*p).cur_chan_bit as ::core::ffi::c_int
        | (*s).voice_bitset as ::core::ffi::c_int) as uint8;
    (*p).key_OFF =
        ((*p).key_OFF as ::core::ffi::c_int | (*s).voice_bitset as ::core::ffi::c_int) as uint8;
    let mut c: *mut Channel = (&raw mut (*p).channel as *mut Channel)
        .offset(((*s).voice_index as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as *mut Channel;
    Channel_SetInstrument_NoSetId(p, c, (*c).instrument_id);
    (*c).final_volume = (*s).volume;
    (*c).pan_flag_with_phase_invert = (*s).phase_invert;
    if (*sr).enabled_voices == 0 {
        (*sr).cur_sound = 0 as uint8;
        (*sr).priority = 0 as uint8;
        (*sr).init_flag = 0 as uint8;
    }
}
static kSfx1InstrListPtrs: [uint16; 66] = [
    0x2b71 as ::core::ffi::c_int as uint16,
    0x2baf as ::core::ffi::c_int as uint16,
    0x2bb7 as ::core::ffi::c_int as uint16,
    0x2bc4 as ::core::ffi::c_int as uint16,
    0x2bd1 as ::core::ffi::c_int as uint16,
    0x2bfc as ::core::ffi::c_int as uint16,
    0x2c2f as ::core::ffi::c_int as uint16,
    0x2c37 as ::core::ffi::c_int as uint16,
    0x2cff as ::core::ffi::c_int as uint16,
    0x2d12 as ::core::ffi::c_int as uint16,
    0x2d1a as ::core::ffi::c_int as uint16,
    0x2d27 as ::core::ffi::c_int as uint16,
    0x2d4b as ::core::ffi::c_int as uint16,
    0x2d5d as ::core::ffi::c_int as uint16,
    0x2d5f as ::core::ffi::c_int as uint16,
    0x2d76 as ::core::ffi::c_int as uint16,
    0x2d95 as ::core::ffi::c_int as uint16,
    0x2d97 as ::core::ffi::c_int as uint16,
    0x2d99 as ::core::ffi::c_int as uint16,
    0x2da6 as ::core::ffi::c_int as uint16,
    0x2da8 as ::core::ffi::c_int as uint16,
    0x2daa as ::core::ffi::c_int as uint16,
    0x2dac as ::core::ffi::c_int as uint16,
    0x2dc8 as ::core::ffi::c_int as uint16,
    0x2de7 as ::core::ffi::c_int as uint16,
    0x2dfe as ::core::ffi::c_int as uint16,
    0x2e00 as ::core::ffi::c_int as uint16,
    0x2e17 as ::core::ffi::c_int as uint16,
    0x2e19 as ::core::ffi::c_int as uint16,
    0x2e1b as ::core::ffi::c_int as uint16,
    0x2e1d as ::core::ffi::c_int as uint16,
    0x2e34 as ::core::ffi::c_int as uint16,
    0x2e36 as ::core::ffi::c_int as uint16,
    0x2e38 as ::core::ffi::c_int as uint16,
    0x2e3a as ::core::ffi::c_int as uint16,
    0x2e68 as ::core::ffi::c_int as uint16,
    0x2eaf as ::core::ffi::c_int as uint16,
    0x2ed0 as ::core::ffi::c_int as uint16,
    0x2ef6 as ::core::ffi::c_int as uint16,
    0x2f2c as ::core::ffi::c_int as uint16,
    0x2f37 as ::core::ffi::c_int as uint16,
    0x2f3f as ::core::ffi::c_int as uint16,
    0x2f47 as ::core::ffi::c_int as uint16,
    0x2f4a as ::core::ffi::c_int as uint16,
    0x2f4d as ::core::ffi::c_int as uint16,
    0x2f50 as ::core::ffi::c_int as uint16,
    0x2fb0 as ::core::ffi::c_int as uint16,
    0x2fb8 as ::core::ffi::c_int as uint16,
    0x2fc3 as ::core::ffi::c_int as uint16,
    0x2fdd as ::core::ffi::c_int as uint16,
    0x2fe5 as ::core::ffi::c_int as uint16,
    0x3040 as ::core::ffi::c_int as uint16,
    0x3048 as ::core::ffi::c_int as uint16,
    0x3055 as ::core::ffi::c_int as uint16,
    0x305d as ::core::ffi::c_int as uint16,
    0x3065 as ::core::ffi::c_int as uint16,
    0x3070 as ::core::ffi::c_int as uint16,
    0x3078 as ::core::ffi::c_int as uint16,
    0x307b as ::core::ffi::c_int as uint16,
    0x308b as ::core::ffi::c_int as uint16,
    0x309b as ::core::ffi::c_int as uint16,
    0x30a8 as ::core::ffi::c_int as uint16,
    0x30d6 as ::core::ffi::c_int as uint16,
    0x30e1 as ::core::ffi::c_int as uint16,
    0x312a as ::core::ffi::c_int as uint16,
    0x313f as ::core::ffi::c_int as uint16,
];
static kSfx1Conf: [uint8; 66] = [
    5 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
];
pub unsafe fn Sfx1_HandleCmdFromSnes(mut p: *mut SpcPlayer) {
    let mut a: uint8 = (*p).input_ports[1 as ::core::ffi::c_int as usize];
    (*p).input_ports[1 as ::core::ffi::c_int as usize] = 255 as uint8;
    if a as ::core::ffi::c_int == 255 as ::core::ffi::c_int
        || a as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || a as ::core::ffi::c_int != 1 as ::core::ffi::c_int
            && a as ::core::ffi::c_int != 2 as ::core::ffi::c_int
            && (*p).sfx1.priority as ::core::ffi::c_int != 0
    {
        if (*p).sfx1.cur_sound != 0 {
            Sfx1_Process(p);
        }
        return;
    }
    if (*p).sfx1.cur_sound != 0 {
        (*p).sfx1.enabled_voices = 0 as uint8;
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(0) as *mut SfxChan,
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(1) as *mut SfxChan,
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(2) as *mut SfxChan,
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(3) as *mut SfxChan,
        );
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 4 as ::core::ffi::c_int {
        (*p).sfx_chans_1[i as usize].legato = 0 as uint8;
        i += 1;
    }
    (*p).sfx1.cur_sound_index =
        ((a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) * 2 as ::core::ffi::c_int) as uint8;
    (*p).sfx1.cur_ptr = kSfx1InstrListPtrs
        [((*p).sfx1.cur_sound_index as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    (*p).sfx1.cur_sound = a;
    match kSfx1Conf[(a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
    {
        0 => {
            (*p).sfx1.voices_to_setup = 1 as uint8;
            (*p).sfx1.priority = 0 as uint8;
        }
        1 => {
            (*p).sfx1.voices_to_setup = 1 as uint8;
            (*p).sfx1.priority = 1 as uint8;
        }
        2 => {
            (*p).sfx1.voices_to_setup = 2 as uint8;
            (*p).sfx1.priority = 0 as uint8;
        }
        3 => {
            (*p).sfx1.voices_to_setup = 3 as uint8;
            (*p).sfx1.priority = 1 as uint8;
        }
        4 => {
            (*p).sfx1.voices_to_setup = 4 as uint8;
            (*p).sfx1.priority = 0 as uint8;
        }
        5 => {
            (*p).sfx1.voices_to_setup = 4 as uint8;
            (*p).sfx1.priority = 0 as uint8;
        }
        _ => {}
    };
}
static kSfx2InstrListPtrs: [uint16; 127] = [
    0x3ab1 as ::core::ffi::c_int as uint16,
    0x3ac3 as ::core::ffi::c_int as uint16,
    0x3ad5 as ::core::ffi::c_int as uint16,
    0x3af1 as ::core::ffi::c_int as uint16,
    0x3af3 as ::core::ffi::c_int as uint16,
    0x3af5 as ::core::ffi::c_int as uint16,
    0x3b0c as ::core::ffi::c_int as uint16,
    0x3b28 as ::core::ffi::c_int as uint16,
    0x3b2a as ::core::ffi::c_int as uint16,
    0x3b3a as ::core::ffi::c_int as uint16,
    0x3b42 as ::core::ffi::c_int as uint16,
    0x3b5e as ::core::ffi::c_int as uint16,
    0x3b73 as ::core::ffi::c_int as uint16,
    0x3b85 as ::core::ffi::c_int as uint16,
    0x3b92 as ::core::ffi::c_int as uint16,
    0x3ba9 as ::core::ffi::c_int as uint16,
    0x3bb4 as ::core::ffi::c_int as uint16,
    0x3bc1 as ::core::ffi::c_int as uint16,
    0x3be7 as ::core::ffi::c_int as uint16,
    0x3c08 as ::core::ffi::c_int as uint16,
    0x3c33 as ::core::ffi::c_int as uint16,
    0x3c3b as ::core::ffi::c_int as uint16,
    0x3c43 as ::core::ffi::c_int as uint16,
    0x3c56 as ::core::ffi::c_int as uint16,
    0x3c90 as ::core::ffi::c_int as uint16,
    0x3cff as ::core::ffi::c_int as uint16,
    0x3d46 as ::core::ffi::c_int as uint16,
    0x3d4e as ::core::ffi::c_int as uint16,
    0x3d65 as ::core::ffi::c_int as uint16,
    0x3d81 as ::core::ffi::c_int as uint16,
    0x3d9b as ::core::ffi::c_int as uint16,
    0x3da8 as ::core::ffi::c_int as uint16,
    0x3dbf as ::core::ffi::c_int as uint16,
    0x3dd6 as ::core::ffi::c_int as uint16,
    0x3ded as ::core::ffi::c_int as uint16,
    0x3e04 as ::core::ffi::c_int as uint16,
    0x3e20 as ::core::ffi::c_int as uint16,
    0x3e41 as ::core::ffi::c_int as uint16,
    0x3e66 as ::core::ffi::c_int as uint16,
    0x3e94 as ::core::ffi::c_int as uint16,
    0x3eb0 as ::core::ffi::c_int as uint16,
    0x3ecc as ::core::ffi::c_int as uint16,
    0x3ece as ::core::ffi::c_int as uint16,
    0x3f03 as ::core::ffi::c_int as uint16,
    0x3f18 as ::core::ffi::c_int as uint16,
    0x3f20 as ::core::ffi::c_int as uint16,
    0x3f44 as ::core::ffi::c_int as uint16,
    0x3f5e as ::core::ffi::c_int as uint16,
    0x3f75 as ::core::ffi::c_int as uint16,
    0x3f82 as ::core::ffi::c_int as uint16,
    0x3f8a as ::core::ffi::c_int as uint16,
    0x3f97 as ::core::ffi::c_int as uint16,
    0x3f9f as ::core::ffi::c_int as uint16,
    0x401f as ::core::ffi::c_int as uint16,
    0x4036 as ::core::ffi::c_int as uint16,
    0x4066 as ::core::ffi::c_int as uint16,
    0x407c as ::core::ffi::c_int as uint16,
    0x407e as ::core::ffi::c_int as uint16,
    0x4086 as ::core::ffi::c_int as uint16,
    0x4088 as ::core::ffi::c_int as uint16,
    0x408a as ::core::ffi::c_int as uint16,
    0x408c as ::core::ffi::c_int as uint16,
    0x409e as ::core::ffi::c_int as uint16,
    0x40a6 as ::core::ffi::c_int as uint16,
    0x40b1 as ::core::ffi::c_int as uint16,
    0x40b4 as ::core::ffi::c_int as uint16,
    0x40bc as ::core::ffi::c_int as uint16,
    0x40ce as ::core::ffi::c_int as uint16,
    0x40d1 as ::core::ffi::c_int as uint16,
    0x40de as ::core::ffi::c_int as uint16,
    0x40f5 as ::core::ffi::c_int as uint16,
    0x410c as ::core::ffi::c_int as uint16,
    0x412c as ::core::ffi::c_int as uint16,
    0x4143 as ::core::ffi::c_int as uint16,
    0x415a as ::core::ffi::c_int as uint16,
    0x4162 as ::core::ffi::c_int as uint16,
    0x416f as ::core::ffi::c_int as uint16,
    0x41a9 as ::core::ffi::c_int as uint16,
    0x41c7 as ::core::ffi::c_int as uint16,
    0x41d9 as ::core::ffi::c_int as uint16,
    0x41ee as ::core::ffi::c_int as uint16,
    0x421c as ::core::ffi::c_int as uint16,
    0x4224 as ::core::ffi::c_int as uint16,
    0x4236 as ::core::ffi::c_int as uint16,
    0x424d as ::core::ffi::c_int as uint16,
    0x4268 as ::core::ffi::c_int as uint16,
    0x4288 as ::core::ffi::c_int as uint16,
    0x429a as ::core::ffi::c_int as uint16,
    0x42af as ::core::ffi::c_int as uint16,
    0x42bf as ::core::ffi::c_int as uint16,
    0x42d4 as ::core::ffi::c_int as uint16,
    0x42d6 as ::core::ffi::c_int as uint16,
    0x42fe as ::core::ffi::c_int as uint16,
    0x4310 as ::core::ffi::c_int as uint16,
    0x4322 as ::core::ffi::c_int as uint16,
    0x4334 as ::core::ffi::c_int as uint16,
    0x433c as ::core::ffi::c_int as uint16,
    0x4347 as ::core::ffi::c_int as uint16,
    0x4352 as ::core::ffi::c_int as uint16,
    0x43c1 as ::core::ffi::c_int as uint16,
    0x43cc as ::core::ffi::c_int as uint16,
    0x43e8 as ::core::ffi::c_int as uint16,
    0x43fa as ::core::ffi::c_int as uint16,
    0x4422 as ::core::ffi::c_int as uint16,
    0x442f as ::core::ffi::c_int as uint16,
    0x4441 as ::core::ffi::c_int as uint16,
    0x4443 as ::core::ffi::c_int as uint16,
    0x4446 as ::core::ffi::c_int as uint16,
    0x444e as ::core::ffi::c_int as uint16,
    0x446f as ::core::ffi::c_int as uint16,
    0x447f as ::core::ffi::c_int as uint16,
    0x448f as ::core::ffi::c_int as uint16,
    0x449c as ::core::ffi::c_int as uint16,
    0x44a4 as ::core::ffi::c_int as uint16,
    0x44b9 as ::core::ffi::c_int as uint16,
    0x44ce as ::core::ffi::c_int as uint16,
    0x44ed as ::core::ffi::c_int as uint16,
    0x4598 as ::core::ffi::c_int as uint16,
    0x459a as ::core::ffi::c_int as uint16,
    0x4618 as ::core::ffi::c_int as uint16,
    0x462d as ::core::ffi::c_int as uint16,
    0x4642 as ::core::ffi::c_int as uint16,
    0x4657 as ::core::ffi::c_int as uint16,
    0x466c as ::core::ffi::c_int as uint16,
    0x4679 as ::core::ffi::c_int as uint16,
    0x4686 as ::core::ffi::c_int as uint16,
    0x468e as ::core::ffi::c_int as uint16,
];
static kSfx2Conf: [uint8; 127] = [
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    0 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
];
pub unsafe fn Sfx2_HandleCmdFromSnes(mut p: *mut SpcPlayer) {
    let mut a: uint8 = (*p).input_ports[2 as ::core::ffi::c_int as usize];
    (*p).input_ports[2 as ::core::ffi::c_int as usize] = 255 as uint8;
    if a as ::core::ffi::c_int == 255 as ::core::ffi::c_int
        || a as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || a as ::core::ffi::c_int != 0x71 as ::core::ffi::c_int
            && a as ::core::ffi::c_int != 0x7e as ::core::ffi::c_int
            && (*p).sfx2.priority as ::core::ffi::c_int != 0
    {
        if (*p).sfx2.cur_sound != 0 {
            Sfx2_Process(p);
        }
        return;
    }
    if (*p).sfx2.cur_sound != 0 {
        (*p).sfx2.enabled_voices = 0 as uint8;
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx2,
            (&raw mut (*p).sfx_chans_2 as *mut SfxChan).offset(0) as *mut SfxChan,
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx2,
            (&raw mut (*p).sfx_chans_2 as *mut SfxChan).offset(1) as *mut SfxChan,
        );
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 2 as ::core::ffi::c_int {
        (*p).sfx_chans_2[i as usize].legato = 0 as uint8;
        i += 1;
    }
    (*p).sfx2.cur_sound_index =
        ((a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) * 2 as ::core::ffi::c_int) as uint8;
    (*p).sfx2.cur_ptr = kSfx2InstrListPtrs
        [((*p).sfx2.cur_sound_index as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    (*p).sfx2.cur_sound = a;
    match kSfx2Conf[(a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
    {
        0 => {
            (*p).sfx2.voices_to_setup = 1 as uint8;
            (*p).sfx2.priority = 0 as uint8;
        }
        1 => {
            (*p).sfx2.voices_to_setup = 1 as uint8;
            (*p).sfx2.priority = 1 as uint8;
        }
        2 => {
            (*p).sfx2.voices_to_setup = 2 as uint8;
            (*p).sfx2.priority = 0 as uint8;
        }
        3 => {
            (*p).sfx2.voices_to_setup = 2 as uint8;
            (*p).sfx2.priority = 1 as uint8;
        }
        _ => {}
    };
}
static kSfx3InstrListPtrs: [uint16; 47] = [
    0x4eed as ::core::ffi::c_int as uint16,
    0x4ef5 as ::core::ffi::c_int as uint16,
    0x4f00 as ::core::ffi::c_int as uint16,
    0x4f4b as ::core::ffi::c_int as uint16,
    0x4f5b as ::core::ffi::c_int as uint16,
    0x4f6b as ::core::ffi::c_int as uint16,
    0x4f73 as ::core::ffi::c_int as uint16,
    0x4f89 as ::core::ffi::c_int as uint16,
    0x4f9f as ::core::ffi::c_int as uint16,
    0x4fb6 as ::core::ffi::c_int as uint16,
    0x4fe1 as ::core::ffi::c_int as uint16,
    0x4ff7 as ::core::ffi::c_int as uint16,
    0x4fff as ::core::ffi::c_int as uint16,
    0x500c as ::core::ffi::c_int as uint16,
    0x5047 as ::core::ffi::c_int as uint16,
    0x50a8 as ::core::ffi::c_int as uint16,
    0x50c4 as ::core::ffi::c_int as uint16,
    0x50d1 as ::core::ffi::c_int as uint16,
    0x50d4 as ::core::ffi::c_int as uint16,
    0x50f0 as ::core::ffi::c_int as uint16,
    0x511a as ::core::ffi::c_int as uint16,
    0x5130 as ::core::ffi::c_int as uint16,
    0x5142 as ::core::ffi::c_int as uint16,
    0x5175 as ::core::ffi::c_int as uint16,
    0x5178 as ::core::ffi::c_int as uint16,
    0x5188 as ::core::ffi::c_int as uint16,
    0x518b as ::core::ffi::c_int as uint16,
    0x51b9 as ::core::ffi::c_int as uint16,
    0x51c1 as ::core::ffi::c_int as uint16,
    0x51d4 as ::core::ffi::c_int as uint16,
    0x51f0 as ::core::ffi::c_int as uint16,
    0x51fd as ::core::ffi::c_int as uint16,
    0x5200 as ::core::ffi::c_int as uint16,
    0x5208 as ::core::ffi::c_int as uint16,
    0x5229 as ::core::ffi::c_int as uint16,
    0x5231 as ::core::ffi::c_int as uint16,
    0x5239 as ::core::ffi::c_int as uint16,
    0x5241 as ::core::ffi::c_int as uint16,
    0x524e as ::core::ffi::c_int as uint16,
    0x5256 as ::core::ffi::c_int as uint16,
    0x5277 as ::core::ffi::c_int as uint16,
    0x5293 as ::core::ffi::c_int as uint16,
    0x52a5 as ::core::ffi::c_int as uint16,
    0x52b0 as ::core::ffi::c_int as uint16,
    0x52c6 as ::core::ffi::c_int as uint16,
    0x52f1 as ::core::ffi::c_int as uint16,
    0x530b as ::core::ffi::c_int as uint16,
];
static kSfx3Conf: [uint8; 47] = [
    0 as ::core::ffi::c_int as uint8,
    1 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    3 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    5 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
    4 as ::core::ffi::c_int as uint8,
    2 as ::core::ffi::c_int as uint8,
];
pub unsafe fn Sfx3_HandleCmdFromSnes(mut p: *mut SpcPlayer) {
    let mut a: uint8 = (*p).input_ports[3 as ::core::ffi::c_int as usize];
    (*p).input_ports[3 as ::core::ffi::c_int as usize] = 255 as uint8;
    if a as ::core::ffi::c_int == 255 as ::core::ffi::c_int
        || a as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || a as ::core::ffi::c_int != 1 as ::core::ffi::c_int
            && ((*p).sfx3.some_value as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                || a as ::core::ffi::c_int != 2 as ::core::ffi::c_int
                    && (*p).sfx3.priority as ::core::ffi::c_int != 0)
    {
        if (*p).sfx3.cur_sound != 0 {
            Sfx3_Process(p);
        }
        return;
    }
    if (*p).sfx3.cur_sound != 0 {
        (*p).sfx3.enabled_voices = 0 as uint8;
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx3,
            (&raw mut (*p).sfx_chans_3 as *mut SfxChan).offset(0) as *mut SfxChan,
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx3,
            (&raw mut (*p).sfx_chans_3 as *mut SfxChan).offset(1) as *mut SfxChan,
        );
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 2 as ::core::ffi::c_int {
        (*p).sfx_chans_3[i as usize].legato = 0 as uint8;
        i += 1;
    }
    (*p).sfx3.cur_sound_index =
        ((a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) * 2 as ::core::ffi::c_int) as uint8;
    (*p).sfx3.cur_ptr = kSfx3InstrListPtrs
        [((*p).sfx3.cur_sound_index as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
    (*p).sfx3.cur_sound = a;
    if (*p).sfx3.cur_sound as ::core::ffi::c_int >= 0xfe as ::core::ffi::c_int {
        Sfx3_Process(p);
        return;
    }
    match kSfx3Conf[(a as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int
    {
        0 => {
            (*p).sfx3.voices_to_setup = 1 as uint8;
            (*p).sfx3.some_value = 0 as uint8;
            (*p).sfx3.priority = 1 as uint8;
        }
        1 => {
            (*p).sfx3.voices_to_setup = 1 as uint8;
            (*p).sfx3.some_value = 2 as uint8;
        }
        2 => {
            (*p).sfx3.voices_to_setup = 1 as uint8;
            (*p).sfx3.priority = 0 as uint8;
        }
        3 => {
            (*p).sfx3.voices_to_setup = 2 as uint8;
            (*p).sfx3.priority = 0 as uint8;
        }
        4 => {
            (*p).sfx3.voices_to_setup = 2 as uint8;
            (*p).sfx3.priority = 1 as uint8;
        }
        5 => {
            (*p).sfx3.voices_to_setup = 1 as uint8;
            (*p).sfx3.priority = 1 as uint8;
        }
        _ => {}
    };
}
pub unsafe fn SfxChan_GetByte(mut p: *mut SpcPlayer, mut s: *mut SfxChan) -> uint8 {
    let fresh31 = (*s).ptr_idx;
    (*s).ptr_idx = (*s).ptr_idx.wrapping_add(1);
    return (*p).ram[((*s).ptr as ::core::ffi::c_int + fresh31 as ::core::ffi::c_int) as usize];
}
pub unsafe fn Sfx_RunChan(mut p: *mut SpcPlayer, mut sr: *mut SfxRegs, mut s: *mut SfxChan) {
    if (*s).disable as ::core::ffi::c_int == 0xff as ::core::ffi::c_int {
        return;
    }
    (*s).instr_timer = (*s).instr_timer.wrapping_sub(1);
    if (*s).instr_timer as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if (*s).legato == 0 {
            (*s).enable_pitch_slide = 0 as uint8;
            (*s).subnote_delta = 0 as uint8;
            (*s).target_note = 0 as uint8;
            if (*s).release_flag as ::core::ffi::c_int != 0xff as ::core::ffi::c_int {
                (*p).key_OFF = ((*p).key_OFF as ::core::ffi::c_int
                    | (*s).voice_bitset as ::core::ffi::c_int)
                    as uint8;
                (*s).release_timer = 2 as uint8;
                (*s).instr_timer = 1 as uint8;
                (*s).release_flag = 0xff as uint8;
            }
            (*s).release_timer = (*s).release_timer.wrapping_sub(1);
            if (*s).release_timer != 0 {
                return;
            }
            (*s).release_flag = 0 as uint8;
            (*p).cur_chan_bit = ((*p).cur_chan_bit as ::core::ffi::c_int
                & (*s).channel_mask as ::core::ffi::c_int) as uint8;
            (*p).reg_NON = ((*p).reg_NON as ::core::ffi::c_int
                & (*s).channel_mask as ::core::ffi::c_int) as uint8;
        }
        let mut a: uint8 = 0;
        loop {
            a = SfxChan_GetByte(p, s);
            if a as ::core::ffi::c_int == 0xf9 as ::core::ffi::c_int {
                (*s).adsr_settings = SfxChan_GetByte(p, s) as uint16;
                (*s).adsr_settings = ((*s).adsr_settings as ::core::ffi::c_int
                    | (SfxChan_GetByte(p, s) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    as uint16;
                (*s).ptr_idx =
                    ((*s).ptr_idx as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8;
                (*s).update_adsr = 0xff as uint8;
            } else {
                if a as ::core::ffi::c_int == 0xf5 as ::core::ffi::c_int
                    || a as ::core::ffi::c_int == 0xf8 as ::core::ffi::c_int
                {
                    (*s).enable_pitch_slide_legato =
                        (if a as ::core::ffi::c_int == 0xf5 as ::core::ffi::c_int {
                            0xf5 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as uint8;
                    (*s).subnote_delta = SfxChan_GetByte(p, s);
                    (*s).target_note = SfxChan_GetByte(p, s);
                    (*s).enable_pitch_slide = 0xff as uint8;
                    a = SfxChan_GetByte(p, s);
                }
                if a as ::core::ffi::c_int == 0xff as ::core::ffi::c_int {
                    Sfx_Chan_Reset(p, sr, s);
                    return;
                }
                if a as ::core::ffi::c_int == 0xfe as ::core::ffi::c_int {
                    (*s).repeat_ctr = SfxChan_GetByte(p, s);
                    (*s).repeat_pt = (*s).ptr_idx;
                    a = SfxChan_GetByte(p, s);
                }
                if a as ::core::ffi::c_int == 0xfb as ::core::ffi::c_int
                    || a as ::core::ffi::c_int == 0xfd as ::core::ffi::c_int
                {
                    if a as ::core::ffi::c_int != 0xfb as ::core::ffi::c_int && {
                        (*s).repeat_ctr = (*s).repeat_ctr.wrapping_sub(1);
                        (*s).repeat_ctr as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                    } {
                        continue;
                    }
                    (*s).ptr_idx = (*s).repeat_pt;
                    a = SfxChan_GetByte(p, s);
                }
                if !(a as ::core::ffi::c_int == 0xfc as ::core::ffi::c_int) {
                    break;
                }
                (*p).reg_NON = ((*p).reg_NON as ::core::ffi::c_int
                    | (*s).voice_bitset as ::core::ffi::c_int)
                    as uint8;
            }
        }
        let mut c: *mut Channel = (&raw mut (*p).channel as *mut Channel)
            .offset(((*s).voice_index as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut Channel;
        Channel_SetInstrument_NoSetId(p, c, a);
        (*c).final_volume = SfxChan_GetByte(p, s);
        (*c).pan_flag_with_phase_invert = 0 as uint8;
        WriteVolumeToDsp(
            p,
            c,
            ((SfxChan_GetByte(p, s) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16,
        );
        a = SfxChan_GetByte(p, s);
        if a as ::core::ffi::c_int != 0xf6 as ::core::ffi::c_int {
            (*s).note = a;
            (*s).subnote = 0 as uint8;
        }
        WritePitchInner(
            p,
            c,
            (((*s).note as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
                | (*s).subnote as ::core::ffi::c_int) as uint16,
        );
        (*s).instr_timer = SfxChan_GetByte(p, s);
        if (*s).update_adsr != 0 {
            Dsp_Write(
                p,
                ((*s).voice_index_mult8 as ::core::ffi::c_int | V0ADSR1 as ::core::ffi::c_int)
                    as uint8_t,
                (*s).adsr_settings as uint8,
            );
            Dsp_Write(
                p,
                ((*s).voice_index_mult8 as ::core::ffi::c_int | V0ADSR2 as ::core::ffi::c_int)
                    as uint8_t,
                ((*s).adsr_settings as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8,
            );
        }
        if (*s).legato == 0 {
            (*p).key_ON = ((*p).key_ON as ::core::ffi::c_int
                | (*s).voice_bitset as ::core::ffi::c_int) as uint8;
        }
    }
    if (*s).enable_pitch_slide as ::core::ffi::c_int != 0xff as ::core::ffi::c_int {
        return;
    }
    if (*s).enable_pitch_slide_legato != 0 {
        (*s).legato = 0xff as uint8;
    }
    if (*s).note as ::core::ffi::c_int >= (*s).target_note as ::core::ffi::c_int {
        let mut t: ::core::ffi::c_int =
            (*s).subnote as ::core::ffi::c_int - (*s).subnote_delta as ::core::ffi::c_int;
        (*s).subnote = t as uint8;
        if t & 256 as ::core::ffi::c_int != 0 && {
            (*s).note = (*s).note.wrapping_sub(1);
            (*s).note as ::core::ffi::c_int == (*s).target_note as ::core::ffi::c_int
        } {
            (*s).legato = 0 as uint8;
            (*s).enable_pitch_slide = (*s).legato;
        }
    } else {
        let mut t_0: ::core::ffi::c_int =
            (*s).subnote as ::core::ffi::c_int + (*s).subnote_delta as ::core::ffi::c_int;
        (*s).subnote = t_0 as uint8;
        if t_0 & 256 as ::core::ffi::c_int != 0 && {
            (*s).note = (*s).note.wrapping_add(1);
            (*s).note as ::core::ffi::c_int == (*s).target_note as ::core::ffi::c_int
        } {
            (*s).legato = 0 as uint8;
            (*s).enable_pitch_slide = (*s).legato;
        }
    }
    WritePitchInner(
        p,
        (&raw mut (*p).channel as *mut Channel)
            .offset(((*s).voice_index as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as *mut Channel,
        (((*s).note as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | (*s).subnote as ::core::ffi::c_int) as uint16,
    );
}
pub unsafe fn Sfx1_Process(mut p: *mut SpcPlayer) {
    if (*p).sfx1.init_flag as ::core::ffi::c_int != 0xff as ::core::ffi::c_int {
        Sfx1_Init(p);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 4 as ::core::ffi::c_int {
            let mut s: *mut SfxChan =
                (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(i as isize) as *mut SfxChan;
            (*s).ptr = *((&raw mut (*p).ram as *mut uint8).offset(
                ((*p).sfx1.cur_ptr as ::core::ffi::c_int + i * 2 as ::core::ffi::c_int) as isize,
            ) as *mut uint8 as *mut uint16);
            (*s).voice_index_mult8 =
                ((*s).voice_index as ::core::ffi::c_int * 8 as ::core::ffi::c_int) as uint8;
            (*s).ptr_idx = 0 as uint8;
            (*s).instr_timer = 1 as uint8;
            i += 1;
        }
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 4 as ::core::ffi::c_int {
        Sfx_RunChan(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(i_0 as isize) as *mut SfxChan,
        );
        i_0 += 1;
    }
}
pub unsafe fn Sfx2_Process(mut p: *mut SpcPlayer) {
    if (*p).sfx2.init_flag as ::core::ffi::c_int != 0xff as ::core::ffi::c_int {
        Sfx2_Init(p);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 2 as ::core::ffi::c_int {
            let mut s: *mut SfxChan =
                (&raw mut (*p).sfx_chans_2 as *mut SfxChan).offset(i as isize) as *mut SfxChan;
            (*s).ptr = *((&raw mut (*p).ram as *mut uint8).offset(
                ((*p).sfx2.cur_ptr as ::core::ffi::c_int + i * 2 as ::core::ffi::c_int) as isize,
            ) as *mut uint8 as *mut uint16);
            (*s).voice_index_mult8 =
                ((*s).voice_index as ::core::ffi::c_int * 8 as ::core::ffi::c_int) as uint8;
            (*s).ptr_idx = 0 as uint8;
            (*s).instr_timer = 1 as uint8;
            i += 1;
        }
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 2 as ::core::ffi::c_int {
        Sfx_RunChan(
            p,
            &raw mut (*p).sfx2,
            (&raw mut (*p).sfx_chans_2 as *mut SfxChan).offset(i_0 as isize) as *mut SfxChan,
        );
        i_0 += 1;
    }
}
pub unsafe fn Sfx3_Process(mut p: *mut SpcPlayer) {
    if (*p).sfx3.init_flag as ::core::ffi::c_int != 0xff as ::core::ffi::c_int {
        Sfx3_Init(p);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 2 as ::core::ffi::c_int {
            let mut s: *mut SfxChan =
                (&raw mut (*p).sfx_chans_3 as *mut SfxChan).offset(i as isize) as *mut SfxChan;
            (*s).ptr = *((&raw mut (*p).ram as *mut uint8).offset(
                ((*p).sfx3.cur_ptr as ::core::ffi::c_int + i * 2 as ::core::ffi::c_int) as isize,
            ) as *mut uint8 as *mut uint16);
            (*s).voice_index_mult8 =
                ((*s).voice_index as ::core::ffi::c_int * 8 as ::core::ffi::c_int) as uint8;
            (*s).ptr_idx = 0 as uint8;
            (*s).instr_timer = 1 as uint8;
            i += 1;
        }
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 2 as ::core::ffi::c_int {
        Sfx_RunChan(
            p,
            &raw mut (*p).sfx3,
            (&raw mut (*p).sfx_chans_3 as *mut SfxChan).offset(i_0 as isize) as *mut SfxChan,
        );
        i_0 += 1;
    }
}
pub unsafe fn Sfx_Init(
    mut p: *mut SpcPlayer,
    mut sr: *mut SfxRegs,
    mut channels: *mut SfxChan,
    mut which_sfx: ::core::ffi::c_int,
) {
    let mut num: ::core::ffi::c_int = if which_sfx == 0 as ::core::ffi::c_int {
        4 as ::core::ffi::c_int
    } else {
        2 as ::core::ffi::c_int
    };
    static kSomeAddrs: [[uint16; 3]; 3] = [
        [
            0x3a6 as ::core::ffi::c_int as uint16,
            0x3aa as ::core::ffi::c_int as uint16,
            0x3af as ::core::ffi::c_int as uint16,
        ],
        [
            0x446 as ::core::ffi::c_int as uint16,
            0x448 as ::core::ffi::c_int as uint16,
            0x44b as ::core::ffi::c_int as uint16,
        ],
        [
            0x47e as ::core::ffi::c_int as uint16,
            0x480 as ::core::ffi::c_int as uint16,
            0x483 as ::core::ffi::c_int as uint16,
        ],
    ];
    (*sr).voice_id = 9 as uint8;
    (*sr).voices_remaining = (*p).is_chan_on;
    (*sr).init_flag = 0xff as uint8;
    (*sr).chan_idx_x2 = 0 as uint8;
    (*sr).chan_idx = (*sr).chan_idx_x2;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num {
        (*channels.offset(i as isize)).voice_bitset = 0 as uint8;
        (*channels.offset(i as isize)).voice_index = 0 as uint8;
        (*channels.offset(i as isize)).channel_mask = 0xff as uint8;
        (*channels.offset(i as isize)).disable = 0xff as uint8;
        i += 1;
    }
    loop {
        (*sr).voice_id = (*sr).voice_id.wrapping_sub(1);
        if !((*sr).voice_id != 0) {
            break;
        }
        let mut t: ::core::ffi::c_int = (*sr).voices_remaining as ::core::ffi::c_int;
        (*sr).voices_remaining =
            (((*sr).voices_remaining as ::core::ffi::c_int) << 1 as ::core::ffi::c_int) as uint8;
        if t & 0x80 as ::core::ffi::c_int == 0 {
            if (*sr).voices_to_setup == 0 {
                return;
            }
            (*sr).voices_to_setup = (*sr).voices_to_setup.wrapping_sub(1);
            let mut c: *mut SfxChan = channels.offset((*sr).chan_idx as isize) as *mut SfxChan;
            (*c).disable = 0 as uint8;
            (*sr).chan_voice_bitset_ptr =
                (kSomeAddrs[which_sfx as usize][0 as ::core::ffi::c_int as usize]
                    as ::core::ffi::c_int
                    + (*sr).chan_idx as ::core::ffi::c_int) as uint16;
            (*sr).chan_voice_mask_ptr =
                (kSomeAddrs[which_sfx as usize][1 as ::core::ffi::c_int as usize]
                    as ::core::ffi::c_int
                    + (*sr).chan_idx as ::core::ffi::c_int) as uint16;
            (*sr).chan_voice_index_ptr =
                (kSomeAddrs[which_sfx as usize][2 as ::core::ffi::c_int as usize]
                    as ::core::ffi::c_int
                    + (*sr).chan_idx as ::core::ffi::c_int) as uint16;
            (*sr).chan_idx =
                ((*sr).chan_idx as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8;
            (*sr).chan_idx_x2 =
                ((*sr).chan_idx_x2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8;
            (*sr).voice_index = (((*sr).voice_id as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * 2 as ::core::ffi::c_int) as uint8;
            (*c).voice_index = (*sr).voice_index;
            let mut t_0: ::core::ffi::c_int =
                (*sr).voice_id as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            (*c).volume = ((*p).channel[t_0 as usize].final_volume as ::core::ffi::c_int
                | ((*p).channel[t_0 as usize].pan_flag_with_phase_invert as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int) as uint8;
            let mut m: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << t_0;
            (*p).is_chan_on = ((*p).is_chan_on as ::core::ffi::c_int | m) as uint8;
            (*p).cur_chan_bit = ((*p).cur_chan_bit as ::core::ffi::c_int & !m) as uint8;
            (*p).reg_EON = ((*p).reg_EON as ::core::ffi::c_int & !m) as uint8;
            (*sr).enabled_voices = ((*sr).enabled_voices as ::core::ffi::c_int | m) as uint8;
            (*c).voice_bitset = m as uint8;
            (*c).channel_mask = !m as uint8;
        }
    }
}
pub unsafe fn Sfx1_Init(mut p: *mut SpcPlayer) {
    Sfx_Init(
        p,
        &raw mut (*p).sfx1,
        &raw mut (*p).sfx_chans_1 as *mut SfxChan,
        0 as ::core::ffi::c_int,
    );
}
pub unsafe fn Sfx2_Init(mut p: *mut SpcPlayer) {
    Sfx_Init(
        p,
        &raw mut (*p).sfx2,
        &raw mut (*p).sfx_chans_2 as *mut SfxChan,
        1 as ::core::ffi::c_int,
    );
}
pub unsafe fn Sfx3_Init(mut p: *mut SpcPlayer) {
    Sfx_Init(
        p,
        &raw mut (*p).sfx3,
        &raw mut (*p).sfx_chans_3 as *mut SfxChan,
        2 as ::core::ffi::c_int,
    );
}
pub unsafe fn SpcPlayer_Create() -> *mut SpcPlayer {
    let mut p: *mut SpcPlayer =
        malloc(::core::mem::size_of::<SpcPlayer>() as size_t) as *mut SpcPlayer;
    memset(
        p as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<SpcPlayer>() as size_t,
    );
    (*p).dsp = dsp_init(&raw mut (*p).ram as *mut uint8_t);
    (*p).reg_write_history = ::core::ptr::null_mut::<DspRegWriteHistory>();
    return p;
}
pub unsafe fn SpcPlayer_Initialize(mut p: *mut SpcPlayer) {
    Vector_Reset_Spc(p);
    Spc_Loop_Part1(p);
}
pub unsafe fn SpcPlayer_CopyVariablesToRam(mut p: *mut SpcPlayer) {
    let mut c: *mut Channel = &raw mut (*p).channel as *mut Channel;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        let mut m: *const MemMap =
            (&raw const kChannel_Maps as *const MemMap).offset(0) as *const MemMap;
        while m
            != (&raw const kChannel_Maps as *const MemMap).offset(
                (::core::mem::size_of::<[MemMap; 47]>() as usize)
                    .wrapping_div(::core::mem::size_of::<MemMap>() as usize)
                    as isize,
            ) as *const MemMap
        {
            memcpy(
                (&raw mut (*p).ram as *mut uint8).offset(
                    (((*m).org_off as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        + i * 2 as ::core::ffi::c_int) as isize,
                ) as *mut uint8 as *mut ::core::ffi::c_void,
                (c as *mut uint8).offset((*m).off as ::core::ffi::c_int as isize)
                    as *const ::core::ffi::c_void,
                (if (*m).org_off as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as size_t,
            );
            m = m.offset(1);
        }
        i += 1;
        c = c.offset(1);
    }
    let mut m_0: *const MemMapSized =
        (&raw const kSpcPlayer_Maps as *const MemMapSized).offset(0) as *const MemMapSized;
    while m_0
        != (&raw const kSpcPlayer_Maps as *const MemMapSized).offset(
            (::core::mem::size_of::<[MemMapSized; 283]>() as usize)
                .wrapping_div(::core::mem::size_of::<MemMapSized>() as usize) as isize,
        ) as *const MemMapSized
    {
        memcpy(
            (&raw mut (*p).ram as *mut uint8).offset((*m_0).org_off as isize) as *mut uint8
                as *mut ::core::ffi::c_void,
            (p as *mut uint8).offset((*m_0).off as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            (*m_0).size as size_t,
        );
        m_0 = m_0.offset(1);
    }
}
pub unsafe fn SpcPlayer_CopyVariablesFromRam(mut p: *mut SpcPlayer) {
    let mut c: *mut Channel = &raw mut (*p).channel as *mut Channel;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        let mut m: *const MemMap =
            (&raw const kChannel_Maps as *const MemMap).offset(0) as *const MemMap;
        while m
            != (&raw const kChannel_Maps as *const MemMap).offset(
                (::core::mem::size_of::<[MemMap; 47]>() as usize)
                    .wrapping_div(::core::mem::size_of::<MemMap>() as usize)
                    as isize,
            ) as *const MemMap
        {
            memcpy(
                (c as *mut uint8).offset((*m).off as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                (&raw mut (*p).ram as *mut uint8).offset(
                    (((*m).org_off as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        + i * 2 as ::core::ffi::c_int) as isize,
                ) as *mut uint8 as *const ::core::ffi::c_void,
                (if (*m).org_off as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as size_t,
            );
            m = m.offset(1);
        }
        i += 1;
        c = c.offset(1);
    }
    let mut m_0: *const MemMapSized =
        (&raw const kSpcPlayer_Maps as *const MemMapSized).offset(0) as *const MemMapSized;
    while m_0
        != (&raw const kSpcPlayer_Maps as *const MemMapSized).offset(
            (::core::mem::size_of::<[MemMapSized; 283]>() as usize)
                .wrapping_div(::core::mem::size_of::<MemMapSized>() as usize) as isize,
        ) as *const MemMapSized
    {
        memcpy(
            (p as *mut uint8).offset((*m_0).off as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            (&raw mut (*p).ram as *mut uint8).offset((*m_0).org_off as isize) as *mut uint8
                as *const ::core::ffi::c_void,
            (*m_0).size as size_t,
        );
        m_0 = m_0.offset(1);
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 8 as ::core::ffi::c_int {
        (*p).channel[i_0 as usize].index = i_0 as uint8;
        i_0 += 1;
    }
}
pub unsafe fn SpcPlayer_GenerateSamples(mut p: *mut SpcPlayer) {
    '_c2rust_label: {
        if (*p).timer_cycles as ::core::ffi::c_int <= 64 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"p->timer_cycles <= 64\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/spc_player.c\0" as *const u8 as *const ::core::ffi::c_char,
                1550 as ::core::ffi::c_uint,
                b"void SpcPlayer_GenerateSamples(SpcPlayer *)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if (*(*p).dsp).sampleOffset as ::core::ffi::c_int <= 534 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"p->dsp->sampleOffset <= 534\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/spc_player.c\0" as *const u8 as *const ::core::ffi::c_char,
                1552 as ::core::ffi::c_uint,
                b"void SpcPlayer_GenerateSamples(SpcPlayer *)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    };
    loop {
        if (*p).timer_cycles as ::core::ffi::c_int >= 64 as ::core::ffi::c_int {
            Spc_Loop_Part2(
                p,
                ((*p).timer_cycles as ::core::ffi::c_int >> 6 as ::core::ffi::c_int) as uint8,
            );
            Spc_Loop_Part1(p);
            (*p).timer_cycles =
                ((*p).timer_cycles as ::core::ffi::c_int & 63 as ::core::ffi::c_int) as uint8;
        }
        let mut n: ::core::ffi::c_int =
            534 as ::core::ffi::c_int - (*(*p).dsp).sampleOffset as ::core::ffi::c_int;
        if n > 64 as ::core::ffi::c_int - (*p).timer_cycles as ::core::ffi::c_int {
            n = 64 as ::core::ffi::c_int - (*p).timer_cycles as ::core::ffi::c_int;
        }
        (*p).timer_cycles = ((*p).timer_cycles as ::core::ffi::c_int + n) as uint8;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < n {
            dsp_cycle((*p).dsp);
            i += 1;
        }
        if (*(*p).dsp).sampleOffset as ::core::ffi::c_int == 534 as ::core::ffi::c_int {
            break;
        }
    }
}
pub unsafe fn SpcPlayer_Upload(mut p: *mut SpcPlayer, mut data: *const uint8_t) {
    Dsp_Write(p, EVOLL as ::core::ffi::c_int as uint8_t, 0 as uint8);
    Dsp_Write(p, EVOLR as ::core::ffi::c_int as uint8_t, 0 as uint8);
    Dsp_Write(p, KOF as ::core::ffi::c_int as uint8_t, 0xff as uint8);
    loop {
        let mut numbytes: ::core::ffi::c_int = *(data as *mut uint16) as ::core::ffi::c_int;
        if numbytes == 0 as ::core::ffi::c_int {
            break;
        }
        let mut target: ::core::ffi::c_int = *(data.offset(2) as *mut uint16) as ::core::ffi::c_int;
        data = data.offset(4);
        loop {
            let fresh32 = data;
            data = data.offset(1);
            let fresh33 = target;
            target = target + 1;
            (*p).ram[(fresh33 & 0xffff as ::core::ffi::c_int) as usize] = *fresh32 as uint8;
            numbytes -= 1;
            if !(numbytes != 0) {
                break;
            }
        }
    }
    (*p).port_to_snes[0 as ::core::ffi::c_int as usize] = 0 as uint8;
    (*p).input_ports[3 as ::core::ffi::c_int as usize] = 255 as uint8;
    (*p).input_ports[2 as ::core::ffi::c_int as usize] =
        (*p).input_ports[3 as ::core::ffi::c_int as usize];
    (*p).input_ports[1 as ::core::ffi::c_int as usize] =
        (*p).input_ports[2 as ::core::ffi::c_int as usize];
    (*p).input_ports[0 as ::core::ffi::c_int as usize] =
        (*p).input_ports[1 as ::core::ffi::c_int as usize];
    (*p).music_ptr_toplevel =
        *((&raw mut (*p).ram as *mut uint8).offset(0x581e) as *mut uint8 as *mut uint16);
    (*p).counter_sf0c = 2 as uint16;
    (*p).key_OFF =
        ((*p).key_OFF as ::core::ffi::c_int | !((*p).is_chan_on as ::core::ffi::c_int)) as uint8;
}
