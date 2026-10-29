//! Owned requests and immediate receipts for the retained audio transport.
#![forbid(unsafe_code)]

pub use sm_data::{CueAction, CuePhase, EmittedCue, MusicCue, SoundCue};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeliveryId {
    pub epoch: u64,
    pub input_tick: u64,
    pub ordinal: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionPosition {
    pub branch: u8,
    pub pc: Option<u32>,
    pub native_frame: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortTarget {
    Queue,
    Physical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioOutput {
    InputBegin {
        epoch: u64,
        tick: u64,
    },
    ExecutionBegin {
        position: ExecutionPosition,
    },
    CueBegin {
        id: DeliveryId,
        emission: Option<sm_data::EmittedCue>,
    },
    CueComplete {
        id: DeliveryId,
    },
    Port {
        id: DeliveryId,
        cue: Option<DeliveryId>,
        position: ExecutionPosition,
        target: PortTarget,
        port: u8,
        value: u8,
    },
    InputComplete {
        epoch: u64,
        tick: u64,
    },
}

/// A request lives only through the immediate adapter call, never in a journal.
pub(crate) struct CueRequest {
    pub(crate) id: DeliveryId,
    pub(crate) emission: Option<sm_data::EmittedCue>,
}

pub(crate) struct PortRequest {
    id: DeliveryId,
    cue: Option<DeliveryId>,
    position: ExecutionPosition,
    target: PortTarget,
    pub(crate) port: u8,
    pub(crate) value: u8,
}

/// Every normal/headless adapter uses this same accepting transport. The receipt
/// consumer acknowledges each owned output immediately; it cannot retain Game.
/// No complete replay or physical upload is buffered by this owner.
pub struct AudioOutputSink {
    epoch: u64,
    input_tick: u64,
    ordinal: u64,
    active: Option<DeliveryId>,
    input_open: bool,
    failure: Option<String>,
    consume: Box<dyn FnMut(AudioOutput) -> Result<(), String>>,
}

impl Default for AudioOutputSink {
    fn default() -> Self {
        Self::new(|_| Ok(()))
    }
}

impl AudioOutputSink {
    pub fn new(consume: impl FnMut(AudioOutput) -> Result<(), String> + 'static) -> Self {
        Self {
            epoch: 0,
            input_tick: 0,
            ordinal: 0,
            active: None,
            input_open: false,
            failure: None,
            consume: Box::new(consume),
        }
    }

    fn publish(&mut self, output: AudioOutput) {
        // A failed receiver has rejected this output namespace. Preserve the
        // native delivery prefix, retain the first diagnostic, and bound further
        // recording until the caller reaches its existing safe error boundary.
        if self.failure.is_some() {
            return;
        }
        if let Err(error) = (self.consume)(output) {
            if self.failure.is_none() {
                self.failure = Some(error);
            }
        }
    }

    fn next_id(&mut self) -> DeliveryId {
        let id = DeliveryId {
            epoch: self.epoch,
            input_tick: self.input_tick,
            ordinal: self.ordinal,
        };
        self.ordinal = self.ordinal.checked_add(1).expect("audio ordinal overflow");
        id
    }

    pub(crate) fn begin_input(&mut self, tick: u64) {
        if self.input_open || self.active.is_some() {
            self.failure
                .get_or_insert_with(|| "incomplete prior audio input".into());
        }
        self.input_tick = tick;
        self.ordinal = 0;
        self.input_open = true;
        self.publish(AudioOutput::InputBegin {
            epoch: self.epoch,
            tick,
        });
    }

    pub(crate) fn complete_input(&mut self) {
        if self.active.is_some() || !self.input_open {
            self.failure
                .get_or_insert_with(|| "incomplete audio request/input".into());
        }
        self.publish(AudioOutput::InputComplete {
            epoch: self.epoch,
            tick: self.input_tick,
        });
        self.input_open = false;
    }

    pub(crate) fn enter_execution(&mut self, position: ExecutionPosition) {
        if self.active.is_some() {
            self.failure
                .get_or_insert_with(|| "incomplete Cue at audio execution boundary".into());
        }
        self.publish(AudioOutput::ExecutionBegin { position });
    }

    /// A reset/restore starts a new live-output namespace; semantic pending Cues
    /// remain with their existing gameplay owner and persisted request schema.
    pub fn new_epoch(&mut self) {
        self.epoch = self.epoch.checked_add(1).expect("audio epoch overflow");
        self.ordinal = 0;
        self.active = None;
        self.input_open = false;
        self.failure = None;
    }

    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    pub(crate) fn accept_cue(&mut self, emission: Option<sm_data::EmittedCue>) -> CueRequest {
        if self.active.is_some() {
            self.failure
                .get_or_insert_with(|| "nested audio Cue acceptance".into());
        }
        let id = self.next_id();
        self.active = Some(id);
        self.publish(AudioOutput::CueBegin {
            id,
            emission: emission.clone(),
        });
        CueRequest { id, emission }
    }

    pub(crate) fn complete_cue(&mut self, request: CueRequest) {
        if self.active != Some(request.id) {
            self.failure
                .get_or_insert_with(|| "unpaired audio Cue completion".into());
        }
        self.publish(AudioOutput::CueComplete { id: request.id });
        self.active = None;
    }

    pub(crate) fn accept_port(
        &mut self,
        position: ExecutionPosition,
        target: PortTarget,
        port: u8,
        value: u8,
    ) -> PortRequest {
        PortRequest {
            id: self.next_id(),
            cue: self.active,
            position,
            target,
            port,
            value,
        }
    }

    /// Called only after the queue assignment or native port mutation succeeds.
    /// Failed native calls unwind with their original prefix and no false receipt.
    pub(crate) fn complete_port(&mut self, request: PortRequest) {
        self.publish(AudioOutput::Port {
            id: request.id,
            cue: request.cue,
            position: request.position,
            target: request.target,
            port: request.port,
            value: request.value,
        });
    }
}
