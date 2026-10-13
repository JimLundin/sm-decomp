use port::{
    WRAM_SIZE, Wram,
    game_host::{self, Entry, Host, Outcome, Request, Response},
};

#[derive(Default)]
struct Recording {
    requests: Vec<Request>,
    replies: std::collections::VecDeque<Result<Response, &'static str>>,
    observed: Vec<(u16, u16)>,
}

impl Host for Recording {
    fn request(&mut self, ram: &mut Wram<'_>, request: Request) -> Result<Response, &'static str> {
        self.requests.push(request);
        self.observed.push((ram.word(0xce2), ram.word(0x1cf4)));
        let reply = self.replies.pop_front().unwrap_or_else(|| {
            Ok(match request {
                Request::SpawnPlm { .. } => Response::Plm(0),
                Request::SpawnHdma { .. } => Response::Hdma(0xffff),
                Request::FlushVram => Response::VramFlushed,
            })
        })?;
        if request == Request::FlushVram {
            assert_eq!(ram.word(0x962) & 0xff, 1);
            self.observed.last_mut().unwrap().0 = ram.word(0x8f7);
            self.observed.last_mut().unwrap().1 = ram.word(0x990);
            for offset in [0x962, 0x970, 0x97e, 0x98c] {
                ram.set_byte(offset, 0);
            }
            ram.set_word(0x330, 0);
        } else {
            ram.set_word(0x1cf4, ram.word(0x1cf4) + 1);
        }
        Ok(reply)
    }
}

fn power_bomb() -> Vec<u8> {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for (offset, value) in [
        (0x7a5, 16),
        (0x7a7, 16),
        (0x7a9, 1),
        (0x7ab, 1),
        (0xc4a, 0xfff8),
        (0xc5e, 2),
        (0xc72, 0xb0ae),
        (0xc22, 0x400),
        (0xc86, 1),
        (0xb6e, 100),
        (0xb82, 120),
    ] {
        ram.set_word(offset, value);
    }
    bytes
}

fn room() -> Vec<u8> {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x7a5, 32);
    ram.set_byte(0x58, 0x51);
    ram.set_byte(0x59, 0x49);
    for (i, tile) in [0x11, 0x22, 0x33, 0x44].into_iter().enumerate() {
        ram.set_word(0xa008 + i * 2, tile);
    }
    for i in 0..0x8000 {
        ram.set_word(0x10000 + i * 2, 1);
    }
    bytes
}

#[test]
fn whole_projectile_frame_transfers_current_wram_between_ordered_hdma_requests() {
    let mut bytes = power_bomb();
    let mut host = Recording::default();
    let outcome = game_host::execute(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, Entry::SamusProjectiles);
    assert_eq!(outcome, Outcome::Completed { value: 0, requests: 2 });
    assert_eq!(host.requests, [Request::SpawnHdma { script: 0x8ace }, Request::SpawnHdma { script: 0x8b80 }]);
    assert_eq!(host.observed, [(100, 0), (100, 1)]);
    let ram = Wram::new(&mut bytes).unwrap();
    assert_eq!((ram.word(0xc86), ram.word(0x592), ram.word(0x1cf4)), (0, 0x8000, 2));
    assert_eq!(ram.word(0xdde), 0xfffe);
}

#[test]
fn unsupported_later_slot_refuses_without_publishing_earlier_scheduler_work() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    for slot in [18, 0] {
        ram.set_word(0xc40 + slot, 0xfff8);
        ram.set_word(0xc54 + slot, 4);
        ram.set_word(0xc68 + slot, if slot == 18 { 0xb169 } else { 0xffff });
    }
    let before = bytes.clone();
    let mut host = Recording::default();
    assert!(matches!(
        game_host::execute(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, Entry::SamusProjectiles),
        Outcome::Refused { reason: "unsupported projectile frame preinstruction" }
    ));
    assert_eq!(bytes, before);
    assert!(host.requests.is_empty());
}

#[test]
fn first_failing_request_retains_rust_ownership() {
    let mut bytes = power_bomb();
    let before = bytes.clone();
    let mut host = Recording { replies: [Err("native request failed")].into(), ..Default::default() };
    assert_eq!(
        game_host::execute(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, Entry::SamusProjectiles),
        Outcome::OwnedFailure { reason: "native request failed", requests: 1 }
    );
    assert_eq!(host.requests.len(), 1);
    assert_eq!(bytes, before);
}

