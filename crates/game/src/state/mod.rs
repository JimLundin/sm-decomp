//! The State (D3): the whole Session as public plain data, and the Checkpoint around it.
//!
//! Slice 0's domains only (ARCHITECTURE §15 step 4). Values are typed; the original's
//! numbers appear only at the decoder, projections and encoders.
// open: the sound domain (`SoundCpu`, the driver model, D18), `progress` and `menus` join
// with their first rows; slice 0's item pickup needs `progress` once the decoder shows
// which collected-item bits its rows read.

pub mod hud;
pub mod room;
pub mod samus;
pub mod slots;
pub mod sram;
pub mod strand;
pub mod video;

use canonical::{Canonical, Schema};
use data::id::Cycles;
use slots::FlagSet;

use hud::Hud;
use room::RoomScoped;
use samus::Samus;
use strand::{IrqStrand, MainStrand, Message};
use video::Video;

/// Ticks since the Session opened.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(pub u64);

/// A wrapping 16-bit frame counter, as the original keeps its NMI counts.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameCount(pub u16);

/// The NMI counters.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    /// Every NMI.
    pub nmi: FrameCount,
    /// Requested NMIs.
    pub completed: FrameCount,
    /// Lag NMIs since the last requested one.
    pub lag: FrameCount,
    /// The longest lag run seen.
    pub max_lag: FrameCount,
    /// Requested NMIs in eight bits, which door transitions reset; the HUD's auto-cancel
    /// flash reads it.
    pub requested_byte: u8,
}

/// The in-game time on the file.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GameTime {
    /// Hours.
    pub hours: u16,
    /// Minutes.
    pub minutes: u16,
    /// Seconds.
    pub seconds: u16,
    /// Frames within the second.
    pub frames: u16,
}

/// Time.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Clock {
    /// The Tick counter.
    pub tick: Tick,
    /// The NMI counters.
    pub counters: Counters,
    /// The in-game time.
    pub game_time: GameTime,
}

/// The RNG and its call count, which the `rng` stream compares.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rng {
    /// The original's RNG word.
    pub word: u16,
    /// Calls since the Session opened.
    pub calls: u32,
}

/// A pad button.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    /// B.
    B,
    /// Y.
    Y,
    /// Select.
    Select,
    /// Start.
    Start,
    /// Up.
    Up,
    /// Down.
    Down,
    /// Left.
    Left,
    /// Right.
    Right,
    /// A.
    A,
    /// X.
    X,
    /// L.
    L,
    /// R.
    R,
}

impl Button {
    /// The button's bit in a [`Buttons`] set: its place in declaration order.
    #[must_use]
    pub const fn bit(self) -> u32 {
        let place = match self {
            Self::B => 0,
            Self::Y => 1,
            Self::Select => 2,
            Self::Start => 3,
            Self::Up => 4,
            Self::Down => 5,
            Self::Left => 6,
            Self::Right => 7,
            Self::A => 8,
            Self::X => 9,
            Self::L => 10,
            Self::R => 11,
        };
        1 << place
    }
}

impl slots::Flag for Button {
    fn bit(self) -> u32 {
        Self::bit(self)
    }
}
/// A set of pad buttons.
pub type Buttons = FlagSet<Button>;

/// The controller as the game sees it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Controller {
    /// The pad as last auto-read, latched at the next requested NMI.
    pub auto_read: Buttons,
    /// Held this frame.
    pub held: Buttons,
    /// Held now but not last frame.
    pub pressed: Buttons,
    /// Held last frame.
    pub previous: Buttons,
    /// Auto-repeat presses.
    pub repeat: Buttons,
    /// Frames until the next auto-repeat.
    pub repeat_delay: slots::Frames,
}

/// The game state.
// open: the original's other game states (title, file select, doors, pause, death, ...),
// each with its data, join with their clusters; slice 0 runs in gameplay only.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameMode {
    /// Main gameplay.
    #[default]
    Gameplay,
}

/// Whether an NMI arrived after the main strand yielded or while it was still working (D8).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NmiTiming {
    /// After the main strand yielded.
    #[default]
    OnTime,
    /// While the main strand was still working.
    Overrun,
    /// No NMI: the game had NMIs disabled at that vblank, so no wait was answered.
    Disabled,
}

/// The NMI timing state and the cost model's accumulator (D8, D9).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Timing {
    /// How the last NMI arrived.
    pub last: NmiTiming,
    /// CPU time still owed by a busy routine, under the modeled timing source.
    pub owed: Cycles,
}

