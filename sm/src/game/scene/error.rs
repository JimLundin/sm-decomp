//! Owned display boundary faults, independent of media and platform state.
#![forbid(unsafe_code)]

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayError {
    Identity,
    Revision,
    Incomplete,
    Phase,
    Poisoned,
    Queue,
    Source,
    /// The retained compatibility owner must perform this aliased operation.
    Alias {
        offset: usize,
    },
    Geometry,
    Scene,
}
