use ::core::ffi::c_void;
use ::core::mem::size_of;
extern "C" {
    fn malloc(__size: size_t) -> *mut c_void;
    fn free(__ptr: *mut c_void);
}
pub use crate::types::{
    size_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer,
    WindowLayer,
};
pub unsafe fn input_init(snes: *mut Snes) -> *mut Input {
    let input: *mut Input = malloc(size_of::<Input>() as size_t) as *mut Input;
    (*input).snes = snes;
    (*input).type_0 = 1u8;
    (*input).currentState = 0u16;
    input
}
pub unsafe fn input_free(input: *mut Input) {
    free(input as *mut c_void);
}
pub unsafe fn input_reset(input: *mut Input) {}
