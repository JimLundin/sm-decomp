//! Portable message definitions and immutable prepared display input.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxSize {
    Small,
    Large,
}

const LARGE_BORDER: [u16; 32] = [
    0xe, 0xe, 0xe, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x280f, 0x280f, 0x280f, 0x280f,
    0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f, 0x280f,
    0x280f, 0x280f, 0x284e, 0x284e, 0xe, 0xe, 0xe,
];

const SMALL_BORDER: [u16; 32] = [
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e,
    0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0x284e, 0xe,
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe,
];

const SELECTION: [u16; 96] = [
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0x3c4e, 0x3c4e, 0x38cc, 0x38cd, 0x3cf8, 0x3ce4, 0x3cf2, 0x3c4e,
    0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x2ced, 0x2cee, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0xe,
    0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0x3c4e, 0x3c4e, 0x38cc, 0x38cd,
    0x3cf8, 0x3ce4, 0x3cf2, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x2ced, 0x2cee, 0x3c4e,
    0x3c4e, 0x3c4e, 0x3c4e, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe,
    0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x2cf8, 0x2ce4, 0x2cf2, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0x38cc,
    0x38cd, 0x3ced, 0x3cee, 0x3c4e, 0x3c4e, 0x3c4e, 0x3c4e, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe, 0xe,
];

pub fn border(size: BoxSize) -> &'static [u16; 32] {
    match size {
        BoxSize::Small => &SMALL_BORDER,
        BoxSize::Large => &LARGE_BORDER,
    }
}

pub fn selection_row(selection: u16) -> &'static [u16; 32] {
    let first = if selection == 2 { 64 } else { 32 };
    SELECTION[first..first + 32].try_into().unwrap()
}

pub fn map_offset(size: BoxSize) -> u16 {
    match size {
        BoxSize::Small => 448,
        BoxSize::Large => 416,
    }
}

use std::{collections::BTreeMap, sync::Arc};

