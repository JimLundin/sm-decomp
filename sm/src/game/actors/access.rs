//! Borrowed actor reads and exclusive edits across the single compatibility codec.
#![forbid(unsafe_code)]
use super::state::{Actor, SpawnRecord};
use crate::game::session_access::SessionAccess;
use core::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) struct ActorKey(u16);
impl ActorKey {
    pub(in crate::game) const fn from_legacy(byte_index: u16) -> Self {
        Self(byte_index)
    }
    pub(super) const fn byte_index(self) -> u16 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum AccessMode {
    LittleEndian,
    Native,
}

pub(in crate::game) struct ActorRead<'a> {
    repr: ReadRepr<'a>,
}
enum ReadRepr<'a> {
    Borrowed(&'a Actor),
    Projected(Actor),
}
impl Deref for ActorRead<'_> {
    type Target = Actor;
    fn deref(&self) -> &Actor {
        match &self.repr {
            ReadRepr::Borrowed(actor) => actor,
            ReadRepr::Projected(actor) => actor,
        }
    }
}
/// Both read variants borrow Self. No projected-value detach constructor exists.
pub(in crate::game) trait ActorAccess: SessionAccess {
    #[cfg(sm_verify_dll)]
    fn actor_movement_observe(&mut self, _tag: u8) {}
    fn resolve_actor<'a>(&'a self, key: ActorKey, mode: AccessMode) -> ActorRead<'a>;
    /// R is independent of the actor borrow; references cannot escape the edit.
    fn edit_actor<R>(
        &mut self,
        key: ActorKey,
        mode: AccessMode,
        edit: impl for<'edit> FnOnce(&'edit mut Actor) -> R,
    ) -> R;
}
/// Only this guard retains a projected mutable value; the whole host is borrowed.
struct ProjectedEdit<'a, H: SessionAccess + ?Sized, T> {
    host: &'a mut H,
    key: ActorKey,
    mode: AccessMode,
    original: T,
    edited: T,
    commit: fn(&mut H, ActorKey, AccessMode, &T, &T, fn(&mut H, usize, u8)),
    write_byte: fn(&mut H, usize, u8),
}

// Canonical state encodes words little-endian. Native access may borrow it
// directly only on little-endian hosts; every u16 key remains admissible.
fn canonical_index(key: ActorKey, mode: AccessMode) -> Option<usize> {
    let index = usize::from(key.byte_index());
    (index < 32 * 64
        && index % 64 == 0
        && (mode == AccessMode::LittleEndian || cfg!(target_endian = "little")))
    .then_some(index / 64)
}

impl<H: SessionAccess + ?Sized, T> Drop for ProjectedEdit<'_, H, T> {
    fn drop(&mut self) {
        (self.commit)(
            self.host,
            self.key,
            self.mode,
            &self.original,
            &self.edited,
            self.write_byte,
        );
    }
}

/// Both variants retain the host lifetime, including exceptional projections.
/// End this borrow before any separate content capability can diagnose/mutate.
pub(super) fn resolve_actor_with<'a, H: SessionAccess + ?Sized>(
    host: &'a H,
    key: ActorKey,
    mode: AccessMode,
    read_byte: fn(&H, usize) -> u8,
) -> ActorRead<'a> {
    let repr = match canonical_index(key, mode) {
        Some(index) => ReadRepr::Borrowed(&host.session().actors.slots[index]),
        None => ReadRepr::Projected(super::codec::read_actor(host, key, mode, read_byte)),
    };
    ActorRead { repr }
}

/// The edit cannot borrow the host or return an actor reference. Ordinary
/// fields mutate in place; projected changed fields publish on return or unwind.
/// read_byte/write_byte must access the same current aggregate plus its actual
/// unowned RAM, without observers or failure within the admitted actor window.
pub(super) fn edit_actor_with<H: SessionAccess + ?Sized, R>(
    host: &mut H,
    key: ActorKey,
    mode: AccessMode,
    read_byte: fn(&H, usize) -> u8,
    write_byte: fn(&mut H, usize, u8),
    edit: impl for<'edit> FnOnce(&'edit mut Actor) -> R,
) -> R {
    if let Some(index) = canonical_index(key, mode) {
        return edit(&mut host.session_mut().actors.slots[index]);
    }
    let original = super::codec::read_actor(host, key, mode, read_byte);
    let edited = original.duplicate_for_storage();
    let mut guard = ProjectedEdit {
        host,
        key,
        mode,
        original,
        edited,
        commit: super::codec::write_actor_changes,
        write_byte,
    };
    edit(&mut guard.edited)
}

