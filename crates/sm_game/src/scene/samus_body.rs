//! Shared Samus body selection, ordered drawing and direct front-facing marker.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Normal,
    Frozen,
    Death,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Half {
    Top,
    Bottom,
}
pub trait Table {
    type Error;
    fn little_word(&self, offset: usize) -> Result<u16, Self::Error>;
    fn native_word(&self, offset: usize) -> Result<u16, Self::Error>;
}
pub trait Host {
    type Error;
    type Table: Table<Error = Self::Error>;
    fn knockback(&mut self) -> Result<u16, Self::Error>;
    fn invincibility(&mut self) -> Result<u16, Self::Error>;
    fn shine(&mut self) -> Result<u16, Self::Error>;
    fn frame_counter(&mut self) -> Result<u16, Self::Error>;
    fn pose(&mut self) -> Result<u16, Self::Error>;
    fn animation_frame(&mut self) -> Result<u16, Self::Error>;
    fn table(&mut self, half: Half) -> Result<Self::Table, Self::Error>;
    fn set_sprite(&mut self, half: Half, index: u16) -> Result<(), Self::Error>;
    fn sprite_position(&mut self, doubled_pose: u16) -> Result<(u16, u16), Self::Error>;
    fn body_x(&mut self) -> Result<u16, Self::Error>;
    fn body_y(&mut self) -> Result<u16, Self::Error>;
    fn camera_x(&mut self) -> Result<u16, Self::Error>;
    fn camera_y(&mut self) -> Result<u16, Self::Error>;
    fn draw_sprite(&mut self, index: u16, x: u16, y: u16) -> Result<(), Self::Error>;
    fn bottom_visible(&mut self) -> Result<bool, Self::Error>;
    fn queue_tiles(&mut self) -> Result<(), Self::Error>;
}
pub fn sprite_index<H: Host>(
    host: &mut H,
    half: Half,
    pose: u16,
    little: bool,
) -> Result<u16, H::Error> {
    let frame = host.animation_frame()?;
    let table = host.table(half)?;
    let offset = usize::from(pose) * 2;
    let base = if little {
        table.little_word(offset)?
    } else {
        table.native_word(offset)?
    };
    Ok(frame.wrapping_add(base))
}
pub fn body_position<H: Host>(host: &mut H, mode: Body) -> Result<(u16, u16), H::Error> {
    let x = host.body_x()?;
    let y = host.body_y()?;
    Ok(if mode == Body::Death {
        (
            x.wrapping_add(host.camera_x()?),
            y.wrapping_add(host.camera_y()?),
        )
    } else {
        (x, y)
    })
}
pub fn draw<H: Host>(host: &mut H, mode: Body) -> Result<(), H::Error> {
    if mode != Body::Normal
        || host.knockback()? != 0
        || host.invincibility()? == 0
        || host.shine()? != 0
        || host.frame_counter()? & 1 == 0
    {
        let pose = host.pose()?;
        let doubled = pose.wrapping_mul(2);
        let top_pose = host.pose()?;
        let top = sprite_index(host, Half::Top, top_pose, mode == Body::Normal)?;
        if mode == Body::Normal {
            host.set_sprite(Half::Top, top)?;
        }
        let position = host.sprite_position(doubled)?;
        let (x, y) = if mode == Body::Death {
            body_position(host, mode)?
        } else {
            position
        };
        host.draw_sprite(top, x, y)?;
        if host.bottom_visible()? {
            let bottom = sprite_index(host, Half::Bottom, doubled >> 1, mode == Body::Normal)?;
            if mode == Body::Normal {
                host.set_sprite(Half::Bottom, bottom)?;
            }
            let (x, y) = body_position(host, mode)?;
            host.draw_sprite(bottom, x, y)?;
        }
    }
    host.queue_tiles()
}

