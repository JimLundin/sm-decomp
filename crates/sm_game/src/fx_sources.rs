//! Live HDMA sources whose window-control lanes belong to the current menu owner.
#![forbid(unsafe_code)]

use crate::menu_window_state::{Field, WindowState};
use crate::scene::hdma_sources::{HdmaLiveDescription, LiveOwner, LiveSources};
use crate::scene::state::{DisplayError, HdmaState, WordOrder};

/// Supplied alias coordinates in one canonical source allocation. These are
/// buffer offsets, independent of physical banks, addresses and native pointers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowControlLayout {
    pub storage: LiveOwner,
    /// Field::ALL order, each field's low byte then high byte. Absent lanes
    /// are outside this source allocation; no substitute byte is supplied.
    pub lanes: [Option<usize>; 38],
}
impl WindowControlLayout {
    pub fn lane(&self, storage: LiveOwner, offset: usize) -> Option<(Field, usize)> {
        if storage != self.storage {
            return None;
        }
        self.lanes.iter().enumerate().find_map(|(index, &at)| {
            (at == Some(offset)).then_some((Field::ALL[index / 2], index % 2))
        })
    }
}

/// The source buffer and the window state are exclusive borrowed authorities.
/// Excluded buffer lanes are structural placeholders, never window byte copies.
/// Canonical-region resolution makes overlapping source roles see the same owner.
pub struct WindowSources<'a> {
    pub live: &'a mut HdmaLiveDescription,
    pub window: &'a mut WindowState,
}
impl LiveSources for WindowSources<'_> {
    fn length(&self, owner: LiveOwner) -> Result<usize, DisplayError> {
        LiveSources::length(self.live, owner)
    }
    fn read(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
    ) -> Result<u8, DisplayError> {
        if let Some((field, lane)) = self.live.window_control_lane(owner, offset)? {
            if !self.window.is_initialized() {
                return Err(DisplayError::Incomplete);
            }
            return Ok(field.get(self.window).to_le_bytes()[lane]);
        }
        self.live.read(state, owner, offset)
    }
    fn write(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
        value: u8,
    ) -> Result<(), DisplayError> {
        if let Some((field, lane)) = self.live.window_control_lane(owner, offset)? {
            if !self.window.is_initialized() {
                return Err(DisplayError::Incomplete);
            }
            let word = field.get_mut(self.window);
            let mut bytes = word.to_le_bytes();
            bytes[lane] = value;
            *word = u16::from_le_bytes(bytes);
            return Ok(());
        }
        self.live.write(state, owner, offset, value)
    }
}