pub(super) struct SpawnRead<'a> {
    repr: SpawnRepr<'a>,
}
enum SpawnRepr<'a> {
    Borrowed(&'a SpawnRecord),
    Projected(SpawnRecord),
}
impl Deref for SpawnRead<'_> {
    type Target = SpawnRecord;
    fn deref(&self) -> &SpawnRecord {
        match &self.repr {
            SpawnRepr::Borrowed(record) => record,
            SpawnRepr::Projected(record) => record,
        }
    }
}

/// Saved records are mutable gameplay state, sharing the same byte codec with
/// graphics staging. End the read before any observer or other state mutation.
pub(super) trait SpawnAccess: SessionAccess {
    fn resolve_spawn<'a>(&'a self, key: ActorKey, mode: AccessMode) -> SpawnRead<'a>;
    fn edit_spawn<R>(
        &mut self,
        key: ActorKey,
        mode: AccessMode,
        edit: impl for<'edit> FnOnce(&'edit mut SpawnRecord) -> R,
    ) -> R;
}

pub(super) fn resolve_spawn_with<'a, H: SessionAccess + ?Sized>(
    host: &'a H,
    key: ActorKey,
    mode: AccessMode,
    read_byte: fn(&H, usize) -> u8,
) -> SpawnRead<'a> {
    let repr = match canonical_index(key, mode) {
        Some(index) => SpawnRepr::Borrowed(&host.session().actors.saved[index]),
        None => SpawnRepr::Projected(super::codec::read_spawn(host, key, mode, read_byte)),
    };
    SpawnRead { repr }
}

pub(super) fn edit_spawn_with<H: SessionAccess + ?Sized, R>(
    host: &mut H,
    key: ActorKey,
    mode: AccessMode,
    read_byte: fn(&H, usize) -> u8,
    write_byte: fn(&mut H, usize, u8),
    edit: impl for<'edit> FnOnce(&'edit mut SpawnRecord) -> R,
) -> R {
    if let Some(index) = canonical_index(key, mode) {
        return edit(&mut host.session_mut().actors.saved[index]);
    }
    let original = super::codec::read_spawn(host, key, mode, read_byte);
    let edited = original.duplicate_for_storage();
    let mut guard = ProjectedEdit {
        host,
        key,
        mode,
        original,
        edited,
        commit: super::codec::write_spawn_changes,
        write_byte,
    };
    edit(&mut guard.edited)
}

// Reuse the existing store policy: LE prefix writes and preflighted LE/native
// slice writes differ when the final byte is out of bounds.
pub(super) use crate::game::player_projectiles::access::Store as CollectionStore;

/// Only source-audited collection bases. Offsets remain widened and may leave
/// the nominal collection; no generic absolute-memory variant is exposed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CollectionWindow {
    Active,
    Interactive,
    CollisionDirections,
    DrawEntries,
    DrawSizes,
    GraphicsDefinitions,
    GraphicsTiles,
    GraphicsRoutes,
    TileLoad,
    VramQueue,
    /// Caller performs the original draw assertion before access.
    AbsoluteDrawAddress(u16),
}
impl CollectionWindow {
    fn address(self, offset: usize) -> usize {
        let base: usize = match self {
            Self::Active => 0x17ac,
            Self::Interactive => 0x17ec,
            Self::CollisionDirections => 0x182c,
            Self::DrawEntries => 0x0e84,
            Self::DrawSizes => 0x0f68,
            Self::GraphicsDefinitions => 0xef5c,
            Self::GraphicsTiles => 0xef64,
            Self::GraphicsRoutes => 0xef6c,
            Self::TileLoad => 0x0e5a,
            Self::VramQueue => 0x00d0,
            Self::AbsoluteDrawAddress(address) => usize::from(address),
        };
        base.checked_add(offset).expect("actor collection address")
    }
}