pub trait BottomHost {
    type Error;
    fn pose(&mut self) -> Result<u16, Self::Error>;
    fn animation_frame(&mut self) -> Result<u16, Self::Error>;
    fn movement(&mut self) -> Result<u8, Self::Error>;
    fn clear_bottom(&mut self) -> Result<(), Self::Error>;
    fn cursor(&mut self) -> Result<u16, Self::Error>;
    fn player_x(&mut self) -> Result<u16, Self::Error>;
    fn player_y(&mut self) -> Result<u16, Self::Error>;
    fn camera_x(&mut self) -> Result<u16, Self::Error>;
    fn camera_y(&mut self) -> Result<u16, Self::Error>;
    /// Native low byte then high byte, preserving prefixes without word preflight.
    fn marker_word(&mut self, entry: usize, value: u16) -> Result<(), Self::Error>;
    fn publish_cursor(&mut self, value: u16) -> Result<(), Self::Error>;
    fn marker_written(&mut self, entry: usize, x: u16, y: u16) -> Result<(), Self::Error>;
}
fn marker<H: BottomHost>(host: &mut H) -> Result<(), H::Error> {
    let next = host.cursor()?;
    let entry = usize::from(host.cursor()?);
    let x = (i32::from(host.player_x()?) - 7 - i32::from(host.camera_x()?)) as u16;
    host.marker_word(entry, x)?;
    let y = (i32::from(host.player_y()?) - 17 - i32::from(host.camera_y()?)) as u16;
    host.marker_word(entry + 1, y)?;
    host.marker_word(entry + 2, 14369)?;
    host.publish_cursor(next.wrapping_add(4))?;
    host.marker_written(entry, x, y)
}
pub fn bottom_is_visible<H: BottomHost>(host: &mut H) -> Result<bool, H::Error> {
    let pose = host.pose()?;
    let frame = host.animation_frame()?;
    let visible = match host.movement()? {
        0 => {
            if pose == 0 {
                marker(host)?;
            }
            true
        }
        1 | 2 | 5 | 6 | 11 | 12 | 14 | 16 | 21 | 22 | 23 | 24 | 26 => true,
        3 => {
            matches!(pose, 0x81 | 0x82 | 0x1b | 0x1c)
                || frame == 0
                || frame.wrapping_sub(11) & 0x8000 == 0
        }
        4 | 7 | 8 | 9 | 17 | 18 | 19 => false,
        10 => !matches!(pose, 0xd7 | 0xd8) || frame.wrapping_sub(3) & 0x8000 == 0,
        13 => !matches!(pose, 0x65 | 0x66) || frame.wrapping_sub(1) & 0x8000 != 0,
        15 => {
            if pose.wrapping_sub(0xf1) & 0x8000 == 0 {
                true
            } else if pose.wrapping_sub(219) & 0x8000 == 0 {
                frame
                    == if pose.wrapping_sub(221) & 0x8000 != 0 {
                        0
                    } else {
                        2
                    }
            } else {
                matches!(pose, 0x35 | 0x36 | 0x3b | 0x3c)
            }
        }
        20 => !(3..13).contains(&frame),
        25 => !(2..9).contains(&frame),
        27 => {
            if pose.wrapping_sub(0xcf) & 0x8000 != 0 {
                !matches!(pose, 0xcb | 0xcc)
            } else {
                !matches!(pose, 0xe8 | 0xe9) || frame.wrapping_sub(2) & 0x8000 == 0
            }
        }
        _ => panic!("invalid Samus movement type"),
    };
    if !visible {
        host.clear_bottom()?;
    }
    Ok(visible)
}

