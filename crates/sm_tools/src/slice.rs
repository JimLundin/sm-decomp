//! Gate 1's headless evidence: a Morph room replay through the Session interface
//! with the effect outbox in the loop, repeated to show what must not change it.
use crate::trace::{Running, Tick};
use sm_data::prepared::{PreparedContent, PreparedStart};
use sm_host::session::{AudioBatch, HostResult, SaveRequest, Session, TickInput};
use sm_runtime::outbox::{EffectId, Outbox, Sink, TickBatch};
use std::collections::VecDeque;

/// A run's scheduling: nothing here may change the game.
#[derive(Clone, Copy, Debug, Default)]
pub struct Schedule {
    /// The audio queue refuses until this many offers have failed, per batch.
    pub audio_held_offers: u32,
    /// The save sink refuses until this many offers have failed, per request.
    pub save_held_offers: u32,
    /// Read the Scene every this many Ticks; 0 never does.
    pub present_every: u32,
    /// Whether to read the digests every Tick and fold them for the lock.
    pub digests: bool,
}

/// What a run produced.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub ticks: u64,
    pub lock_toml: String,
    /// Each Tick's digest line, when the schedule reads the digests.
    pub digest_lines: Vec<String>,
    pub effect_trace: u64,
    pub audio_batches: u64,
    pub cues: u64,
    pub saves: u64,
    pub receipts: u64,
    /// The most Tick batches ever held, the proof of the bound.
    pub most_waiting: usize,
    pub faulted: bool,
}

struct AudioQueue {
    held_offers: u32,
    refused: u32,
    taken: u64,
    cues: u64,
    last: Option<EffectId>,
}

impl Sink<AudioBatch> for AudioQueue {
    fn offer(&mut self, id: EffectId, batch: AudioBatch) -> Result<(), AudioBatch> {
        if self.refused < self.held_offers {
            self.refused += 1;
            return Err(batch);
        }
        assert!(
            self.last.is_none_or(|last| last < id),
            "a duplicated or reordered sound batch: {id:?}"
        );
        self.refused = 0;
        self.last = Some(id);
        self.taken += 1;
        self.cues += batch.cues.len() as u64;
        Ok(())
    }
}

/// Accepts into a two-entry worker queue; the store finishes on a later offer,
/// and only then does the receipt appear.
struct SaveWorker {
    held_offers: u32,
    refused: u32,
    active: VecDeque<(EffectId, SaveRequest)>,
    receipts: Vec<HostResult>,
    accepted: u64,
    last_sram: Vec<u8>,
}

impl SaveWorker {
    fn turn(&mut self) {
        if let Some((id, request)) = self.active.pop_front() {
            self.last_sram = request.sram;
            self.receipts.push(HostResult::SaveStored {
                tick: id.tick,
                stored: true,
            });
        }
    }
}

impl Sink<SaveRequest> for SaveWorker {
    fn offer(&mut self, id: EffectId, request: SaveRequest) -> Result<(), SaveRequest> {
        if self.refused < self.held_offers || self.active.len() == 2 {
            self.refused += 1;
            return Err(request);
        }
        self.refused = 0;
        self.accepted += 1;
        self.active.push_back((id, request));
        Ok(())
    }
}

/// Replay the first `ticks` of `pads` from the start, with the outbox between the
/// Session and its sinks. Each Tick's closing NMI latches the next word, the one
/// after the last Tick included when `pads` has it, as the verifier's replay does.
pub fn run(
    content: &PreparedContent,
    start: &PreparedStart,
    pads: &[u16],
    ticks: usize,
    schedule: Schedule,
) -> Result<Outcome, String> {
    let opened = Session::open(content, start).map_err(|error| error.to_string())?;
    let mut session = opened.session;
    let mut running = Running::default();
    let mut outbox: Outbox<AudioBatch, SaveRequest> = Outbox::new(1);
    let mut audio = AudioQueue {
        held_offers: schedule.audio_held_offers,
        refused: 0,
        taken: 0,
        cues: 0,
        last: None,
    };
    let mut save = SaveWorker {
        held_offers: schedule.save_held_offers,
        refused: 0,
        active: VecDeque::new(),
        receipts: Vec::new(),
        accepted: 0,
        last_sram: Vec::new(),
    };
    let mut presented = opened
        .initial
        .scene
        .pixels
        .iter()
        .map(|&b| u64::from(b))
        .sum::<u64>();
    let mut most_waiting = 0;
    let mut saves = 0;
    let mut digest_lines = Vec::new();
    for (index, &pad) in pads.iter().take(ticks).enumerate() {
        // Admit: no Tick while the last one's effects wait. The loop stays live:
        // every failed offer returns here, where a real loop handles its events.
        let mut turns = 0u32;
        while !outbox.admits() {
            outbox.offer(&mut audio, &mut save);
            save.turn();
            turns += 1;
            assert!(turns < 1_000_000, "a sink never accepted");
        }
        save.turn();
        let output = session.step(TickInput {
            buttons: pad,
            latched: pads.get(index + 1).copied(),
            host_results: std::mem::take(&mut save.receipts),
        });
        if schedule.present_every != 0 && output.tick % u64::from(schedule.present_every) == 0 {
            presented = presented.wrapping_add(
                output
                    .scene
                    .pixels
                    .iter()
                    .map(|&b| u64::from(b))
                    .sum::<u64>(),
            );
        }
        saves += u64::from(output.effects.save.is_some());
        most_waiting = most_waiting.max(1);
        if outbox
            .put(TickBatch {
                tick: output.tick,
                audio: output.effects.audio,
                save: output.effects.save,
            })
            .is_err()
        {
            return Err("a Tick ran while its predecessor's effects waited".into());
        }
        outbox.offer(&mut audio, &mut save);
        if schedule.digests {
            let machine = session.digests();
            digest_lines.push(session.line_of(&machine));
            running.fold(&Tick {
                index: index as u32,
                tick: index as i32,
                machine,
                typed: None,
                session: None,
            });
        }
    }
    while !outbox.admits() {
        outbox.offer(&mut audio, &mut save);
        save.turn();
    }
    save.turn();
    let _ = presented;
    let delivered = session.step_count();
    let receipts_outstanding = save.receipts.len() as u64;
    Ok(Outcome {
        ticks: delivered,
        lock_toml: running.lock_toml(),
        digest_lines,
        effect_trace: session.effect_trace(),
        audio_batches: audio.taken,
        cues: audio.cues,
        saves,
        receipts: session.receipts().len() as u64 + receipts_outstanding,
        most_waiting,
        faulted: session.faulted(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_audio_queue_refuses_without_losing_the_batch() {
        let mut queue = AudioQueue {
            held_offers: 2,
            refused: 0,
            taken: 0,
            cues: 0,
            last: None,
        };
        let id = EffectId {
            epoch: 1,
            tick: 1,
            ordinal: 0,
        };
        let batch = AudioBatch::default();
        let batch = queue.offer(id, batch).unwrap_err();
        let batch = queue.offer(id, batch).unwrap_err();
        assert!(queue.offer(id, batch).is_ok());
        assert_eq!(queue.taken, 1);
    }
}