/// This private compatibility seam handles source-defined spill/packed aliases.
/// Normal gameplay uses owned fields; it does not expose arbitrary RAM access.
pub(super) trait ActorCollections {
    fn collection_word(
        &self,
        window: CollectionWindow,
        byte_offset: usize,
        mode: AccessMode,
    ) -> u16;
    fn collection_byte(&self, window: CollectionWindow, byte_offset: usize) -> u8;
    fn write_collection_word(
        &mut self,
        window: CollectionWindow,
        byte_offset: usize,
        value: u16,
        store: CollectionStore,
    );
    fn write_collection_byte(&mut self, window: CollectionWindow, byte_offset: usize, value: u8);
    fn write_tile_source(&mut self, descriptor_offset: usize, source: [u8; 3]);
    fn copy_palette_row(&mut self, row: u8, colors: [u8; 32]);
}

pub(super) fn collection_word_with<H: ?Sized>(
    host: &H,
    window: CollectionWindow,
    offset: usize,
    mode: AccessMode,
    read_byte: fn(&H, usize) -> u8,
) -> u16 {
    let at = window.address(offset);
    let bytes = [read_byte(host, at), read_byte(host, at + 1)];
    match mode {
        AccessMode::LittleEndian => u16::from_le_bytes(bytes),
        AccessMode::Native => u16::from_ne_bytes(bytes),
    }
}

pub(super) fn collection_byte_with<H: ?Sized>(
    host: &H,
    window: CollectionWindow,
    offset: usize,
    read_byte: fn(&H, usize) -> u8,
) -> u8 {
    read_byte(host, window.address(offset))
}

/// Slice writer must preflight its whole range before using the same aggregate
/// byte routing. Prefix stores deliberately publish low before testing high.
pub(super) fn write_collection_word_with<H: ?Sized>(
    host: &mut H,
    window: CollectionWindow,
    offset: usize,
    value: u16,
    store: CollectionStore,
    write_byte: fn(&mut H, usize, u8),
    write_slice: fn(&mut H, usize, &[u8]),
) {
    let at = window.address(offset);
    match store {
        CollectionStore::PrefixLe => {
            let bytes = value.to_le_bytes();
            write_byte(host, at, bytes[0]);
            write_byte(host, at + 1, bytes[1]);
        }
        CollectionStore::SliceLe => write_slice(host, at, &value.to_le_bytes()),
        CollectionStore::SliceNative => write_slice(host, at, &value.to_ne_bytes()),
    }
}

pub(super) fn write_collection_byte_with<H: ?Sized>(
    host: &mut H,
    window: CollectionWindow,
    offset: usize,
    value: u8,
    write_byte: fn(&mut H, usize, u8),
) {
    write_byte(host, window.address(offset), value);
}

/// Caller captures all three source bytes before this preflighted copy, exactly
/// as copy_record did. Fifth-entry counter aliases remain ordinary byte writes.
pub(super) fn write_tile_source_with<H: ?Sized>(
    host: &mut H,
    descriptor_offset: usize,
    source: [u8; 3],
    write_slice: fn(&mut H, usize, &[u8]),
) {
    let at = CollectionWindow::TileLoad
        .address(descriptor_offset)
        .checked_add(2)
        .expect("actor tile source address");
    write_slice(host, at, &source);
}

/// A native tileset destination byte selects the row. Do not clamp to palette
/// capacity: high rows intentionally address other valid compatibility bytes.
pub(super) fn copy_palette_row_with<H: ?Sized>(
    host: &mut H,
    row: u8,
    colors: [u8; 32],
    write_slice: fn(&mut H, usize, &[u8]),
) {
    let at = 0xc200 + (usize::from(row) + 8) * 32;
    write_slice(host, at, &colors);
}
