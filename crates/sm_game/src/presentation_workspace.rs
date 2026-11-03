//! Shared mutable screen storage and its supplied allocation provenance.
#![forbid(unsafe_code)]
use crate::scene::records::{AssetRef, Revision};
use crate::scene::state::{DisplayError, WordOrder};
use std::collections::BTreeMap;

pub const FIRST: usize = 0x3000;
pub const BYTES: usize = 0x4000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Pause,
    MessageText,
    MessageDescriptor,
    Menu,
    Equipment,
    SavedBackground,
    Xray,
}
impl Role {
    pub const ALL: [Self; 7] = [
        Self::Pause,
        Self::MessageText,
        Self::MessageDescriptor,
        Self::Menu,
        Self::Equipment,
        Self::SavedBackground,
        Self::Xray,
    ];
    pub const fn compatibility_base(self) -> usize {
        match self {
            Self::Pause => 0x3000,
            Self::MessageText => 0x3200,
            Self::MessageDescriptor => 0x3380,
            Self::Menu => 0x3600,
            Self::Equipment => 0x3800,
            Self::SavedBackground => 0x4100,
            Self::Xray => 0x4000,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub role: Role,
    pub offset: usize,
    physical: Option<usize>,
}
impl Point {
    pub const fn new(role: Role, offset: usize) -> Self {
        Self {
            role,
            offset,
            physical: None,
        }
    }
    /// A retained physical alias. Prepared role accesses never infer this mapping.
    pub const fn physical(at: usize) -> Self {
        Self {
            role: Role::Pause,
            offset: 0,
            physical: Some(at),
        }
    }
    pub fn compatibility_address(self) -> Result<usize, DisplayError> {
        self.physical
            .unwrap_or(self.role.compatibility_base())
            .checked_add(self.offset)
            .ok_or(DisplayError::Source)
    }
    pub(crate) fn physical_address(self) -> Option<usize> {
        self.physical
    }
    pub fn advance(self, bytes: usize) -> Result<Self, DisplayError> {
        Ok(Self {
            offset: self.offset.checked_add(bytes).ok_or(DisplayError::Source)?,
            ..self
        })
    }
}
/// The actual source allocation, not a role-derived physical fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    Shared,
    Allocation(u64),
}
/// A captured coordinate in the actual allocation. Role bindings are queried
/// when this is selected, never again while an in-flight access retains it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcePoint {
    pub origin: Origin,
    pub offset: usize,
}
impl SourcePoint {
    pub fn advance(self, bytes: usize) -> Result<Self, DisplayError> {
        Ok(Self {
            offset: self.offset.checked_add(bytes).ok_or(DisplayError::Source)?,
            ..self
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub origin: Origin,
    pub base: usize,
    pub extent: usize,
    pub asset: AssetRef,
    pub window: Option<Window>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub bank: u8,
    pub address: u16,
    pub length: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bindings(pub BTreeMap<Role, Binding>);

/// All shared aliases occupy this one allocation. Separate supplied allocations
/// retain their identities; two roles may name the same allocation and overlap.
#[derive(Clone, Debug, PartialEq)]
pub struct Workspace {
    pub(crate) initialized: bool,
    pub(crate) shared: Box<[u8]>,
    pub bindings: Bindings,
    pub allocations: BTreeMap<u64, Vec<u8>>,
    pub native_order: WordOrder,
}
impl Workspace {
    pub fn from_owned(
        shared: Box<[u8]>,
        bindings: Bindings,
        allocations: BTreeMap<u64, Vec<u8>>,
        native_order: WordOrder,
        revision: Revision,
    ) -> Result<Self, DisplayError> {
        if shared.len() != BYTES {
            return Err(DisplayError::Incomplete);
        }
        let result = Self {
            initialized: true,
            shared,
            bindings,
            allocations,
            native_order,
        };
        result.validate(revision)?;
        Ok(result)
    }
    pub(crate) fn compatibility(bytes: impl Fn(usize) -> u8) -> Self {
        let mut shared = vec![0; BYTES].into_boxed_slice();
        for (offset, byte) in shared.iter_mut().enumerate() {
            *byte = bytes(FIRST + offset);
        }
        // Compatibility has no prepared content identity. It cannot admit an
        // owned prepared start; tools must supply the actual complete bindings.
        let bindings = Bindings(
            Role::ALL
                .into_iter()
                .map(|role| {
                    (
                        role,
                        Binding {
                            origin: Origin::Shared,
                            base: role.compatibility_base() - FIRST,
                            extent: BYTES - (role.compatibility_base() - FIRST),
                            asset: AssetRef {
                                id: crate::scene::records::AssetId(0),
                                revision: Revision([0; 32]),
                            },
                            window: Some(Window {
                                bank: 0x7e,
                                address: u16::try_from(role.compatibility_base())
                                    .expect("screen anchor"),
                                length: BYTES - (role.compatibility_base() - FIRST),
                            }),
                        },
                    )
                })
                .collect(),
        );
        Self {
            initialized: true,
            shared,
            bindings,
            allocations: BTreeMap::new(),
            native_order: if cfg!(target_endian = "little") {
                WordOrder::Little
            } else {
                WordOrder::Big
            },
        }
    }
    pub(crate) fn empty() -> Self {
        Self::compatibility(|_| 0)
    }
    /// An inaccessible marker while the exclusive allocation is leased locally.
    /// It holds no bytes and cannot validate, render or become a prepared start.
    pub(crate) fn vacant() -> Self {
        Self {
            initialized: false,
            shared: Box::new([]),
            bindings: Bindings(BTreeMap::new()),
            allocations: BTreeMap::new(),
            native_order: WordOrder::Little,
        }
    }
    pub(crate) fn take(&mut self) -> Self {
        std::mem::replace(self, Self::vacant())
    }
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    pub fn validate(&self, revision: Revision) -> Result<(), DisplayError> {
        if !self.initialized
            || self.shared.len() != BYTES
            || self.bindings.0.len() != Role::ALL.len()
        {
            return Err(DisplayError::Incomplete);
        }
        for role in Role::ALL {
            let binding = self.bindings.0.get(&role).ok_or(DisplayError::Incomplete)?;
            if binding.asset.id.0 == 0 {
                return Err(DisplayError::Identity);
            }
            if binding.asset.revision != revision {
                return Err(DisplayError::Revision);
            }
            let bytes = self.storage(binding.origin)?;
            let end = binding
                .base
                .checked_add(binding.extent)
                .ok_or(DisplayError::Source)?;
            if binding.extent == 0 || end > bytes.len() {
                return Err(DisplayError::Source);
            }
            if let Some(window) = binding.window
                && (window.length == 0
                    || window.length > binding.extent
                    || usize::from(window.address)
                        .checked_add(window.length)
                        .is_none_or(|end| end > 0x10000))
            {
                return Err(DisplayError::Source);
            }
        }
        let bindings: Vec<_> = self.bindings.0.values().collect();
        for (index, first) in bindings.iter().enumerate() {
            for second in &bindings[index + 1..] {
                if first.asset == second.asset
                    && (first.origin != second.origin || first.base != second.base)
                {
                    return Err(DisplayError::Identity);
                }
                if let (Some(a), Some(b)) = (first.window, second.window) {
                    let start = usize::from(a.address).max(usize::from(b.address));
                    let end =
                        (usize::from(a.address) + a.length).min(usize::from(b.address) + b.length);
                    if a.bank == b.bank
                        && start < end
                        && (first.origin != second.origin
                            || first.base + start - usize::from(a.address)
                                != second.base + start - usize::from(b.address))
                    {
                        return Err(DisplayError::Identity);
                    }
                }
            }
        }
        Ok(())
    }
    fn storage(&self, origin: Origin) -> Result<&[u8], DisplayError> {
        match origin {
            Origin::Shared => Ok(self.shared.as_ref()),
            Origin::Allocation(id) => self
                .allocations
                .get(&id)
                .map(Vec::as_slice)
                .ok_or(DisplayError::Identity),
        }
    }
    fn storage_mut(&mut self, origin: Origin) -> Result<&mut [u8], DisplayError> {
        match origin {
            Origin::Shared => Ok(self.shared.as_mut()),
            Origin::Allocation(id) => self
                .allocations
                .get_mut(&id)
                .map(Vec::as_mut_slice)
                .ok_or(DisplayError::Identity),
        }
    }
    fn resolve(&self, point: Point) -> Result<(Origin, usize), DisplayError> {
        if let Some(at) = point.physical {
            let at = at.checked_add(point.offset).ok_or(DisplayError::Source)?;
            return Ok((
                Origin::Shared,
                at.checked_sub(FIRST)
                    .ok_or(DisplayError::Alias { offset: at })?,
            ));
        }
        let binding = self
            .bindings
            .0
            .get(&point.role)
            .ok_or(DisplayError::Incomplete)?;
        let at = binding
            .base
            .checked_add(point.offset)
            .ok_or(DisplayError::Source)?;
        // A role extent is metadata, never a clamp on an original alias. The
        // selected allocation's real bounds decide whether the access succeeds.
        Ok((binding.origin, at))
    }
    /// Actual backing identity and displacement; no role-derived address inference.
    pub fn coordinate(&self, point: Point) -> Result<(Origin, usize), DisplayError> {
        let coordinate = self.resolve(point)?;
        if !self.initialized || self.storage(coordinate.0)?.get(coordinate.1).is_none() {
            return Err(DisplayError::Source);
        }
        Ok(coordinate)
    }
    /// Resolve only the requested coordinate; defer byte bounds to its read/store.
    pub fn source_coordinate(&self, point: Point) -> Result<SourcePoint, DisplayError> {
        if !self.initialized {
            return Err(DisplayError::Incomplete);
        }
        let (origin, offset) = self.resolve(point)?;
        Ok(SourcePoint { origin, offset })
    }
    pub fn read_source_point(&self, point: SourcePoint) -> Result<u8, DisplayError> {
        if !self.initialized {
            return Err(DisplayError::Incomplete);
        }
        self.storage(point.origin)?
            .get(point.offset)
            .copied()
            .ok_or(DisplayError::Alias {
                offset: point.offset,
            })
    }
    pub fn write_source_point(
        &mut self,
        point: SourcePoint,
        value: u8,
    ) -> Result<(), DisplayError> {
        if !self.initialized {
            return Err(DisplayError::Incomplete);
        }
        *self
            .storage_mut(point.origin)?
            .get_mut(point.offset)
            .ok_or(DisplayError::Alias {
                offset: point.offset,
            })? = value;
        Ok(())
    }
    /// Supplied bus provenance is metadata, not a replacement for current bytes.
    /// Multiple different physical coordinates require the native candidate path.
    pub fn source_bus_address(
        &self,
        point: SourcePoint,
    ) -> Result<Option<(u8, u16)>, DisplayError> {
        if !self.initialized {
            return Err(DisplayError::Incomplete);
        }
        let mut selected = None;
        let mut ambiguous = false;
        self.visit_source_bus_addresses(point, |bank, address| {
            let candidate = (bank, address);
            if selected.is_some_and(|prior| prior != candidate) {
                ambiguous = true;
            }
            selected = Some(candidate);
        })?;
        if ambiguous {
            return Err(DisplayError::Source);
        }
        Ok(selected)
    }
    /// Visit candidates in supplied binding order. Native selection owns byte
    /// queries, equality and retained ordinal/origin decisions at this boundary.
    pub fn visit_source_bus_addresses(
        &self,
        point: SourcePoint,
        mut visit: impl FnMut(u8, u16),
    ) -> Result<(), DisplayError> {
        if !self.initialized {
            return Err(DisplayError::Incomplete);
        }
        let mut supplied = false;
        for binding in self.bindings.0.values() {
            if binding.origin != point.origin {
                continue;
            }
            let Some(window) = binding.window else {
                continue;
            };
            let Some(offset) = point
                .offset
                .checked_sub(binding.base)
                .filter(|&offset| offset < window.length)
            else {
                continue;
            };
            let address = usize::from(window.address)
                .checked_add(offset)
                .ok_or(DisplayError::Source)?;
            let candidate = (
                window.bank,
                u16::try_from(address).map_err(|_| DisplayError::Source)?,
            );
            supplied = true;
            visit(candidate.0, candidate.1);
        }
        if supplied {
            return Ok(());
        }
        if point.origin == Origin::Shared && point.offset < self.shared.len() {
            let address = FIRST
                .checked_add(point.offset)
                .ok_or(DisplayError::Source)?;
            visit(
                0x7e,
                u16::try_from(address).map_err(|_| DisplayError::Source)?,
            );
        }
        Ok(())
    }
    pub fn byte(&self, point: Point) -> Result<u8, DisplayError> {
        let (origin, at) = self.resolve(point)?;
        self.storage(origin)?
            .get(at)
            .copied()
            .ok_or(DisplayError::Alias { offset: at })
    }
    pub fn write_byte(&mut self, point: Point, value: u8) -> Result<(), DisplayError> {
        let (origin, at) = self.resolve(point)?;
        *self
            .storage_mut(origin)?
            .get_mut(at)
            .ok_or(DisplayError::Alias { offset: at })? = value;
        Ok(())
    }
    pub fn word(&self, point: Point) -> Result<u16, DisplayError> {
        Ok(u16::from_le_bytes([
            self.byte(point)?,
            self.byte(point.advance(1)?)?,
        ]))
    }
    pub fn native_word(&self, point: Point) -> Result<u16, DisplayError> {
        Ok(self
            .native_order
            .decode([self.byte(point)?, self.byte(point.advance(1)?)?]))
    }
    pub fn write_word(&mut self, point: Point, value: u16) -> Result<(), DisplayError> {
        let bytes = value.to_le_bytes();
        self.write_byte(point, bytes[0])?;
        self.write_byte(point.advance(1)?, bytes[1])
    }
    pub fn write_native_word(&mut self, point: Point, value: u16) -> Result<(), DisplayError> {
        let bytes = self.native_order.encode(value);
        self.write_byte(point, bytes[0])?;
        self.write_byte(point.advance(1)?, bytes[1])
    }
    pub fn write_slice(&mut self, point: Point, bytes: &[u8]) -> Result<(), DisplayError> {
        let (origin, at) = self.resolve(point)?;
        let end = at.checked_add(bytes.len()).ok_or(DisplayError::Source)?;
        self.storage_mut(origin)?
            .get_mut(at..end)
            .ok_or(DisplayError::Alias { offset: at })?
            .copy_from_slice(bytes);
        Ok(())
    }
    pub fn write_native_slice(&mut self, point: Point, value: u16) -> Result<(), DisplayError> {
        self.write_slice(point, &self.native_order.encode(value))
    }
    pub fn shared_byte(&self, at: usize) -> Option<u8> {
        at.checked_sub(FIRST)
            .and_then(|offset| self.shared.get(offset).copied())
    }
    pub fn alias_address(&self, point: Point) -> Result<Option<usize>, DisplayError> {
        if point.physical.is_some() {
            return point.compatibility_address().map(Some);
        }
        let (origin, at) = self.resolve(point)?;
        match origin {
            Origin::Shared => Ok(Some(FIRST.checked_add(at).ok_or(DisplayError::Source)?)),
            Origin::Allocation(_) => Ok(None),
        }
    }
    pub fn write_shared_byte(&mut self, at: usize, value: u8) -> bool {
        let Some(byte) = at
            .checked_sub(FIRST)
            .and_then(|offset| self.shared.get_mut(offset))
        else {
            return false;
        };
        *byte = value;
        true
    }
    pub fn shared_bytes(&self) -> &[u8] {
        &self.shared
    }
    /// Transfer the selected owner's actual allocations at a declared boundary.
    pub fn normalize_from(
        &mut self,
        source: &Self,
        range: std::ops::Range<usize>,
    ) -> Result<(), DisplayError> {
        if range.start <= FIRST && range.end >= FIRST + BYTES {
            *self = source.clone();
            return Ok(());
        }
        if self.bindings != source.bindings
            || self.native_order != source.native_order
            || self.initialized != source.initialized
        {
            return Err(DisplayError::Identity);
        }
        for at in range {
            if let Some(value) = source.shared_byte(at) {
                self.write_shared_byte(at, value);
            }
            if let Ok(address) = u16::try_from(at)
                && let Some(point) = source.bus_point(0x7e, address)
            {
                let value = source.byte(point)?;
                self.write_byte(point, value)?;
            }
        }
        Ok(())
    }
    pub fn available(&self, point: Point) -> Result<usize, DisplayError> {
        let (origin, at) = self.resolve(point)?;
        self.storage(origin)?
            .len()
            .checked_sub(at)
            .ok_or(DisplayError::Source)
    }
    /// Fresh binding selection at each upload/readback byte, with no early copy.
    pub fn source_point(
        &self,
        asset: AssetRef,
        displacement: usize,
    ) -> Result<Point, DisplayError> {
        let (role, _) = self
            .bindings
            .0
            .iter()
            .find(|(_, binding)| binding.asset == asset)
            .ok_or(DisplayError::Identity)?;
        Ok(Point::new(*role, displacement))
    }
    pub fn bus_point(&self, bank: u8, address: u16) -> Option<Point> {
        self.bindings.0.iter().find_map(|(&role, binding)| {
            let window = binding.window?;
            let offset = usize::from(address).checked_sub(usize::from(window.address))?;
            (window.bank == bank && offset < window.length).then_some(Point::new(role, offset))
        })
    }
}

/// Narrow compatibility admission of a pure screen operation. Implementations
/// publish completed writes before forwarding errors or opening a foreign effect.
pub trait Access {
    fn screen_byte(&self, point: Point) -> u8;
    fn screen_write_byte(&mut self, point: Point, value: u8);
    fn screen_write_slice(&mut self, point: Point, bytes: &[u8]);
    fn screen_apply<T>(
        &mut self,
        action: impl FnOnce(&mut dyn Canvas) -> Result<T, DisplayError>,
    ) -> T;
    fn screen_word(&self, point: Point) -> u16 {
        u16::from_le_bytes([
            self.screen_byte(point),
            self.screen_byte(point.advance(1).expect("screen offset")),
        ])
    }
    fn screen_native_word(&self, point: Point) -> u16 {
        u16::from_ne_bytes([
            self.screen_byte(point),
            self.screen_byte(point.advance(1).expect("screen offset")),
        ])
    }
    fn screen_write_word(&mut self, point: Point, value: u16) {
        let bytes = value.to_le_bytes();
        self.screen_write_byte(point, bytes[0]);
        self.screen_write_byte(point.advance(1).expect("screen offset"), bytes[1]);
    }
    fn screen_write_native_word(&mut self, point: Point, value: u16) {
        let bytes = value.to_ne_bytes();
        self.screen_write_byte(point, bytes[0]);
        self.screen_write_byte(point.advance(1).expect("screen offset"), bytes[1]);
    }
    fn screen_write_native_slice(&mut self, point: Point, value: u16) {
        self.screen_write_slice(point, &value.to_ne_bytes());
    }
}
pub trait PaletteAccess {
    fn palette_byte(&self, target: bool, byte_offset: usize) -> u8;
    fn palette_native_word(&self, target: bool, byte_offset: usize) -> u16;
    fn palette_word(&self, target: bool, byte_offset: usize) -> u16;
    fn palette_write_word(&mut self, target: bool, byte_offset: usize, value: u16);
    fn palette_write_native_word(&mut self, target: bool, byte_offset: usize, value: u16);
    fn palette_write_native_slice(&mut self, target: bool, byte_offset: usize, value: u16);
}
/// Linear queue positions preserve menu/ending's native slice identity. The
/// queue_hud producer keeps its separate per-u16-offset wrapping policy.
pub trait QueueAccess: crate::display::queued_tilemaps::QueueAccess {
    fn queue_byte_at(&mut self, offset: usize, value: u8);
    fn queue_native_slice_at(&mut self, offset: usize, value: u16);
}
/// Context-specific callers interpret these existing shared control lanes.
/// No second ScreenExecution or mode-based cancellation is introduced.
pub trait ContinuationAccess {
    fn screen_primary(&self) -> u8;
    fn screen_set_primary(&mut self, value: u8);
    fn demo_phase(&self) -> u8;
    fn demo_set_phase(&mut self, value: u8);
    /// The secondary continuation ($0613): the resume point of a pause hook
    /// or the room mechanisms' pass.
    fn secondary_continuation(&self) -> u8;
    fn set_secondary_continuation(&mut self, value: u8);
    /// The outer continuation ($0611): the frame state, or the boot's resume
    /// point.
    fn outer_continuation(&self) -> u8;
    fn set_outer_continuation(&mut self, value: u8);
    /// The message box continuation ($0615).
    fn message_continuation(&self) -> u8;
    /// Clear the pass continuations ($0612..$0615), the last first, as a
    /// frame starts.
    fn clear_pass_continuations(&mut self);
}

/// Successful slices publish before the next canvas operation or foreign effect.
pub trait Publication {
    fn published(&mut self, workspace: &Workspace, first: SourcePoint, bytes: usize);
}
pub trait SliceWriter {
    fn write_slice(&mut self, point: Point, bytes: &[u8]) -> Result<(), DisplayError>;
}
/// Lazy screen reads and ordered stores. Each successful byte publishes before
/// the next access; slice stores retain their complete extent preflight.
pub trait Canvas: SliceWriter {
    fn byte(&self, point: Point) -> Result<u8, DisplayError>;
    fn native_order(&self) -> WordOrder;
    fn write_byte(&mut self, point: Point, value: u8) -> Result<(), DisplayError>;
    fn word(&self, point: Point) -> Result<u16, DisplayError> {
        Ok(u16::from_le_bytes([
            self.byte(point)?,
            self.byte(point.advance(1)?)?,
        ]))
    }
    fn native_word(&self, point: Point) -> Result<u16, DisplayError> {
        Ok(self
            .native_order()
            .decode([self.byte(point)?, self.byte(point.advance(1)?)?]))
    }
    fn write_native_word(&mut self, point: Point, value: u16) -> Result<(), DisplayError> {
        let bytes = self.native_order().encode(value);
        self.write_byte(point, bytes[0])?;
        self.write_byte(point.advance(1)?, bytes[1])
    }
    fn write_native_slice(&mut self, point: Point, value: u16) -> Result<(), DisplayError> {
        self.write_slice(point, &self.native_order().encode(value))
    }
}
impl Canvas for Workspace {
    fn byte(&self, point: Point) -> Result<u8, DisplayError> {
        Workspace::byte(self, point)
    }
    fn native_order(&self) -> WordOrder {
        self.native_order
    }
    fn write_byte(&mut self, point: Point, value: u8) -> Result<(), DisplayError> {
        Workspace::write_byte(self, point, value)
    }
}
impl SliceWriter for Workspace {
    fn write_slice(&mut self, point: Point, bytes: &[u8]) -> Result<(), DisplayError> {
        Workspace::write_slice(self, point, bytes)
    }
}
pub(crate) struct PublishedWorkspace<'a, P: Publication + ?Sized> {
    pub workspace: &'a mut Workspace,
    pub publication: &'a mut P,
}
impl<P: Publication + ?Sized> SliceWriter for PublishedWorkspace<'_, P> {
    fn write_slice(&mut self, point: Point, bytes: &[u8]) -> Result<(), DisplayError> {
        self.workspace.write_slice(point, bytes)?;
        if !bytes.is_empty() {
            let first = self
                .workspace
                .source_coordinate(point)
                .expect("successful prepared Workspace slice");
            self.publication
                .published(self.workspace, first, bytes.len());
        }
        Ok(())
    }
}
