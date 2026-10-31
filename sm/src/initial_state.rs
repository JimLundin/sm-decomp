//! Typed power-on values for detached hardware and oracle records.
#![forbid(unsafe_code)]

// This constructs values, not byte representations. Implicit padding is not state.
pub(crate) trait InitialState {
    const INITIAL: Self;
}
macro_rules! scalars {
    ($($ty:ty = $value:expr),* $(,)?) => {
        $(impl InitialState for $ty { const INITIAL: Self = $value; })*
    };
}
scalars!(
    u8 = 0,
    u16 = 0,
    u32 = 0,
    i8 = 0,
    i16 = 0,
    i32 = 0,
    f64 = 0.0,
    bool = false
);
impl InitialState for sm_runtime::audio::dsp::DspState {
    const INITIAL: Self = Self::INITIAL;
}
impl InitialState for sm_runtime::audio::dsp::DspChannel {
    const INITIAL: Self = Self::INITIAL;
}
impl<T> InitialState for *mut T {
    const INITIAL: Self = core::ptr::null_mut();
}
impl<T: InitialState + Copy, const N: usize> InitialState for [T; N] {
    const INITIAL: Self = [T::INITIAL; N];
}