#[test]
fn later_failing_request_does_not_repeat_the_first_effect_or_publish_partial_wram() {
    let mut bytes = power_bomb();
    let before = bytes.clone();
    let mut host =
        Recording { replies: [Ok(Response::Hdma(2)), Err("second allocation failed")].into(), ..Default::default() };
    assert_eq!(
        game_host::execute(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, Entry::SamusProjectiles),
        Outcome::OwnedFailure { reason: "second allocation failed", requests: 2 }
    );
    assert_eq!(host.requests.len(), 2);
    assert_eq!(host.observed, [(100, 0), (100, 1)]);
    assert_eq!(bytes, before);
}

#[test]
fn unsupported_lower_slot_after_host_effects_is_an_owned_failure() {
    let mut bytes = power_bomb();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xc40, 0xfff8);
    ram.set_word(0xc68, 0xffff);
    let before = bytes.clone();
    let mut host = Recording::default();
    assert_eq!(
        game_host::execute(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, Entry::SamusProjectiles),
        Outcome::OwnedFailure { reason: "unsupported projectile frame preinstruction", requests: 2 }
    );
    assert_eq!(host.requests.len(), 2);
    assert_eq!(bytes, before);
}

#[test]
fn response_kind_mismatch_is_an_owned_failure() {
    let mut bytes = power_bomb();
    let mut host = Recording { replies: [Ok(Response::Plm(0))].into(), ..Default::default() };
    assert_eq!(
        game_host::execute(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, Entry::SamusProjectiles),
        Outcome::OwnedFailure { reason: "game host HDMA response kind differs", requests: 1 }
    );
}

#[test]
fn whole_projectile_frame_routes_repeated_bomb_terrain_contacts_through_shared_plm_request() {
    let mut bytes = power_bomb();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xc86, 0);
    ram.set_word(0xcee, 1);
    ram.set_word(0xce2, 100);
    ram.set_word(0xce4, 120);
    ram.set_word(0x10002 + 2 * 118, 0x4000);
    let mut rom = vec![0; 0x200000];
    for (address, word) in [(0x949ea6u32, 0xd064u16), (0x84d064, 0xce6b), (0x84d066, 0xcadf)] {
        let offset = ((address >> 16 & 0x7f) * 0x8000 + (address & 0x7fff)) as usize;
        rom[offset..offset + 2].copy_from_slice(&word.to_le_bytes());
    }
    let mut host = Recording::default();
    assert_eq!(
        game_host::execute(&mut Wram::new(&mut bytes).unwrap(), &rom, &mut host, Entry::SamusProjectiles),
        Outcome::Completed { value: 0, requests: 4 }
    );
    assert_eq!(host.requests, vec![Request::SpawnPlm { header: 0xd064 }; 4]);
    assert_eq!(host.observed, [(100, 0), (100, 1), (100, 2), (100, 3)]);
}

#[test]
fn second_existing_module_flushes_all_seventeen_columns_synchronously() {
    let mut bytes = room();
    let mut host = Recording::default();
    assert_eq!(
        game_host::execute(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, Entry::RoomDisplay),
        Outcome::Completed { value: 0, requests: 17 }
    );
    assert_eq!(host.requests, vec![Request::FlushVram; 17]);
    assert_eq!(host.observed, (0..17).map(|i| (i, i)).collect::<Vec<_>>());
    let ram = Wram::new(&mut bytes).unwrap();
    assert_eq!((ram.word(0x98e), ram.word(0x8f7), ram.word(0x330)), (0x800, 17, 0));
}

#[test]
fn malformed_later_room_column_refuses_before_the_first_flush() {
    let mut bytes = room();
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x7a5, 255);
    ram.set_word(0x915, 0x7ff0);
    let before = bytes.clone();
    let mut host = Recording::default();
    assert!(matches!(
        game_host::execute(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, Entry::RoomDisplay),
        Outcome::Refused { .. }
    ));
    assert_eq!(bytes, before);
    assert!(host.requests.is_empty());
}

#[test]
fn fixture_compatibility_rejects_unknown_or_parameterized_calls_before_requests() {
    let mut bytes = power_bomb();
    let before = bytes.clone();
    let mut host = Recording::default();
    for (symbol, a, b) in [("HandleProjectile", 1, 0), ("HandleProjectile", 0, 1), ("Unknown", 0, 0)] {
        assert!(game_host::dispatch_host(&mut Wram::new(&mut bytes).unwrap(), &[], &mut host, symbol, a, b).is_err());
    }
    assert_eq!(bytes, before);
    assert!(host.requests.is_empty());
}
