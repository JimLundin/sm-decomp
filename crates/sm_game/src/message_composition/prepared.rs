//! Prepared message state and ordered extraction through the existing dialogue runner.
use super::*;
use crate::dialogue as d;
pub use sm_data::message::{Display, DisplayLine, FullDisplay, PreparedStart};
use std::sync::Arc;
const MAP_BASE: usize = 0x5880;
const MAP_WORDS: usize = 896;
const WORK_WORDS: usize = 768;

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
    AdvanceSoundEffects,
    /// The HDMA object handler, which also handles the music queue.
    UpdateHdma,
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
pub struct StepOutput<Frame> {
    pub outcome: Outcome,
    pub frame: Frame,
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
    pub fn open<Frame>(
        definitions: Arc<Definitions>,
        start: PreparedStart,
        mut render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<(Self, Frame), Fault> {
        if start.saved.media_id.0 == 0
            || start.active.media_id.0 == 0
            || start.restored.media_id.0 == 0
        {
            return Err(Fault::MissingDisplay);
        }
        if definitions.revision() != start.active.revision
            || start.saved.revision != start.active.revision
            || start.restored.revision != start.active.revision
        {
            return Err(Fault::ChangedContent);
        }
        for (index, word) in definitions.font().words.iter().enumerate() {
            if start
                .active
                .media
                .word(usize::from(definitions.font().first_word) + index)?
                != *word
            {
                return Err(Fault::MissingDisplay);
            }
        }
        for line in start.active.lines.iter() {
            if line.mode != 1
                || line.backgrounds[2].map_words != 0x5800
                || line.backgrounds[2].tile_words != definitions.font().first_word
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
        let initial = render(owner.extract_display(&owner.start.saved, None, false))?;
        let clear_map = [0x000e; MAP_WORDS];
        render(owner.extract_display(&owner.start.active, Some(&clear_map), true))?;
        render(owner.extract_display(&owner.start.restored, Some(&owner.state.backup), false))?;
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
        if let Some((completed, selection)) = self.state.completed
            && completed == id
        {
            self.state.completed = None;
            if self.state.runner.continuation == d::Continuation::Start {
                self.state.runner.message = d::Message::Content(MessageId(0));
            }
            return Ok(Some(selection));
        }
        self.queue(id)?;
        Ok(None)
    }
    pub fn resume<Frame>(
        &mut self,
        interval: u64,
        sample: InputSample,
        render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<StepOutput<Frame>, Fault> {
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
        let frame = match self.render(render) {
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
    pub fn reopen<Frame>(
        definitions: Arc<Definitions>,
        start: PreparedStart,
        checkpoint: Checkpoint,
        render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<(Self, Frame), Fault> {
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
        let frame = owner.render(render)?;
        Ok((owner, frame))
    }
    pub fn reset<Frame>(
        &self,
        start: PreparedStart,
        render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<(Self, Frame), Fault> {
        Self::open(self.definitions.clone(), start, render)
    }
    fn healthy(&self) -> Result<(), Fault> {
        if self.fault.is_some() {
            Err(Fault::Poisoned)
        } else {
            Ok(())
        }
    }
    pub fn render<Frame>(
        &self,
        mut render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<Frame, Fault> {
        self.healthy()?;
        let display = if self.state.visible {
            &self.start.active
        } else if self.state.restored {
            &self.start.restored
        } else {
            &self.start.saved
        };
        let map = (self.state.visible || self.state.restored).then_some(&self.state.map);
        render(self.extract_display(display, map, self.state.visible))
    }
    fn extract_display<'a>(
        &self,
        display: &'a FullDisplay,
        map: Option<&'a [u16; MAP_WORDS]>,
        active: bool,
    ) -> Display<'a> {
        Display {
            display,
            map,
            active,
            scroll: self.state.drawn_scroll,
        }
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
        Ok(())
    }
    fn event(&mut self, effect: Effect) {
        if self.effects.len() == 16 {
            self.fault = Some(Fault::EffectLimit);
        } else {
            self.effects.push(effect);
        }
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
            d::Effect::ClearSelection => state.selection = d::Selection::Accept,
            d::Effect::PrepareDisplay => self.state.visible = true,
            // The map holds VRAM, so only the uploads change it.
            d::Effect::SaveBackground
            | d::Effect::FillBackground
            | d::Effect::DrawTopBorder
            | d::Effect::PrepareScroll
            | d::Effect::EnableScroll => {}
            d::Effect::UploadBackground => self.state.map.fill(0x000e),
            d::Effect::WriteMessage => state.radius = 0,
            d::Effect::UploadMessage => {
                if let Err(fault) = self.prepare(state.message) {
                    self.fault = Some(fault);
                }
            }
            d::Effect::SaveSound => self.event(Effect::SaveSound),
            d::Effect::AdvanceAudio => self.event(Effect::AdvanceAudio),
            d::Effect::AdvanceSoundEffects => self.event(Effect::AdvanceSoundEffects),
            d::Effect::UpdateHdma => self.event(Effect::UpdateHdma),
            d::Effect::RestoreBackground => self.state.map = self.state.backup,
            d::Effect::RestoreDisplay => {
                self.state.visible = false;
                self.state.restored = true;
            }
            d::Effect::ResumeMovementSound => self.event(Effect::ResumeMovementSound),
            d::Effect::SelectPause => self.event(Effect::SelectPause),
            d::Effect::DrawRadius => self.state.drawn_scroll = radius_values(state.radius),
            d::Effect::StageSelection => {
                self.state.tiles[128..160]
                    .copy_from_slice(selection_row(selection(state.selection)));
            }
            d::Effect::UploadSelection => {
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
