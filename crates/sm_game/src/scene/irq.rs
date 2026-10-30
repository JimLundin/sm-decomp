//! Ordered scanline interrupt decisions over the actual owned display authority.
#![forbid(unsafe_code)]

use super::state::{DisplayError, DisplayState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Current,
    Pending,
    Enable,
    PreviousCre,
    CurrentCre,
    TransitionFlag,
}

impl Control {
    pub fn read(self, state: &DisplayState) -> u16 {
        match self {
            Self::Current => state.interrupt.current,
            Self::Pending => state.interrupt.pending,
            Self::Enable => {
                u16::from_le_bytes([state.controls.nmi_enable, state.controls.hdma_enable])
            }
            Self::PreviousCre => state.interrupt.previous_cre,
            Self::CurrentCre => state.interrupt.current_cre,
            Self::TransitionFlag => state.interrupt.transition_flag,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Store {
    Current,
    Pending,
    Enable,
}

impl Store {
    pub fn write(self, state: &mut DisplayState, value: u16) {
        match self {
            Self::Current => state.interrupt.current = value,
            Self::Pending => state.interrupt.pending = value,
            Self::Enable => {
                let [low, high] = value.to_le_bytes();
                state.controls.nmi_enable = low;
                state.controls.hdma_enable = high;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mirror {
    Background3Map,
    ColorWindow,
    ColorArithmetic,
    MainLayers,
    InterruptEnable,
}

/// Private integration effects, not register addresses or public Scene records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Background3Map,
    ColorWindow,
    ColorArithmetic,
    MainLayers,
    VerticalLow,
    VerticalHigh,
    HorizontalLow,
    HorizontalHigh,
    InterruptEnable,
}

/// Work-state access alone does not require a native display capability.
pub trait Controls {
    fn read_control(&mut self, field: Control) -> u16;
    fn write_control(&mut self, field: Store, value: u16);
}

/// Every access reaches the current authority at the original source point.
/// A display write or transition may change later reads. No state borrow may
/// survive either capability, and native callers retain their original contract.
pub trait Host: Controls {
    type Error;

    fn read_mirror(&mut self, field: Mirror) -> u8;
    fn write_display(&mut self, field: Output, value: u8) -> Result<(), Self::Error>;
    fn follow_transition(&mut self) -> Result<(), Self::Error>;
}

/// The real G room capability polls once synchronously using this same owner.
/// Its completed display/room effects survive a returned fault or unwinding.
pub trait OwnedIrqEffects {
    fn follow_transition(&mut self, state: &mut DisplayState) -> Result<(), DisplayError>;
}

struct OwnedControls<'a> {
    state: &'a mut DisplayState,
}
impl Controls for OwnedControls<'_> {
    fn read_control(&mut self, field: Control) -> u16 {
        field.read(self.state)
    }
    fn write_control(&mut self, field: Store, value: u16) {
        field.write(self.state, value);
    }
}

struct OwnedHost<'a, E> {
    controls: OwnedControls<'a>,
    effects: &'a mut E,
}
impl<E> Controls for OwnedHost<'_, E> {
    fn read_control(&mut self, field: Control) -> u16 {
        self.controls.read_control(field)
    }
    fn write_control(&mut self, field: Store, value: u16) {
        self.controls.write_control(field, value);
    }
}
impl<E: OwnedIrqEffects> Host for OwnedHost<'_, E> {
    type Error = DisplayError;

    fn read_mirror(&mut self, field: Mirror) -> u8 {
        let controls = &self.controls.state.controls;
        match field {
            Mirror::Background3Map => controls.previous_background3_map,
            Mirror::ColorWindow => controls.previous_color_selection,
            Mirror::ColorArithmetic => controls.previous_color_math,
            Mirror::MainLayers => controls.previous_main_layers,
            Mirror::InterruptEnable => controls.nmi_enable,
        }
    }
    fn write_display(&mut self, field: Output, value: u8) -> Result<(), DisplayError> {
        deliver_output(self.controls.state, field, value);
        Ok(())
    }
    fn follow_transition(&mut self) -> Result<(), DisplayError> {
        self.effects.follow_transition(self.controls.state)
    }
}

/// Apply one actual delivered byte. Staged mirrors are never rewritten here.
/// Timing byte publications retain the native nine-bit masking and order.
pub fn deliver_output(state: &mut DisplayState, field: Output, value: u8) {
    match field {
        Output::Background3Map => {
            state.effective.background3_map_wider = value & 1 != 0;
            state.effective.background3_map_higher = value & 2 != 0;
            state.effective.background3_map_words = u16::from(value & 0xfc) << 8;
        }
        Output::ColorWindow => {
            state.effective.direct_color = value & 1 != 0;
            state.effective.color_add_subscreen = value & 2 != 0;
            state.effective.color_prevent_math = (value >> 4) & 3;
            state.effective.color_clip = (value >> 6) & 3;
        }
        Output::ColorArithmetic => {
            state.effective.color_subtract = value & 0x80 != 0;
            state.effective.color_half = value & 0x40 != 0;
            for (layer, enabled) in state.effective.color_math_enabled.iter_mut().enumerate() {
                *enabled = value & (1 << layer) != 0;
            }
        }
        Output::MainLayers => state.effective.main_layers = value,
        Output::VerticalLow => {
            state.interrupt.vertical = (state.interrupt.vertical & 0x100) | u16::from(value);
        }
        Output::VerticalHigh => {
            state.interrupt.vertical =
                (state.interrupt.vertical & 0xff) | (u16::from(value & 1) << 8);
        }
        Output::HorizontalLow => {
            state.interrupt.horizontal = (state.interrupt.horizontal & 0x100) | u16::from(value);
        }
        Output::HorizontalHigh => {
            state.interrupt.horizontal =
                (state.interrupt.horizontal & 0xff) | (u16::from(value & 1) << 8);
        }
        Output::InterruptEnable => super::delivery::interrupt_enable(state, value),
    }
}

pub fn owned_advance<E: OwnedIrqEffects>(
    state: &mut DisplayState,
    effects: &mut E,
) -> Result<(), DisplayError> {
    advance(&mut OwnedHost {
        controls: OwnedControls { state },
        effects,
    })
}

fn pending(host: &mut impl Controls, fallback: u16) -> u16 {
    let next = host.read_control(Control::Pending);
    host.write_control(Store::Pending, 0);
    if next == 0 { fallback } else { next }
}

fn room_layers(host: &mut impl Controls) -> u8 {
    let previous = host.read_control(Control::PreviousCre);
    let current = host.read_control(Control::CurrentCre);
    if (previous | current) & 1 != 0 {
        16
    } else {
        17
    }
}

fn vertical<H: Host>(host: &mut H, value: u16) -> Result<(), H::Error> {
    host.write_display(Output::VerticalLow, value as u8)?;
    host.write_display(Output::VerticalHigh, (value >> 8) as u8)
}

fn horizontal<H: Host>(host: &mut H, value: u16) -> Result<(), H::Error> {
    host.write_display(Output::HorizontalLow, value as u8)?;
    host.write_display(Output::HorizontalHigh, (value >> 8) as u8)
}

fn schedule<H: Host>(
    host: &mut H,
    next: u16,
    next_vertical: u16,
    next_horizontal: u16,
) -> Result<(), H::Error> {
    host.write_control(Store::Current, next);
    vertical(host, next_vertical)?;
    horizontal(host, next_horizontal)
}

/// Changes only the source's full-width enable mirror; no immediate delivery.
pub fn disable(host: &mut impl Controls) {
    let enable = host.read_control(Control::Enable);
    host.write_control(Store::Enable, enable & !0x30);
}

pub fn enable<H: Host>(host: &mut H, immediate: bool) -> Result<(), H::Error> {
    vertical(host, 0)?;
    horizontal(host, 152)?;
    let enable = host.read_control(Control::Enable);
    host.write_control(Store::Enable, enable | 0x30);
    if immediate {
        let value = host.read_mirror(Mirror::InterruptEnable);
        host.write_display(Output::InterruptEnable, value)?;
    }
    Ok(())
}

fn restore<H: Host>(host: &mut H, mirror: Mirror, output: Output) -> Result<(), H::Error> {
    let value = host.read_mirror(mirror);
    host.write_display(output, value)
}

/// Retains the original invalid-handler panic before pending/state/display effects.
/// Host failures and Rust unwinding preserve all completed writes and callbacks.
pub fn advance<H: Host>(host: &mut H) -> Result<(), H::Error> {
    let handler = host.read_control(Control::Current) >> 1;
    let (next, next_vertical, next_horizontal) = match handler {
        0 => (pending(host, 0), 0, 0),
        1 => {
            disable(host);
            (0, 0, 0)
        }
        2 => {
            host.write_display(Output::Background3Map, 0x5a)?;
            host.write_display(Output::ColorWindow, 0)?;
            host.write_display(Output::ColorArithmetic, 0)?;
            host.write_display(Output::MainLayers, 4)?;
            (6, 31, 152)
        }
        3 => {
            restore(host, Mirror::ColorWindow, Output::ColorWindow)?;
            restore(host, Mirror::ColorArithmetic, Output::ColorArithmetic)?;
            restore(host, Mirror::Background3Map, Output::Background3Map)?;
            restore(host, Mirror::MainLayers, Output::MainLayers)?;
            (pending(host, 4), 0, 152)
        }
        4 => {
            host.write_display(Output::Background3Map, 0x5a)?;
            host.write_display(Output::MainLayers, 4)?;
            host.write_display(Output::ColorWindow, 0)?;
            host.write_display(Output::ColorArithmetic, 0)?;
            (10, 31, 152)
        }
        5 => {
            let layers = room_layers(host);
            host.write_display(Output::MainLayers, layers)?;
            (pending(host, 8), 0, 152)
        }
        6 | 8 | 11 => {
            host.write_display(Output::MainLayers, 4)?;
            host.write_display(Output::ColorWindow, 0)?;
            host.write_display(Output::ColorArithmetic, 0)?;
            (handler * 2 + 2, 31, 152)
        }
        7 => {
            restore(host, Mirror::Background3Map, Output::Background3Map)?;
            restore(host, Mirror::ColorWindow, Output::ColorWindow)?;
            restore(host, Mirror::ColorArithmetic, Output::ColorArithmetic)?;
            (pending(host, 12), 0, 152)
        }
        9 | 12 => {
            let layers = room_layers(host);
            host.write_display(Output::MainLayers, layers)?;
            host.write_display(Output::ColorWindow, 0)?;
            host.write_display(Output::ColorArithmetic, 0)?;
            if host.read_control(Control::TransitionFlag) & 0x8000 == 0 {
                host.follow_transition()?;
            }
            (handler * 2 + 2, if handler == 9 { 216 } else { 160 }, 152)
        }
        10 | 13 => (pending(host, handler * 2 - 4), 0, 152),
        _ => panic!("invalid scanline IRQ handler: {handler}"),
    };
    schedule(host, next, next_vertical, next_horizontal)
}
