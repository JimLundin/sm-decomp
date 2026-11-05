//! The coordinator's effect outbox: one completed Tick's effects, offered to
//! each sink until it takes them (docs/target-architecture/CONTRACTS.md).
//!
//! A sink takes ownership of its effect or hands it back unchanged. Acceptance
//! means enqueued, not played or durable. A sink that accepted is never offered
//! the batch again, so a blocked save cannot duplicate an accepted sound. No new
//! Tick is admitted while a batch waits.
#![forbid(unsafe_code)]

/// An effect's identity within one running timeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectId {
    pub epoch: u64,
    pub tick: u64,
    pub ordinal: u32,
}

/// A runtime adapter: it takes ownership of `effect`, or returns it unchanged.
pub trait Sink<T> {
    fn offer(&mut self, id: EffectId, effect: T) -> Result<(), T>;
}

/// One completed Tick's effects.
pub struct TickBatch<A, S> {
    pub tick: u64,
    pub audio: A,
    pub save: Option<S>,
}

/// What an offer left waiting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Waiting {
    pub audio: bool,
    pub save: bool,
}

impl Waiting {
    pub fn any(self) -> bool {
        self.audio || self.save
    }
}

/// Holds at most one Tick's batch, with one acceptance per sink.
pub struct Outbox<A, S> {
    epoch: u64,
    tick: u64,
    audio: Option<A>,
    save: Option<S>,
}

impl<A, S> Outbox<A, S> {
    pub fn new(epoch: u64) -> Self {
        Self {
            epoch,
            tick: 0,
            audio: None,
            save: None,
        }
    }

    /// Whether the coordinator may run the next Tick.
    pub fn admits(&self) -> bool {
        self.audio.is_none() && self.save.is_none()
    }

    /// Move a completed Tick's effects in; `false` hands nothing over because a
    /// batch still waits, and the caller must not have run that Tick.
    pub fn put(&mut self, batch: TickBatch<A, S>) -> Result<(), TickBatch<A, S>> {
        if !self.admits() {
            return Err(batch);
        }
        self.tick = batch.tick;
        self.audio = Some(batch.audio);
        self.save = batch.save;
        Ok(())
    }

    /// Offer whatever waits to its sink, audio first, without blocking.
    pub fn offer(&mut self, audio: &mut impl Sink<A>, save: &mut impl Sink<S>) -> Waiting {
        let id = |ordinal| EffectId {
            epoch: self.epoch,
            tick: self.tick,
            ordinal,
        };
        if let Some(effect) = self.audio.take() {
            self.audio = audio.offer(id(0), effect).err();
        }
        if let Some(effect) = self.save.take() {
            self.save = save.offer(id(1), effect).err();
        }
        Waiting {
            audio: self.audio.is_some(),
            save: self.save.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sink that refuses until `open`, recording what it accepted.
    #[derive(Default)]
    struct Held {
        open: bool,
        taken: Vec<(EffectId, u32)>,
    }

    impl Sink<u32> for Held {
        fn offer(&mut self, id: EffectId, effect: u32) -> Result<(), u32> {
            if self.open {
                self.taken.push((id, effect));
                Ok(())
            } else {
                Err(effect)
            }
        }
    }

    fn batch(tick: u64, save: Option<u32>) -> TickBatch<u32, u32> {
        TickBatch {
            tick,
            audio: tick as u32,
            save,
        }
    }

    #[test]
    fn a_held_audio_queue_stops_admission_and_loses_nothing() {
        let mut outbox = Outbox::new(1);
        let (mut audio, mut save) = (Held::default(), Held::default());
        assert!(outbox.put(batch(1, None)).is_ok());
        for _ in 0..1000 {
            assert!(outbox.offer(&mut audio, &mut save).audio);
            assert!(!outbox.admits());
        }
        assert!(outbox.put(batch(2, None)).is_err());
        audio.open = true;
        assert!(!outbox.offer(&mut audio, &mut save).any());
        assert!(outbox.admits());
        assert_eq!(audio.taken.len(), 1);
        assert_eq!(audio.taken[0].0.tick, 1);
    }

    #[test]
    fn a_blocked_save_never_duplicates_an_accepted_sound() {
        let mut outbox = Outbox::new(1);
        let mut audio = Held {
            open: true,
            ..Held::default()
        };
        let mut save = Held::default();
        assert!(outbox.put(batch(7, Some(70))).is_ok());
        for _ in 0..100 {
            let waiting = outbox.offer(&mut audio, &mut save);
            assert!(waiting.save && !waiting.audio);
            assert!(!outbox.admits());
        }
        assert_eq!(audio.taken.len(), 1);
        save.open = true;
        assert!(!outbox.offer(&mut audio, &mut save).any());
        assert_eq!(audio.taken.len(), 1);
        assert_eq!(save.taken.len(), 1);
        assert_eq!(save.taken[0].0.ordinal, 1);
        assert_eq!(save.taken[0].0.tick, 7);
    }

    #[test]
    fn identities_are_ordered_across_ticks() {
        let mut outbox = Outbox::new(3);
        let (mut audio, mut save) = (
            Held {
                open: true,
                ..Held::default()
            },
            Held {
                open: true,
                ..Held::default()
            },
        );
        for tick in 1..=10 {
            assert!(outbox.put(batch(tick, Some(1))).is_ok());
            assert!(!outbox.offer(&mut audio, &mut save).any());
        }
        assert!(audio.taken.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(audio.taken.iter().all(|taken| taken.0.epoch == 3));
    }
}
