//! Game flow outside gameplay: the cinematic function and the demo playback
//! position, owned by `Session::flow`.
//!
//! # Design
//!
//! **Cohort.** `cinematic_function`, `demo_timer`, `demo_set`, `demo_scene`
//! and `num_demo_sets` ($1F51..$1F5B). No other RAM name aliases them. Each
//! is the little-endian value of its two RAM bytes.
//!
//! **Encoding boundary.** `session_encoding` projects every byte and decodes
//! or visits the cohort as one part; the auxiliary gameplay checkpoint
//! carries it. The cinematic function is also the lease's `cinematic`
//! observation, so a lease refreshes it from this owner after each store.
//!
//! **Consumers.** The title, intro, ending, credits, options and demo code
//! run on the compatibility host outside a lease. They read the owner with
//! [`current`] and store with [`set`], which go through the host's
//! projected bytes: the owned Session's projection while one is installed,
//! the byte image otherwise, with the same bytes and order as the named
//! words they replace. A consumer that moves onto a lease uses
//! `session().flow` directly.
use crate::ram::CompatibilityRam;

/// The flow words; see the module documentation for the RAM layout.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Flow {
    /// The cinematic and title sequence step's handler ($1F51).
    pub cinematic_function: u16,
    /// Frames left in the demo scene ($1F53).
    pub demo_timer: u16,
    /// The demo set and the scene within it ($1F55, $1F57).
    pub demo_set: u16,
    pub demo_scene: u16,
    /// How many demo sets the cartridge has ($1F59).
    pub num_demo_sets: u16,
}

/// The flow as the host's projection holds it.
pub fn current<H: CompatibilityRam + ?Sized>(h: &H) -> Flow {
    crate::session_encoding::flow(&crate::projection::reader(h))
}

/// A flow word, by its RAM name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    CinematicFunction,
    DemoTimer,
    DemoSet,
    DemoScene,
    NumDemoSets,
}

/// Store `value` to `word`, low byte then high, as the named word store did.
pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
    crate::projection::set_word(h, crate::session_encoding::FLOW + 2 * word as usize, value);
}
