//! Prepared message state and ordered extraction through the existing dialogue runner.
use super::*;
use crate::dialogue as d;
use crate::message_workspace as canvas;
use crate::presentation_workspace::{Point, Role, Workspace};
pub use sm_data::message::{Display, DisplayLine, FullDisplay, PreparedStart};
use std::sync::Arc;
const MAP_BASE: usize = 0x5880;
const MAP_WORDS: usize = 896;

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
    map: [u16; MAP_WORDS],
    request: Option<MessageId>,
    completed: Option<(MessageId, u16)>,
    visible: bool,
    restored: bool,
    wait_pending: bool,
    last_interval: u64,
    drawn_scroll: RadiusValues,
}
/// Complete supplied execution state for a selected image or a checkpoint.
/// Workspace sources are borrowed separately; this map is the actual VRAM result.
#[derive(Clone)]
pub struct ExecutionDescription {
    pub message: d::Message<MessageId>,
    pub continuation: d::Continuation,
    pub child: d::ChildContinuation,
    pub radius: u16,
    pub selection: d::Selection,
    pub delay: u16,
    pub map: [u16; MAP_WORDS],
    pub request: Option<MessageId>,
    pub completed: Option<(MessageId, u16)>,
    pub visible: bool,
    pub restored: bool,
    pub wait_pending: bool,
    pub last_interval: u64,
    pub drawn_scroll: RadiusValues,
}
#[derive(Clone)]
pub struct Checkpoint {
    schema: u16,
    definitions: Arc<Definitions>,
    start: Arc<PreparedStart>,
    workspace: Workspace,
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

fn validate_content(definitions: &Definitions, start: &PreparedStart) -> Result<(), Fault> {
    if start.saved.media_id.0 == 0 || start.active.media_id.0 == 0 || start.restored.media_id.0 == 0
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
    Ok(())
}
fn validate_execution(
    start: &PreparedStart,
    description: &ExecutionDescription,
    workspace: &Workspace,
) -> Result<(), Fault> {
    workspace
        .validate(crate::scene::records::Revision(start.active.revision.0))
        .map_err(|_| Fault::MissingDisplay)?;
    if description.last_interval < start.initial_interval {
        return Err(Fault::Interval);
    }
    // Active, pending and completed IDs remain full values. Resolve them only
    // at the original draw/request/poll phase, including after suspension.
    Ok(())
}
impl From<ExecutionDescription> for State {
    fn from(description: ExecutionDescription) -> Self {
        Self {
            runner: RunnerState {
                message: description.message,
                continuation: description.continuation,
                child: description.child,
                radius: description.radius,
                selection: description.selection,
                delay: description.delay,
            },
            map: description.map,
            request: description.request,
            completed: description.completed,
            visible: description.visible,
            restored: description.restored,
            wait_pending: description.wait_pending,
            last_interval: description.last_interval,
            drawn_scroll: description.drawn_scroll,
        }
    }
}
impl PreparedOwner {
    /// Restore complete supplied execution without rendering or emitting effects.
    /// The content envelope must already carry the validated full display/media closure.
    pub fn from_execution(
        definitions: Arc<Definitions>,
        start: Arc<PreparedStart>,
        execution: ExecutionDescription,
        workspace: &Workspace,
    ) -> Result<Self, Fault> {
        validate_content(&definitions, &start)?;
        for display in [&start.saved, &start.active, &start.restored] {
            for line in display.lines.iter() {
                if line.extended_background {
                    return Err(Fault::ExtendedBackground);
                }
                if line.mode != 1 && line.mode != 7 {
                    return Err(sm_data::raster::RasterFault::Mode(line.mode).into());
                }
            }
        }
        validate_execution(&start, &execution, workspace)?;
        for (role, required) in [
            (Role::MessageText, 1536),
            (Role::SavedBackground, 1792),
            (Role::Pause, 0x1e0),
            (Role::MessageDescriptor, 0x6c),
            (Role::Equipment, 0x700),
        ] {
            if workspace
                .available(Point::new(role, 0))
                .map_err(|_| Fault::MissingDisplay)?
                < required
            {
                return Err(Fault::MissingDisplay);
            }
        }
        Ok(Self {
            definitions,
            start,
            state: execution.into(),
            effects: Vec::new(),
            fault: None,
            sample: None,
        })
    }
    pub fn open<Frame>(
        definitions: Arc<Definitions>,
        start: PreparedStart,
        workspace: &mut Workspace,
        mut render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<(Self, Frame), Fault> {
        validate_content(&definitions, &start)?;
        workspace
            .validate(crate::scene::records::Revision(start.active.revision.0))
            .map_err(|_| Fault::MissingDisplay)?;
        for (index, tile) in start.tile_workspace.iter().enumerate() {
            if workspace
                .word(Point::new(Role::MessageText, index * 2))
                .map_err(|_| Fault::MissingDisplay)?
                != *tile
            {
                return Err(Fault::MissingDisplay);
            }
        }
        let mut map = [0; MAP_WORDS];
        for (index, word) in map.iter_mut().enumerate() {
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
            map,
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
        render(owner.extract_display(&owner.start.restored, Some(&owner.state.map), false))?;
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
    /// Extraction-only compatibility entry; coupled execution uses resume_with_effects.
    pub fn resume<Frame>(
        &mut self,
        interval: u64,
        sample: InputSample,
        workspace: &mut Workspace,
        render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<StepOutput<Frame>, Fault> {
        self.resume_with_effects(interval, sample, workspace, |_, _, _| Ok(()), render)
    }
    /// Deliver each effect at its original runner phase. Returned effects are receipts;
    /// a caller that used this sink must never deliver them a second time.
    pub fn resume_with_effects<Frame>(
        &mut self,
        interval: u64,
        sample: InputSample,
        workspace: &mut Workspace,
        effects: impl FnMut(Effect, &mut Workspace, &mut ExecutionDescription) -> Result<(), Fault>,
        render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<StepOutput<Frame>, Fault> {
        self.resume_with_publications(
            interval,
            sample,
            workspace,
            &mut ExtractionEffects(effects),
            render,
        )
    }
    pub fn resume_with_publications<Frame>(
        &mut self,
        interval: u64,
        sample: InputSample,
        workspace: &mut Workspace,
        effects: &mut impl WorkspaceEffects,
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
        let state = self.state.runner.dialogue();
        let (state, outcome) = d::run(
            state,
            request,
            &mut Runner {
                owner: self,
                workspace: &mut *workspace,
                effects,
            },
        );
        // A failed context already holds the actual selected prefix. The
        // shared runner's remaining local bookkeeping is not a committed state.
        if let Some(fault) = self.fault {
            return Err(fault);
        }
        self.state.runner = RunnerState::from_dialogue(state);
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
        let frame = match self.render(workspace, render) {
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
    pub fn execution(&self) -> ExecutionDescription {
        ExecutionDescription {
            message: self.state.runner.message,
            continuation: self.state.runner.continuation,
            child: self.state.runner.child,
            radius: self.state.runner.radius,
            selection: self.state.runner.selection,
            delay: self.state.runner.delay,
            map: self.state.map,
            request: self.state.request,
            completed: self.state.completed,
            visible: self.state.visible,
            restored: self.state.restored,
            wait_pending: self.state.wait_pending,
            last_interval: self.state.last_interval,
            drawn_scroll: self.state.drawn_scroll,
        }
    }
    pub fn checkpoint(&self, workspace: &Workspace) -> Result<Checkpoint, Fault> {
        self.healthy()?;
        workspace
            .validate(crate::scene::records::Revision(
                self.start.active.revision.0,
            ))
            .map_err(|_| Fault::MissingDisplay)?;
        Ok(Checkpoint {
            schema: 1,
            definitions: self.definitions.clone(),
            start: self.start.clone(),
            workspace: workspace.clone(),
            state: self.state.clone(),
        })
    }
    pub fn reopen<Frame>(
        definitions: Arc<Definitions>,
        start: PreparedStart,
        checkpoint: Checkpoint,
        workspace: &mut Workspace,
        render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<(Self, Frame), Fault> {
        if checkpoint.schema != 1
            || *definitions != *checkpoint.definitions
            || start != *checkpoint.start
            || *workspace != checkpoint.workspace
        {
            return Err(Fault::ChangedContent);
        }
        workspace
            .validate(crate::scene::records::Revision(start.active.revision.0))
            .map_err(|_| Fault::MissingDisplay)?;
        let owner = Self {
            definitions,
            start: Arc::new(start),
            state: checkpoint.state,
            effects: Vec::new(),
            fault: None,
            sample: None,
        };
        let frame = owner.render(workspace, render)?;
        Ok((owner, frame))
    }
    pub fn reset<Frame>(
        &self,
        start: PreparedStart,
        workspace: &mut Workspace,
        render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<(Self, Frame), Fault> {
        Self::open(self.definitions.clone(), start, workspace, render)
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
        workspace: &Workspace,
        mut render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<Frame, Fault> {
        self.healthy()?;
        workspace
            .validate(crate::scene::records::Revision(
                self.start.active.revision.0,
            ))
            .map_err(|_| Fault::MissingDisplay)?;
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
    fn event(&mut self, effect: Effect) {
        if self.effects.len() == 16 {
            self.fault = Some(Fault::EffectLimit);
        } else {
            self.effects.push(effect);
        }
    }
}

struct Runner<'a> {
    owner: &'a mut PreparedOwner,
    workspace: &'a mut Workspace,
    effects: &'a mut dyn WorkspaceEffects,
}
impl Runner<'_> {
    fn emit(&mut self, state: d::Dialogue<MessageId>, effect: Effect) -> d::Dialogue<MessageId> {
        self.owner.state.runner = RunnerState::from_dialogue(state);
        self.owner.event(effect);
        if self.owner.fault.is_some() {
            return self.owner.state.runner.dialogue();
        }
        let mut execution = self.owner.execution();
        // The sink hands the sole Workspace and published execution back to
        // their owner before cross-owner effects, then returns the actual prefix.
        let delivered = self.effects.effect(effect, self.workspace, &mut execution);
        // Adopt the actual typed prefix without eagerly querying changed live
        // requests or media. Their original consuming phases decide failure.
        self.owner.state = execution.into();
        self.owner.fault = delivered.err();
        self.owner.state.runner.dialogue()
    }
    fn apply(
        &mut self,
        action: impl FnOnce(
            &mut dyn crate::presentation_workspace::SliceWriter,
        ) -> Result<(), crate::scene::state::DisplayError>,
    ) {
        let mut workspace = crate::presentation_workspace::PublishedWorkspace {
            workspace: self.workspace,
            publication: self.effects,
        };
        if action(&mut workspace).is_err() {
            self.owner.fault = Some(Fault::Definition);
        }
    }
    fn upload(&mut self, first: usize, bytes: usize, role: Role) {
        for (index, destination) in self.owner.state.map[first..first + bytes / 2]
            .iter_mut()
            .enumerate()
        {
            match self.workspace.word(Point::new(role, index * 2)) {
                Ok(word) => *destination = word,
                Err(_) => {
                    self.owner.fault = Some(Fault::MissingDisplay);
                    break;
                }
            }
        }
    }
}
impl d::Context for Runner<'_> {
    type Id = MessageId;
    fn effect(
        &mut self,
        state: d::Dialogue<MessageId>,
        effect: d::Effect,
    ) -> d::Dialogue<MessageId> {
        if self.owner.fault.is_some() {
            return state;
        }
        // Fallible canvas operations publish their current runner prefix too.
        self.owner.state.runner = RunnerState::from_dialogue(state);
        let mut state = self.owner.state.runner.dialogue();
        match effect {
            d::Effect::CancelSounds => return self.emit(state, Effect::CancelSounds),
            d::Effect::ClearSelection => state.selection = d::Selection::Accept,
            d::Effect::PrepareDisplay => {
                self.owner.state.visible = true;
                self.apply(|workspace| {
                    workspace.write_slice(Point::new(Role::Pause, 0), &[0; 0x82])
                });
            }
            d::Effect::SaveBackground => {
                for (index, word) in self.owner.state.map.iter().enumerate() {
                    if (crate::presentation_workspace::PublishedWorkspace {
                        workspace: self.workspace,
                        publication: self.effects,
                    })
                    .write_slice(
                        Point::new(Role::SavedBackground, index * 2),
                        &word.to_le_bytes(),
                    )
                    .is_err()
                    {
                        self.owner.fault = Some(Fault::MissingDisplay);
                        break;
                    }
                }
            }
            d::Effect::FillBackground => self.apply(|workspace| canvas::background(workspace)),
            d::Effect::UploadBackground => self.upload(0, MAP_WORDS * 2, Role::Equipment),
            d::Effect::DrawTopBorder => match definition(&self.owner.definitions, state.message) {
                Ok(message) => {
                    let size = message.size;
                    self.apply(|workspace| canvas::row(workspace, 0, border(size)));
                }
                Err(fault) => self.owner.fault = Some(fault),
            },
            d::Effect::WriteMessage => {
                state.radius = 0;
                self.owner.state.runner.radius = 0;
                self.apply(|workspace| {
                    workspace.write_slice(Point::new(Role::Pause, 0), &[0; 0xe0])
                });
                if self.owner.fault.is_some() {
                    return state;
                }
                match definition(&self.owner.definitions, state.message) {
                    Ok(message) => {
                        let size = message.size;
                        let mut cursor = TextCursor::new(
                            u16::try_from(message.text.len() * 2)
                                .expect("validated message length"),
                        );
                        loop {
                            let tile = message.text[usize::from(cursor.source_offset >> 1)];
                            if canvas::tile(
                                &mut crate::presentation_workspace::PublishedWorkspace {
                                    workspace: self.workspace,
                                    publication: self.effects,
                                },
                                cursor.word,
                                tile,
                            )
                            .is_err()
                            {
                                self.owner.fault = Some(Fault::Definition);
                                break;
                            }
                            if cursor.advance() {
                                break;
                            }
                        }
                        if self.owner.fault.is_none() {
                            let end = cursor.word;
                            self.apply(|workspace| canvas::row(workspace, end, border(size)));
                        }
                    }
                    Err(fault) => self.owner.fault = Some(fault),
                }
            }
            d::Effect::PrepareScroll => match definition(&self.owner.definitions, state.message) {
                Ok(message) => {
                    let modifier = message.modifier;
                    let offset = message.button_byte_offset;
                    if modifier != ButtonModifier::None {
                        let buttons = if modifier == ButtonModifier::Shoot {
                            self.owner.start.shoot_button
                        } else {
                            self.owner.start.run_button
                        };
                        if let Some(offset) = offset {
                            self.apply(|workspace| {
                                workspace.write_slice(
                                    Point::new(Role::MessageText, usize::from(offset)),
                                    &button_letter(buttons).to_le_bytes(),
                                )
                            });
                        } else {
                            self.owner.fault = Some(Fault::Definition);
                        }
                    }
                    if self.owner.fault.is_none() {
                        self.apply(|workspace| {
                            workspace.write_slice(
                                Point::new(Role::MessageDescriptor, 0),
                                &HDMA_DESCRIPTOR,
                            )
                        });
                    }
                }
                Err(fault) => self.owner.fault = Some(fault),
            },
            d::Effect::EnableScroll => {}
            d::Effect::UploadMessage => match definition(&self.owner.definitions, state.message) {
                Ok(message) => {
                    let first = message.upload_offset();
                    let bytes = usize::from(dma_length(
                        u16::try_from(message.text.len() * 2).expect("validated message length"),
                    ));
                    self.upload(first, bytes, Role::MessageText);
                }
                Err(fault) => self.owner.fault = Some(fault),
            },
            d::Effect::SaveSound => return self.emit(state, Effect::SaveSound),
            d::Effect::AdvanceAudio => return self.emit(state, Effect::AdvanceAudio),
            d::Effect::AdvanceSoundEffects => return self.emit(state, Effect::AdvanceSoundEffects),
            d::Effect::UpdateHdma => return self.emit(state, Effect::UpdateHdma),
            d::Effect::RestoreBackground => self.upload(0, MAP_WORDS * 2, Role::SavedBackground),
            d::Effect::RestoreDisplay => {
                self.owner.state.visible = false;
                self.owner.state.restored = true;
            }
            d::Effect::ResumeMovementSound => return self.emit(state, Effect::ResumeMovementSound),
            d::Effect::SelectPause => return self.emit(state, Effect::SelectPause),
            d::Effect::DrawRadius => {
                match canvas::radius(
                    &mut crate::presentation_workspace::PublishedWorkspace {
                        workspace: self.workspace,
                        publication: self.effects,
                    },
                    state.radius,
                ) {
                    Ok(values) => self.owner.state.drawn_scroll = values,
                    Err(_) => self.owner.fault = Some(Fault::Definition),
                }
            }
            d::Effect::StageSelection => {
                self.apply(|workspace| {
                    canvas::row(workspace, 128, selection_row(selection(state.selection)))
                });
            }
            d::Effect::UploadSelection => {
                let first = 0x5800 + usize::from(map_offset(BoxSize::Large)) - MAP_BASE;
                self.upload(first, 0x180, Role::MessageText);
            }
            d::Effect::SelectionSound => return self.emit(state, Effect::SelectionSound),
        }
        if self.owner.fault.is_none() {
            self.owner.state.runner = RunnerState::from_dialogue(state);
            return self.owner.state.runner.dialogue();
        }
        state
    }
    fn wait(&mut self, state: d::Dialogue<MessageId>) -> (d::Dialogue<MessageId>, d::Outcome) {
        if self.owner.fault.is_some() {
            return (
                state,
                d::Outcome::Suspended(std::num::NonZeroU8::new(1).unwrap()),
            );
        }
        if self.owner.state.wait_pending {
            self.owner.state.wait_pending = false;
            (state, d::Outcome::Complete)
        } else {
            self.owner.state.wait_pending = true;
            (
                state,
                d::Outcome::Suspended(std::num::NonZeroU8::new(1).unwrap()),
            )
        }
    }
    fn input(&mut self, state: d::Dialogue<MessageId>) -> (d::Dialogue<MessageId>, d::Input) {
        if self.owner.fault.is_some() {
            // This inert result drains only discarded local runner bookkeeping;
            // it never samples input or changes the retained failed execution.
            return (
                state,
                d::Input {
                    confirm: false,
                    cancel: false,
                    toggle: false,
                    dismiss: false,
                },
            );
        }
        let sample = self
            .owner
            .sample
            .expect("resume admits input before dialogue polling");
        (
            state,
            d::Input {
                confirm: sample.new_keys & 0x80 != 0,
                cancel: sample.new_keys & 0x8000 != 0,
                toggle: sample.new_keys & (0x2000 | 0x200 | 0x100) != 0,
                dismiss: sample.held_keys != 0,
            },
        )
    }
}

/// One sink receives byte publication while the actual allocation is leased,
/// and receives synchronous effects only after its actual execution is exposed.
pub trait WorkspaceEffects: crate::presentation_workspace::Publication {
    fn effect(
        &mut self,
        effect: Effect,
        workspace: &mut Workspace,
        execution: &mut ExecutionDescription,
    ) -> Result<(), Fault>;
}
struct ExtractionEffects<F>(F);
impl<F> crate::presentation_workspace::Publication for ExtractionEffects<F> {
    fn published(
        &mut self,
        _: &Workspace,
        _: crate::presentation_workspace::SourcePoint,
        _: usize,
    ) {
    }
}
impl<F: FnMut(Effect, &mut Workspace, &mut ExecutionDescription) -> Result<(), Fault>>
    WorkspaceEffects for ExtractionEffects<F>
{
    fn effect(
        &mut self,
        effect: Effect,
        workspace: &mut Workspace,
        execution: &mut ExecutionDescription,
    ) -> Result<(), Fault> {
        (self.0)(effect, workspace, execution)
    }
}

use crate::presentation_workspace::SliceWriter as _;
