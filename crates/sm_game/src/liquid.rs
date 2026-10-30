//! Room liquid (FX) and HDMA objects as one Session-owned domain: design.
//!
//! Nothing here is implemented yet; this records the plan the coordinator
//! reviews before code lands. Constraints come from docs/ENGINE_STATE_MIGRATION.md.
//!
//! # Scope and owner types
//!
//! Step 1, liquid: `Room::liquid: Liquid` replaces the five `room.liquid_*`
//! words and adds the rest of the $195C–$1980 block (not $1966–$196A, which
//! is unnamed; not the layer blending words at $1982–$1986, which belong to
//! display):
//!
//! | RAM | field | today |
//! |---|---|---|
//! | $195C/$195E | `surface: Position` (fx_y_subpos/pos) | RAM only |
//! | $1960/$1962 | `lava_acid: Position` | RAM only |
//! | $1964 | `tilemap: u16` | RAM only |
//! | $196C | `rising: u16` (bank $88 handler) | RAM only |
//! | $196E | `kind: u16` (fx_type) | RAM only |
//! | $1970/$1972 | `offset: Position` | RAM only |
//! | $1974 | `tide_phase: u16` | RAM only |
//! | $1976/$1978 | `base: Position` (whole = old `liquid_height`) | whole owned |
//! | $197A | `target: u16` | owned (`liquid_target`) |
//! | $197C | `velocity: u16` | owned |
//! | $197E | `options: u16` | owned |
//! | $1980 | `delay: u16` | owned |
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
//! - `liquid_effects` (sm_game; HDMA pre-instructions): closed. Each callback
//!   runs on one lease and uses `session().room.liquid` / `presentation.hdma`.
//! - `room_state::set_fx_kind`, `player_transitions_runtime` (reset to
//!   $FFFF): closed; they already hold a Runtime, so they write typed fields.
//! - `sm_a6` (Ridley/Ceres rising lava) and `sm_aa` (Torizo acid): closed by
//!   writing the three words through one lease where the original wrote them
//!   together.
//! - `visual_runtime` HDMA interpreter: bridged in step 1; closed in step 2,
//!   when it runs the slot table from `presentation.hdma`.
//! - Readers (grapple, sky, x-ray, Crocomire, escape critters, explosion
//!   windows, Mother Brain, menus, display refresh): already read through the
//!   projection while the owner is current; converted to typed reads as their
//!   own modules migrate.
//!
//! # Risks
//!
//! - Extra leases add import/export transfers, which some verifier checks
//!   count; per-callback leases must replace the old accesses one-for-one.
//! - Step 1 is a complete closure only for the liquid words; it lands as one
//!   commit, step 2 as another, each behind a full corpus.
#![forbid(unsafe_code)]
