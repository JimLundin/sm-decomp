//! Actor-private tests of actual aggregate routing and actual access guards.
use super::access::*;
use crate::game::{
    external_inputs::ExternalInputs, room_entry::world::WorldBinding, session::Session,
    session_access::SessionAccess, session_encoding as encoding,
};
use sm_game::actors::state::Actor;
use std::panic::{AssertUnwindSafe, catch_unwind};

struct Host {
    session: Session,
    backing: Box<[u8; 131072]>,
    inputs: ExternalInputs,
}
impl Host {
    fn patterned() -> Self {
        let bytes: Box<[u8; 131072]> = (0..131072)
            .map(|i| ((i * 37 + 19) & 255) as u8)
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .try_into()
            .unwrap();
        let session = encoding::decode(
            &|at| bytes[at],
            bytes.len(),
            WorldBinding::canonical_legacy(),
        );
        Self {
            session,
            backing: bytes,
            inputs: ExternalInputs::default(),
        }
    }
    fn byte(&self, at: usize) -> u8 {
        encoding::projected_byte(&self.session, at).unwrap_or(self.backing[at])
    }
    fn write(&mut self, at: usize, value: u8) {
        if !encoding::write_projected_byte(&mut self.session, at, value) {
            self.backing[at] = value;
        }
    }
    fn image(&self) -> Vec<u8> {
        (0..131072).map(|at| self.byte(at)).collect()
    }
}
impl SessionAccess for Host {
    fn session(&self) -> &Session {
        &self.session
    }
    fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }
    fn inputs(&self) -> &ExternalInputs {
        &self.inputs
    }
    fn probe(&self, _: &'static str) {}
}
fn assert_named_panic(error: Box<dyn core::any::Any + Send>) {
    assert_eq!(
        error.downcast_ref::<&str>(),
        Some(&"actor guard fixture unwind")
    );
}
fn bytes(value: u16, mode: AccessMode) -> [u8; 2] {
    match mode {
        AccessMode::LittleEndian => value.to_le_bytes(),
        AccessMode::Native => value.to_ne_bytes(),
    }
}

#[test]
fn actor_edits_publish_only_changed_fields_on_normal_return_and_unwind() {
    for key in [0, 64, 1, 0xe3ea, 0xffff] {
        for mode in [AccessMode::LittleEndian, AccessMode::Native] {
            let mut host = Host::patterned();
            let mut expected = host.image();
            let base = 0xf78 + usize::from(key);
            let observed =
                resolve_actor_with(&host, ActorKey::from_legacy(key), mode, Host::byte).health;
            let original_pair = [expected[base + 0x14], expected[base + 0x15]];
            assert_eq!(
                observed,
                match mode {
                    AccessMode::LittleEndian => u16::from_le_bytes(original_pair),
                    AccessMode::Native => u16::from_ne_bytes(original_pair),
                }
            );
            let result = catch_unwind(AssertUnwindSafe(|| {
                edit_actor_with(
                    &mut host,
                    ActorKey::from_legacy(key),
                    mode,
                    Host::byte,
                    Host::write,
                    |actor| {
                        actor.health = 0x1267;
                        actor.spritemap = 0x3456;
                        panic!("actor guard fixture unwind");
                    },
                );
            }));
            assert_named_panic(result.unwrap_err());
            expected[base + 0x14..base + 0x16].copy_from_slice(&bytes(0x1267, mode));
            expected[base + 0x16..base + 0x18].copy_from_slice(&bytes(0x3456, mode));
            assert_eq!(
                host.image() == expected,
                true,
                "actor.contract.guard_unwind_changed_bytes key={key:04x} mode={mode:?}"
            );
            edit_actor_with(
                &mut host,
                ActorKey::from_legacy(key),
                mode,
                Host::byte,
                Host::write,
                |actor| actor.health = 0x9abc,
            );
            expected[base + 0x14..base + 0x16].copy_from_slice(&bytes(0x9abc, mode));
            assert_eq!(
                host.image() == expected,
                true,
                "actor.contract.guard_return_changed_bytes"
            );
        }
    }
}

#[test]
fn saved_record_odd_and_broad_edits_publish_words_and_name_bytes_on_unwind() {
    for key in [0, 1, 0x7c0, 0x800, 0xffff] {
        for mode in [AccessMode::LittleEndian, AccessMode::Native] {
            let mut host = Host::patterned();
            let mut expected = host.image();
            let base = 0x7000 + usize::from(key);
            let result = catch_unwind(AssertUnwindSafe(|| {
                edit_spawn_with(
                    &mut host,
                    ActorKey::from_legacy(key),
                    mode,
                    Host::byte,
                    Host::write,
                    |saved| {
                        saved.death_cause = 0xcdef;
                        saved.name[17] = 0x37;
                        panic!("actor guard fixture unwind");
                    },
                );
            }));
            assert_named_panic(result.unwrap_err());
            expected[base + 2..base + 4].copy_from_slice(&bytes(0xcdef, mode));
            expected[base + 63] = 0x37;
            assert_eq!(
                host.image() == expected,
                true,
                "actor.contract.spawn_guard_unwind_changed_bytes key={key:04x} mode={mode:?}"
            );
        }
    }
}

#[test]
fn broad_actor_map_write_reaches_the_shared_room_processing_stage() {
    let mut host = Host::patterned();
    let key = ActorKey::from_legacy(0xe3ea);
    edit_actor_with(
        &mut host,
        key,
        AccessMode::LittleEndian,
        Host::byte,
        Host::write,
        |actor: &mut Actor| actor.spritemap = 9,
    );
    assert_eq!([host.byte(0xf378), host.byte(0xf379)], [9, 0]);
    host.write(0xf378, 0x6a);
    assert_eq!(
        resolve_actor_with(&host, key, AccessMode::LittleEndian, Host::byte).spritemap,
        0x6a
    );
}
