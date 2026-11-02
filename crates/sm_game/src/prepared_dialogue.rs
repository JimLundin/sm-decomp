//! Complete suspended dialogue execution on the persistent Session.
#![forbid(unsafe_code)]
use crate::dialogue as d;
use crate::message_composition::prepared::{
    ExecutionDescription, InputSample, PreparedOwner, StepOutput,
};
use crate::presentation_workspace::Workspace;
use sm_data::message::{Definitions, Display, Fault, MessageId, PreparedStart, RadiusValues};
use std::{collections::BTreeMap, sync::Arc};

/// Immutable caller supplied content and its independently sealed manifest.
#[derive(Clone)]
pub struct Content {
    pub manifest: [u8; 32],
    pub definitions: Arc<Definitions>,
    pub start: Arc<PreparedStart>,
    pub native_messages: BTreeMap<u16, MessageId>,
}
impl Content {
    pub fn declarations(&self) -> Vec<u8> {
        let mut out = self.manifest.to_vec();
        out.extend_from_slice(&self.definitions.revision().0);
        let font = self.definitions.font();
        out.extend_from_slice(&font.id.0.to_le_bytes());
        out.extend_from_slice(&font.revision.0);
        put_word(&mut out, font.first_word);
        put_words(&mut out, &font.words);
        let messages: Vec<_> = self.definitions.entries().collect();
        out.extend_from_slice(&(messages.len() as u64).to_le_bytes());
        for message in messages {
            put_id(&mut out, message.id);
            out.push(match message.kind {
                sm_data::message::MessageKind::Content => 0,
                sm_data::message::MessageKind::Map => 1,
                sm_data::message::MessageKind::Energy => 2,
                sm_data::message::MessageKind::Missiles => 3,
                sm_data::message::MessageKind::Confirmation => 4,
                sm_data::message::MessageKind::SavePrompt => 5,
                sm_data::message::MessageKind::Saved => 6,
            });
            out.push(match message.size {
                sm_data::message::BoxSize::Small => 0,
                sm_data::message::BoxSize::Large => 1,
            });
            out.push(match message.modifier {
                sm_data::message::ButtonModifier::None => 0,
                sm_data::message::ButtonModifier::Shoot => 1,
                sm_data::message::ButtonModifier::Run => 2,
            });
            out.push(u8::from(message.button_byte_offset.is_some()));
            if let Some(offset) = message.button_byte_offset {
                put_word(&mut out, offset);
            }
            put_words(&mut out, &message.text);
        }
        for display in [&self.start.saved, &self.start.active, &self.start.restored] {
            put_display(&mut out, display);
        }
        out.extend_from_slice(&self.start.initial_interval.to_le_bytes());
        out.push(u8::from(self.start.initial_wait_pending));
        for word in self
            .start
            .tile_workspace
            .iter()
            .copied()
            .chain([self.start.shoot_button, self.start.run_button])
        {
            out.extend_from_slice(&word.to_le_bytes());
        }
        out.extend_from_slice(&(self.native_messages.len() as u64).to_le_bytes());
        for (native, id) in &self.native_messages {
            out.extend_from_slice(&native.to_le_bytes());
            out.extend_from_slice(&id.0.to_le_bytes());
        }
        out
    }
    pub fn validate(
        &self,
        execution: &ExecutionDescription,
        workspace: &Workspace,
    ) -> Result<(), Fault> {
        if self.manifest == [0; 32] {
            return Err(Fault::ChangedContent);
        }
        PreparedOwner::from_execution(
            self.definitions.clone(),
            self.start.clone(),
            execution.clone(),
            workspace,
        )
        .map(|_| ())
    }
    pub fn resume<Frame>(
        &self,
        execution: &mut ExecutionDescription,
        interval: u64,
        sample: InputSample,
        effects: &mut impl crate::message_composition::prepared::WorkspaceEffects,
        workspace: &mut Workspace,
        render: impl FnMut(Display<'_>) -> Result<Frame, Fault>,
    ) -> Result<StepOutput<Frame>, Fault> {
        let owner = PreparedOwner::from_execution(
            self.definitions.clone(),
            self.start.clone(),
            execution.clone(),
            workspace,
        )?;
        let mut lease = ExecutionLease { owner, execution };
        lease
            .owner
            .resume_with_publications(interval, sample, workspace, effects, render)
    }
}

/// Rejoin the execution before outer Session unwind handling observes it.
/// Workspace remains the independently leased, actual allocation throughout.
struct ExecutionLease<'a> {
    owner: PreparedOwner,
    execution: &'a mut ExecutionDescription,
}
impl Drop for ExecutionLease<'_> {
    fn drop(&mut self) {
        *self.execution = self.owner.execution();
    }
}

