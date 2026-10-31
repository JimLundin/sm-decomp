//! Samus's bound content and its authority: the prepared roles, the ROM
//! registrations they read, and whether the frame and its leases may use them.
use super::owned_samus::SamusExecution;
use super::records::SceneMedia;
use super::samus::SamusCatalog;
use super::samus_body::Half;
use super::samus_content::{
    AllocationId, ContentDescription, PreparedBytes, Registration, Role, RoleLease,
    ValidatedContent,
};
use super::samus_native_content::{CapturedRom, RegisteredRom};
use super::samus_position_tiles::TileRequest;
use super::state::DisplayError;
use crate::room_entry::world::WorldBinding;
use sm_data::cartridge::RomImage;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamusAuthority {
    LegacyUnrouted,
    Available,
    Revoked,
}

/// Supplied initialization, independently expected before the selection is published.
pub struct SamusBindingDescription {
    pub world: Arc<WorldBinding>,
    pub roles: ContentDescription,
    pub spritemaps: Arc<SamusCatalog>,
    pub request_media: Arc<SceneMedia>,
    pub execution: SamusExecution,
}

/// The world and the loaded and native ROM images a binding's registrations read.
#[derive(Clone, Copy)]
pub struct SamusImages<'a> {
    pub world: &'a Arc<WorldBinding>,
    pub loaded: &'a Option<RomImage>,
    pub native: &'a Option<RomImage>,
}
impl<'a> SamusImages<'a> {
    /// The ROM image a Samus registration reads.
    fn image(self, registration: Registration) -> Result<&'a RomImage, DisplayError> {
        match registration {
            Registration::Loaded => self.loaded,
            Registration::Native => self.native,
        }
        .as_ref()
        .ok_or(DisplayError::Identity)
    }
}

pub struct SamusBinding {
    world: Arc<WorldBinding>,
    roles: Arc<ValidatedContent>,
    expected_roles: Arc<ContentDescription>,
    spritemaps: Arc<SamusCatalog>,
    request_media: Arc<SceneMedia>,
    registrations: BTreeMap<(Registration, AllocationId), RegisteredRom>,
    execution: SamusExecution,
}
/// Unpublished; only a validated preparation constructs this complete replacement.
pub struct PreparedSamusBinding(Rc<SamusBinding>);
impl PreparedSamusBinding {
    pub fn binding(&self) -> &SamusBinding {
        &self.0
    }
}

/// Whether `role` is a native position-offset table, which retained native
/// execution also reads from the prepared content.
fn native_offset_role(role: Role, registration: Registration) -> bool {
    registration == Registration::Native
        && matches!(role, Role::Landing | Role::Crouching | Role::Drained)
}

impl SamusBinding {
    pub fn execution(&self) -> SamusExecution {
        self.execution
    }
    pub fn prepare(
        supplied: SamusBindingDescription,
        expected: &SamusBindingDescription,
        images: SamusImages<'_>,
    ) -> Result<PreparedSamusBinding, DisplayError> {
        if !Arc::ptr_eq(&supplied.world, &expected.world)
            || !Arc::ptr_eq(&supplied.world, images.world)
        {
            return Err(DisplayError::Identity);
        }
        if supplied.execution != expected.execution {
            return Err(DisplayError::Incomplete);
        }
        supplied
            .request_media
            .validate()
            .map_err(|_| DisplayError::Scene)?;
        if supplied.request_media != expected.request_media {
            return Err(DisplayError::Incomplete);
        }
        supplied.spritemaps.validate(&supplied.request_media)?;
        supplied
            .spritemaps
            .validate_expected(&expected.spritemaps)?;
        let mut expected_requests = expected.roles.requests.iter();
        let roles = Arc::new(ValidatedContent::construct(
            supplied.roles,
            &expected.roles,
            |request| {
                request.validate(
                    expected_requests
                        .next()
                        .ok_or(DisplayError::Incomplete)?
                        .request
                        .pointer,
                    &supplied.request_media,
                )
            },
        )?);
        let mut registrations = BTreeMap::new();
        for (&(role, registration), declaration) in &roles.description().roles {
            if native_offset_role(role, registration) {
                continue;
            }
            let key = (registration, declaration.allocation);
            if registrations.contains_key(&key) {
                continue;
            }
            let allocation = roles
                .description()
                .allocations
                .get(&declaration.allocation)
                .ok_or(DisplayError::Incomplete)?;
            let expected_allocation = expected
                .roles
                .allocations
                .get(&declaration.allocation)
                .ok_or(DisplayError::Incomplete)?;
            let image = images.image(registration)?.clone();
            registrations.insert(
                key,
                RegisteredRom::admit(registration, image, allocation.clone(), expected_allocation)?,
            );
        }
        Ok(PreparedSamusBinding(Rc::new(SamusBinding {
            world: supplied.world,
            roles,
            expected_roles: Arc::new(expected.roles.clone()),
            spritemaps: supplied.spritemaps,
            request_media: supplied.request_media,
            registrations,
            execution: supplied.execution,
        })))
    }
    /// The binding's world is `images`' world, and each registration still
    /// captures its image at the declared extent.
    fn check_registrations(&self, images: SamusImages<'_>) -> Result<(), DisplayError> {
        if !Arc::ptr_eq(&self.world, images.world) {
            return Err(DisplayError::Identity);
        }
        for (&(registration, identity), admitted) in &self.registrations {
            let image = images.image(registration)?;
            let declaration = &self.roles.description().allocations[&identity];
            if image.len() != declaration.extent {
                return Err(DisplayError::Source);
            }
            admitted.capture(registration, image)?;
        }
        Ok(())
    }
    /// The registration check, and for prepared execution, each image's full
    /// bytes against the expected allocations.
    pub fn check(&self, images: SamusImages<'_>) -> Result<(), DisplayError> {
        self.check_registrations(images)?;
        if self.execution == SamusExecution::Prepared {
            for &(registration, identity) in self.registrations.keys() {
                let image = images.image(registration)?;
                let declaration = &self.roles.description().allocations[&identity];
                RegisteredRom::admit(
                    registration,
                    image.clone(),
                    declaration.clone(),
                    &self.expected_roles.allocations[&identity],
                )?;
            }
        }
        Ok(())
    }
}