pub(crate) const MAP_BASE: usize = 0x5880;
pub(crate) const MAP_WORDS: usize = 896;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Revision(pub [u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    Content,
    Map,
    Energy,
    Missiles,
    Confirmation,
    SavePrompt,
    Saved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonModifier {
    None,
    Shoot,
    Run,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageDefinition {
    pub id: MessageId,
    pub kind: MessageKind,
    pub size: BoxSize,
    pub modifier: ButtonModifier,
    pub button_byte_offset: Option<u16>,
    pub text: Arc<[u16]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontDefinition {
    pub id: AssetId,
    pub revision: Revision,
    pub first_word: u16,
    pub words: Arc<[u16]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definitions {
    revision: Revision,
    font: FontDefinition,
    messages: BTreeMap<MessageId, MessageDefinition>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Definition,
    MissingMessage(MessageId),
    MissingDisplay,
    ExtendedBackground,
    ChangedContent,
    Interval,
    NoRequest,
    Poisoned,
    EffectLimit,
    Raster(crate::raster::RasterFault),
}
impl From<crate::raster::RasterFault> for Fault {
    fn from(error: crate::raster::RasterFault) -> Self {
        Self::Raster(error)
    }
}

impl MessageDefinition {
    pub fn upload_offset(&self) -> usize {
        0x5800
            + usize::from(map_offset(if self.modifier == ButtonModifier::None {
                self.size
            } else {
                BoxSize::Large
            }))
            - MAP_BASE
    }
}
impl Definitions {
    pub fn new(
        revision: Revision,
        font: FontDefinition,
        messages: Vec<MessageDefinition>,
    ) -> Result<Self, Fault> {
        if font.id.0 == 0
            || font.revision != revision
            || font.words.is_empty()
            || !font.words.len().is_multiple_of(8)
            || usize::from(font.first_word) + font.words.len() > 0x8000
            || messages.len() > 256
        {
            return Err(Fault::Definition);
        }
        let mut result = Self {
            revision,
            font,
            messages: BTreeMap::new(),
        };
        for message in messages {
            let length = message.text.len();
            if message.id.0 == 0
                || length == 0
                || length + 64 + message.upload_offset() > MAP_WORDS
                || message.button_byte_offset.is_some_and(|offset| {
                    offset & 1 != 0 || usize::from(offset) >= 2 * (length + 64)
                })
                || (message.modifier != ButtonModifier::None
                    && message.button_byte_offset.is_none())
                || result.messages.contains_key(&message.id)
                || (message.kind != MessageKind::Content
                    && result.messages.values().any(|old| old.kind == message.kind))
            {
                return Err(Fault::Definition);
            }
            for tile in message.text.iter().chain(border(message.size)) {
                if (usize::from(tile & 0x3ff) + 1) * 8 > result.font.words.len() {
                    return Err(Fault::Definition);
                }
            }
            result.messages.insert(message.id, message);
        }
        for tile in selection_row(0).iter().chain(selection_row(2)).chain(
            [
                0x28e0, 0x3ce1, 0x2cf7, 0x38f8, 0x38d0, 0x38eb, 0x38f1, 0x284e,
            ]
            .iter(),
        ) {
            if (usize::from(tile & 0x3ff) + 1) * 8 > result.font.words.len() {
                return Err(Fault::Definition);
            }
        }
        if result
            .messages
            .values()
            .any(|message| message.kind == MessageKind::SavePrompt)
            && !result
                .messages
                .values()
                .any(|message| message.kind == MessageKind::Saved)
        {
            return Err(Fault::Definition);
        }
        Ok(result)
    }
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn font(&self) -> &FontDefinition {
        &self.font
    }
    pub fn message(&self, id: MessageId) -> Result<&MessageDefinition, Fault> {
        self.messages.get(&id).ok_or(Fault::MissingMessage(id))
    }
    pub fn kind(&self, kind: MessageKind) -> Result<&MessageDefinition, Fault> {
        self.messages
            .values()
            .find(|message| message.kind == kind)
            .ok_or(Fault::Definition)
    }
}

const LINES: usize = 224;
const WORK_WORDS: usize = 768;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayLine {
    pub mode: u8,
    /// The frozen core has no EXTBG composition; required EXTBG rejects explicitly.
    pub extended_background: bool,
    pub forced_blank: bool,
    pub backgrounds: [crate::raster::Background; 3],
    pub windows: crate::raster::Windows,
    pub routing: crate::raster::Routing,
    pub affine: crate::raster::Affine,
    pub color: crate::raster::ColorMath,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullDisplay {
    /// Stable authored ID of the supplied indexed media within this revision.
    pub media_id: AssetId,
    /// Revision of the identified media and message/font definitions.
    pub revision: Revision,
    pub media: Arc<crate::raster::IndexedImage>,
    pub sprites: Arc<crate::raster::PreparedSprites>,
    pub mosaic: Arc<crate::raster::PreparedMosaic>,
    pub lines: Box<[DisplayLine; LINES]>,
    pub overflow: crate::raster::Overflow,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedStart {
    /// Actual complete saved, active and restored display records; no guessed routing.
    pub saved: FullDisplay,
    pub active: FullDisplay,
    pub restored: FullDisplay,
    pub tile_workspace: [u16; WORK_WORDS],
    pub shoot_button: u16,
    pub run_button: u16,
    pub initial_interval: u64,
    pub initial_wait_pending: bool,
}
/// Complete input to one message render; supplied only after game extraction.
pub struct Display<'a> {
    pub display: &'a FullDisplay,
    pub map: Option<&'a [u16; MAP_WORDS]>,
    pub active: bool,
    pub scroll: RadiusValues,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadiusValues {
    pub top_scroll: u16,
    pub bottom_scroll: u16,
    /// Final source scratch words at05a4,05a6,05a8,05aa, in write order.
    pub scratch: [u16; 4],
}

pub const HDMA_DESCRIPTOR: [u8; 7] = [0xff, 0x00, 0x30, 0xe1, 0xfe, 0x30, 0x00];

/// Fixed indirect two-byte transfers:127 repeating rows at3000, then97 at30fe.
/// This maps emitted scroll words; it does not replace them with a rectangle.
pub fn scanline_scroll(values: RadiusValues, line_zero_based: usize) -> u16 {
    crate::table::check_index(line_zero_based, 224);
    let block = if line_zero_based < usize::from(HDMA_DESCRIPTOR[0] & 0x7f) {
        0
    } else {
        3
    };
    let start = if block == 0 {
        0
    } else {
        usize::from(HDMA_DESCRIPTOR[0] & 0x7f)
    };
    let control = HDMA_DESCRIPTOR[block];
    let pointer = u16::from_le_bytes([HDMA_DESCRIPTOR[block + 1], HDMA_DESCRIPTOR[block + 2]]);
    let word = usize::from(pointer - 0x3000) / 2
        + if control & 0x80 != 0 {
            line_zero_based - start
        } else {
            0
        };
    match word {
        94..124 => values.top_scroll,
        124..154 => values.bottom_scroll,
        _ => 0,
    }
}