/// Executes the original synchronous game effect at its emitted source phase.
/// All current owners, including the exact selected execution, are rejoined.
pub trait Effects {
    /// Existing extraction-only callers retain their admitted wait state.
    /// The actor binding overrides this with the actual 0782 request/ack owner.
    fn wait(&mut self, session: &mut crate::session::Session) -> Result<d::Outcome, Fault> {
        let execution = session
            .prepared_dialogue
            .as_mut()
            .ok_or(Fault::MissingDisplay)?;
        let pending = execution.wait_pending;
        execution.wait_pending = !pending;
        Ok(if pending {
            d::Outcome::Complete
        } else {
            d::Outcome::Suspended(std::num::NonZeroU8::new(1).unwrap())
        })
    }
    /// Rejoin the existing full owner only when an effect needs actor execution.
    /// Existing extraction callers continue to use their Session-only sink.
    fn apply_owner(
        &mut self,
        effect: crate::message_composition::prepared::Effect,
        owner: &mut crate::persistent_session::PersistentSession,
    ) -> Result<(), Fault> {
        self.apply(effect, &mut owner.gameplay)
    }
    fn input_owner(
        &mut self,
        _owner: &mut crate::persistent_session::PersistentSession,
        sample: crate::message_composition::prepared::InputSample,
    ) -> Result<crate::message_composition::prepared::InputSample, Fault> {
        Ok(sample)
    }
    fn apply(
        &mut self,
        effect: crate::message_composition::prepared::Effect,
        session: &mut crate::session::Session,
    ) -> Result<(), Fault>;
}
pub(crate) fn publish(
    session: &mut crate::session::Session,
    content: &Content,
) -> Result<(), Fault> {
    let execution = session
        .prepared_dialogue
        .as_ref()
        .ok_or(Fault::MissingDisplay)?;
    let active = match execution.message {
        d::Message::Content(MessageId(0)) => 0,
        d::Message::Content(id) => *content
            .native_messages
            .iter()
            .find(|(_, candidate)| **candidate == id)
            .map(|(native, _)| native)
            .ok_or(Fault::Definition)?,
        d::Message::MapAcquired => 20,
        d::Message::EnergyRestored => 21,
        d::Message::MissilesRestored => 22,
        d::Message::SaveConfirmation => 23,
        d::Message::SavePrompt => 28,
        d::Message::Saved => 24,
    };
    let encoded = crate::dialogue_ram::encode(d::Dialogue {
        message: crate::dialogue_ram::message(active),
        continuation: execution.continuation,
        child: execution.child,
        delay_remaining: execution.delay,
        radius: execution.radius,
        selection: execution.selection,
    });
    session.messages.active_index = encoded.index;
    session.messages.delay = encoded.delay;
    session.messages.box_radius = encoded.radius;
    session.messages.selection = encoded.selection;
    session.messages.box_scratch = execution.drawn_scroll.scratch;
    session.control.write_projected_byte(0x615, encoded.child);
    session.control.write_projected_byte(0x614, encoded.parent);
    Ok(())
}