use super::{
    records::{
        ObjectSize, ObjectVariants, ProducedObject, ProducerObjectId, SceneMedia, SceneObject,
        ScreenPosition,
    },
    state::{DisplayError, DisplayPhase, DisplayState, WordOrder},
};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyTable {
    pub bytes: Arc<[u8]>,
    pub native_order: WordOrder,
}
impl BodyTable {
    fn bytes(&self, offset: usize) -> Result<[u8; 2], DisplayError> {
        let first = *self.bytes.get(offset).ok_or(DisplayError::Source)?;
        let next = offset.checked_add(1).ok_or(DisplayError::Source)?;
        let second = *self.bytes.get(next).ok_or(DisplayError::Source)?;
        Ok([first, second])
    }
}
impl Table for BodyTable {
    type Error = DisplayError;
    fn little_word(&self, offset: usize) -> Result<u16, DisplayError> {
        Ok(u16::from_le_bytes(self.bytes(offset)?))
    }
    fn native_word(&self, offset: usize) -> Result<u16, DisplayError> {
        Ok(self.native_order.decode(self.bytes(offset)?))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkerPublication {
    pub producer: ProducerObjectId,
    pub variants: Arc<ObjectVariants>,
}
impl MarkerPublication {
    pub fn validate(&self, media: &SceneMedia) -> Result<(), DisplayError> {
        self.variants
            .validate(media, 14369 & 255)
            .map_err(|_| DisplayError::Scene)
    }
}
/// Every observation/effect belongs to its actual G owner; position/queue code
/// is supplied explicitly and is not rebuilt from display or inferred content.
pub trait Inputs: super::samus::Inputs + super::samus_position_tiles::Inputs {
    type BodyTable: Table<Error = DisplayError>;
    fn knockback(&mut self) -> Result<u16, DisplayError>;
    fn invincibility(&mut self) -> Result<u16, DisplayError>;
    fn shine(&mut self) -> Result<u16, DisplayError>;
    fn frame_counter(&mut self) -> Result<u16, DisplayError>;
    fn table(&mut self, half: Half) -> Result<Self::BodyTable, DisplayError>;
    fn set_sprite(&mut self, half: Half, index: u16) -> Result<(), DisplayError>;
}
pub fn draw_owned(
    state: &mut DisplayState,
    media: &SceneMedia,
    producer: ProducerObjectId,
    marker: &MarkerPublication,
    mode: Body,
    inputs: &mut impl Inputs,
) -> Result<(), DisplayError> {
    if state.phase == DisplayPhase::Failed {
        return Err(DisplayError::Poisoned);
    }
    if state.phase != DisplayPhase::Producing {
        return Err(DisplayError::Phase);
    }
    let mut guard = super::delivery::FaultGuard::new(state);
    let result = draw(
        &mut Owned {
            state: guard.state(),
            media,
            producer,
            marker,
            inputs,
        },
        mode,
    );
    if result.is_ok() {
        guard.disarm();
    }
    result
}
pub fn bottom_owned(
    state: &mut DisplayState,
    media: &SceneMedia,
    producer: ProducerObjectId,
    marker: &MarkerPublication,
    inputs: &mut impl Inputs,
) -> Result<bool, DisplayError> {
    if state.phase == DisplayPhase::Failed {
        return Err(DisplayError::Poisoned);
    }
    if state.phase != DisplayPhase::Producing {
        return Err(DisplayError::Phase);
    }
    let mut guard = super::delivery::FaultGuard::new(state);
    let result = bottom_is_visible(&mut Owned {
        state: guard.state(),
        media,
        producer,
        marker,
        inputs,
    });
    if result.is_ok() {
        guard.disarm();
    }
    result
}
struct Owned<'a, I> {
    state: &'a mut DisplayState,
    media: &'a SceneMedia,
    producer: ProducerObjectId,
    marker: &'a MarkerPublication,
    inputs: &'a mut I,
}
impl<I: Inputs> Host for Owned<'_, I> {
    type Error = DisplayError;
    type Table = I::BodyTable;
    fn knockback(&mut self) -> Result<u16, DisplayError> {
        self.inputs.knockback()
    }
    fn invincibility(&mut self) -> Result<u16, DisplayError> {
        self.inputs.invincibility()
    }
    fn shine(&mut self) -> Result<u16, DisplayError> {
        self.inputs.shine()
    }
    fn frame_counter(&mut self) -> Result<u16, DisplayError> {
        self.inputs.frame_counter()
    }
    fn pose(&mut self) -> Result<u16, DisplayError> {
        self.inputs.pose()
    }
    fn animation_frame(&mut self) -> Result<u16, DisplayError> {
        self.inputs.animation_frame()
    }
    fn table(&mut self, half: Half) -> Result<Self::Table, DisplayError> {
        self.inputs.table(half)
    }
    fn set_sprite(&mut self, half: Half, index: u16) -> Result<(), DisplayError> {
        self.inputs.set_sprite(half, index)
    }
    fn sprite_position(&mut self, doubled: u16) -> Result<(u16, u16), DisplayError> {
        super::samus_position_tiles::position_owned(self.state, self.media, doubled, self.inputs)
    }
    fn body_x(&mut self) -> Result<u16, DisplayError> {
        self.inputs.body_x()
    }
    fn body_y(&mut self) -> Result<u16, DisplayError> {
        self.inputs.body_y()
    }
    fn camera_x(&mut self) -> Result<u16, DisplayError> {
        super::samus_position_tiles::Inputs::camera_x(self.inputs)
    }
    fn camera_y(&mut self) -> Result<u16, DisplayError> {
        super::samus_position_tiles::Inputs::camera_y(self.inputs)
    }
    fn draw_sprite(&mut self, index: u16, x: u16, y: u16) -> Result<(), DisplayError> {
        super::samus::draw_owned(
            self.state,
            self.media,
            self.producer,
            index,
            (x, y),
            self.inputs,
        )
    }
    fn bottom_visible(&mut self) -> Result<bool, DisplayError> {
        bottom_is_visible(self)
    }
    fn queue_tiles(&mut self) -> Result<(), DisplayError> {
        super::samus_position_tiles::tiles_owned(self.state, self.media, self.inputs)
    }
}
impl<I: Inputs> BottomHost for Owned<'_, I> {
    type Error = DisplayError;
    fn pose(&mut self) -> Result<u16, DisplayError> {
        self.inputs.pose()
    }
    fn animation_frame(&mut self) -> Result<u16, DisplayError> {
        self.inputs.animation_frame()
    }
    fn movement(&mut self) -> Result<u8, DisplayError> {
        self.inputs.movement()
    }
    fn clear_bottom(&mut self) -> Result<(), DisplayError> {
        self.inputs.set_sprite(Half::Bottom, 0)
    }
    fn cursor(&mut self) -> Result<u16, DisplayError> {
        Ok(u16::from_le_bytes([
            self.state.objects.byte(544)?,
            self.state.objects.byte(545)?,
        ]))
    }
    fn player_x(&mut self) -> Result<u16, DisplayError> {
        self.inputs.player_x()
    }
    fn player_y(&mut self) -> Result<u16, DisplayError> {
        self.inputs.player_y()
    }
    fn camera_x(&mut self) -> Result<u16, DisplayError> {
        super::samus_position_tiles::Inputs::camera_x(self.inputs)
    }
    fn camera_y(&mut self) -> Result<u16, DisplayError> {
        super::samus_position_tiles::Inputs::camera_y(self.inputs)
    }
    fn marker_word(&mut self, entry: usize, value: u16) -> Result<(), DisplayError> {
        super::objects::Host::write_native_bytes(&mut self.state.objects, entry, value)
    }
    fn publish_cursor(&mut self, value: u16) -> Result<(), DisplayError> {
        let [low, high] = value.to_le_bytes();
        self.state.objects.write_byte(544, low)?;
        self.state.objects.write_byte(545, high)
    }
    fn marker_written(&mut self, entry: usize, x: u16, y: u16) -> Result<(), DisplayError> {
        let context = self
            .inputs
            .object_context(&self.state.controls, &self.state.effective)?;
        if entry >= 512 || entry & 3 != 0 {
            return Err(DisplayError::Alias { offset: entry });
        }
        let bits = self.state.objects.byte(512 + entry / 16)? >> (2 * (entry / 4 & 3));
        let size = if bits & 2 != 0 {
            ObjectSize::Large
        } else {
            ObjectSize::Small
        };
        let binding = self
            .marker
            .variants
            .binding(context.selection.size_set, size)
            .map_err(|_| DisplayError::Incomplete)?;
        let definition = binding
            .definition(self.media)
            .map_err(|_| DisplayError::Scene)?;
        let word = 14369;
        let record = ProducedObject {
            producer: self.marker.producer,
            variants: self.marker.variants.clone(),
            context,
            native_word: word,
            object: SceneObject {
                order: (entry / 4) as u16,
                visual: binding.visual,
                frame: binding.frame,
                part: binding.part,
                atlas: definition.atlas,
                material: definition.material,
                tile: word & 255,
                position: ScreenPosition {
                    x: x as i16,
                    y: y as i16,
                },
                size,
                palette: ((word >> 9) & 7) as u8,
                priority: ((word >> 12) & 3) as u8,
                flip_horizontal: word & 0x4000 != 0,
                flip_vertical: word & 0x8000 != 0,
            },
        }
        .for_line(self.media, context, &self.state.objects.bytes)
        .map_err(|_| DisplayError::Scene)?;
        self.state
            .producer_ids
            .validate_id(record.producer)
            .map_err(|_| DisplayError::Identity)?;
        self.state
            .objects
            .produced
            .retain(|old| old.object.order != record.object.order);
        self.state.objects.produced.push(record);
        self.state
            .objects
            .produced
            .sort_by_key(|record| record.object.order);
        Ok(())
    }
}
