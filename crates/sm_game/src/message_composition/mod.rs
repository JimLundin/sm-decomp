//! Message state and native layout share one implementation.
mod native;
pub use native::*;
pub use sm_data::message::{
    AssetId, ButtonModifier, Definitions, Fault, FontDefinition, MessageDefinition, MessageId,
    MessageKind, Revision,
};
pub mod prepared;
