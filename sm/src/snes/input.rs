extern "C" {
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16_t, int32_t, int8_t,
    size_t, uint16_t, uint32_t, uint8_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma,
    DmaChannel, Dsp, DspChannel, DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs,
    PpuZbufType, Snes, Spc, Timer, WindowLayer,
};
pub unsafe fn input_init(snes: *mut Snes) -> *mut Input {
    let input: *mut Input = malloc(::core::mem::size_of::<Input>() as size_t) as *mut Input;
    (*input).snes = snes;
    (*input).type_0 = 1 as uint8_t;
    (*input).currentState = 0 as uint16_t;
    input
}
pub unsafe fn input_free(input: *mut Input) {
    free(input as *mut ::core::ffi::c_void);
}
pub unsafe fn input_reset(input: *mut Input) {}
