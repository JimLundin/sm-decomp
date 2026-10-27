//! Actual prepared consumer of the existing dialogue runner and shared rasterizer.
use super::*;
use crate::{game::dialogue as d, rendering as r};
use r::{
    IndexedSource,
    prepared::{IndexedImage, PreparedMosaic, PreparedSprites},
};

const LINES: usize = 224;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayLine {
    pub mode: u8,
    /// The frozen core has no EXTBG composition; required EXTBG rejects explicitly.
    pub extended_background: bool,
    pub forced_blank: bool,
    pub backgrounds: [r::Background; 3],
    pub windows: r::Windows,
    pub routing: r::Routing,
    pub affine: r::Affine,
    pub color: r::ColorMath,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullDisplay {
    /// Stable authored ID of the supplied indexed media within this revision.
    pub media_id: AssetId,
    /// Revision of the identified media and message/font definitions.
    pub revision: Revision,
    pub media: Arc<IndexedImage>,
    pub sprites: Arc<PreparedSprites>,
    pub mosaic: Arc<PreparedMosaic>,
    pub lines: Box<[DisplayLine; LINES]>,
    pub overflow: r::Overflow,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputSample {
    /// Values at the original dialogue input-poll phase, not physical-device events.
    pub new_keys: u16,
    pub held_keys: u16,
    pub bug_fix_counter: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    CancelSounds,
    AdvanceAudio,
    SaveSound,
    SelectionSound,
    ResumeMovementSound,
    SelectPause,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Suspended,
    Complete,
}
pub struct MessageFrame {
    pub pixels: Vec<r::Bgra>,
    pub overflow: r::Overflow,
    pub revision: Revision,
}
pub struct StepOutput {
    pub outcome: Outcome,
    pub frame: MessageFrame,
    pub effects: Vec<Effect>,
}

#[derive(Clone, Copy)]
struct RunnerState {
    message: d::Message<MessageId>,
    continuation: d::Continuation,
    child: d::ChildContinuation,
    radius: u16,
    selection: d::Selection,
    delay: u16,
}
impl RunnerState {
    fn dialogue(self) -> d::Dialogue<MessageId> {
        d::Dialogue {
            message: self.message,
            continuation: self.continuation,
            child: self.child,
            radius: self.radius,
            selection: self.selection,
            delay_remaining: self.delay,
        }
    }
    fn from_dialogue(state: d::Dialogue<MessageId>) -> Self {
        Self {
            message: state.message,
            continuation: state.continuation,
            child: state.child,
            radius: state.radius,
            selection: state.selection,
            delay: state.delay_remaining,
        }
    }
}
#[derive(Clone)]
struct State {
    runner: RunnerState,
    tiles: [u16; WORK_WORDS],
    map: [u16; MAP_WORDS],
    backup: [u16; MAP_WORDS],
    request: Option<MessageId>,
    completed: Option<(MessageId, u16)>,
    visible: bool,
    restored: bool,
    wait_pending: bool,
    last_interval: u64,
    drawn_scroll: RadiusValues,
}
#[derive(Clone)]
pub struct Checkpoint {
    schema: u16,
    definitions: Arc<Definitions>,
    start: Arc<PreparedStart>,
    state: State,
}
pub struct PreparedOwner {
    definitions: Arc<Definitions>,
    start: Arc<PreparedStart>,
    state: State,
    effects: Vec<Effect>,
    fault: Option<Fault>,
    sample: Option<InputSample>,
}

fn requested(definition: &MessageDefinition) -> d::Message<MessageId> {
    match definition.kind {
        MessageKind::Content => d::Message::Content(definition.id),
        MessageKind::Map => d::Message::MapAcquired,
        MessageKind::Energy => d::Message::EnergyRestored,
        MessageKind::Missiles => d::Message::MissilesRestored,
        MessageKind::Confirmation => d::Message::SaveConfirmation,
        MessageKind::SavePrompt => d::Message::SavePrompt,
        MessageKind::Saved => d::Message::Saved,
    }
}
fn definition(
    definitions: &Definitions,
    message: d::Message<MessageId>,
) -> Result<&MessageDefinition, Fault> {
    match message {
        d::Message::Content(id) => definitions.message(id),
        d::Message::MapAcquired => definitions.kind(MessageKind::Map),
        d::Message::EnergyRestored => definitions.kind(MessageKind::Energy),
        d::Message::MissilesRestored => definitions.kind(MessageKind::Missiles),
        d::Message::SaveConfirmation => definitions.kind(MessageKind::Confirmation),
        d::Message::SavePrompt => definitions.kind(MessageKind::SavePrompt),
        d::Message::Saved => definitions.kind(MessageKind::Saved),
    }
}
fn selection(selection: d::Selection) -> u16 {
    match selection {
        d::Selection::Accept => 0,
        d::Selection::Decline => 2,
        d::Selection::Other(value) => value,
    }
}

impl PreparedOwner {
    pub fn open(
        definitions: Arc<Definitions>,
        start: PreparedStart,
    ) -> Result<(Self, MessageFrame), Fault> {
        if start.saved.media_id.0 == 0
            || start.active.media_id.0 == 0
            || start.restored.media_id.0 == 0
        {
            return Err(Fault::MissingDisplay);
        }
        if definitions.revision != start.active.revision
            || start.saved.revision != start.active.revision
            || start.restored.revision != start.active.revision
        {
            return Err(Fault::ChangedContent);
        }
        for (index, word) in definitions.font.words.iter().enumerate() {
            if start
                .active
                .media
                .word(usize::from(definitions.font.first_word) + index)?
                != *word
            {
                return Err(Fault::MissingDisplay);
            }
        }
        for line in start.active.lines.iter() {
            if line.mode != 1
                || line.backgrounds[2].map_words != 0x5800
                || line.backgrounds[2].tile_words != definitions.font.first_word
                || line.backgrounds[2].horizontal_scroll != 0
                || line.backgrounds[2].wider
                || line.backgrounds[2].higher
            {
                return Err(Fault::MissingDisplay);
            }
        }
        let mut backup = [0; MAP_WORDS];
        for (index, word) in backup.iter_mut().enumerate() {
            *word = start.saved.media.word(MAP_BASE + index)?;
        }
        let state = State {
            runner: RunnerState {
                message: d::Message::Content(MessageId(0)),
                continuation: d::Continuation::Start,
                child: d::ChildContinuation::Start,
                radius: 0,
                selection: d::Selection::Accept,
                delay: 0,
            },
            tiles: start.tile_workspace,
            map: backup,
            backup,
            request: None,
            completed: None,
            visible: false,
            restored: false,
            wait_pending: start.initial_wait_pending,
            last_interval: start.initial_interval,
            drawn_scroll: radius_values(0),
        };
        let owner = Self {
            definitions,
            start: Arc::new(start),
            state,
            effects: Vec::new(),
            fault: None,
            sample: None,
        };
        // All three supplied display/media closures must be complete before opening.
        let initial = owner.render_display(&owner.start.saved, None, false)?;
        let clear_map = [0x000e; MAP_WORDS];
        owner.render_display(&owner.start.active, Some(&clear_map), true)?;
        owner.render_display(&owner.start.restored, Some(&owner.state.backup), false)?;
        Ok((owner, initial))
    }
    /// Keep a live request; content is read when a new message starts.
    pub fn queue(&mut self, id: MessageId) -> Result<(), Fault> {
        self.healthy()?;
        self.state.request = Some(id);
        Ok(())
    }
    pub fn poll(&mut self, id: MessageId) -> Result<Option<u16>, Fault> {
        self.healthy()?;
        if let Some((completed, selection)) = self.state.completed {
            if completed == id {
                self.state.completed = None;
                if self.state.runner.continuation == d::Continuation::Start {
                    self.state.runner.message = d::Message::Content(MessageId(0));
                }
                return Ok(Some(selection));
            }
        }
        self.queue(id)?;
        Ok(None)
    }
    pub fn resume(&mut self, interval: u64, sample: InputSample) -> Result<StepOutput, Fault> {
        self.healthy()?;
        if self.state.last_interval.checked_add(1) != Some(interval) {
            return Err(Fault::Interval);
        }
        let request = if self.state.runner.continuation == d::Continuation::Start {
            let id = self.state.request.ok_or(Fault::NoRequest)?;
            requested(self.definitions.message(id)?)
        } else {
            // Suspension retains the captured message independently of the live queue.
            self.state.runner.message
        };
        self.sample = Some(sample);
        self.effects.clear();
        let (state, outcome) = d::run(self.state.runner.dialogue(), request, self);
        self.state.runner = RunnerState::from_dialogue(state);
        if let Some(fault) = self.fault {
            return Err(fault);
        }
        if outcome == d::Outcome::Complete {
            let completed = match definition(&self.definitions, self.state.runner.message) {
                Ok(message) => message.id,
                Err(fault) => {
                    self.fault = Some(fault);
                    return Err(fault);
                }
            };
            self.state.completed = Some((completed, selection(self.state.runner.selection)));
            // Completion clears the current live request, even if it changed while suspended.
            self.state.request = None;
        }
        let frame = match self.render() {
            Ok(frame) => frame,
            Err(fault) => {
                self.fault = Some(fault);
                return Err(fault);
            }
        };
        self.state.last_interval = interval;
        Ok(StepOutput {
            outcome: if outcome == d::Outcome::Complete {
                Outcome::Complete
            } else {
                Outcome::Suspended
            },
            frame,
            effects: std::mem::take(&mut self.effects),
        })
    }
    pub fn checkpoint(&self) -> Result<Checkpoint, Fault> {
        self.healthy()?;
        Ok(Checkpoint {
            schema: 1,
            definitions: self.definitions.clone(),
            start: self.start.clone(),
            state: self.state.clone(),
        })
    }
    pub fn reopen(
        definitions: Arc<Definitions>,
        start: PreparedStart,
        checkpoint: Checkpoint,
    ) -> Result<(Self, MessageFrame), Fault> {
        if checkpoint.schema != 1
            || *definitions != *checkpoint.definitions
            || start != *checkpoint.start
        {
            return Err(Fault::ChangedContent);
        }
        let owner = Self {
            definitions,
            start: Arc::new(start),
            state: checkpoint.state,
            effects: Vec::new(),
            fault: None,
            sample: None,
        };
        let frame = owner.render()?;
        Ok((owner, frame))
    }
    pub fn reset(&self, start: PreparedStart) -> Result<(Self, MessageFrame), Fault> {
        Self::open(self.definitions.clone(), start)
    }
    fn healthy(&self) -> Result<(), Fault> {
        if self.fault.is_some() {
            Err(Fault::Poisoned)
        } else {
            Ok(())
        }
    }
    pub fn render(&self) -> Result<MessageFrame, Fault> {
        self.healthy()?;
        let display = if self.state.visible {
            &self.start.active
        } else if self.state.restored {
            &self.start.restored
        } else {
            &self.start.saved
        };
        let map = (self.state.visible || self.state.restored).then_some(&self.state.map);
        self.render_display(display, map, self.state.visible)
    }
    fn prepare(&mut self, message: d::Message<MessageId>) -> Result<(), Fault> {
        let message = definition(&self.definitions, message)?;
        let mut cursor = TextCursor::new((message.text.len() * 2) as u16);
        self.state.tiles[..32].copy_from_slice(border(message.size));
        loop {
            let tile = message.text[usize::from(cursor.source_offset >> 1)];
            let offset = destination_byte_offset(cursor.word);
            self.state.tiles[(offset - 0x200) / 2] = tile;
            if cursor.advance() {
                break;
            }
        }
        let end = usize::from(cursor.word);
        self.state.tiles[end..end + 32].copy_from_slice(border(message.size));
        if message.modifier != ButtonModifier::None {
            let buttons = if message.modifier == ButtonModifier::Shoot {
                self.start.shoot_button
            } else {
                self.start.run_button
            };
            self.state.tiles
                [usize::from(message.button_byte_offset.ok_or(Fault::Definition)?) / 2] =
                button_letter(buttons);
        }
        let length = usize::from(dma_length((message.text.len() * 2) as u16)) / 2;
        let first = message.upload_offset();
        self.state.map[first..first + length].copy_from_slice(&self.state.tiles[..length]);
        self.state.drawn_scroll = radius_values(0);
        Ok(())
    }
    fn event(&mut self, effect: Effect) {
        if self.effects.len() == 16 {
            self.fault = Some(Fault::EffectLimit);
        } else {
            self.effects.push(effect);
        }
    }
    fn render_display(
        &self,
        display: &FullDisplay,
        map: Option<&[u16; MAP_WORDS]>,
        active: bool,
    ) -> Result<MessageFrame, Fault> {
        let media = ComposedMedia {
            base: &display.media,
            map,
            message_palette: active,
        };
        let mut pixels = Vec::with_capacity(r::WIDTH * LINES);
        let mut overflow = display.overflow;
        for (index, line) in display.lines.iter().enumerate() {
            if line.extended_background {
                return Err(Fault::ExtendedBackground);
            }
            if line.mode != 1 && line.mode != 7 {
                return Err(r::RasterFault::Mode(line.mode).into());
            }
            let mut sprites = r::SpriteLine::cleared(overflow);
            if !line.forced_blank {
                let source = &display.sprites;
                let input = r::SpriteInput {
                    words: &source.words,
                    extension: &source.extension,
                    tile_words: source.tile_words,
                    size: source.size,
                    rotation: source.rotation,
                    interlace: source.interlace,
                    even_frame: source.even_frame,
                };
                let (has_tiles, flags) = r::evaluate_sprites(
                    &media,
                    &input,
                    index as i32,
                    &mut sprites.pixels,
                    overflow,
                )?;
                sprites.has_tiles = has_tiles;
                sprites.overflow = flags;
            }
            let mut backgrounds = line.backgrounds;
            if active {
                backgrounds[2].vertical_scroll = scanline_scroll(self.state.drawn_scroll, index);
            }
            let brightness = r::BrightnessTables::for_level(line.color.brightness);
            let mosaic = &display.mosaic;
            let config = r::LineConfig {
                mode: line.mode,
                forced_blank: line.forced_blank,
                backgrounds,
                windows: line.windows,
                routing: line.routing,
                affine: line.affine,
                color: line.color,
                brightness: brightness.borrowed(),
                has_sprites: sprites.has_tiles,
                mosaic: r::Mosaic {
                    native_enabled: mosaic.native_enabled,
                    background0_enabled: mosaic.background0_enabled,
                    size: mosaic.size,
                    start_line: mosaic.start_line,
                    modulo: &mosaic.modulo,
                    left: 0,
                },
            };
            let mut row = [[0; 4]; r::WIDTH];
            r::render_line(&media, &config, &sprites, index as i32 + 1, &mut row)?;
            pixels.extend_from_slice(&row);
            overflow = sprites.overflow;
        }
        Ok(MessageFrame {
            pixels,
            overflow,
            revision: display.revision,
        })
    }
}

struct ComposedMedia<'a> {
    base: &'a IndexedImage,
    map: Option<&'a [u16; MAP_WORDS]>,
    message_palette: bool,
}
impl IndexedSource for ComposedMedia<'_> {
    fn word(&self, index: usize) -> Result<u16, r::RasterFault> {
        if let Some(map) = self.map {
            if (MAP_BASE..MAP_BASE + MAP_WORDS).contains(&index) {
                return Ok(map[index - MAP_BASE]);
            }
        }
        self.base.word(index)
    }
    fn palette(&self, index: u8) -> Result<u16, r::RasterFault> {
        if self.message_palette {
            match index {
                25 => return Ok(0x0bb1),
                26 => return Ok(0x001f),
                _ => {}
            }
        }
        self.base.palette(index)
    }
}
impl d::Context for PreparedOwner {
    type Id = MessageId;
    fn effect(
        &mut self,
        mut state: d::Dialogue<MessageId>,
        effect: d::Effect,
    ) -> d::Dialogue<MessageId> {
        if self.fault.is_some() {
            return state;
        }
        match effect {
            d::Effect::CancelSounds => self.event(Effect::CancelSounds),
            d::Effect::PrepareDisplay => {
                self.state.visible = true;
                state.selection = d::Selection::Accept;
            }
            d::Effect::ClearDisplay => self.state.map.fill(0x000e),
            d::Effect::PrepareMessage => {
                state.radius = 0;
                if let Err(fault) = self.prepare(state.message) {
                    self.fault = Some(fault);
                }
            }
            d::Effect::SaveSound => self.event(Effect::SaveSound),
            d::Effect::AdvanceAudio => self.event(Effect::AdvanceAudio),
            d::Effect::RestoreDisplay => {
                self.state.map = self.state.backup;
                self.state.visible = false;
                self.state.restored = true;
            }
            d::Effect::ResumeMovementSound => self.event(Effect::ResumeMovementSound),
            d::Effect::SelectPause => self.event(Effect::SelectPause),
            d::Effect::DrawRadius => self.state.drawn_scroll = radius_values(state.radius),
            d::Effect::DrawSelection => {
                self.state.tiles[128..160]
                    .copy_from_slice(selection_row(selection(state.selection)));
                let first = 0x5800 + usize::from(map_offset(BoxSize::Large)) - MAP_BASE;
                self.state.map[first..first + 192].copy_from_slice(&self.state.tiles[..192]);
            }
            d::Effect::SelectionSound => self.event(Effect::SelectionSound),
        }
        state
    }
    fn wait(&mut self, state: d::Dialogue<MessageId>) -> (d::Dialogue<MessageId>, d::Outcome) {
        if self.fault.is_some() {
            return (
                state,
                d::Outcome::Suspended(std::num::NonZeroU8::new(1).unwrap()),
            );
        }
        if self.state.wait_pending {
            self.state.wait_pending = false;
            (state, d::Outcome::Complete)
        } else {
            self.state.wait_pending = true;
            (
                state,
                d::Outcome::Suspended(std::num::NonZeroU8::new(1).unwrap()),
            )
        }
    }
    fn input(&mut self, state: d::Dialogue<MessageId>) -> (d::Dialogue<MessageId>, d::Input) {
        let sample = self
            .sample
            .expect("resume admits input before dialogue polling");
        (
            state,
            d::Input {
                confirm: sample.new_keys & 0x80 != 0,
                cancel: sample.new_keys & 0x8000 != 0,
                toggle: sample.new_keys & (0x2000 | 0x200 | 0x100) != 0,
                dismiss: if sample.bug_fix_counter == 0 {
                    sample.new_keys != 0
                } else {
                    sample.held_keys != 0
                },
            },
        )
    }
}
