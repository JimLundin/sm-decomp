//! Ordered Samus body position and tile-request protocol, shared by both hosts.
#![forbid(unsafe_code)]

use super::{
    records::SceneMedia,
    samus_body::Half,
    state::{DisplayError, DisplayState, SamusHalf, SourceSpan, TransferSource, WordOrder},
};

pub(crate) trait PositionHost {
    type Error;
    fn ceres(&mut self) -> Result<u16, Self::Error>;
    fn player_x(&mut self) -> Result<u16, Self::Error>;
    fn player_y(&mut self) -> Result<u16, Self::Error>;
    fn set_player_x(&mut self, value: u16) -> Result<(), Self::Error>;
    fn set_player_y(&mut self, value: u16) -> Result<(), Self::Error>;
    fn project(&mut self, x: u16, y: u16) -> Result<(u16, u16), Self::Error>;
    fn frame(&mut self) -> Result<u16, Self::Error>;
    fn movement(&mut self) -> Result<u8, Self::Error>;
    fn landing_word(&self, bytes: [u8; 2]) -> u16;
    fn landing_offset(&mut self, offset: usize) -> Result<u16, Self::Error> {
        Ok(self.landing_word(default_landing_bytes(offset)))
    }
    fn crouching_offset(&mut self, offset: usize) -> Result<i8, Self::Error> {
        Ok(default_crouching_byte(offset))
    }
    fn drained_offset(&mut self, offset: usize) -> Result<i8, Self::Error> {
        Ok(default_drained_byte(offset))
    }
    fn fallback_byte(&mut self, offset: usize) -> Result<u8, Self::Error>;
    fn camera_x(&mut self) -> Result<u16, Self::Error>;
    fn camera_y(&mut self) -> Result<u16, Self::Error>;
    fn set_body_x(&mut self, value: u16) -> Result<(), Self::Error>;
    fn set_body_y(&mut self, value: u16) -> Result<(), Self::Error>;
    fn body_x(&mut self) -> Result<u16, Self::Error>;
    fn body_y(&mut self) -> Result<u16, Self::Error>;
}
pub(crate) fn default_landing_bytes(offset: usize) -> [u8; 2] {
    static LANDING: [u8; 16] = [3, 6, 0, 0, 3, 6, 0, 0, 3, 3, 6, 0, 3, 3, 6, 0];
    [LANDING[offset], LANDING[offset + 1]]
}
pub(crate) fn default_crouching_byte(offset: usize) -> i8 {
    static CROUCHING: [i8; 24] = [
        -8, 0, -8, 0, -4, -2, -4, -2, 0, 0, 0, 0, -4, 0, -4, 0, 5, 4, 5, 4, 0, 0, 0, 0,
    ];
    CROUCHING[offset]
}
pub(crate) fn default_drained_byte(offset: usize) -> i8 {
    static DRAINED: [i8; 32] = [
        7, 5, -8, -8, -8, -8, -8, -5, 4, 4, 4, 4, 0, 0, 4, -3, -5, 0, 0, 4, -3, -5, -3, 4, 0, 0, 4,
        0, 0, 4, 0, 0,
    ];
    DRAINED[offset]
}
pub(crate) fn sprite_y_offset<H: PositionHost>(
    host: &mut H,
    doubled: u16,
) -> Result<i32, H::Error> {
    let pose = doubled >> 1;
    let frame = host.frame()?;
    match host.movement()? {
        0 => {
            if matches!(pose, 0 | 0x9b) {
                if frame.wrapping_sub(2) & 0x8000 == 0 {
                    return Ok(-1);
                }
            } else if (0xa4..0xa8).contains(&pose) {
                let offset = usize::from(frame.wrapping_add(4 * (pose - 0xa4)));
                return Ok(-i32::from(host.landing_offset(offset)?));
            }
        }
        15 if (0x35..0x41).contains(&pose) => {
            return Ok(i32::from(host.crouching_offset(usize::from(
                frame.wrapping_add(2 * (pose - 0x35)),
            ))?));
        }
        27 => {
            if matches!(pose, 0xe8 | 0xe9) {
                return Ok(i32::from(host.drained_offset(usize::from(frame))?));
            }
            if matches!(pose, 0xea | 0xeb) && (frame as i16).wrapping_sub(5) >= 0 {
                return Ok(-3);
            }
        }
        1..=26 => {}
        _ => panic!("invalid Samus movement type"),
    }
    Ok(-i32::from(
        host.fallback_byte(4 + 4 * usize::from(doubled))? as i8,
    ))
}
pub(crate) fn calculate_sprite_position<H: PositionHost>(
    host: &mut H,
    doubled: u16,
) -> Result<(u16, u16), H::Error> {
    let offset = sprite_y_offset(host, doubled)?;
    let y = (i32::from(host.player_y()?) + offset - i32::from(host.camera_y()?)) as u16;
    host.set_body_y(y)?;
    let x = host.player_x()?.wrapping_sub(host.camera_x()?);
    host.set_body_x(x)?;
    let x = host.body_x()?;
    let y = host.body_y()?;
    Ok((x, y))
}
pub(crate) fn sprite_position<H: PositionHost>(
    host: &mut H,
    doubled: u16,
) -> Result<(u16, u16), H::Error> {
    if host.ceres()? & 0x8000 == 0 {
        return calculate_sprite_position(host, doubled);
    }
    let old_x = host.player_x()?;
    let old_y = host.player_y()?;
    let (x, y) = host.project(old_x, old_y)?;
    host.set_player_x(x)?;
    host.set_player_y(y)?;
    let result = calculate_sprite_position(host, doubled)?;
    host.set_player_y(old_y)?;
    host.set_player_x(old_x)?;
    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileTable {
    Pose,
    Top,
    Bottom,
}
pub(crate) trait Table {
    type Error;
    fn native_word(&self, offset: usize) -> Result<u16, Self::Error>;
    fn byte(&self, offset: usize) -> Result<u8, Self::Error>;
}
pub(crate) trait TileHost {
    type Error;
    type Table: Table<Error = Self::Error>;
    fn table(&mut self, which: TileTable) -> Result<Self::Table, Self::Error>;
    fn pose(&mut self) -> Result<u16, Self::Error>;
    fn frame(&mut self) -> Result<u16, Self::Error>;
    fn definition(&mut self, cursor: u16) -> Result<Self::Table, Self::Error>;
    fn request_word(&mut self, half: Half, pointer: u16) -> Result<(), Self::Error>;
    fn request_flag(&mut self, half: Half) -> Result<(), Self::Error>;
    fn request_written(&mut self, half: Half, pointer: u16) -> Result<(), Self::Error>;
}
pub(crate) fn queue_body_tiles<H: TileHost>(host: &mut H) -> Result<(), H::Error> {
    let table = host.table(TileTable::Pose)?;
    let pose = host.pose()?;
    let base = table.native_word(usize::from(pose) * 2)?;
    let cursor = base.wrapping_add(host.frame()?.wrapping_mul(4));
    let definition = host.definition(cursor)?;
    let top_offset = u16::from(definition.byte(1)?) * 7;
    let top_table = host.table(TileTable::Top)?;
    let top_index = definition.byte(0)?;
    let top = top_offset.wrapping_add(top_table.native_word(usize::from(top_index) * 2)?);
    host.request_word(Half::Top, top)?;
    host.request_flag(Half::Top)?;
    host.request_written(Half::Top, top)?;
    if definition.byte(2)? != 255 {
        let bottom_offset = u16::from(definition.byte(3)?) * 7;
        let bottom_table = host.table(TileTable::Bottom)?;
        let bottom_index = definition.byte(2)?;
        let bottom =
            bottom_offset.wrapping_add(bottom_table.native_word(usize::from(bottom_index) * 2)?);
        host.request_word(Half::Bottom, bottom)?;
        host.request_flag(Half::Bottom)?;
        host.request_written(Half::Bottom, bottom)?;
    }
    Ok(())
}

/// An immutable source captured at the original table/definition observation.
/// Its caller supplies and validates content identity and native word order.
#[derive(Clone, Debug)]
pub struct SourceTable {
    pub source: SourceSpan,
    pub native_order: WordOrder,
}
impl Table for SourceTable {
    type Error = DisplayError;
    fn native_word(&self, offset: usize) -> Result<u16, DisplayError> {
        let low = self.byte(offset)?;
        let high = self.byte(offset.checked_add(1).ok_or(DisplayError::Source)?)?;
        Ok(self.native_order.decode([low, high]))
    }
    fn byte(&self, offset: usize) -> Result<u8, DisplayError> {
        let at = self
            .source
            .offset
            .checked_add(offset)
            .ok_or(DisplayError::Source)?;
        self.source
            .bytes
            .get(at)
            .copied()
            .ok_or(DisplayError::Source)
    }
}
/// Authored request associated with a compatibility pointer by the G content
/// owner. Neither the descriptor nor transfer identities are decoded from it.
#[derive(Clone, Debug)]
pub struct TileRequest {
    pub pointer: u16,
    pub half: SamusHalf,
}
impl TileRequest {
    pub(crate) fn validate(&self, pointer: u16, media: &SceneMedia) -> Result<(), DisplayError> {
        if self.pointer != pointer || self.half.enabled != 1 {
            return Err(DisplayError::Incomplete);
        }
        media
            .asset(self.half.descriptor)
            .map_err(|_| DisplayError::Identity)?;
        for (transfer, active) in [
            (&self.half.first, true),
            (&self.half.second, self.half.second.count != 0),
        ] {
            transfer.validate(media.revision())?;
            let TransferSource::Asset(source) = &transfer.source else {
                return Err(DisplayError::Incomplete);
            };
            source.validate(media.revision())?;
            source.validate_media(media)?;
            if active && source.bytes.len() - source.offset < transfer.byte_count() {
                return Err(DisplayError::Source);
            }
        }
        Ok(())
    }
}
/// G provides its actual position owner and validated selected content/request
/// bindings. Publications perform original ordered writes through that owner;
/// pointer values are compatibility selectors, never inferred media identities.
pub(crate) trait Inputs {
    fn ceres(&mut self) -> Result<u16, DisplayError>;
    fn player_x(&mut self) -> Result<u16, DisplayError>;
    fn player_y(&mut self) -> Result<u16, DisplayError>;
    fn set_player_x(&mut self, state: &mut DisplayState, value: u16) -> Result<(), DisplayError>;
    fn set_player_y(&mut self, state: &mut DisplayState, value: u16) -> Result<(), DisplayError>;
    fn project(
        &mut self,
        state: &mut DisplayState,
        x: u16,
        y: u16,
    ) -> Result<(u16, u16), DisplayError>;
    fn animation_frame(&mut self) -> Result<u16, DisplayError>;
    fn movement(&mut self) -> Result<u8, DisplayError>;
    fn landing_order(&self) -> WordOrder;
    fn position_source(&mut self) -> Result<SourceTable, DisplayError>;
    fn camera_x(&mut self) -> Result<u16, DisplayError>;
    fn camera_y(&mut self) -> Result<u16, DisplayError>;
    fn set_body_x(&mut self, state: &mut DisplayState, value: u16) -> Result<(), DisplayError>;
    fn set_body_y(&mut self, state: &mut DisplayState, value: u16) -> Result<(), DisplayError>;
    fn body_x(&mut self) -> Result<u16, DisplayError>;
    fn body_y(&mut self) -> Result<u16, DisplayError>;
    fn pose(&mut self) -> Result<u16, DisplayError>;
    fn tile_table(&mut self, which: TileTable) -> Result<SourceTable, DisplayError>;
    fn tile_definition(
        &mut self,
        state: &mut DisplayState,
        cursor: u16,
    ) -> Result<SourceTable, DisplayError>;
    /// Publish the compatibility pointer low byte/observer/high byte/observer
    /// through the same owner, retaining prefixes before metadata admission.
    fn request_word(
        &mut self,
        state: &mut DisplayState,
        half: Half,
        pointer: u16,
    ) -> Result<(), DisplayError>;
    fn request_flag(&mut self, state: &mut DisplayState, half: Half) -> Result<(), DisplayError>;
    fn tile_request(
        &mut self,
        state: &DisplayState,
        half: Half,
        pointer: u16,
    ) -> Result<TileRequest, DisplayError>;
}
struct Owned<'a, I> {
    state: &'a mut DisplayState,
    media: &'a SceneMedia,
    inputs: &'a mut I,
}
pub(crate) fn position_owned(
    state: &mut DisplayState,
    media: &SceneMedia,
    doubled: u16,
    inputs: &mut impl Inputs,
) -> Result<(u16, u16), DisplayError> {
    sprite_position(
        &mut Owned {
            state,
            media,
            inputs,
        },
        doubled,
    )
}
pub(crate) fn tiles_owned(
    state: &mut DisplayState,
    media: &SceneMedia,
    inputs: &mut impl Inputs,
) -> Result<(), DisplayError> {
    queue_body_tiles(&mut Owned {
        state,
        media,
        inputs,
    })
}
impl<I: Inputs> PositionHost for Owned<'_, I> {
    type Error = DisplayError;
    fn ceres(&mut self) -> Result<u16, DisplayError> {
        self.inputs.ceres()
    }
    fn player_x(&mut self) -> Result<u16, DisplayError> {
        self.inputs.player_x()
    }
    fn player_y(&mut self) -> Result<u16, DisplayError> {
        self.inputs.player_y()
    }
    fn set_player_x(&mut self, v: u16) -> Result<(), DisplayError> {
        self.inputs.set_player_x(self.state, v)
    }
    fn set_player_y(&mut self, v: u16) -> Result<(), DisplayError> {
        self.inputs.set_player_y(self.state, v)
    }
    fn project(&mut self, x: u16, y: u16) -> Result<(u16, u16), DisplayError> {
        self.inputs.project(self.state, x, y)
    }
    fn frame(&mut self) -> Result<u16, DisplayError> {
        self.inputs.animation_frame()
    }
    fn movement(&mut self) -> Result<u8, DisplayError> {
        self.inputs.movement()
    }
    fn landing_word(&self, b: [u8; 2]) -> u16 {
        self.inputs.landing_order().decode(b)
    }
    fn fallback_byte(&mut self, o: usize) -> Result<u8, DisplayError> {
        let source = self.inputs.position_source()?;
        source.source.validate(self.media.revision())?;
        source.byte(o)
    }
    fn camera_x(&mut self) -> Result<u16, DisplayError> {
        self.inputs.camera_x()
    }
    fn camera_y(&mut self) -> Result<u16, DisplayError> {
        self.inputs.camera_y()
    }
    fn set_body_x(&mut self, v: u16) -> Result<(), DisplayError> {
        self.inputs.set_body_x(self.state, v)
    }
    fn set_body_y(&mut self, v: u16) -> Result<(), DisplayError> {
        self.inputs.set_body_y(self.state, v)
    }
    fn body_x(&mut self) -> Result<u16, DisplayError> {
        self.inputs.body_x()
    }
    fn body_y(&mut self) -> Result<u16, DisplayError> {
        self.inputs.body_y()
    }
}
impl<I: Inputs> TileHost for Owned<'_, I> {
    type Error = DisplayError;
    type Table = SourceTable;
    fn table(&mut self, w: TileTable) -> Result<SourceTable, DisplayError> {
        let table = self.inputs.tile_table(w)?;
        table.source.validate(self.media.revision())?;
        Ok(table)
    }
    fn pose(&mut self) -> Result<u16, DisplayError> {
        self.inputs.pose()
    }
    fn frame(&mut self) -> Result<u16, DisplayError> {
        self.inputs.animation_frame()
    }
    fn definition(&mut self, c: u16) -> Result<SourceTable, DisplayError> {
        let table = self.inputs.tile_definition(self.state, c)?;
        table.source.validate(self.media.revision())?;
        Ok(table)
    }
    fn request_word(&mut self, h: Half, p: u16) -> Result<(), DisplayError> {
        self.inputs.request_word(self.state, h, p)
    }
    fn request_flag(&mut self, h: Half) -> Result<(), DisplayError> {
        self.inputs.request_flag(self.state, h)
    }
    fn request_written(&mut self, half: Half, pointer: u16) -> Result<(), DisplayError> {
        let request = self.inputs.tile_request(self.state, half, pointer)?;
        request.validate(pointer, self.media)?;
        let slot = match half {
            Half::Top => 0,
            Half::Bottom => 1,
        };
        self.state.samus[slot] = request.half;
        Ok(())
    }
}
