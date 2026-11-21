//! The current number shared by random-sequence callers and consumers.

use serde::{Deserialize, Serialize};

/// Spec: `AdvanceRandom`. A whole sequence value; every `u16` is valid, including zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RandomNumber(pub u16);

/// Spec: `AdvanceRandom`. The complete state of the shared random sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RandomSequence {
    /// The current number. Read it without drawing, or set it to any saved number to resume.
    pub number: RandomNumber,
}
