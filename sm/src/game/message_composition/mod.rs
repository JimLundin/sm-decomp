//! Owned message composition; native acquisition and display settlement stay outside.
#![forbid(unsafe_code)]

mod native;
pub use native::*;
pub mod prepared;

use std::{collections::BTreeMap, sync::Arc};

const MAP_BASE: usize = 0x5880;
const MAP_WORDS: usize = 896;
const WORK_WORDS: usize = 768;

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
    Raster(crate::rendering::RasterFault),
}
impl From<crate::rendering::RasterFault> for Fault {
    fn from(error: crate::rendering::RasterFault) -> Self {
        Self::Raster(error)
    }
}

impl MessageDefinition {
    fn upload_offset(&self) -> usize {
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
            || font.words.len() % 8 != 0
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
    fn message(&self, id: MessageId) -> Result<&MessageDefinition, Fault> {
        self.messages.get(&id).ok_or(Fault::MissingMessage(id))
    }
    fn kind(&self, kind: MessageKind) -> Result<&MessageDefinition, Fault> {
        self.messages
            .values()
            .find(|message| message.kind == kind)
            .ok_or(Fault::Definition)
    }
}
