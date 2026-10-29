//! Explicit admission of the actual mutable cartridge allocation for Samus roles.
#![forbid(unsafe_code)]

use super::{
    samus_content::{AllocationDeclaration, AllocationId, CapturedBytes, Registration},
    state::DisplayError,
};
use sm_data::cartridge::RomImage;
use std::sync::Arc;

/// G owns this registration. Loaded and Native may register different images.
/// Replacement/adoption requires explicit admission; it never retargets leases.
pub struct RegisteredRom {
    source: Registration,
    declaration: Arc<AllocationDeclaration>,
    image: RomImage,
}
impl RegisteredRom {
    /// Compare the full backing bytes once at the supplied content-admission
    /// seam. The declaration comes from G's independently validated role content.
    pub fn admit(
        source: Registration,
        image: RomImage,
        supplied: Arc<AllocationDeclaration>,
        expected: &AllocationDeclaration,
    ) -> Result<Self, DisplayError> {
        if supplied.identity.0 == [0; 32] {
            return Err(DisplayError::Identity);
        }
        if supplied.revision.0 == [0; 32] {
            return Err(DisplayError::Revision);
        }
        if supplied.as_ref() != expected {
            return Err(DisplayError::Incomplete);
        }
        if supplied.extent != supplied.bytes.len() || image.len() != supplied.extent {
            return Err(DisplayError::Source);
        }
        for (offset, expected_byte) in supplied.bytes.iter().copied().enumerate() {
            if image.read(offset) != expected_byte {
                return Err(DisplayError::Source);
            }
        }
        Ok(Self {
            source,
            declaration: supplied,
            image,
        })
    }

    /// Invoke after the original selection/diagnostic observation. No payload
    /// comparison or value snapshot occurs during this lazy capture.
    pub fn capture(
        &self,
        source: Registration,
        selected: &RomImage,
    ) -> Result<CapturedRom, DisplayError> {
        if source != self.source || !self.image.same_allocation(selected) {
            return Err(DisplayError::Identity);
        }
        Ok(CapturedRom {
            source,
            declaration: self.declaration.clone(),
            image: self.image.clone(),
        })
    }
}

/// Retains the original Rc<Cell> image. Returning callback writes to that same
/// allocation remain visible; changing G's selection cannot change these reads.
/// Native mutation/revocation and successor adoption remain G dependencies.
pub struct CapturedRom {
    source: Registration,
    declaration: Arc<AllocationDeclaration>,
    image: RomImage,
}
impl CapturedRom {
    pub fn registration(&self) -> Registration {
        self.source
    }
}
impl CapturedBytes for CapturedRom {
    fn allocation(&self) -> AllocationId {
        self.declaration.identity
    }
    fn extent(&self) -> usize {
        self.declaration.extent
    }
    fn byte(&self, allocation_offset: usize) -> Result<u8, DisplayError> {
        // Preserve RomImage's original indexed failure, including unwinding.
        Ok(self.image.read(allocation_offset))
    }
}