/// Every actual display source query borrows the current workspace and window.
/// The borrow ends before HDMA calls a destination effect or resolves another byte.
pub struct DisplaySources<'a> {
    pub live: &'a mut HdmaLiveDescription,
    pub window: Option<&'a mut WindowState>,
    pub lighting: &'a mut crate::visual_state::FirefleaLighting,
    pub workspace: &'a mut crate::presentation_workspace::Workspace,
}
impl DisplaySources<'_> {
    fn read_scalar(&self, state: &HdmaState, lane: usize) -> Result<u8, DisplayError> {
        let at = crate::fx_encoding::control_address(lane);
        match lane {
            26..=37 => crate::fx_encoding::projected_lighting_byte(self.lighting, at)
                .ok_or(DisplayError::Incomplete),
            38..=75 => {
                let window = self
                    .window
                    .as_deref()
                    .filter(|window| window.is_initialized())
                    .ok_or(DisplayError::Incomplete)?;
                crate::menu_window_encoding::projected_byte(window, at)
                    .ok_or(DisplayError::Incomplete)
            }
            76..=247 => Ok(
                crate::fx_encoding::common_scalar_word(&state.common, at & !1).to_le_bytes()
                    [at % 2],
            ),
            _ => Err(DisplayError::Incomplete),
        }
    }
    fn write_scalar(
        &mut self,
        state: &mut HdmaState,
        lane: usize,
        byte: u8,
    ) -> Result<(), DisplayError> {
        let at = crate::fx_encoding::control_address(lane);
        match lane {
            26..=37 => {
                if !crate::fx_encoding::write_lighting_byte(self.lighting, at, byte) {
                    return Err(DisplayError::Incomplete);
                }
            }
            38..=75 => {
                let window = self
                    .window
                    .as_deref_mut()
                    .filter(|window| window.is_initialized())
                    .ok_or(DisplayError::Incomplete)?;
                if !crate::menu_window_encoding::write_byte(window, at, byte) {
                    return Err(DisplayError::Incomplete);
                }
            }
            76..=247 => {
                let word = crate::fx_encoding::common_scalar_word_mut(&mut state.common, at & !1);
                let mut bytes = word.to_le_bytes();
                bytes[at % 2] = byte;
                *word = u16::from_le_bytes(bytes);
            }
            _ => return Err(DisplayError::Incomplete),
        }
        Ok(())
    }
}
impl LiveSources for DisplaySources<'_> {
    fn length(&self, owner: LiveOwner) -> Result<usize, DisplayError> {
        let length = LiveSources::length(self.live, owner)?;
        if let Some(point) = self.live.workspace_point(owner, 0)?
            && self.workspace.available(point)? < length
        {
            return Err(DisplayError::Source);
        }
        Ok(length)
    }
    fn read(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
    ) -> Result<u8, DisplayError> {
        match crate::fx_encoding::current_source_owner(
            self.live,
            self.workspace,
            state,
            owner,
            offset,
        )? {
            crate::fx_encoding::ControlOwner::Scalar(lane) => self.read_scalar(state, lane),
            crate::fx_encoding::ControlOwner::Workspace(point) => {
                self.workspace.read_source_point(point)
            }
            crate::fx_encoding::ControlOwner::Source(owner, offset) => {
                self.live.read_buffer(owner, offset)
            }
        }
    }
    fn write(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
        value: u8,
    ) -> Result<(), DisplayError> {
        let selected = crate::fx_encoding::current_source_owner(
            self.live,
            self.workspace,
            state,
            owner,
            offset,
        )?;
        match selected {
            crate::fx_encoding::ControlOwner::Scalar(lane) => {
                self.write_scalar(state, lane, value)?
            }
            crate::fx_encoding::ControlOwner::Workspace(point) => {
                self.workspace.write_source_point(point, value)?
            }
            crate::fx_encoding::ControlOwner::Source(owner, offset) => {
                self.live.write_buffer(owner, offset, value)?
            }
        }
        forget_current_origins(self.live, self.workspace, state, selected);
        Ok(())
    }
}

/// Native and LE stores retain the original first byte before a second-byte
/// failure. No whole-word preflight, rollback, sorted writes or fresh selector.
pub fn write_word(
    sources: &mut impl LiveSources,
    state: &mut HdmaState,
    owner: LiveOwner,
    offset: usize,
    order: WordOrder,
    value: u16,
) -> Result<(), DisplayError> {
    let bytes = order.encode(value);
    sources.write(state, owner, offset, bytes[0])?;
    let next = offset
        .checked_add(1)
        .ok_or(DisplayError::Alias { offset })?;
    sources.write(state, owner, next, bytes[1])
}

pub fn read_word(
    sources: &mut impl LiveSources,
    state: &mut HdmaState,
    owner: LiveOwner,
    offset: usize,
    order: WordOrder,
) -> Result<u16, DisplayError> {
    let first = sources.read(state, owner, offset)?;
    let next = offset
        .checked_add(1)
        .ok_or(DisplayError::Alias { offset })?;
    let second = sources.read(state, owner, next)?;
    Ok(order.decode([first, second]))
}

