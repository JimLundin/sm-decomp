extern "C" {
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
}
pub use crate::types::{
    __int16_t, __int8_t, __uint16_t, __uint8_t, int16_t, int8_t, size_t, uint16_t, uint8_t, Dsp,
    DspChannel, SaveLoadFunc,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut rateValues: [::core::ffi::c_int; 32] = [
    0 as ::core::ffi::c_int,
    2048 as ::core::ffi::c_int,
    1536 as ::core::ffi::c_int,
    1280 as ::core::ffi::c_int,
    1024 as ::core::ffi::c_int,
    768 as ::core::ffi::c_int,
    640 as ::core::ffi::c_int,
    512 as ::core::ffi::c_int,
    384 as ::core::ffi::c_int,
    320 as ::core::ffi::c_int,
    256 as ::core::ffi::c_int,
    192 as ::core::ffi::c_int,
    160 as ::core::ffi::c_int,
    128 as ::core::ffi::c_int,
    96 as ::core::ffi::c_int,
    80 as ::core::ffi::c_int,
    64 as ::core::ffi::c_int,
    48 as ::core::ffi::c_int,
    40 as ::core::ffi::c_int,
    32 as ::core::ffi::c_int,
    24 as ::core::ffi::c_int,
    20 as ::core::ffi::c_int,
    16 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    10 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    1 as ::core::ffi::c_int,
];
static mut gaussValues: [uint16_t; 512] = [
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x1 as ::core::ffi::c_int as uint16_t,
    0x2 as ::core::ffi::c_int as uint16_t,
    0x2 as ::core::ffi::c_int as uint16_t,
    0x2 as ::core::ffi::c_int as uint16_t,
    0x2 as ::core::ffi::c_int as uint16_t,
    0x2 as ::core::ffi::c_int as uint16_t,
    0x2 as ::core::ffi::c_int as uint16_t,
    0x2 as ::core::ffi::c_int as uint16_t,
    0x3 as ::core::ffi::c_int as uint16_t,
    0x3 as ::core::ffi::c_int as uint16_t,
    0x3 as ::core::ffi::c_int as uint16_t,
    0x3 as ::core::ffi::c_int as uint16_t,
    0x3 as ::core::ffi::c_int as uint16_t,
    0x4 as ::core::ffi::c_int as uint16_t,
    0x4 as ::core::ffi::c_int as uint16_t,
    0x4 as ::core::ffi::c_int as uint16_t,
    0x4 as ::core::ffi::c_int as uint16_t,
    0x4 as ::core::ffi::c_int as uint16_t,
    0x5 as ::core::ffi::c_int as uint16_t,
    0x5 as ::core::ffi::c_int as uint16_t,
    0x5 as ::core::ffi::c_int as uint16_t,
    0x5 as ::core::ffi::c_int as uint16_t,
    0x6 as ::core::ffi::c_int as uint16_t,
    0x6 as ::core::ffi::c_int as uint16_t,
    0x6 as ::core::ffi::c_int as uint16_t,
    0x6 as ::core::ffi::c_int as uint16_t,
    0x7 as ::core::ffi::c_int as uint16_t,
    0x7 as ::core::ffi::c_int as uint16_t,
    0x7 as ::core::ffi::c_int as uint16_t,
    0x8 as ::core::ffi::c_int as uint16_t,
    0x8 as ::core::ffi::c_int as uint16_t,
    0x8 as ::core::ffi::c_int as uint16_t,
    0x9 as ::core::ffi::c_int as uint16_t,
    0x9 as ::core::ffi::c_int as uint16_t,
    0x9 as ::core::ffi::c_int as uint16_t,
    0xa as ::core::ffi::c_int as uint16_t,
    0xa as ::core::ffi::c_int as uint16_t,
    0xa as ::core::ffi::c_int as uint16_t,
    0xb as ::core::ffi::c_int as uint16_t,
    0xb as ::core::ffi::c_int as uint16_t,
    0xb as ::core::ffi::c_int as uint16_t,
    0xc as ::core::ffi::c_int as uint16_t,
    0xc as ::core::ffi::c_int as uint16_t,
    0xd as ::core::ffi::c_int as uint16_t,
    0xd as ::core::ffi::c_int as uint16_t,
    0xe as ::core::ffi::c_int as uint16_t,
    0xe as ::core::ffi::c_int as uint16_t,
    0xf as ::core::ffi::c_int as uint16_t,
    0xf as ::core::ffi::c_int as uint16_t,
    0xf as ::core::ffi::c_int as uint16_t,
    0x10 as ::core::ffi::c_int as uint16_t,
    0x10 as ::core::ffi::c_int as uint16_t,
    0x11 as ::core::ffi::c_int as uint16_t,
    0x11 as ::core::ffi::c_int as uint16_t,
    0x12 as ::core::ffi::c_int as uint16_t,
    0x13 as ::core::ffi::c_int as uint16_t,
    0x13 as ::core::ffi::c_int as uint16_t,
    0x14 as ::core::ffi::c_int as uint16_t,
    0x14 as ::core::ffi::c_int as uint16_t,
    0x15 as ::core::ffi::c_int as uint16_t,
    0x15 as ::core::ffi::c_int as uint16_t,
    0x16 as ::core::ffi::c_int as uint16_t,
    0x17 as ::core::ffi::c_int as uint16_t,
    0x17 as ::core::ffi::c_int as uint16_t,
    0x18 as ::core::ffi::c_int as uint16_t,
    0x18 as ::core::ffi::c_int as uint16_t,
    0x19 as ::core::ffi::c_int as uint16_t,
    0x1a as ::core::ffi::c_int as uint16_t,
    0x1b as ::core::ffi::c_int as uint16_t,
    0x1b as ::core::ffi::c_int as uint16_t,
    0x1c as ::core::ffi::c_int as uint16_t,
    0x1d as ::core::ffi::c_int as uint16_t,
    0x1d as ::core::ffi::c_int as uint16_t,
    0x1e as ::core::ffi::c_int as uint16_t,
    0x1f as ::core::ffi::c_int as uint16_t,
    0x20 as ::core::ffi::c_int as uint16_t,
    0x20 as ::core::ffi::c_int as uint16_t,
    0x21 as ::core::ffi::c_int as uint16_t,
    0x22 as ::core::ffi::c_int as uint16_t,
    0x23 as ::core::ffi::c_int as uint16_t,
    0x24 as ::core::ffi::c_int as uint16_t,
    0x24 as ::core::ffi::c_int as uint16_t,
    0x25 as ::core::ffi::c_int as uint16_t,
    0x26 as ::core::ffi::c_int as uint16_t,
    0x27 as ::core::ffi::c_int as uint16_t,
    0x28 as ::core::ffi::c_int as uint16_t,
    0x29 as ::core::ffi::c_int as uint16_t,
    0x2a as ::core::ffi::c_int as uint16_t,
    0x2b as ::core::ffi::c_int as uint16_t,
    0x2c as ::core::ffi::c_int as uint16_t,
    0x2d as ::core::ffi::c_int as uint16_t,
    0x2e as ::core::ffi::c_int as uint16_t,
    0x2f as ::core::ffi::c_int as uint16_t,
    0x30 as ::core::ffi::c_int as uint16_t,
    0x31 as ::core::ffi::c_int as uint16_t,
    0x32 as ::core::ffi::c_int as uint16_t,
    0x33 as ::core::ffi::c_int as uint16_t,
    0x34 as ::core::ffi::c_int as uint16_t,
    0x35 as ::core::ffi::c_int as uint16_t,
    0x36 as ::core::ffi::c_int as uint16_t,
    0x37 as ::core::ffi::c_int as uint16_t,
    0x38 as ::core::ffi::c_int as uint16_t,
    0x3a as ::core::ffi::c_int as uint16_t,
    0x3b as ::core::ffi::c_int as uint16_t,
    0x3c as ::core::ffi::c_int as uint16_t,
    0x3d as ::core::ffi::c_int as uint16_t,
    0x3e as ::core::ffi::c_int as uint16_t,
    0x40 as ::core::ffi::c_int as uint16_t,
    0x41 as ::core::ffi::c_int as uint16_t,
    0x42 as ::core::ffi::c_int as uint16_t,
    0x43 as ::core::ffi::c_int as uint16_t,
    0x45 as ::core::ffi::c_int as uint16_t,
    0x46 as ::core::ffi::c_int as uint16_t,
    0x47 as ::core::ffi::c_int as uint16_t,
    0x49 as ::core::ffi::c_int as uint16_t,
    0x4a as ::core::ffi::c_int as uint16_t,
    0x4c as ::core::ffi::c_int as uint16_t,
    0x4d as ::core::ffi::c_int as uint16_t,
    0x4e as ::core::ffi::c_int as uint16_t,
    0x50 as ::core::ffi::c_int as uint16_t,
    0x51 as ::core::ffi::c_int as uint16_t,
    0x53 as ::core::ffi::c_int as uint16_t,
    0x54 as ::core::ffi::c_int as uint16_t,
    0x56 as ::core::ffi::c_int as uint16_t,
    0x57 as ::core::ffi::c_int as uint16_t,
    0x59 as ::core::ffi::c_int as uint16_t,
    0x5a as ::core::ffi::c_int as uint16_t,
    0x5c as ::core::ffi::c_int as uint16_t,
    0x5e as ::core::ffi::c_int as uint16_t,
    0x5f as ::core::ffi::c_int as uint16_t,
    0x61 as ::core::ffi::c_int as uint16_t,
    0x63 as ::core::ffi::c_int as uint16_t,
    0x64 as ::core::ffi::c_int as uint16_t,
    0x66 as ::core::ffi::c_int as uint16_t,
    0x68 as ::core::ffi::c_int as uint16_t,
    0x6a as ::core::ffi::c_int as uint16_t,
    0x6b as ::core::ffi::c_int as uint16_t,
    0x6d as ::core::ffi::c_int as uint16_t,
    0x6f as ::core::ffi::c_int as uint16_t,
    0x71 as ::core::ffi::c_int as uint16_t,
    0x73 as ::core::ffi::c_int as uint16_t,
    0x75 as ::core::ffi::c_int as uint16_t,
    0x76 as ::core::ffi::c_int as uint16_t,
    0x78 as ::core::ffi::c_int as uint16_t,
    0x7a as ::core::ffi::c_int as uint16_t,
    0x7c as ::core::ffi::c_int as uint16_t,
    0x7e as ::core::ffi::c_int as uint16_t,
    0x80 as ::core::ffi::c_int as uint16_t,
    0x82 as ::core::ffi::c_int as uint16_t,
    0x84 as ::core::ffi::c_int as uint16_t,
    0x86 as ::core::ffi::c_int as uint16_t,
    0x89 as ::core::ffi::c_int as uint16_t,
    0x8b as ::core::ffi::c_int as uint16_t,
    0x8d as ::core::ffi::c_int as uint16_t,
    0x8f as ::core::ffi::c_int as uint16_t,
    0x91 as ::core::ffi::c_int as uint16_t,
    0x93 as ::core::ffi::c_int as uint16_t,
    0x96 as ::core::ffi::c_int as uint16_t,
    0x98 as ::core::ffi::c_int as uint16_t,
    0x9a as ::core::ffi::c_int as uint16_t,
    0x9c as ::core::ffi::c_int as uint16_t,
    0x9f as ::core::ffi::c_int as uint16_t,
    0xa1 as ::core::ffi::c_int as uint16_t,
    0xa3 as ::core::ffi::c_int as uint16_t,
    0xa6 as ::core::ffi::c_int as uint16_t,
    0xa8 as ::core::ffi::c_int as uint16_t,
    0xab as ::core::ffi::c_int as uint16_t,
    0xad as ::core::ffi::c_int as uint16_t,
    0xaf as ::core::ffi::c_int as uint16_t,
    0xb2 as ::core::ffi::c_int as uint16_t,
    0xb4 as ::core::ffi::c_int as uint16_t,
    0xb7 as ::core::ffi::c_int as uint16_t,
    0xba as ::core::ffi::c_int as uint16_t,
    0xbc as ::core::ffi::c_int as uint16_t,
    0xbf as ::core::ffi::c_int as uint16_t,
    0xc1 as ::core::ffi::c_int as uint16_t,
    0xc4 as ::core::ffi::c_int as uint16_t,
    0xc7 as ::core::ffi::c_int as uint16_t,
    0xc9 as ::core::ffi::c_int as uint16_t,
    0xcc as ::core::ffi::c_int as uint16_t,
    0xcf as ::core::ffi::c_int as uint16_t,
    0xd2 as ::core::ffi::c_int as uint16_t,
    0xd4 as ::core::ffi::c_int as uint16_t,
    0xd7 as ::core::ffi::c_int as uint16_t,
    0xda as ::core::ffi::c_int as uint16_t,
    0xdd as ::core::ffi::c_int as uint16_t,
    0xe0 as ::core::ffi::c_int as uint16_t,
    0xe3 as ::core::ffi::c_int as uint16_t,
    0xe6 as ::core::ffi::c_int as uint16_t,
    0xe9 as ::core::ffi::c_int as uint16_t,
    0xec as ::core::ffi::c_int as uint16_t,
    0xef as ::core::ffi::c_int as uint16_t,
    0xf2 as ::core::ffi::c_int as uint16_t,
    0xf5 as ::core::ffi::c_int as uint16_t,
    0xf8 as ::core::ffi::c_int as uint16_t,
    0xfb as ::core::ffi::c_int as uint16_t,
    0xfe as ::core::ffi::c_int as uint16_t,
    0x101 as ::core::ffi::c_int as uint16_t,
    0x104 as ::core::ffi::c_int as uint16_t,
    0x107 as ::core::ffi::c_int as uint16_t,
    0x10b as ::core::ffi::c_int as uint16_t,
    0x10e as ::core::ffi::c_int as uint16_t,
    0x111 as ::core::ffi::c_int as uint16_t,
    0x114 as ::core::ffi::c_int as uint16_t,
    0x118 as ::core::ffi::c_int as uint16_t,
    0x11b as ::core::ffi::c_int as uint16_t,
    0x11e as ::core::ffi::c_int as uint16_t,
    0x122 as ::core::ffi::c_int as uint16_t,
    0x125 as ::core::ffi::c_int as uint16_t,
    0x129 as ::core::ffi::c_int as uint16_t,
    0x12c as ::core::ffi::c_int as uint16_t,
    0x130 as ::core::ffi::c_int as uint16_t,
    0x133 as ::core::ffi::c_int as uint16_t,
    0x137 as ::core::ffi::c_int as uint16_t,
    0x13a as ::core::ffi::c_int as uint16_t,
    0x13e as ::core::ffi::c_int as uint16_t,
    0x141 as ::core::ffi::c_int as uint16_t,
    0x145 as ::core::ffi::c_int as uint16_t,
    0x148 as ::core::ffi::c_int as uint16_t,
    0x14c as ::core::ffi::c_int as uint16_t,
    0x150 as ::core::ffi::c_int as uint16_t,
    0x153 as ::core::ffi::c_int as uint16_t,
    0x157 as ::core::ffi::c_int as uint16_t,
    0x15b as ::core::ffi::c_int as uint16_t,
    0x15f as ::core::ffi::c_int as uint16_t,
    0x162 as ::core::ffi::c_int as uint16_t,
    0x166 as ::core::ffi::c_int as uint16_t,
    0x16a as ::core::ffi::c_int as uint16_t,
    0x16e as ::core::ffi::c_int as uint16_t,
    0x172 as ::core::ffi::c_int as uint16_t,
    0x176 as ::core::ffi::c_int as uint16_t,
    0x17a as ::core::ffi::c_int as uint16_t,
    0x17d as ::core::ffi::c_int as uint16_t,
    0x181 as ::core::ffi::c_int as uint16_t,
    0x185 as ::core::ffi::c_int as uint16_t,
    0x189 as ::core::ffi::c_int as uint16_t,
    0x18d as ::core::ffi::c_int as uint16_t,
    0x191 as ::core::ffi::c_int as uint16_t,
    0x195 as ::core::ffi::c_int as uint16_t,
    0x19a as ::core::ffi::c_int as uint16_t,
    0x19e as ::core::ffi::c_int as uint16_t,
    0x1a2 as ::core::ffi::c_int as uint16_t,
    0x1a6 as ::core::ffi::c_int as uint16_t,
    0x1aa as ::core::ffi::c_int as uint16_t,
    0x1ae as ::core::ffi::c_int as uint16_t,
    0x1b2 as ::core::ffi::c_int as uint16_t,
    0x1b7 as ::core::ffi::c_int as uint16_t,
    0x1bb as ::core::ffi::c_int as uint16_t,
    0x1bf as ::core::ffi::c_int as uint16_t,
    0x1c3 as ::core::ffi::c_int as uint16_t,
    0x1c8 as ::core::ffi::c_int as uint16_t,
    0x1cc as ::core::ffi::c_int as uint16_t,
    0x1d0 as ::core::ffi::c_int as uint16_t,
    0x1d5 as ::core::ffi::c_int as uint16_t,
    0x1d9 as ::core::ffi::c_int as uint16_t,
    0x1dd as ::core::ffi::c_int as uint16_t,
    0x1e2 as ::core::ffi::c_int as uint16_t,
    0x1e6 as ::core::ffi::c_int as uint16_t,
    0x1eb as ::core::ffi::c_int as uint16_t,
    0x1ef as ::core::ffi::c_int as uint16_t,
    0x1f3 as ::core::ffi::c_int as uint16_t,
    0x1f8 as ::core::ffi::c_int as uint16_t,
    0x1fc as ::core::ffi::c_int as uint16_t,
    0x201 as ::core::ffi::c_int as uint16_t,
    0x205 as ::core::ffi::c_int as uint16_t,
    0x20a as ::core::ffi::c_int as uint16_t,
    0x20f as ::core::ffi::c_int as uint16_t,
    0x213 as ::core::ffi::c_int as uint16_t,
    0x218 as ::core::ffi::c_int as uint16_t,
    0x21c as ::core::ffi::c_int as uint16_t,
    0x221 as ::core::ffi::c_int as uint16_t,
    0x226 as ::core::ffi::c_int as uint16_t,
    0x22a as ::core::ffi::c_int as uint16_t,
    0x22f as ::core::ffi::c_int as uint16_t,
    0x233 as ::core::ffi::c_int as uint16_t,
    0x238 as ::core::ffi::c_int as uint16_t,
    0x23d as ::core::ffi::c_int as uint16_t,
    0x241 as ::core::ffi::c_int as uint16_t,
    0x246 as ::core::ffi::c_int as uint16_t,
    0x24b as ::core::ffi::c_int as uint16_t,
    0x250 as ::core::ffi::c_int as uint16_t,
    0x254 as ::core::ffi::c_int as uint16_t,
    0x259 as ::core::ffi::c_int as uint16_t,
    0x25e as ::core::ffi::c_int as uint16_t,
    0x263 as ::core::ffi::c_int as uint16_t,
    0x267 as ::core::ffi::c_int as uint16_t,
    0x26c as ::core::ffi::c_int as uint16_t,
    0x271 as ::core::ffi::c_int as uint16_t,
    0x276 as ::core::ffi::c_int as uint16_t,
    0x27b as ::core::ffi::c_int as uint16_t,
    0x280 as ::core::ffi::c_int as uint16_t,
    0x284 as ::core::ffi::c_int as uint16_t,
    0x289 as ::core::ffi::c_int as uint16_t,
    0x28e as ::core::ffi::c_int as uint16_t,
    0x293 as ::core::ffi::c_int as uint16_t,
    0x298 as ::core::ffi::c_int as uint16_t,
    0x29d as ::core::ffi::c_int as uint16_t,
    0x2a2 as ::core::ffi::c_int as uint16_t,
    0x2a6 as ::core::ffi::c_int as uint16_t,
    0x2ab as ::core::ffi::c_int as uint16_t,
    0x2b0 as ::core::ffi::c_int as uint16_t,
    0x2b5 as ::core::ffi::c_int as uint16_t,
    0x2ba as ::core::ffi::c_int as uint16_t,
    0x2bf as ::core::ffi::c_int as uint16_t,
    0x2c4 as ::core::ffi::c_int as uint16_t,
    0x2c9 as ::core::ffi::c_int as uint16_t,
    0x2ce as ::core::ffi::c_int as uint16_t,
    0x2d3 as ::core::ffi::c_int as uint16_t,
    0x2d8 as ::core::ffi::c_int as uint16_t,
    0x2dc as ::core::ffi::c_int as uint16_t,
    0x2e1 as ::core::ffi::c_int as uint16_t,
    0x2e6 as ::core::ffi::c_int as uint16_t,
    0x2eb as ::core::ffi::c_int as uint16_t,
    0x2f0 as ::core::ffi::c_int as uint16_t,
    0x2f5 as ::core::ffi::c_int as uint16_t,
    0x2fa as ::core::ffi::c_int as uint16_t,
    0x2ff as ::core::ffi::c_int as uint16_t,
    0x304 as ::core::ffi::c_int as uint16_t,
    0x309 as ::core::ffi::c_int as uint16_t,
    0x30e as ::core::ffi::c_int as uint16_t,
    0x313 as ::core::ffi::c_int as uint16_t,
    0x318 as ::core::ffi::c_int as uint16_t,
    0x31d as ::core::ffi::c_int as uint16_t,
    0x322 as ::core::ffi::c_int as uint16_t,
    0x326 as ::core::ffi::c_int as uint16_t,
    0x32b as ::core::ffi::c_int as uint16_t,
    0x330 as ::core::ffi::c_int as uint16_t,
    0x335 as ::core::ffi::c_int as uint16_t,
    0x33a as ::core::ffi::c_int as uint16_t,
    0x33f as ::core::ffi::c_int as uint16_t,
    0x344 as ::core::ffi::c_int as uint16_t,
    0x349 as ::core::ffi::c_int as uint16_t,
    0x34e as ::core::ffi::c_int as uint16_t,
    0x353 as ::core::ffi::c_int as uint16_t,
    0x357 as ::core::ffi::c_int as uint16_t,
    0x35c as ::core::ffi::c_int as uint16_t,
    0x361 as ::core::ffi::c_int as uint16_t,
    0x366 as ::core::ffi::c_int as uint16_t,
    0x36b as ::core::ffi::c_int as uint16_t,
    0x370 as ::core::ffi::c_int as uint16_t,
    0x374 as ::core::ffi::c_int as uint16_t,
    0x379 as ::core::ffi::c_int as uint16_t,
    0x37e as ::core::ffi::c_int as uint16_t,
    0x383 as ::core::ffi::c_int as uint16_t,
    0x388 as ::core::ffi::c_int as uint16_t,
    0x38c as ::core::ffi::c_int as uint16_t,
    0x391 as ::core::ffi::c_int as uint16_t,
    0x396 as ::core::ffi::c_int as uint16_t,
    0x39b as ::core::ffi::c_int as uint16_t,
    0x39f as ::core::ffi::c_int as uint16_t,
    0x3a4 as ::core::ffi::c_int as uint16_t,
    0x3a9 as ::core::ffi::c_int as uint16_t,
    0x3ad as ::core::ffi::c_int as uint16_t,
    0x3b2 as ::core::ffi::c_int as uint16_t,
    0x3b7 as ::core::ffi::c_int as uint16_t,
    0x3bb as ::core::ffi::c_int as uint16_t,
    0x3c0 as ::core::ffi::c_int as uint16_t,
    0x3c5 as ::core::ffi::c_int as uint16_t,
    0x3c9 as ::core::ffi::c_int as uint16_t,
    0x3ce as ::core::ffi::c_int as uint16_t,
    0x3d2 as ::core::ffi::c_int as uint16_t,
    0x3d7 as ::core::ffi::c_int as uint16_t,
    0x3dc as ::core::ffi::c_int as uint16_t,
    0x3e0 as ::core::ffi::c_int as uint16_t,
    0x3e5 as ::core::ffi::c_int as uint16_t,
    0x3e9 as ::core::ffi::c_int as uint16_t,
    0x3ed as ::core::ffi::c_int as uint16_t,
    0x3f2 as ::core::ffi::c_int as uint16_t,
    0x3f6 as ::core::ffi::c_int as uint16_t,
    0x3fb as ::core::ffi::c_int as uint16_t,
    0x3ff as ::core::ffi::c_int as uint16_t,
    0x403 as ::core::ffi::c_int as uint16_t,
    0x408 as ::core::ffi::c_int as uint16_t,
    0x40c as ::core::ffi::c_int as uint16_t,
    0x410 as ::core::ffi::c_int as uint16_t,
    0x415 as ::core::ffi::c_int as uint16_t,
    0x419 as ::core::ffi::c_int as uint16_t,
    0x41d as ::core::ffi::c_int as uint16_t,
    0x421 as ::core::ffi::c_int as uint16_t,
    0x425 as ::core::ffi::c_int as uint16_t,
    0x42a as ::core::ffi::c_int as uint16_t,
    0x42e as ::core::ffi::c_int as uint16_t,
    0x432 as ::core::ffi::c_int as uint16_t,
    0x436 as ::core::ffi::c_int as uint16_t,
    0x43a as ::core::ffi::c_int as uint16_t,
    0x43e as ::core::ffi::c_int as uint16_t,
    0x442 as ::core::ffi::c_int as uint16_t,
    0x446 as ::core::ffi::c_int as uint16_t,
    0x44a as ::core::ffi::c_int as uint16_t,
    0x44e as ::core::ffi::c_int as uint16_t,
    0x452 as ::core::ffi::c_int as uint16_t,
    0x455 as ::core::ffi::c_int as uint16_t,
    0x459 as ::core::ffi::c_int as uint16_t,
    0x45d as ::core::ffi::c_int as uint16_t,
    0x461 as ::core::ffi::c_int as uint16_t,
    0x465 as ::core::ffi::c_int as uint16_t,
    0x468 as ::core::ffi::c_int as uint16_t,
    0x46c as ::core::ffi::c_int as uint16_t,
    0x470 as ::core::ffi::c_int as uint16_t,
    0x473 as ::core::ffi::c_int as uint16_t,
    0x477 as ::core::ffi::c_int as uint16_t,
    0x47a as ::core::ffi::c_int as uint16_t,
    0x47e as ::core::ffi::c_int as uint16_t,
    0x481 as ::core::ffi::c_int as uint16_t,
    0x485 as ::core::ffi::c_int as uint16_t,
    0x488 as ::core::ffi::c_int as uint16_t,
    0x48c as ::core::ffi::c_int as uint16_t,
    0x48f as ::core::ffi::c_int as uint16_t,
    0x492 as ::core::ffi::c_int as uint16_t,
    0x496 as ::core::ffi::c_int as uint16_t,
    0x499 as ::core::ffi::c_int as uint16_t,
    0x49c as ::core::ffi::c_int as uint16_t,
    0x49f as ::core::ffi::c_int as uint16_t,
    0x4a2 as ::core::ffi::c_int as uint16_t,
    0x4a6 as ::core::ffi::c_int as uint16_t,
    0x4a9 as ::core::ffi::c_int as uint16_t,
    0x4ac as ::core::ffi::c_int as uint16_t,
    0x4af as ::core::ffi::c_int as uint16_t,
    0x4b2 as ::core::ffi::c_int as uint16_t,
    0x4b5 as ::core::ffi::c_int as uint16_t,
    0x4b7 as ::core::ffi::c_int as uint16_t,
    0x4ba as ::core::ffi::c_int as uint16_t,
    0x4bd as ::core::ffi::c_int as uint16_t,
    0x4c0 as ::core::ffi::c_int as uint16_t,
    0x4c3 as ::core::ffi::c_int as uint16_t,
    0x4c5 as ::core::ffi::c_int as uint16_t,
    0x4c8 as ::core::ffi::c_int as uint16_t,
    0x4cb as ::core::ffi::c_int as uint16_t,
    0x4cd as ::core::ffi::c_int as uint16_t,
    0x4d0 as ::core::ffi::c_int as uint16_t,
    0x4d2 as ::core::ffi::c_int as uint16_t,
    0x4d5 as ::core::ffi::c_int as uint16_t,
    0x4d7 as ::core::ffi::c_int as uint16_t,
    0x4d9 as ::core::ffi::c_int as uint16_t,
    0x4dc as ::core::ffi::c_int as uint16_t,
    0x4de as ::core::ffi::c_int as uint16_t,
    0x4e0 as ::core::ffi::c_int as uint16_t,
    0x4e3 as ::core::ffi::c_int as uint16_t,
    0x4e5 as ::core::ffi::c_int as uint16_t,
    0x4e7 as ::core::ffi::c_int as uint16_t,
    0x4e9 as ::core::ffi::c_int as uint16_t,
    0x4eb as ::core::ffi::c_int as uint16_t,
    0x4ed as ::core::ffi::c_int as uint16_t,
    0x4ef as ::core::ffi::c_int as uint16_t,
    0x4f1 as ::core::ffi::c_int as uint16_t,
    0x4f3 as ::core::ffi::c_int as uint16_t,
    0x4f5 as ::core::ffi::c_int as uint16_t,
    0x4f6 as ::core::ffi::c_int as uint16_t,
    0x4f8 as ::core::ffi::c_int as uint16_t,
    0x4fa as ::core::ffi::c_int as uint16_t,
    0x4fb as ::core::ffi::c_int as uint16_t,
    0x4fd as ::core::ffi::c_int as uint16_t,
    0x4ff as ::core::ffi::c_int as uint16_t,
    0x500 as ::core::ffi::c_int as uint16_t,
    0x502 as ::core::ffi::c_int as uint16_t,
    0x503 as ::core::ffi::c_int as uint16_t,
    0x504 as ::core::ffi::c_int as uint16_t,
    0x506 as ::core::ffi::c_int as uint16_t,
    0x507 as ::core::ffi::c_int as uint16_t,
    0x508 as ::core::ffi::c_int as uint16_t,
    0x50a as ::core::ffi::c_int as uint16_t,
    0x50b as ::core::ffi::c_int as uint16_t,
    0x50c as ::core::ffi::c_int as uint16_t,
    0x50d as ::core::ffi::c_int as uint16_t,
    0x50e as ::core::ffi::c_int as uint16_t,
    0x50f as ::core::ffi::c_int as uint16_t,
    0x510 as ::core::ffi::c_int as uint16_t,
    0x511 as ::core::ffi::c_int as uint16_t,
    0x511 as ::core::ffi::c_int as uint16_t,
    0x512 as ::core::ffi::c_int as uint16_t,
    0x513 as ::core::ffi::c_int as uint16_t,
    0x514 as ::core::ffi::c_int as uint16_t,
    0x514 as ::core::ffi::c_int as uint16_t,
    0x515 as ::core::ffi::c_int as uint16_t,
    0x516 as ::core::ffi::c_int as uint16_t,
    0x516 as ::core::ffi::c_int as uint16_t,
    0x517 as ::core::ffi::c_int as uint16_t,
    0x517 as ::core::ffi::c_int as uint16_t,
    0x517 as ::core::ffi::c_int as uint16_t,
    0x518 as ::core::ffi::c_int as uint16_t,
    0x518 as ::core::ffi::c_int as uint16_t,
    0x518 as ::core::ffi::c_int as uint16_t,
    0x518 as ::core::ffi::c_int as uint16_t,
    0x518 as ::core::ffi::c_int as uint16_t,
    0x519 as ::core::ffi::c_int as uint16_t,
    0x519 as ::core::ffi::c_int as uint16_t,
];
#[no_mangle]
pub unsafe extern "C" fn dsp_init(mut ram: *mut uint8_t) -> *mut Dsp {
    let mut dsp: *mut Dsp = malloc(::core::mem::size_of::<Dsp>() as size_t) as *mut Dsp;
    (*dsp).apu_ram = ram;
    return dsp;
}
#[no_mangle]
pub unsafe extern "C" fn dsp_free(mut dsp: *mut Dsp) {
    free(dsp as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn dsp_reset(mut dsp: *mut Dsp) {
    memset(
        &raw mut (*dsp).ram as *mut uint8_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint8_t; 128]>() as size_t,
    );
    (*dsp).ram[0x7c as ::core::ffi::c_int as usize] = 0xff as uint8_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        (*dsp).channel[i as usize].pitch = 0 as uint16_t;
        (*dsp).channel[i as usize].pitchCounter = 0 as uint16_t;
        (*dsp).channel[i as usize].pitchModulation = false_0 != 0;
        memset(
            &raw mut (*(&raw mut (*dsp).channel as *mut DspChannel).offset(i as isize)).decodeBuffer
                as *mut int16_t as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[int16_t; 19]>() as size_t,
        );
        (*dsp).channel[i as usize].srcn = 0 as uint8_t;
        (*dsp).channel[i as usize].decodeOffset = 0 as uint16_t;
        (*dsp).channel[i as usize].previousFlags = 0 as uint8_t;
        (*dsp).channel[i as usize].old = 0 as int16_t;
        (*dsp).channel[i as usize].older = 0 as int16_t;
        (*dsp).channel[i as usize].useNoise = false_0 != 0;
        memset(
            &raw mut (*(&raw mut (*dsp).channel as *mut DspChannel).offset(i as isize)).adsrRates
                as *mut uint16_t as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[uint16_t; 4]>() as size_t,
        );
        (*dsp).channel[i as usize].rateCounter = 0 as uint16_t;
        (*dsp).channel[i as usize].adsrState = 0 as uint8_t;
        (*dsp).channel[i as usize].sustainLevel = 0 as uint16_t;
        (*dsp).channel[i as usize].useGain = false_0 != 0;
        (*dsp).channel[i as usize].gainMode = 0 as uint8_t;
        (*dsp).channel[i as usize].directGain = false_0 != 0;
        (*dsp).channel[i as usize].gainValue = 0 as uint16_t;
        (*dsp).channel[i as usize].gain = 0 as uint16_t;
        (*dsp).channel[i as usize].keyOn = false_0 != 0;
        (*dsp).channel[i as usize].keyOff = false_0 != 0;
        (*dsp).channel[i as usize].sampleOut = 0 as int16_t;
        (*dsp).channel[i as usize].volumeL = 0 as int8_t;
        (*dsp).channel[i as usize].volumeR = 0 as int8_t;
        (*dsp).channel[i as usize].echoEnable = false_0 != 0;
        i += 1;
    }
    (*dsp).dirPage = 0 as uint16_t;
    (*dsp).evenCycle = false_0 != 0;
    (*dsp).mute = true_0 != 0;
    (*dsp).reset = true_0 != 0;
    (*dsp).masterVolumeL = 0 as int8_t;
    (*dsp).masterVolumeR = 0 as int8_t;
    (*dsp).noiseSample = -(0x4000 as ::core::ffi::c_int) as int16_t;
    (*dsp).noiseRate = 0 as uint16_t;
    (*dsp).noiseCounter = 0 as uint16_t;
    (*dsp).echoWrites = false_0 != 0;
    (*dsp).echoVolumeL = 0 as int8_t;
    (*dsp).echoVolumeR = 0 as int8_t;
    (*dsp).feedbackVolume = 0 as int8_t;
    (*dsp).echoBufferAdr = 0 as uint16_t;
    (*dsp).echoDelay = 1 as uint16_t;
    (*dsp).echoRemain = 1 as uint16_t;
    (*dsp).echoBufferIndex = 0 as uint16_t;
    (*dsp).firBufferIndex = 0 as uint8_t;
    memset(
        &raw mut (*dsp).firValues as *mut int8_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[int8_t; 8]>() as size_t,
    );
    memset(
        &raw mut (*dsp).firBufferL as *mut int16_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[int16_t; 8]>() as size_t,
    );
    memset(
        &raw mut (*dsp).firBufferR as *mut int16_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[int16_t; 8]>() as size_t,
    );
    memset(
        &raw mut (*dsp).sampleBuffer as *mut int16_t as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[int16_t; 1068]>() as size_t,
    );
    (*dsp).sampleOffset = 0 as uint16_t;
}
#[no_mangle]
pub unsafe extern "C" fn dsp_saveload(
    mut dsp: *mut Dsp,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*dsp).ram as *mut ::core::ffi::c_void,
        (::core::mem::size_of::<Dsp>() as size_t).wrapping_sub(8 as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn dsp_cycle(mut dsp: *mut Dsp) {
    let mut totalL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut totalR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        dsp_cycleChannel(dsp, i);
        totalL += (*dsp).channel[i as usize].sampleOut as ::core::ffi::c_int
            * (*dsp).channel[i as usize].volumeL as ::core::ffi::c_int
            >> 6 as ::core::ffi::c_int;
        totalR += (*dsp).channel[i as usize].sampleOut as ::core::ffi::c_int
            * (*dsp).channel[i as usize].volumeR as ::core::ffi::c_int
            >> 6 as ::core::ffi::c_int;
        totalL = if totalL < -(0x8000 as ::core::ffi::c_int) {
            -(0x8000 as ::core::ffi::c_int)
        } else if totalL > 0x7fff as ::core::ffi::c_int {
            0x7fff as ::core::ffi::c_int
        } else {
            totalL
        };
        totalR = if totalR < -(0x8000 as ::core::ffi::c_int) {
            -(0x8000 as ::core::ffi::c_int)
        } else if totalR > 0x7fff as ::core::ffi::c_int {
            0x7fff as ::core::ffi::c_int
        } else {
            totalR
        };
        i += 1;
    }
    totalL = totalL * (*dsp).masterVolumeL as ::core::ffi::c_int >> 7 as ::core::ffi::c_int;
    totalR = totalR * (*dsp).masterVolumeR as ::core::ffi::c_int >> 7 as ::core::ffi::c_int;
    totalL = if totalL < -(0x8000 as ::core::ffi::c_int) {
        -(0x8000 as ::core::ffi::c_int)
    } else if totalL > 0x7fff as ::core::ffi::c_int {
        0x7fff as ::core::ffi::c_int
    } else {
        totalL
    };
    totalR = if totalR < -(0x8000 as ::core::ffi::c_int) {
        -(0x8000 as ::core::ffi::c_int)
    } else if totalR > 0x7fff as ::core::ffi::c_int {
        0x7fff as ::core::ffi::c_int
    } else {
        totalR
    };
    dsp_handleEcho(dsp, &raw mut totalL, &raw mut totalR);
    if (*dsp).mute {
        totalL = 0 as ::core::ffi::c_int;
        totalR = 0 as ::core::ffi::c_int;
    }
    dsp_handleNoise(dsp);
    if ((*dsp).sampleOffset as ::core::ffi::c_int) < 534 as ::core::ffi::c_int {
        (*dsp).sampleBuffer
            [((*dsp).sampleOffset as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as usize] =
            totalL as int16_t;
        (*dsp).sampleBuffer[((*dsp).sampleOffset as ::core::ffi::c_int * 2 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as usize] = totalR as int16_t;
        (*dsp).sampleOffset = (*dsp).sampleOffset.wrapping_add(1);
    }
    (*dsp).evenCycle = !(*dsp).evenCycle;
}
unsafe extern "C" fn dsp_handleEcho(
    mut dsp: *mut Dsp,
    mut outputL: *mut ::core::ffi::c_int,
    mut outputR: *mut ::core::ffi::c_int,
) {
    let mut adr: uint16_t = ((*dsp).echoBufferAdr as ::core::ffi::c_int
        + (*dsp).echoBufferIndex as ::core::ffi::c_int * 4 as ::core::ffi::c_int)
        as uint16_t;
    (*dsp).firBufferL[(*dsp).firBufferIndex as usize] =
        (*(*dsp).apu_ram.offset(adr as isize) as ::core::ffi::c_int
            + ((*(*dsp).apu_ram.offset(
                (adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
                    as isize,
            ) as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int)) as int16_t;
    (*dsp).firBufferL[(*dsp).firBufferIndex as usize] =
        ((*dsp).firBufferL[(*dsp).firBufferIndex as usize] as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as int16_t;
    (*dsp).firBufferR[(*dsp).firBufferIndex as usize] = (*(*dsp).apu_ram.offset(
        (adr as ::core::ffi::c_int + 2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
            as isize,
    ) as ::core::ffi::c_int
        + ((*(*dsp).apu_ram.offset(
            (adr as ::core::ffi::c_int + 3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
                as isize,
        ) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int))
        as int16_t;
    (*dsp).firBufferR[(*dsp).firBufferIndex as usize] =
        ((*dsp).firBufferR[(*dsp).firBufferIndex as usize] as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as int16_t;
    let mut sumL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sumR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        sumL += (*dsp).firBufferL[((*dsp).firBufferIndex as ::core::ffi::c_int
            + i
            + 1 as ::core::ffi::c_int
            & 0x7 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
            * (*dsp).firValues[i as usize] as ::core::ffi::c_int
            >> 6 as ::core::ffi::c_int;
        sumR += (*dsp).firBufferR[((*dsp).firBufferIndex as ::core::ffi::c_int
            + i
            + 1 as ::core::ffi::c_int
            & 0x7 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
            * (*dsp).firValues[i as usize] as ::core::ffi::c_int
            >> 6 as ::core::ffi::c_int;
        if i == 6 as ::core::ffi::c_int {
            sumL = (sumL & 0xffff as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int;
            sumR = (sumR & 0xffff as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int;
        }
        i += 1;
    }
    sumL = if sumL < -(0x8000 as ::core::ffi::c_int) {
        -(0x8000 as ::core::ffi::c_int)
    } else if sumL > 0x7fff as ::core::ffi::c_int {
        0x7fff as ::core::ffi::c_int
    } else {
        sumL
    };
    sumR = if sumR < -(0x8000 as ::core::ffi::c_int) {
        -(0x8000 as ::core::ffi::c_int)
    } else if sumR > 0x7fff as ::core::ffi::c_int {
        0x7fff as ::core::ffi::c_int
    } else {
        sumR
    };
    let mut outL: ::core::ffi::c_int =
        *outputL + (sumL * (*dsp).echoVolumeL as ::core::ffi::c_int >> 7 as ::core::ffi::c_int);
    let mut outR: ::core::ffi::c_int =
        *outputR + (sumR * (*dsp).echoVolumeR as ::core::ffi::c_int >> 7 as ::core::ffi::c_int);
    *outputL = if outL < -(0x8000 as ::core::ffi::c_int) {
        -(0x8000 as ::core::ffi::c_int)
    } else if outL > 0x7fff as ::core::ffi::c_int {
        0x7fff as ::core::ffi::c_int
    } else {
        outL
    };
    *outputR = if outR < -(0x8000 as ::core::ffi::c_int) {
        -(0x8000 as ::core::ffi::c_int)
    } else if outR > 0x7fff as ::core::ffi::c_int {
        0x7fff as ::core::ffi::c_int
    } else {
        outR
    };
    let mut inL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut inR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 8 as ::core::ffi::c_int {
        if (*dsp).channel[i_0 as usize].echoEnable {
            inL += (*dsp).channel[i_0 as usize].sampleOut as ::core::ffi::c_int
                * (*dsp).channel[i_0 as usize].volumeL as ::core::ffi::c_int
                >> 6 as ::core::ffi::c_int;
            inR += (*dsp).channel[i_0 as usize].sampleOut as ::core::ffi::c_int
                * (*dsp).channel[i_0 as usize].volumeR as ::core::ffi::c_int
                >> 6 as ::core::ffi::c_int;
            inL = if inL < -(0x8000 as ::core::ffi::c_int) {
                -(0x8000 as ::core::ffi::c_int)
            } else if inL > 0x7fff as ::core::ffi::c_int {
                0x7fff as ::core::ffi::c_int
            } else {
                inL
            };
            inR = if inR < -(0x8000 as ::core::ffi::c_int) {
                -(0x8000 as ::core::ffi::c_int)
            } else if inR > 0x7fff as ::core::ffi::c_int {
                0x7fff as ::core::ffi::c_int
            } else {
                inR
            };
        }
        i_0 += 1;
    }
    inL += sumL * (*dsp).feedbackVolume as ::core::ffi::c_int >> 7 as ::core::ffi::c_int;
    inR += sumR * (*dsp).feedbackVolume as ::core::ffi::c_int >> 7 as ::core::ffi::c_int;
    inL = if inL < -(0x8000 as ::core::ffi::c_int) {
        -(0x8000 as ::core::ffi::c_int)
    } else if inL > 0x7fff as ::core::ffi::c_int {
        0x7fff as ::core::ffi::c_int
    } else {
        inL
    };
    inR = if inR < -(0x8000 as ::core::ffi::c_int) {
        -(0x8000 as ::core::ffi::c_int)
    } else if inR > 0x7fff as ::core::ffi::c_int {
        0x7fff as ::core::ffi::c_int
    } else {
        inR
    };
    inL &= 0xfffe as ::core::ffi::c_int;
    inR &= 0xfffe as ::core::ffi::c_int;
    if (*dsp).echoWrites {
        *(*dsp).apu_ram.offset(adr as isize) = (inL & 0xff as ::core::ffi::c_int) as uint8_t;
        *(*dsp).apu_ram.offset(
            (adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
                as isize,
        ) = (inL >> 8 as ::core::ffi::c_int) as uint8_t;
        *(*dsp).apu_ram.offset(
            (adr as ::core::ffi::c_int + 2 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
                as isize,
        ) = (inR & 0xff as ::core::ffi::c_int) as uint8_t;
        *(*dsp).apu_ram.offset(
            (adr as ::core::ffi::c_int + 3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
                as isize,
        ) = (inR >> 8 as ::core::ffi::c_int) as uint8_t;
    }
    (*dsp).firBufferIndex = (*dsp).firBufferIndex.wrapping_add(1);
    (*dsp).firBufferIndex =
        ((*dsp).firBufferIndex as ::core::ffi::c_int & 7 as ::core::ffi::c_int) as uint8_t;
    (*dsp).echoBufferIndex = (*dsp).echoBufferIndex.wrapping_add(1);
    (*dsp).echoRemain = (*dsp).echoRemain.wrapping_sub(1);
    if (*dsp).echoRemain as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*dsp).echoRemain = (*dsp).echoDelay;
        (*dsp).echoBufferIndex = 0 as uint16_t;
    }
}
unsafe extern "C" fn dsp_cycleChannel(mut dsp: *mut Dsp, mut ch: ::core::ffi::c_int) {
    let mut pitch: uint16_t = (*dsp).channel[ch as usize].pitch;
    if ch > 0 as ::core::ffi::c_int
        && (*dsp).channel[ch as usize].pitchModulation as ::core::ffi::c_int != 0
    {
        let mut factor: ::core::ffi::c_int = ((*dsp).channel
            [(ch - 1 as ::core::ffi::c_int) as usize]
            .sampleOut as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)
            + 0x400 as ::core::ffi::c_int;
        pitch = (pitch as ::core::ffi::c_int * factor >> 10 as ::core::ffi::c_int) as uint16_t;
        if pitch as ::core::ffi::c_int > 0x3fff as ::core::ffi::c_int {
            pitch = 0x3fff as uint16_t;
        }
    }
    let mut newCounter: ::core::ffi::c_int = (*dsp).channel[ch as usize].pitchCounter
        as ::core::ffi::c_int
        + pitch as ::core::ffi::c_int;
    if newCounter > 0xffff as ::core::ffi::c_int {
        dsp_decodeBrr(dsp, ch);
    }
    (*dsp).channel[ch as usize].pitchCounter = newCounter as uint16_t;
    let mut sample: int16_t = 0 as int16_t;
    if (*dsp).channel[ch as usize].useNoise {
        sample = (*dsp).noiseSample;
    } else {
        sample = dsp_getSample(
            dsp,
            ch,
            (*dsp).channel[ch as usize].pitchCounter as ::core::ffi::c_int
                >> 12 as ::core::ffi::c_int,
            (*dsp).channel[ch as usize].pitchCounter as ::core::ffi::c_int
                >> 4 as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_int,
        );
    }
    if (*dsp).reset {
        (*dsp).channel[ch as usize].adsrState = 4 as uint8_t;
        (*dsp).channel[ch as usize].gain = 0 as uint16_t;
    }
    let mut doingDirectGain: bool = (*dsp).channel[ch as usize].adsrState as ::core::ffi::c_int
        != 4 as ::core::ffi::c_int
        && (*dsp).channel[ch as usize].useGain as ::core::ffi::c_int != 0
        && (*dsp).channel[ch as usize].directGain as ::core::ffi::c_int != 0;
    let mut rate: uint16_t = (if (*dsp).channel[ch as usize].adsrState as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        0 as ::core::ffi::c_int
    } else {
        (*dsp).channel[ch as usize].adsrRates[(*dsp).channel[ch as usize].adsrState as usize]
            as ::core::ffi::c_int
    }) as uint16_t;
    if (*dsp).channel[ch as usize].adsrState as ::core::ffi::c_int != 4 as ::core::ffi::c_int
        && !doingDirectGain
        && rate as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        (*dsp).channel[ch as usize].rateCounter =
            (*dsp).channel[ch as usize].rateCounter.wrapping_add(1);
    }
    if (*dsp).channel[ch as usize].adsrState as ::core::ffi::c_int == 4 as ::core::ffi::c_int
        || !doingDirectGain
            && (*dsp).channel[ch as usize].rateCounter as ::core::ffi::c_int
                >= rate as ::core::ffi::c_int
            && rate as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        if (*dsp).channel[ch as usize].adsrState as ::core::ffi::c_int != 4 as ::core::ffi::c_int {
            (*dsp).channel[ch as usize].rateCounter = 0 as uint16_t;
        }
        dsp_handleGain(dsp, ch);
    }
    if doingDirectGain {
        (*dsp).channel[ch as usize].gain = (*dsp).channel[ch as usize].gainValue;
    }
    (*dsp).ram[(ch << 4 as ::core::ffi::c_int | 8 as ::core::ffi::c_int) as usize] =
        ((*dsp).channel[ch as usize].gain as ::core::ffi::c_int >> 4 as ::core::ffi::c_int)
            as uint8_t;
    sample = (sample as ::core::ffi::c_int * (*dsp).channel[ch as usize].gain as ::core::ffi::c_int
        >> 11 as ::core::ffi::c_int) as int16_t;
    (*dsp).ram[(ch << 4 as ::core::ffi::c_int | 9 as ::core::ffi::c_int) as usize] =
        (sample as ::core::ffi::c_int >> 7 as ::core::ffi::c_int) as uint8_t;
    (*dsp).channel[ch as usize].sampleOut = sample;
}
unsafe extern "C" fn dsp_handleGain(mut dsp: *mut Dsp, mut ch: ::core::ffi::c_int) {
    match (*dsp).channel[ch as usize].adsrState as ::core::ffi::c_int {
        0 => {
            let mut rate: uint16_t = (*dsp).channel[ch as usize].adsrRates
                [(*dsp).channel[ch as usize].adsrState as usize];
            (*dsp).channel[ch as usize].gain = ((*dsp).channel[ch as usize].gain
                as ::core::ffi::c_int
                + if rate as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                    1024 as ::core::ffi::c_int
                } else {
                    32 as ::core::ffi::c_int
                }) as uint16_t;
            if (*dsp).channel[ch as usize].gain as ::core::ffi::c_int >= 0x7e0 as ::core::ffi::c_int
            {
                (*dsp).channel[ch as usize].adsrState = 1 as uint8_t;
            }
            if (*dsp).channel[ch as usize].gain as ::core::ffi::c_int > 0x7ff as ::core::ffi::c_int
            {
                (*dsp).channel[ch as usize].gain = 0x7ff as uint16_t;
            }
        }
        1 => {
            (*dsp).channel[ch as usize].gain =
                ((*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                    - (((*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int
                        >> 8 as ::core::ffi::c_int)
                        + 1 as ::core::ffi::c_int)) as uint16_t;
            if ((*dsp).channel[ch as usize].gain as ::core::ffi::c_int)
                < (*dsp).channel[ch as usize].sustainLevel as ::core::ffi::c_int
            {
                (*dsp).channel[ch as usize].adsrState = 2 as uint8_t;
            }
        }
        2 => {
            (*dsp).channel[ch as usize].gain =
                ((*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                    - (((*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int
                        >> 8 as ::core::ffi::c_int)
                        + 1 as ::core::ffi::c_int)) as uint16_t;
        }
        3 => match (*dsp).channel[ch as usize].gainMode as ::core::ffi::c_int {
            0 => {
                (*dsp).channel[ch as usize].gain =
                    ((*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                        - 32 as ::core::ffi::c_int) as uint16_t;
                if (*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                    > 0x7ff as ::core::ffi::c_int
                {
                    (*dsp).channel[ch as usize].gain = 0 as uint16_t;
                }
            }
            1 => {
                (*dsp).channel[ch as usize].gain =
                    ((*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                        - (((*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int
                            >> 8 as ::core::ffi::c_int)
                            + 1 as ::core::ffi::c_int)) as uint16_t;
            }
            2 => {
                (*dsp).channel[ch as usize].gain =
                    ((*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                        + 32 as ::core::ffi::c_int) as uint16_t;
                if (*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                    > 0x7ff as ::core::ffi::c_int
                {
                    (*dsp).channel[ch as usize].gain = 0x7ff as uint16_t;
                }
            }
            3 => {
                (*dsp).channel[ch as usize].gain = ((*dsp).channel[ch as usize].gain
                    as ::core::ffi::c_int
                    + if ((*dsp).channel[ch as usize].gain as ::core::ffi::c_int)
                        < 0x600 as ::core::ffi::c_int
                    {
                        32 as ::core::ffi::c_int
                    } else {
                        8 as ::core::ffi::c_int
                    }) as uint16_t;
                if (*dsp).channel[ch as usize].gain as ::core::ffi::c_int
                    > 0x7ff as ::core::ffi::c_int
                {
                    (*dsp).channel[ch as usize].gain = 0x7ff as uint16_t;
                }
            }
            _ => {}
        },
        4 => {
            (*dsp).channel[ch as usize].gain =
                ((*dsp).channel[ch as usize].gain as ::core::ffi::c_int - 8 as ::core::ffi::c_int)
                    as uint16_t;
            if (*dsp).channel[ch as usize].gain as ::core::ffi::c_int > 0x7ff as ::core::ffi::c_int
            {
                (*dsp).channel[ch as usize].gain = 0 as uint16_t;
            }
        }
        _ => {}
    };
}
unsafe extern "C" fn dsp_getSample(
    mut dsp: *mut Dsp,
    mut ch: ::core::ffi::c_int,
    mut sampleNum: ::core::ffi::c_int,
    mut offset: ::core::ffi::c_int,
) -> int16_t {
    let mut news: int16_t =
        (*dsp).channel[ch as usize].decodeBuffer[(sampleNum + 3 as ::core::ffi::c_int) as usize];
    let mut olds: int16_t =
        (*dsp).channel[ch as usize].decodeBuffer[(sampleNum + 2 as ::core::ffi::c_int) as usize];
    let mut olders: int16_t =
        (*dsp).channel[ch as usize].decodeBuffer[(sampleNum + 1 as ::core::ffi::c_int) as usize];
    let mut oldests: int16_t = (*dsp).channel[ch as usize].decodeBuffer[sampleNum as usize];
    let mut out: ::core::ffi::c_int = gaussValues[(0xff as ::core::ffi::c_int - offset) as usize]
        as ::core::ffi::c_int
        * oldests as ::core::ffi::c_int
        >> 10 as ::core::ffi::c_int;
    out += gaussValues[(0x1ff as ::core::ffi::c_int - offset) as usize] as ::core::ffi::c_int
        * olders as ::core::ffi::c_int
        >> 10 as ::core::ffi::c_int;
    out += gaussValues[(0x100 as ::core::ffi::c_int + offset) as usize] as ::core::ffi::c_int
        * olds as ::core::ffi::c_int
        >> 10 as ::core::ffi::c_int;
    out = (out & 0xffff as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int;
    out += gaussValues[offset as usize] as ::core::ffi::c_int * news as ::core::ffi::c_int
        >> 10 as ::core::ffi::c_int;
    out = if out < -(0x8000 as ::core::ffi::c_int) {
        -(0x8000 as ::core::ffi::c_int)
    } else if out > 0x7fff as ::core::ffi::c_int {
        0x7fff as ::core::ffi::c_int
    } else {
        out
    };
    return (out >> 1 as ::core::ffi::c_int) as int16_t;
}
unsafe extern "C" fn dsp_decodeBrr(mut dsp: *mut Dsp, mut ch: ::core::ffi::c_int) {
    (*dsp).channel[ch as usize].decodeBuffer[0 as ::core::ffi::c_int as usize] =
        (*dsp).channel[ch as usize].decodeBuffer[16 as ::core::ffi::c_int as usize];
    (*dsp).channel[ch as usize].decodeBuffer[1 as ::core::ffi::c_int as usize] =
        (*dsp).channel[ch as usize].decodeBuffer[17 as ::core::ffi::c_int as usize];
    (*dsp).channel[ch as usize].decodeBuffer[2 as ::core::ffi::c_int as usize] =
        (*dsp).channel[ch as usize].decodeBuffer[18 as ::core::ffi::c_int as usize];
    if (*dsp).channel[ch as usize].previousFlags as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        || (*dsp).channel[ch as usize].previousFlags as ::core::ffi::c_int
            == 3 as ::core::ffi::c_int
    {
        let mut samplePointer: uint16_t = ((*dsp).dirPage as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int * (*dsp).channel[ch as usize].srcn as ::core::ffi::c_int)
            as uint16_t;
        (*dsp).channel[ch as usize].decodeOffset = *(*dsp).apu_ram.offset(
            (samplePointer as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                & 0xffff as ::core::ffi::c_int) as isize,
        ) as uint16_t;
        (*dsp).channel[ch as usize].decodeOffset =
            ((*dsp).channel[ch as usize].decodeOffset as ::core::ffi::c_int
                | (*(*dsp).apu_ram.offset(
                    (samplePointer as ::core::ffi::c_int + 3 as ::core::ffi::c_int
                        & 0xffff as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int) as uint16_t;
        if (*dsp).channel[ch as usize].previousFlags as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            (*dsp).channel[ch as usize].adsrState = 4 as uint8_t;
            (*dsp).channel[ch as usize].gain = 0 as uint16_t;
        }
        (*dsp).ram[0x7c as ::core::ffi::c_int as usize] =
            ((*dsp).ram[0x7c as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int) << ch) as uint8_t;
    }
    let fresh0 = (*dsp).channel[ch as usize].decodeOffset;
    (*dsp).channel[ch as usize].decodeOffset =
        (*dsp).channel[ch as usize].decodeOffset.wrapping_add(1);
    let mut header: uint8_t = *(*dsp).apu_ram.offset(fresh0 as isize);
    let mut shift: ::core::ffi::c_int = header as ::core::ffi::c_int >> 4 as ::core::ffi::c_int;
    let mut filter: ::core::ffi::c_int =
        (header as ::core::ffi::c_int & 0xc as ::core::ffi::c_int) >> 2 as ::core::ffi::c_int;
    (*dsp).channel[ch as usize].previousFlags =
        (header as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int) as uint8_t;
    let mut curByte: uint8_t = 0 as uint8_t;
    let mut old: ::core::ffi::c_int = (*dsp).channel[ch as usize].old as ::core::ffi::c_int;
    let mut older: ::core::ffi::c_int = (*dsp).channel[ch as usize].older as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 16 as ::core::ffi::c_int {
        let mut s: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if i & 1 as ::core::ffi::c_int != 0 {
            s = curByte as ::core::ffi::c_int & 0xf as ::core::ffi::c_int;
        } else {
            let fresh1 = (*dsp).channel[ch as usize].decodeOffset;
            (*dsp).channel[ch as usize].decodeOffset =
                (*dsp).channel[ch as usize].decodeOffset.wrapping_add(1);
            curByte = *(*dsp).apu_ram.offset(fresh1 as isize);
            s = curByte as ::core::ffi::c_int >> 4 as ::core::ffi::c_int;
        }
        if s > 7 as ::core::ffi::c_int {
            s -= 16 as ::core::ffi::c_int;
        }
        if shift <= 0xc as ::core::ffi::c_int {
            s = s << shift >> 1 as ::core::ffi::c_int;
        } else {
            s = (s >> 3 as ::core::ffi::c_int) << 12 as ::core::ffi::c_int;
        }
        match filter {
            1 => {
                s += old + (-old >> 4 as ::core::ffi::c_int);
            }
            2 => {
                s += 2 as ::core::ffi::c_int * old
                    + (3 as ::core::ffi::c_int * -old >> 5 as ::core::ffi::c_int)
                    - older
                    + (older >> 4 as ::core::ffi::c_int);
            }
            3 => {
                s += 2 as ::core::ffi::c_int * old
                    + (13 as ::core::ffi::c_int * -old >> 6 as ::core::ffi::c_int)
                    - older
                    + (3 as ::core::ffi::c_int * older >> 4 as ::core::ffi::c_int);
            }
            _ => {}
        }
        s = if s < -(0x8000 as ::core::ffi::c_int) {
            -(0x8000 as ::core::ffi::c_int)
        } else if s > 0x7fff as ::core::ffi::c_int {
            0x7fff as ::core::ffi::c_int
        } else {
            s
        };
        s = ((s & 0x7fff as ::core::ffi::c_int) << 1 as ::core::ffi::c_int) as int16_t
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        older = old;
        old = s;
        (*dsp).channel[ch as usize].decodeBuffer[(i + 3 as ::core::ffi::c_int) as usize] =
            s as int16_t;
        i += 1;
    }
    (*dsp).channel[ch as usize].older = older as int16_t;
    (*dsp).channel[ch as usize].old = old as int16_t;
}
unsafe extern "C" fn dsp_handleNoise(mut dsp: *mut Dsp) {
    if (*dsp).noiseRate as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*dsp).noiseCounter = (*dsp).noiseCounter.wrapping_add(1);
    }
    if (*dsp).noiseCounter as ::core::ffi::c_int >= (*dsp).noiseRate as ::core::ffi::c_int
        && (*dsp).noiseRate as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        let mut bit: ::core::ffi::c_int = (*dsp).noiseSample as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            ^ (*dsp).noiseSample as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int;
        (*dsp).noiseSample = ((*dsp).noiseSample as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
            & 0x3fff as ::core::ffi::c_int
            | bit << 14 as ::core::ffi::c_int) as int16_t;
        (*dsp).noiseSample =
            ((((*dsp).noiseSample as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                << 1 as ::core::ffi::c_int) as int16_t as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as int16_t;
        (*dsp).noiseCounter = 0 as uint16_t;
    }
}
#[no_mangle]
pub unsafe extern "C" fn dsp_read(mut dsp: *mut Dsp, mut adr: uint8_t) -> uint8_t {
    return (*dsp).ram[adr as usize];
}
#[no_mangle]
pub unsafe extern "C" fn dsp_write(mut dsp: *mut Dsp, mut adr: uint8_t, mut val: uint8_t) {
    let mut ch: ::core::ffi::c_int = adr as ::core::ffi::c_int >> 4 as ::core::ffi::c_int;
    match adr as ::core::ffi::c_int {
        0 | 16 | 32 | 48 | 64 | 80 | 96 | 112 => {
            (*dsp).channel[ch as usize].volumeL = val as int8_t;
        }
        1 | 17 | 33 | 49 | 65 | 81 | 97 | 113 => {
            (*dsp).channel[ch as usize].volumeR = val as int8_t;
        }
        2 | 18 | 34 | 50 | 66 | 82 | 98 | 114 => {
            (*dsp).channel[ch as usize].pitch =
                ((*dsp).channel[ch as usize].pitch as ::core::ffi::c_int
                    & 0x3f00 as ::core::ffi::c_int
                    | val as ::core::ffi::c_int) as uint16_t;
        }
        3 | 19 | 35 | 51 | 67 | 83 | 99 | 115 => {
            (*dsp).channel[ch as usize].pitch =
                (((*dsp).channel[ch as usize].pitch as ::core::ffi::c_int
                    & 0xff as ::core::ffi::c_int
                    | (val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                    & 0x3fff as ::core::ffi::c_int) as uint16_t;
        }
        4 | 20 | 36 | 52 | 68 | 84 | 100 | 116 => {
            (*dsp).channel[ch as usize].srcn = val;
        }
        5 | 21 | 37 | 53 | 69 | 85 | 101 | 117 => {
            (*dsp).channel[ch as usize].adsrRates[0 as ::core::ffi::c_int as usize] = rateValues
                [((val as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) * 2 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as usize]
                as uint16_t;
            (*dsp).channel[ch as usize].adsrRates[1 as ::core::ffi::c_int as usize] =
                rateValues[(((val as ::core::ffi::c_int & 0x70 as ::core::ffi::c_int)
                    >> 4 as ::core::ffi::c_int)
                    * 2 as ::core::ffi::c_int
                    + 16 as ::core::ffi::c_int) as usize] as uint16_t;
            (*dsp).channel[ch as usize].useGain =
                val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
        }
        6 | 22 | 38 | 54 | 70 | 86 | 102 | 118 => {
            (*dsp).channel[ch as usize].adsrRates[2 as ::core::ffi::c_int as usize] = rateValues
                [(val as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as usize]
                as uint16_t;
            (*dsp).channel[ch as usize].sustainLevel =
                ((((val as ::core::ffi::c_int & 0xe0 as ::core::ffi::c_int)
                    >> 5 as ::core::ffi::c_int)
                    + 1 as ::core::ffi::c_int)
                    * 0x100 as ::core::ffi::c_int) as uint16_t;
        }
        7 | 23 | 39 | 55 | 71 | 87 | 103 | 119 => {
            (*dsp).channel[ch as usize].directGain =
                val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            if val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
                (*dsp).channel[ch as usize].gainMode =
                    ((val as ::core::ffi::c_int & 0x60 as ::core::ffi::c_int)
                        >> 5 as ::core::ffi::c_int) as uint8_t;
                (*dsp).channel[ch as usize].adsrRates[3 as ::core::ffi::c_int as usize] = rateValues
                    [(val as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as usize]
                    as uint16_t;
            } else {
                (*dsp).channel[ch as usize].gainValue =
                    ((val as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int)
                        * 16 as ::core::ffi::c_int) as uint16_t;
            }
        }
        12 => {
            (*dsp).masterVolumeL = val as int8_t;
        }
        28 => {
            (*dsp).masterVolumeR = val as int8_t;
        }
        44 => {
            (*dsp).echoVolumeL = val as int8_t;
        }
        60 => {
            (*dsp).echoVolumeR = val as int8_t;
        }
        76 => {
            let mut ch_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while ch_0 < 8 as ::core::ffi::c_int {
                (*dsp).channel[ch_0 as usize].keyOn =
                    val as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << ch_0 != 0;
                if (*dsp).channel[ch_0 as usize].keyOn {
                    (*dsp).channel[ch_0 as usize].keyOn = false_0 != 0;
                    (*dsp).channel[ch_0 as usize].previousFlags = 0 as uint8_t;
                    let mut samplePointer: uint16_t = ((*dsp).dirPage as ::core::ffi::c_int
                        + 4 as ::core::ffi::c_int
                            * (*dsp).channel[ch_0 as usize].srcn as ::core::ffi::c_int)
                        as uint16_t;
                    (*dsp).channel[ch_0 as usize].decodeOffset =
                        *(*dsp).apu_ram.offset(samplePointer as isize) as uint16_t;
                    (*dsp).channel[ch_0 as usize].decodeOffset =
                        ((*dsp).channel[ch_0 as usize].decodeOffset as ::core::ffi::c_int
                            | (*(*dsp).apu_ram.offset(
                                (samplePointer as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                                    & 0xffff as ::core::ffi::c_int)
                                    as isize,
                            ) as ::core::ffi::c_int)
                                << 8 as ::core::ffi::c_int) as uint16_t;
                    memset(
                        &raw mut (*(&raw mut (*dsp).channel as *mut DspChannel)
                            .offset(ch_0 as isize))
                        .decodeBuffer as *mut int16_t
                            as *mut ::core::ffi::c_void,
                        0 as ::core::ffi::c_int,
                        ::core::mem::size_of::<[int16_t; 19]>() as size_t,
                    );
                    (*dsp).channel[ch_0 as usize].gain = 0 as uint16_t;
                    (*dsp).channel[ch_0 as usize].adsrState =
                        (if (*dsp).channel[ch_0 as usize].useGain as ::core::ffi::c_int != 0 {
                            3 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as uint8_t;
                }
                ch_0 += 1;
            }
        }
        92 => {
            let mut ch_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while ch_1 < 8 as ::core::ffi::c_int {
                (*dsp).channel[ch_1 as usize].keyOff =
                    val as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << ch_1 != 0;
                if (*dsp).channel[ch_1 as usize].keyOff {
                    (*dsp).channel[ch_1 as usize].adsrState = 4 as uint8_t;
                }
                ch_1 += 1;
            }
        }
        108 => {
            (*dsp).reset = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
            (*dsp).mute = val as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0;
            (*dsp).echoWrites =
                val as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*dsp).noiseRate = rateValues
                [(val as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int) as usize]
                as uint16_t;
        }
        124 => {
            val = 0 as uint8_t;
        }
        13 => {
            (*dsp).feedbackVolume = val as int8_t;
        }
        45 => {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 8 as ::core::ffi::c_int {
                (*dsp).channel[i as usize].pitchModulation =
                    val as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << i != 0;
                i += 1;
            }
        }
        61 => {
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < 8 as ::core::ffi::c_int {
                (*dsp).channel[i_0 as usize].useNoise =
                    val as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << i_0 != 0;
                i_0 += 1;
            }
        }
        77 => {
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < 8 as ::core::ffi::c_int {
                (*dsp).channel[i_1 as usize].echoEnable =
                    val as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << i_1 != 0;
                i_1 += 1;
            }
        }
        93 => {
            (*dsp).dirPage = ((val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
        }
        109 => {
            (*dsp).echoBufferAdr =
                ((val as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
        }
        125 => {
            (*dsp).echoDelay = ((val as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
                * 512 as ::core::ffi::c_int) as uint16_t;
            if (*dsp).echoDelay as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                (*dsp).echoDelay = 1 as uint16_t;
            }
        }
        15 | 31 | 47 | 63 | 79 | 95 | 111 | 127 => {
            (*dsp).firValues[ch as usize] = val as int8_t;
        }
        _ => {}
    }
    (*dsp).ram[adr as usize] = val;
}
#[no_mangle]
pub unsafe extern "C" fn dsp_getSamples(
    mut dsp: *mut Dsp,
    mut sampleData: *mut int16_t,
    mut samplesPerFrame: ::core::ffi::c_int,
) {
    let mut adder: ::core::ffi::c_double = 534.0f64 / samplesPerFrame as ::core::ffi::c_double;
    let mut location: ::core::ffi::c_double = 0.0f64;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < samplesPerFrame {
        *sampleData.offset((i * 2 as ::core::ffi::c_int) as isize) = (*dsp).sampleBuffer
            [(location as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as usize];
        *sampleData.offset((i * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
            (*dsp).sampleBuffer[(location as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as usize];
        location += adder;
        i += 1;
    }
    (*dsp).sampleOffset = 0 as uint16_t;
}
