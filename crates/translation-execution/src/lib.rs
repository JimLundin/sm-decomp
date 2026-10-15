//! Rust-owned Translation scope attempts. Only source replacements cross the
//! rewrite seam; trusted adapters own execution, evidence and publication.
pub mod budget;
pub mod checkout;
pub mod compiler_runtime;
pub mod confinement;
mod executor;
mod journal;
pub mod model;
pub mod model_rewrite;
pub mod operation;
pub mod retention;
pub mod rewrite;
pub mod rewrite_adapter;

pub use executor::{
    ExecutionAdapter, ExecutionPending, FinalizationJournal, PublicationJournal, Supervisor,
    authenticate_completion_history, authenticate_finalization_tail, authenticate_finalized_history,
    authenticate_publication_extent,
};
pub use model::*;
