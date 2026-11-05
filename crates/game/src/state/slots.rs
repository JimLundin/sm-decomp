//! The slot store (D12) and the instruction-list cursor its families run (D13).
//!
//! The store is generic over a family's common fields and body. Its policy is a value in
//! `game`'s rules, not State, so only the slots themselves are here.

use std::marker::PhantomData;

use canonical::Canonical;
use data::id::ProgramId;

/// A frame count: a delay, a timer or an invincibility window.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frames(pub u16);

/// A slot's position in its family's table: the spawn and run order key and the tie-break.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotIndex<F>(pub u8, pub PhantomData<F>);

/// A reference the original keeps as a raw slot number: it sees whatever occupies the slot
/// now, reuse included.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlotRef<F>(pub SlotIndex<F>);

/// How many times a slot has been filled. It wraps; a `Handle` only needs it to differ.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Generation(pub u16);

/// A reference that must not follow reuse: it is stale once the slot's generation moves on.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Handle<F> {
    /// The slot.
    pub slot: SlotIndex<F>,
    /// The slot's generation when the handle was taken.
    pub generation: Generation,
}

/// What a slot holds.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occupancy<B> {
    /// Nothing; spawn may take it.
    Free,
    /// A running actor.
    Live(B),
    /// Deleted, but not yet free: an enemy is freed by the next determine pass.
    Deleting(B),
}

/// One slot. The common fields sit outside the occupancy because the store never clears
/// them: a field the spawn code doesn't write keeps the last occupant's value (J11).
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot<C, B> {
    /// Bumped by each spawn into this slot.
    pub generation: Generation,
    /// The family's common fields.
    pub common: C,
    /// The typed body, if any.
    pub occupancy: Occupancy<B>,
}

impl<C: Default, B> Default for Slot<C, B> {
    /// A free slot with default common fields, as at boot.
    fn default() -> Self {
        Self {
            generation: Generation::default(),
            common: C::default(),
            occupancy: Occupancy::Free,
        }
    }
}

/// A family's `N` slots, in table order.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct SlotStore<C, B, const N: usize> {
    /// The slots; `SlotIndex` indexes them.
    pub slots: [Slot<C, B>; N],
}

impl<C: Default, B, const N: usize> Default for SlotStore<C, B, N> {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| Slot::default()),
        }
    }
}

/// An index into an instruction list.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepIndex(pub u16);

/// A place in family `F`'s programs: what the original holds as an instruction pointer.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Label<F> {
    /// The instruction list.
    pub program: ProgramId,
    /// The step within it.
    pub step: StepIndex,
    /// The family, so one family's label can't run in another's interpreter.
    pub family: PhantomData<F>,
}

/// One actor's place in its instruction list (D13).
// open: the family's pre-op (`Family::PreOp`) joins when the interpreter lane adds the
// `Family` trait; until then a family that needs one keeps it in its body.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor<F> {
    /// The next step to run.
    pub at: Label<F>,
    /// Frames left on the current `Show`.
    pub delay: Frames,
    /// Where a `Return` goes.
    pub link: Option<Label<F>>,
    /// The counter the decrement-and-branch op uses.
    pub timer: u16,
}

/// A set of `E`s: bit `n` holds `E`'s `n`th variant in declaration order.
// open: the queries (`contains`, `insert`) arrive with the first rule that reads a set; the
// decoder maps the original's bit layout onto declaration order at the edge.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlagSet<E> {
    /// The members, one bit per variant.
    pub bits: u32,
    /// The member type.
    pub members: PhantomData<E>,
}

impl<E> Default for FlagSet<E> {
    /// The empty set.
    fn default() -> Self {
        Self {
            bits: 0,
            members: PhantomData,
        }
    }
}