/// Samus's binding and authority as the game holds them.
pub struct SamusOwner {
    binding: Option<Rc<SamusBinding>>,
    authority: SamusAuthority,
    mutation_depth: usize,
}
impl Default for SamusOwner {
    fn default() -> Self {
        Self {
            binding: None,
            authority: SamusAuthority::LegacyUnrouted,
            mutation_depth: 0,
        }
    }
}
impl SamusOwner {
    pub fn binding(&self) -> Option<&Rc<SamusBinding>> {
        self.binding.as_ref()
    }
    pub fn mutation_depth(&self) -> usize {
        self.mutation_depth
    }
    /// The raw authority, without the live checks `authority` applies.
    pub fn recorded_authority(&self) -> SamusAuthority {
        self.authority
    }
    /// Publish `prepared` as the available binding.
    pub fn install(&mut self, prepared: PreparedSamusBinding) {
        self.binding = Some(prepared.0);
        self.authority = SamusAuthority::Available;
    }
    /// Adopt a snapshot's binding: available when bound, otherwise unrouted.
    pub fn adopt(&mut self, binding: Option<Rc<SamusBinding>>) {
        self.authority = if binding.is_some() {
            SamusAuthority::Available
        } else {
            SamusAuthority::LegacyUnrouted
        };
        self.binding = binding;
    }
    pub fn authority(&self, images: SamusImages<'_>) -> SamusAuthority {
        if self.binding.is_some() && self.mutation_depth != 0 {
            return SamusAuthority::Revoked;
        }
        if self.authority == SamusAuthority::Available
            && let Some(binding) = &self.binding
            && binding.execution == SamusExecution::Prepared
            && binding.check_registrations(images).is_err()
        {
            return SamusAuthority::Revoked;
        }
        self.authority
    }
    /// Revoke a bound authority.
    pub fn revoke(&mut self) {
        if self.binding.is_some() {
            self.authority = SamusAuthority::Revoked;
        }
    }
    /// Revoke the authority whether or not content is bound.
    pub fn force_revoke(&mut self) {
        self.authority = SamusAuthority::Revoked;
    }
    /// Enter an external mutation interval. A supplied prepared owner cannot
    /// enter unrestricted native mutation while a Session is installed.
    pub fn begin_mutation(&mut self, legacy_session: bool) -> Result<(), DisplayError> {
        if !legacy_session
            && self
                .binding
                .as_ref()
                .is_some_and(|binding| binding.execution == SamusExecution::Prepared)
        {
            return Err(DisplayError::Phase);
        }
        self.mutation_depth = self
            .mutation_depth
            .checked_add(1)
            .ok_or(DisplayError::Phase)?;
        Ok(())
    }
    pub fn end_mutation(&mut self) {
        self.mutation_depth -= 1;
    }
    pub fn preflight(&self, images: SamusImages<'_>) -> Result<(), DisplayError> {
        let Some(binding) = &self.binding else {
            return Ok(());
        };
        if self.authority != SamusAuthority::Available {
            return Err(DisplayError::Poisoned);
        }
        binding.check_registrations(images)?;
        // A fresh numeric compatibility interval inside a retained native call is
        // permitted; it never promotes prepared authority during an outer load.
        if self.mutation_depth != 0 && binding.execution == SamusExecution::Prepared {
            return Err(DisplayError::Phase);
        }
        Ok(())
    }
    /// Whether a snapshot of this owner can be adopted again.
    pub fn admitted(&self) -> bool {
        self.binding.is_none() || self.authority == SamusAuthority::Available
    }
    pub fn execution(
        &self,
        images: SamusImages<'_>,
    ) -> Result<Option<SamusExecution>, DisplayError> {
        let Some(binding) = &self.binding else {
            return Ok(None);
        };
        if !Arc::ptr_eq(&binding.world, images.world) {
            return Err(DisplayError::Identity);
        }
        if binding.execution == SamusExecution::Prepared
            && (self.authority == SamusAuthority::Revoked || self.mutation_depth != 0)
        {
            return Err(DisplayError::Poisoned);
        }
        if binding.execution == SamusExecution::Prepared {
            binding.check_registrations(images)?;
        }
        Ok(Some(binding.execution))
    }
    /// The binding, once `execution` admits it.
    fn admitted_binding(
        &self,
        images: SamusImages<'_>,
    ) -> Result<Option<&SamusBinding>, DisplayError> {
        Ok(self.execution(images)?.and(self.binding.as_deref()))
    }
    /// The caller performs this observation's original width/diagnostic preflight first.
    /// Displacement is linear from the sealed role base; no consumer supplies an identity.
    pub fn capture_role(
        &self,
        images: SamusImages<'_>,
        role: Role,
        registration: Registration,
        displacement: usize,
        selected: &RomImage,
    ) -> Result<Option<RoleLease<CapturedRom>>, DisplayError> {
        let Some(execution) = self.execution(images)? else {
            return Ok(None);
        };
        if execution != SamusExecution::RetainedNative {
            return Err(DisplayError::Phase);
        }
        let binding = self.binding.as_ref().unwrap();
        let declaration = binding
            .roles
            .description()
            .roles
            .get(&(role, registration))
            .ok_or(DisplayError::Incomplete)?;
        let image = images.image(registration)?;
        if !image.same_allocation(selected)
            || image.len()
                != binding.roles.description().allocations[&declaration.allocation].extent
        {
            return Err(DisplayError::Identity);
        }
        let admitted = binding
            .registrations
            .get(&(registration, declaration.allocation))
            .ok_or(DisplayError::Incomplete)?;
        let capsule = admitted.capture(registration, selected)?;
        if capsule.registration() != registration {
            return Err(DisplayError::Identity);
        }
        match registration {
            Registration::Loaded => binding.roles.acquire_loaded(role, displacement, capsule),
            Registration::Native => binding.roles.acquire_native(role, displacement, capsule),
        }
        .map(Some)
    }
    pub fn prepared_role(
        &self,
        images: SamusImages<'_>,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> Result<Option<RoleLease<PreparedBytes>>, DisplayError> {
        let Some(execution) = self.execution(images)? else {
            return Ok(None);
        };
        if execution == SamusExecution::RetainedNative && !native_offset_role(role, registration) {
            return Err(DisplayError::Phase);
        }
        self.binding
            .as_ref()
            .unwrap()
            .roles
            .acquire_prepared(role, registration, displacement)
            .map(Some)
    }
    pub fn role_base(
        &self,
        images: SamusImages<'_>,
        role: Role,
        registration: Registration,
    ) -> Result<Option<usize>, DisplayError> {
        let Some(binding) = self.admitted_binding(images)? else {
            return Ok(None);
        };
        binding
            .roles
            .description()
            .roles
            .get(&(role, registration))
            .map(|declaration| Some(declaration.mapped_base))
            .ok_or(DisplayError::Incomplete)
    }
    pub fn request(
        &self,
        images: SamusImages<'_>,
        half: Half,
        pointer: u16,
    ) -> Result<Option<TileRequest>, DisplayError> {
        self.admitted_binding(images)?
            .map(|binding| binding.roles.request(half, pointer))
            .transpose()
    }
    pub fn spritemaps(
        &self,
        images: SamusImages<'_>,
    ) -> Result<Option<Arc<SamusCatalog>>, DisplayError> {
        Ok(self
            .admitted_binding(images)?
            .map(|binding| binding.spritemaps.clone()))
    }
    pub fn request_media(
        &self,
        images: SamusImages<'_>,
    ) -> Result<Option<Arc<SceneMedia>>, DisplayError> {
        Ok(self
            .admitted_binding(images)?
            .map(|binding| binding.request_media.clone()))
    }
}