/// Scalar fields outside DisplayState, borrowed for the actual settlement
/// interval. Common HDMA and lighting remain borrowed directly from that state.
pub(crate) struct ExternalControls<'a> {
    pub fx: &'a mut crate::fx_state::FxState,
    pub window: &'a mut WindowState,
    pub room_primary: &'a mut u16,
    pub room_liquid: &'a mut u16,
    pub ownership: &'a mut crate::fx_state::ControlOwnership,
}
pub(crate) struct CurrentSources<'a, 'b> {
    pub live: &'a mut HdmaLiveDescription,
    pub lighting: &'a mut crate::visual_state::FirefleaLighting,
    pub workspace: &'a mut crate::presentation_workspace::Workspace,
    pub controls: &'a mut ExternalControls<'b>,
}
impl CurrentSources<'_, '_> {
    pub(crate) fn scalar_byte(
        &self,
        common: &crate::hdma::HdmaObjects,
        lane: usize,
    ) -> Result<u8, DisplayError> {
        let at = crate::fx_encoding::control_address(lane);
        let byte = match lane {
            0..=1 => self.controls.room_primary.to_le_bytes()[lane],
            2..=3 => self.controls.room_liquid.to_le_bytes()[lane - 2],
            4..=25 => crate::fx_encoding::projected_byte(self.controls.fx, at)
                .ok_or(DisplayError::Source)?,
            26..=37 => crate::fx_encoding::projected_lighting_byte(self.lighting, at)
                .ok_or(DisplayError::Source)?,
            38..=75 => {
                if !self.controls.window.is_initialized() {
                    return Err(DisplayError::Incomplete);
                }
                crate::menu_window_encoding::projected_byte(self.controls.window, at)
                    .ok_or(DisplayError::Source)?
            }
            _ => crate::fx_encoding::common_scalar_word(common, at & !1).to_le_bytes()[at % 2],
        };
        Ok(byte)
    }
    pub(crate) fn store_scalar_byte(
        &mut self,
        common: &mut crate::hdma::HdmaObjects,
        lane: usize,
        byte: u8,
    ) -> Result<(), DisplayError> {
        let at = crate::fx_encoding::control_address(lane);
        let (word, byte_lane) = match lane {
            0..=1 => (&mut *self.controls.room_primary, lane),
            2..=3 => (&mut *self.controls.room_liquid, lane - 2),
            4..=25 => {
                assert!(crate::fx_encoding::write_byte(self.controls.fx, at, byte));
                return Ok(());
            }
            26..=37 => {
                assert!(crate::fx_encoding::write_lighting_byte(
                    self.lighting,
                    at,
                    byte
                ));
                return Ok(());
            }
            38..=75 => {
                if !self.controls.window.is_initialized() {
                    return Err(DisplayError::Incomplete);
                }
                assert!(crate::menu_window_encoding::write_byte(
                    self.controls.window,
                    at,
                    byte
                ));
                return Ok(());
            }
            _ => (
                crate::fx_encoding::common_scalar_word_mut(common, at & !1),
                at % 2,
            ),
        };
        let mut bytes = word.to_le_bytes();
        bytes[byte_lane] = byte;
        *word = u16::from_le_bytes(bytes);
        Ok(())
    }
}
impl LiveSources for CurrentSources<'_, '_> {
    fn length(&self, owner: LiveOwner) -> Result<usize, DisplayError> {
        LiveSources::length(self.live, owner)
    }
    fn read(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
    ) -> Result<u8, DisplayError> {
        use crate::fx_encoding::ControlOwner;
        match crate::fx_encoding::current_source_owner(
            self.live,
            self.workspace,
            state,
            owner,
            offset,
        )? {
            ControlOwner::Workspace(point) => self.workspace.read_source_point(point),
            ControlOwner::Scalar(lane) => self.scalar_byte(&state.common, lane),
            ControlOwner::Source(owner, offset) => self.live.read_buffer(owner, offset),
        }
    }
    fn write(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
        byte: u8,
    ) -> Result<(), DisplayError> {
        use crate::fx_encoding::ControlOwner;
        let selected = crate::fx_encoding::current_source_owner(
            self.live,
            self.workspace,
            state,
            owner,
            offset,
        )?;
        match selected {
            ControlOwner::Workspace(point) => self.workspace.write_source_point(point, byte)?,
            ControlOwner::Scalar(lane) => self.store_scalar_byte(&mut state.common, lane, byte)?,
            ControlOwner::Source(owner, offset) => self.live.write_buffer(owner, offset, byte)?,
        }
        let mut origins = Vec::new();
        for &(owner, offset) in self.live.native_origins().keys() {
            if crate::fx_encoding::current_source_owner(
                self.live,
                self.workspace,
                state,
                owner,
                offset,
            )
            .is_ok_and(|owner| owner == selected)
            {
                origins.push((owner, offset));
            }
        }
        for (owner, offset) in origins {
            self.live.forget_native_origin(owner, offset)?;
        }
        Ok(())
    }
}

