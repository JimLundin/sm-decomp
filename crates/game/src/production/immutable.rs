//! Immutable clean rule contexts whose model types do not yet expose comparison traits.

use std::fmt;
use std::sync::Arc;

/// Complete block rules held by identity, without reconstructing their tables.
#[derive(Clone)]
pub struct BlockRules(Arc<world_blocks::Rules>);

impl BlockRules {
    /// Bind the complete model rules once.
    #[must_use]
    pub fn new(rules: world_blocks::Rules) -> Self {
        Self(Arc::new(rules))
    }

    pub(super) fn get(&self) -> &world_blocks::Rules {
        &self.0
    }
}

impl fmt::Debug for BlockRules {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BlockRules")
    }
}

impl PartialEq for BlockRules {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for BlockRules {}

/// Complete following-camera parameters bound to their immutable identity.
#[derive(Clone, Debug)]
pub struct CameraRules(Arc<world_camera::Rules>);

impl CameraRules {
    /// Bind the complete model rules once.
    #[must_use]
    pub fn new(rules: world_camera::Rules) -> Self {
        Self(Arc::new(rules))
    }

    pub(super) fn get(&self) -> &world_camera::Rules {
        &self.0
    }
}

impl PartialEq for CameraRules {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CameraRules {}
