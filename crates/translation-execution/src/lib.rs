//! Rust-owned Translation scope attempts. Only source replacements cross the
//! rewrite seam; trusted adapters own execution, evidence and publication.
pub mod model;
pub mod rewrite;
pub mod budget;
pub mod confinement;
pub mod checkout;
mod journal;
mod executor;

pub use executor::{ExecutionAdapter, ExecutionPending, Supervisor};
pub use model::*;
