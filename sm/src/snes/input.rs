use crate::c_library::{free, malloc};
pub use crate::types::{
    size_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer,
    WindowLayer,
};
use ::core::ffi::c_void;
use ::core::mem::size_of;
pub unsafe fn input_init(snes: *mut Snes) -> *mut Input {
    let input = malloc(size_of::<Input>() as size_t) as *mut Input;
    (*input).snes = snes;
    (*input).type_0 = 1;
    (*input).currentState = 0;
    input
}
pub unsafe fn input_reset(input: *mut Input) {}