fn put_words(out: &mut Vec<u8>, words: &[u16]) {
    out.extend_from_slice(&(words.len() as u64).to_le_bytes());
    for word in words {
        put_word(out, *word);
    }
}
fn put_display(out: &mut Vec<u8>, display: &sm_data::message::FullDisplay) {
    out.extend_from_slice(&display.media_id.0.to_le_bytes());
    out.extend_from_slice(&display.revision.0);
    put_words(out, display.media.words());
    put_words(out, display.media.colors());
    let sprites = &display.sprites;
    put_words(out, &sprites.words);
    out.extend_from_slice(&sprites.extension);
    put_words(out, &sprites.tile_words);
    out.extend_from_slice(&[sprites.size, u8::from(sprites.rotation.is_some())]);
    if let Some(rotation) = sprites.rotation {
        out.push(rotation);
    }
    out.extend_from_slice(&[u8::from(sprites.interlace), u8::from(sprites.even_frame)]);
    let mosaic = &display.mosaic;
    out.extend_from_slice(&[
        mosaic.native_enabled,
        u8::from(mosaic.background0_enabled),
        mosaic.size,
        mosaic.start_line,
    ]);
    out.extend_from_slice(&mosaic.modulo);
    for line in display.lines.iter() {
        out.extend_from_slice(&[
            line.mode,
            u8::from(line.extended_background),
            u8::from(line.forced_blank),
        ]);
        for background in line.backgrounds {
            for word in [
                background.horizontal_scroll,
                background.vertical_scroll,
                background.map_words,
                background.tile_words,
            ] {
                put_word(out, word);
            }
            out.extend_from_slice(&[u8::from(background.wider), u8::from(background.higher)]);
        }
        out.extend_from_slice(&line.windows.first);
        out.extend_from_slice(&line.windows.second);
        out.extend_from_slice(&line.windows.native_flags.to_le_bytes());
        for window in line.windows.layers {
            out.extend_from_slice(&[
                u8::from(window.first_enabled),
                u8::from(window.first_inverted),
                u8::from(window.second_enabled),
                u8::from(window.second_inverted),
                window.logic,
            ]);
        }
        out.extend_from_slice(&line.routing.enabled);
        out.extend_from_slice(&line.routing.windowed);
        for word in line.affine.matrix {
            out.extend_from_slice(&word.to_le_bytes());
        }
        out.extend_from_slice(&[
            u8::from(line.affine.horizontal_flip),
            u8::from(line.affine.vertical_flip),
            u8::from(line.affine.large_field),
            u8::from(line.affine.character_fill),
        ]);
        out.extend_from_slice(&[line.color.clip, line.color.prevent]);
        for enabled in line.color.enabled {
            out.push(u8::from(enabled));
        }
        out.extend_from_slice(&[
            u8::from(line.color.add_subscreen),
            u8::from(line.color.subtract),
            u8::from(line.color.half),
        ]);
        out.extend_from_slice(&line.color.fixed);
        out.push(line.color.brightness);
    }
    out.extend_from_slice(&[
        u8::from(display.overflow.range),
        u8::from(display.overflow.time),
    ]);
}
fn put_id(out: &mut Vec<u8>, id: MessageId) {
    out.extend_from_slice(&id.0.to_le_bytes());
}
fn put_option_id(out: &mut Vec<u8>, id: Option<MessageId>) {
    out.push(u8::from(id.is_some()));
    if let Some(id) = id {
        put_id(out, id);
    }
}
fn put_word(out: &mut Vec<u8>, word: u16) {
    out.extend_from_slice(&word.to_le_bytes());
}
pub fn encode(execution: &ExecutionDescription) -> Vec<u8> {
    let mut out = Vec::new();
    match execution.message {
        d::Message::Content(id) => {
            out.push(0);
            put_id(&mut out, id);
        }
        d::Message::MapAcquired => out.push(1),
        d::Message::EnergyRestored => out.push(2),
        d::Message::MissilesRestored => out.push(3),
        d::Message::SaveConfirmation => out.push(4),
        d::Message::SavePrompt => out.push(5),
        d::Message::Saved => out.push(6),
    }
    match execution.continuation {
        d::Continuation::Start => out.extend_from_slice(&[0, 0]),
        d::Continuation::Invalid(value) => out.extend_from_slice(&[1, value]),
        d::Continuation::Waiting(phase) => {
            let (tag, value) = match phase {
                d::Phase::Opening(cycle) => (2, cycle_tag(cycle)),
                d::Phase::Interacting(cycle) => (3, cycle_tag(cycle)),
                d::Phase::Closing(cycle) => (4, cycle_tag(cycle)),
                d::Phase::FeedbackDelay => (5, 0),
                d::Phase::Running(script) => (6, script_tag(script)),
            };
            out.extend_from_slice(&[tag, value]);
        }
    }
    let (tag, value) = match execution.child {
        d::ChildContinuation::Start => (0, 0),
        d::ChildContinuation::Waiting => (1, 0),
        d::ChildContinuation::ReadingDelay => (2, 0),
        d::ChildContinuation::Dismissing => (3, 0),
        d::ChildContinuation::Polling => (4, 0),
        d::ChildContinuation::Toggling => (5, 0),
        d::ChildContinuation::Step(value) => (6, value),
        d::ChildContinuation::Invalid(value) => (7, value),
    };
    out.extend_from_slice(&[tag, value]);
    put_word(&mut out, execution.radius);
    put_word(&mut out, execution.delay);
    match execution.selection {
        d::Selection::Accept => out.push(0),
        d::Selection::Decline => out.push(1),
        d::Selection::Other(value) => {
            out.push(2);
            put_word(&mut out, value);
        }
    }
    for word in execution.map {
        put_word(&mut out, word);
    }
    put_option_id(&mut out, execution.request);
    out.push(u8::from(execution.completed.is_some()));
    if let Some((id, selection)) = execution.completed {
        put_id(&mut out, id);
        put_word(&mut out, selection);
    }
    out.extend_from_slice(&[
        u8::from(execution.visible),
        u8::from(execution.restored),
        u8::from(execution.wait_pending),
    ]);
    out.extend_from_slice(&execution.last_interval.to_le_bytes());
    for word in [
        execution.drawn_scroll.top_scroll,
        execution.drawn_scroll.bottom_scroll,
    ]
    .into_iter()
    .chain(execution.drawn_scroll.scratch)
    {
        put_word(&mut out, word);
    }
    out
}
fn cycle_tag(cycle: d::Cycle) -> u8 {
    match cycle {
        d::Cycle::Message => 0,
        d::Cycle::SaveFeedback => 1,
    }
}
fn script_tag(script: d::Script) -> u8 {
    match script {
        d::Script::Open => 0,
        d::Script::Saving => 1,
        d::Script::Feedback => 2,
        d::Script::Close => 3,
    }
}
type Read<'a> = crate::actors::snapshot::CheckpointReader<'a>;
fn flag(read: &mut Read<'_>) -> Result<bool, String> {
    match read.byte()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err("dialogue flag".into()),
    }
}
fn id(read: &mut Read<'_>) -> Result<MessageId, String> {
    Ok(MessageId(u32::from_le_bytes(
        read.bytes(4)?.try_into().map_err(|_| "dialogue id")?,
    )))
}
fn option_id(read: &mut Read<'_>) -> Result<Option<MessageId>, String> {
    if flag(read)? {
        Ok(Some(id(read)?))
    } else {
        Ok(None)
    }
}
fn cycle(value: u8) -> Result<d::Cycle, String> {
    match value {
        0 => Ok(d::Cycle::Message),
        1 => Ok(d::Cycle::SaveFeedback),
        _ => Err("dialogue cycle".into()),
    }
}
pub fn decode(bytes: &[u8]) -> Result<ExecutionDescription, String> {
    let mut read = crate::actors::snapshot::CheckpointReader(bytes);
    let message = match read.byte()? {
        0 => d::Message::Content(id(&mut read)?),
        1 => d::Message::MapAcquired,
        2 => d::Message::EnergyRestored,
        3 => d::Message::MissilesRestored,
        4 => d::Message::SaveConfirmation,
        5 => d::Message::SavePrompt,
        6 => d::Message::Saved,
        _ => return Err("dialogue message".into()),
    };
    let tag = read.byte()?;
    let value = read.byte()?;
    let continuation = match tag {
        0 if value == 0 => d::Continuation::Start,
        1 => d::Continuation::Invalid(value),
        2 => d::Continuation::Waiting(d::Phase::Opening(cycle(value)?)),
        3 => d::Continuation::Waiting(d::Phase::Interacting(cycle(value)?)),
        4 => d::Continuation::Waiting(d::Phase::Closing(cycle(value)?)),
        5 if value == 0 => d::Continuation::Waiting(d::Phase::FeedbackDelay),
        6 => d::Continuation::Waiting(d::Phase::Running(match value {
            0 => d::Script::Open,
            1 => d::Script::Saving,
            2 => d::Script::Feedback,
            3 => d::Script::Close,
            _ => return Err("dialogue script".into()),
        })),
        _ => return Err("dialogue parent".into()),
    };
    let tag = read.byte()?;
    let value = read.byte()?;
    let child = match (tag, value) {
        (0, 0) => d::ChildContinuation::Start,
        (1, 0) => d::ChildContinuation::Waiting,
        (2, 0) => d::ChildContinuation::ReadingDelay,
        (3, 0) => d::ChildContinuation::Dismissing,
        (4, 0) => d::ChildContinuation::Polling,
        (5, 0) => d::ChildContinuation::Toggling,
        (6, value) => d::ChildContinuation::Step(value),
        (7, value) => d::ChildContinuation::Invalid(value),
        _ => return Err("dialogue child".into()),
    };
    let radius = read.word()?;
    let delay = read.word()?;
    let selection = match read.byte()? {
        0 => d::Selection::Accept,
        1 => d::Selection::Decline,
        2 => d::Selection::Other(read.word()?),
        _ => return Err("dialogue selection".into()),
    };
    let mut map = [0; 896];
    for word in &mut map {
        *word = read.word()?;
    }
    let request = option_id(&mut read)?;
    let completed = if flag(&mut read)? {
        Some((id(&mut read)?, read.word()?))
    } else {
        None
    };
    let visible = flag(&mut read)?;
    let restored = flag(&mut read)?;
    let wait_pending = flag(&mut read)?;
    let last_interval = read.counter()?;
    let drawn_scroll = RadiusValues {
        top_scroll: read.word()?,
        bottom_scroll: read.word()?,
        scratch: [read.word()?, read.word()?, read.word()?, read.word()?],
    };
    if !read.0.is_empty() {
        return Err("dialogue trailing fields".into());
    }
    Ok(ExecutionDescription {
        message,
        continuation,
        child,
        radius,
        selection,
        delay,
        map,
        request,
        completed,
        visible,
        restored,
        wait_pending,
        last_interval,
        drawn_scroll,
    })
}
impl PartialEq for ExecutionDescription {
    fn eq(&self, other: &Self) -> bool {
        encode(self) == encode(other)
    }
}
impl std::fmt::Debug for ExecutionDescription {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("ExecutionDescription")
            .field("message", &self.message)
            .field("continuation", &self.continuation)
            .field("child", &self.child)
            .field("last_interval", &self.last_interval)
            .finish_non_exhaustive()
    }
}
