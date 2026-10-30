//! Supplied nonrender Samus content and allocation-bound, lazy observations.

use super::{
    records::Revision,
    samus_body::Half,
    samus_position_tiles::TileRequest,
    state::{DisplayError, TileTransfer, TransferSource, WordOrder},
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    BodyTop,
    BodyBottom,
    PositionFallback,
    Landing,
    Crouching,
    Drained,
    TilePose,
    TileDefinition,
    TileTop,
    TileBottom,
    UploadDescriptor,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Registration {
    Loaded,
    Native,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllocationId(pub [u8; 32]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocationDeclaration {
    pub identity: AllocationId,
    pub revision: Revision,
    pub extent: usize,
    /// Entire backing allocation, including bytes outside every named table.
    pub bytes: Arc<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleDeclaration {
    pub allocation: AllocationId,
    /// Mapping occurs once before capture; subsequent displacements are linear.
    pub mapped_base: usize,
    pub native_order: WordOrder,
}
#[derive(Clone, Debug)]
pub struct RequestBinding {
    pub half: Half,
    pub request: TileRequest,
}
#[derive(Clone, Debug)]
pub struct ContentDescription {
    pub gameplay: [u8; 32],
    /// Nonrender content revision, independent of the request's SceneMedia.
    pub revision: Revision,
    pub allocations: BTreeMap<AllocationId, Arc<AllocationDeclaration>>,
    pub roles: BTreeMap<(Role, Registration), RoleDeclaration>,
    pub requests: Arc<[RequestBinding]>,
}

/// A native implementation must retain the actual captured Rc<Cell> allocation.
/// Reads observe later writes to that allocation, never the current selection.
/// The production adapter and its callback/adoption dependency remain required.
pub trait CapturedBytes {
    fn allocation(&self) -> AllocationId;
    fn extent(&self) -> usize;
    fn byte(&self, allocation_offset: usize) -> Result<u8, DisplayError>;
}
/// Prepared backing is created only from this content's sealed declaration.
/// No caller-created declaration can be substituted into a prepared lease.
pub struct PreparedBytes(Arc<AllocationDeclaration>);
impl CapturedBytes for PreparedBytes {
    fn allocation(&self) -> AllocationId {
        self.0.identity
    }
    fn extent(&self) -> usize {
        self.0.extent
    }
    fn byte(&self, offset: usize) -> Result<u8, DisplayError> {
        self.0
            .bytes
            .get(offset)
            .copied()
            .ok_or(DisplayError::Source)
    }
}
pub struct RoleLease<B> {
    role: Role,
    registration: Registration,
    allocation: Arc<AllocationDeclaration>,
    mapped_base: usize,
    native_order: WordOrder,
    backing: B,
}
impl<B: CapturedBytes> RoleLease<B> {
    pub fn identity(&self) -> (Role, Registration, AllocationId) {
        (self.role, self.registration, self.allocation.identity)
    }
    pub fn native_order(&self) -> WordOrder {
        self.native_order
    }
    pub fn byte(&self, displacement: usize) -> Result<u8, DisplayError> {
        let at = self
            .mapped_base
            .checked_add(displacement)
            .ok_or(DisplayError::Source)?;
        self.backing.byte(at)
    }
    pub fn word(&self, displacement: usize, order: WordOrder) -> Result<u16, DisplayError> {
        let first = self.byte(displacement)?;
        let second = self.byte(displacement.checked_add(1).ok_or(DisplayError::Source)?)?;
        Ok(order.decode([first, second]))
    }
    pub fn native_word(&self, displacement: usize) -> Result<u16, DisplayError> {
        self.word(displacement, self.native_order)
    }
    pub fn little_word(&self, displacement: usize) -> Result<u16, DisplayError> {
        self.word(displacement, WordOrder::Little)
    }
}

pub struct ValidatedContent(ContentDescription);
impl ValidatedContent {
    /// The dependency must invoke the existing TileRequest media/transfer
    /// validator after its shared byte-source rule is exposed by the integrator.
    /// This module neither decodes requests nor promotes native mutable bytes.
    pub fn construct(
        supplied: ContentDescription,
        expected: &ContentDescription,
        mut validate_request: impl FnMut(&TileRequest) -> Result<(), DisplayError>,
    ) -> Result<Self, DisplayError> {
        if supplied.gameplay == [0; 32] || supplied.gameplay != expected.gameplay {
            return Err(DisplayError::Identity);
        }
        if supplied.revision.0 == [0; 32] || supplied.revision != expected.revision {
            return Err(DisplayError::Revision);
        }
        if supplied.allocations != expected.allocations
            || supplied.roles != expected.roles
            || supplied.requests.len() != expected.requests.len()
            || !supplied
                .requests
                .iter()
                .zip(expected.requests.iter())
                .all(|(a, b)| same_request(a, b))
        {
            return Err(DisplayError::Incomplete);
        }
        for (id, allocation) in &supplied.allocations {
            if id.0 == [0; 32] || *id != allocation.identity {
                return Err(DisplayError::Identity);
            }
            if allocation.revision != supplied.revision {
                return Err(DisplayError::Revision);
            }
            if allocation.extent != allocation.bytes.len() {
                return Err(DisplayError::Source);
            }
        }
        use Registration::{Loaded, Native};
        use Role::*;
        for key in [
            (BodyTop, Loaded),
            (BodyBottom, Loaded),
            (PositionFallback, Native),
            (Landing, Native),
            (Crouching, Native),
            (Drained, Native),
            (TilePose, Loaded),
            (TileDefinition, Native),
            (TileTop, Loaded),
            (TileBottom, Loaded),
            (UploadDescriptor, Native),
            (UploadDescriptor, Loaded),
        ] {
            if !supplied.roles.contains_key(&key) {
                return Err(DisplayError::Incomplete);
            }
        }
        for declaration in supplied.roles.values() {
            let allocation = supplied
                .allocations
                .get(&declaration.allocation)
                .ok_or(DisplayError::Incomplete)?;
            if declaration.mapped_base > allocation.extent {
                return Err(DisplayError::Source);
            }
        }
        for (index, binding) in supplied.requests.iter().enumerate() {
            if supplied.requests[..index].iter().any(|prior| {
                prior.half == binding.half && prior.request.pointer == binding.request.pointer
            }) {
                return Err(DisplayError::Incomplete);
            }
            validate_request(&binding.request)?;
        }
        Ok(Self(supplied))
    }
    pub fn description(&self) -> &ContentDescription {
        &self.0
    }
    pub fn acquire_prepared(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> Result<RoleLease<PreparedBytes>, DisplayError> {
        let declaration = self
            .0
            .roles
            .get(&(role, registration))
            .ok_or(DisplayError::Incomplete)?;
        let allocation = self
            .0
            .allocations
            .get(&declaration.allocation)
            .ok_or(DisplayError::Incomplete)?
            .clone();
        self.acquire(role, registration, displacement, PreparedBytes(allocation))
    }
    pub fn acquire_loaded<B: CapturedBytes>(
        &self,
        role: Role,
        displacement: usize,
        backing: B,
    ) -> Result<RoleLease<B>, DisplayError> {
        self.acquire(role, Registration::Loaded, displacement, backing)
    }
    pub fn acquire_native<B: CapturedBytes>(
        &self,
        role: Role,
        displacement: usize,
        backing: B,
    ) -> Result<RoleLease<B>, DisplayError> {
        self.acquire(role, Registration::Native, displacement, backing)
    }
    fn acquire<B: CapturedBytes>(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
        backing: B,
    ) -> Result<RoleLease<B>, DisplayError> {
        let declaration = self
            .0
            .roles
            .get(&(role, registration))
            .ok_or(DisplayError::Incomplete)?;
        let allocation = self
            .0
            .allocations
            .get(&declaration.allocation)
            .ok_or(DisplayError::Incomplete)?
            .clone();
        if backing.allocation() != allocation.identity || backing.extent() != allocation.extent {
            return Err(DisplayError::Identity);
        }
        Ok(RoleLease {
            role,
            registration,
            allocation,
            mapped_base: declaration
                .mapped_base
                .checked_add(displacement)
                .ok_or(DisplayError::Source)?,
            native_order: declaration.native_order,
            backing,
        })
    }
    /// Invoke only after the original pointer bytes and flag have been written.
    pub fn request(&self, half: Half, pointer: u16) -> Result<TileRequest, DisplayError> {
        self.0
            .requests
            .iter()
            .find(|binding| binding.half == half && binding.request.pointer == pointer)
            .map(|binding| binding.request.clone())
            .ok_or(DisplayError::Incomplete)
    }
}
fn same_request(a: &RequestBinding, b: &RequestBinding) -> bool {
    a.half == b.half
        && a.request.pointer == b.request.pointer
        && a.request.half.enabled == b.request.half.enabled
        && a.request.half.descriptor == b.request.half.descriptor
        && same_transfer(&a.request.half.first, &b.request.half.first)
        && same_transfer(&a.request.half.second, &b.request.half.second)
}
fn same_transfer(a: &TileTransfer, b: &TileTransfer) -> bool {
    a.count == b.count
        && a.destination == b.destination
        && match (&a.source, &b.source) {
            (TransferSource::Asset(a), TransferSource::Asset(b)) => {
                a.asset == b.asset && a.offset == b.offset && a.bytes == b.bytes
            }
            // Authored TileRequest admission requires explicit asset sources.
            _ => false,
        }
}