/// The whole Session at a cut.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// Time.
    pub clock: Clock,
    /// The RNG.
    pub rng: Rng,
    /// The controller.
    pub controller: Controller,
    /// The game state.
    pub mode: GameMode,
    /// The main strand.
    pub main: MainStrand,
    /// The interrupt strand.
    pub irq: IrqStrand,
    /// NMI timing.
    pub timing: Timing,
    /// Video.
    pub video: Video,
    /// Samus.
    pub samus: Samus,
    /// The current room.
    pub room: RoomScoped,
    /// The elevator ride, which carries over from room to room.
    pub elevator: room::ElevatorRide,
    /// The enemy palette cycle, which carries over from room to room.
    pub enemy_palette_cycle: room::EnemyPaletteCycle,
    /// The HUD.
    pub hud: Hud,
    /// The message the message box shows, or showed last.
    pub message: Message,
    /// The items picked up on the file, by their collected bit.
    pub collected_items: CollectedItems,
    /// The cartridge's save RAM.
    pub sram: sram::SaveRam,
}

/// Words of the collected item bits (`$7E:D870`, 64 bytes).
pub const COLLECTED_ITEM_WORDS: usize = 32;

/// The items picked up on the file: bit `n` is the item whose room argument is `n`
/// (bank $84 log, `$84:8899`; `$84:887C` tests it).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollectedItems(pub [u16; COLLECTED_ITEM_WORDS]);

impl CollectedItems {
    /// The word and mask of bit `bit`: byte `bit / 8`'s bit `bit % 8`, read as words.
    fn place(bit: u16) -> (usize, u16) {
        (usize::from(bit >> 4), 1 << (bit & 0xf))
    }

    /// Whether item bit `bit` is set; past the table, no.
    #[must_use]
    pub fn contains(&self, bit: u16) -> bool {
        let (word, mask) = Self::place(bit);
        self.0.get(word).is_some_and(|value| value & mask != 0)
    }

    /// Set item bit `bit`.
    // open: past the table the original sets a bit in whatever work RAM follows; slice 0's
    // arguments are inside it.
    pub fn insert(&mut self, bit: u16) {
        let (word, mask) = Self::place(bit);
        if let Some(value) = self.0.get_mut(word) {
            *value |= mask;
        }
    }
}

/// The State's schema: [`State`]'s structural hash.
///
/// A Checkpoint only opens against the State shape that wrote it: another shape's bytes fail
/// with [`canonical::DecodeError::SchemaMismatch`] before the State is read.
pub type StateSchema = Schema<State>;

/// The content fingerprint: a Checkpoint only opens against the content it was made with.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fingerprint(pub u64);

/// A saved State (FOUNDATION §4.6), encoded canonically.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    /// The schema that wrote it, always this build's.
    pub schema: StateSchema,
    /// The content it was made against.
    pub fingerprint: Fingerprint,
    /// The State.
    pub state: State,
}

impl Checkpoint {
    /// `state`, saved against the content `fingerprint` names.
    #[must_use]
    pub const fn new(fingerprint: Fingerprint, state: State) -> Self {
        Self {
            schema: Schema::new(),
            fingerprint,
            state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Checkpoint, Fingerprint, State};
    use canonical::{Canonical, DecodeError, from_bytes, to_bytes};

    #[test]
    fn a_default_state_round_trips() {
        let state = State::default();
        assert_eq!(from_bytes::<State>(&to_bytes(&state)).as_ref(), Ok(&state));
    }

    #[test]
    fn a_checkpoint_round_trips_and_diffs_by_path() {
        let checkpoint = Checkpoint::new(Fingerprint(2), State::default());
        let bytes = to_bytes(&checkpoint);
        assert_eq!(bytes[..8], State::SCHEMA.to_le_bytes());
        let decoded = from_bytes::<Checkpoint>(&bytes);
        assert_eq!(decoded.as_ref(), Ok(&checkpoint));
        let mut moved = checkpoint.clone();
        moved.state.samus.position.x.pixels = 1;
        let difference = checkpoint
            .diff(&moved)
            .map(|difference| difference.to_string());
        assert_eq!(
            difference.as_deref(),
            Some("state.samus.position.x.pixels: 0000 != 0100")
        );
    }

    #[test]
    fn a_checkpoint_from_another_schema_does_not_open() {
        let mut bytes = to_bytes(&Checkpoint::new(Fingerprint(2), State::default()));
        let other = State::SCHEMA ^ 1;
        bytes[..8].copy_from_slice(&other.to_le_bytes());
        assert_eq!(
            from_bytes::<Checkpoint>(&bytes),
            Err(DecodeError::SchemaMismatch {
                expected: State::SCHEMA,
                found: other,
            })
        );
    }
}