impl CurrentSources<'_, '_> {
    pub(crate) fn read_control(&self, state: &HdmaState, lane: usize) -> Result<u8, DisplayError> {
        match crate::fx_encoding::current_control_owner(self.live, self.workspace, state, lane)? {
            crate::fx_encoding::ControlOwner::Scalar(lane) => self.scalar_byte(&state.common, lane),
            crate::fx_encoding::ControlOwner::Workspace(point) => {
                self.workspace.read_source_point(point)
            }
            crate::fx_encoding::ControlOwner::Source(owner, offset) => {
                self.live.read_buffer(owner, offset)
            }
        }
    }
    pub(crate) fn write_control(
        &mut self,
        state: &mut HdmaState,
        lane: usize,
        value: u8,
    ) -> Result<(), DisplayError> {
        let selected =
            crate::fx_encoding::current_control_owner(self.live, self.workspace, state, lane)?;
        match selected {
            crate::fx_encoding::ControlOwner::Scalar(lane) => {
                self.store_scalar_byte(&mut state.common, lane, value)?
            }
            crate::fx_encoding::ControlOwner::Workspace(point) => {
                self.workspace.write_source_point(point, value)?
            }
            crate::fx_encoding::ControlOwner::Source(owner, offset) => {
                self.live.write_buffer(owner, offset, value)?
            }
        }
        forget_current_origins(self.live, self.workspace, state, selected);
        Ok(())
    }
}
pub(crate) fn forget_current_origins(
    live: &mut HdmaLiveDescription,
    workspace: &crate::presentation_workspace::Workspace,
    state: &HdmaState,
    selected: crate::fx_encoding::ControlOwner,
) {
    let origins: Vec<_> = live
        .native_origins()
        .keys()
        .copied()
        .filter(|&(owner, offset)| {
            crate::fx_encoding::current_source_owner(live, workspace, state, owner, offset)
                .is_ok_and(|owner| owner == selected)
        })
        .collect();
    for (owner, offset) in origins {
        live.forget_native_origin(owner, offset)
            .expect("existing source coordinate");
    }
}
/// The actual successful Workspace publication retires stale native selection
/// for its source class; no bytes, epochs or publication logs are copied.
pub(crate) fn publish_workspace_byte(
    display: &mut crate::scene::state::DisplayState,
    point: crate::presentation_workspace::Point,
    value: u8,
) -> Result<(), DisplayError> {
    display.workspace.write_byte(point, value)?;
    let selected = display.workspace.source_coordinate(point)?;
    forget_current_origins(
        &mut display.hdma_live,
        &display.workspace,
        &display.hdma,
        crate::fx_encoding::ControlOwner::Workspace(selected),
    );
    Ok(())
}
pub(crate) fn publish_workspace_slice(
    display: &mut crate::scene::state::DisplayState,
    point: crate::presentation_workspace::Point,
    bytes: &[u8],
) -> Result<(), DisplayError> {
    display.workspace.write_slice(point, bytes)?;
    if bytes.is_empty() {
        return Ok(());
    }
    let first = display.workspace.source_coordinate(point)?;
    let end = first
        .offset
        .checked_add(bytes.len())
        .expect("successful Workspace slice");
    let origins: Vec<_> = display
        .hdma_live
        .native_origins()
        .keys()
        .copied()
        .filter(|&(owner, offset)| {
            crate::fx_encoding::current_source_owner(
                &display.hdma_live,
                &display.workspace,
                &display.hdma,
                owner,
                offset,
            )
            .is_ok_and(|selected| {
                matches!(selected, crate::fx_encoding::ControlOwner::Workspace(at)
                if at.origin == first.origin && (first.offset..end).contains(&at.offset))
            })
        })
        .collect();
    for (owner, offset) in origins {
        display
            .hdma_live
            .forget_native_origin(owner, offset)
            .expect("existing source coordinate");
    }
    Ok(())
}

pub(crate) fn forget_workspace_range(
    live: &mut HdmaLiveDescription,
    state: &HdmaState,
    workspace: &crate::presentation_workspace::Workspace,
    first: crate::presentation_workspace::SourcePoint,
    bytes: usize,
) {
    let end = first
        .offset
        .checked_add(bytes)
        .expect("successful Workspace slice");
    let origins: Vec<_> = live
        .native_origins()
        .keys()
        .copied()
        .filter(|&(owner, offset)| {
            crate::fx_encoding::current_source_owner(live, workspace, state, owner, offset)
                .is_ok_and(|selected| {
                    matches!(selected, crate::fx_encoding::ControlOwner::Workspace(at)
                if at.origin == first.origin && (first.offset..end).contains(&at.offset))
                })
        })
        .collect();
    for (owner, offset) in origins {
        live.forget_native_origin(owner, offset)
            .expect("existing source coordinate");
    }
}
