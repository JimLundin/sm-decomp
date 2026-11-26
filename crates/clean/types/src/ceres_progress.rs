//! Progress events published by the Ceres room and consumed by run progress.

use serde::{Deserialize, Serialize};

/// Spec: `WarningEnds`. A defeat in Ceres, published when the escape activates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CeresDefeat {
    /// Ceres Ridley counts as defeated for subsequent room loads.
    MainBoss,
}
