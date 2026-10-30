//! Room liquid (FX) and HDMA objects as one Session-owned domain: design.
//!
//! Step 1 (liquid) is implemented below; step 2 (HDMA objects) is planned.
//! Constraints come from docs/ENGINE_STATE_MIGRATION.md.
//!
//! # Scope and owner types
//!
//! Step 1, liquid: `Room::liquid: Liquid` joins the five existing `room.liquid_*`
//! words (kept as they are) with the rest of the $195C–$1980 block (not $1966–$196A, which
//! is unnamed; not the layer blending words at $1982–$1986, which belong to
//! display):
//!
//! | RAM | field | before step 1 |
//! |---|---|---|
//! | $195C/$195E | `surface: Position` (fx_y_subpos/pos) | RAM only |
//! | $1960/$1962 | `lava_acid: Position` | RAM only |
//! | $1964 | `tilemap: u16` | RAM only |
//! | $196C | `rising: u16` (bank $88 handler) | RAM only |
//! | $196E | `kind: u16` (fx_type) | RAM only |
//! | $1970/$1972 | `offset: Position` | RAM only |
//! | $1974 | `tide_phase: u16` | RAM only |
//! | $1976 | `base_fraction: u16` (whole stays `room.liquid_height`) | RAM only |
//! | $1978–$1980 | `room.liquid_{height,target,velocity,options,delay}` | owned |
//!
//! Step 2, HDMA objects: `Presentation::hdma: HdmaObjects` takes $18B0–$1958:
//! the enable flag, selected index, and six slots of channel mask, bank,
//! instruction list, table, instruction timer, pre-instruction (+bank),
//! timer and the six behaviour words A–F. The liquid routines are HDMA
//! pre-instructions over these slots, so step 2 completes step 1's callers.
//!
//! Every word keeps its full stored `u16` (no enums yet): behaviours write
//! unknown values and off-table indices alias neighbouring words.
//!
//! # Encoder and checkpoints
//!
//! - `session_encoding`: one `$f(addr, field)` entry per word above, so import
//!   decodes and export encodes exactly the same bytes (little-endian words).
//! - Corpus checkpoints (`.smvchk`) serialise WRAM plus actors, not Session,
//!   so their layout fingerprint and seed assets do not change.
//! - The persistent `SessionCheckpoint` codec (`checkpoint_fields!` for Room
//!   and Presentation) gains the new fields deliberately, with a roundtrip check.
//!
//! # Writers: closed or bridged
//!
//! One authority per interval: a Runtime lease owns the domain while open;
//! compat writes outside a lease write RAM, bump the epoch and are re-imported
//! at the next lease (the existing bridge), so none is lost.
//!
//! Verifier-observed lease and transfer counts must not change, so no writer
//! gains a lease it did not already have:
//!
//! - `room_state` (FX kind and tilemap) and `player_transitions_runtime`
//!   (surface reset to $FFFF): closed. They already hold a Runtime lease, so
//!   they write the typed fields.
//! - `liquid_effects` (sm_game; HDMA pre-instructions), `sm_a6` (Ridley/Ceres
//!   rising lava), `sm_aa` (Torizo acid) and `visual_runtime`: bridged. They run
//!   on `Game` without a lease; their RAM writes suspend the owner, which is
//!   re-imported at the next lease. They close when their callers run on one.
//! - `visual_runtime` HDMA interpreter: bridged in step 1; closed in step 2,
//!   when it runs the slot table from `presentation.hdma`.
//! - Readers (grapple, sky, x-ray, Crocomire, escape critters, explosion
//!   windows, Mother Brain, menus, display refresh): already read through the
//!   projection while the owner is current; converted to typed reads as their
//!   own modules migrate.
//!
//! # Risks
//!
//! - Step 1 is a complete closure only for the liquid words; it lands as one
//!   commit, step 2 as another, each behind a full corpus.
#![forbid(unsafe_code)]
use crate::geometry::Position;

/// The room liquid's motion and surface words not already in `Room`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Liquid {
    /// $195C/$195E: the drawn liquid surface.
    pub surface: Position,
    /// $1960/$1962: the lava or acid surface used for damage.
    pub lava_acid: Position,
    /// $1964: FX tilemap pointer.
    pub tilemap: u16,
    /// $196C: the bank $88 rising handler.
    pub rising: u16,
    /// $196E: FX type.
    pub kind: u16,
    /// $1970/$1972: the surface's wave offset.
    pub offset: Position,
    /// $1974.
    pub tide_phase: u16,
    /// $1976: the fraction of the base height, whose whole is `Room::liquid_height`.
    pub base_fraction: u16,
}
