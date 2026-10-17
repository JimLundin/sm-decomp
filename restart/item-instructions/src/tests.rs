use super::*;
use fixtures::{Expectation, Fixture, body_fixtures, refusal_fixtures};
use std::ffi::OsString;
use std::sync::OnceLock;

fn sealed_input(path: OsString, size: usize, hash: &str) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut bytes = Vec::with_capacity(size);
    std::fs::File::open(path)
        .expect("open retained assertion input")
        .take(size as u64 + 1)
        .read_to_end(&mut bytes)
        .expect("read retained assertion input");
    assert_eq!(bytes.len(), size, "exact assertion input size");
    assert_eq!(format!("{:x}", Sha256::digest(&bytes)), hash, "exact assertion input hash");
    bytes
}

fn inputs() -> &'static (Vec<u8>, Vec<u8>) {
    static INPUTS: OnceLock<(Vec<u8>, Vec<u8>)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let seed = std::env::var_os("SM_ITEM_FIXTURE_SEED").unwrap_or_else(|| {
            "/home/jim/sm-decomp-restart/restart/out/first-missile/author-05/missile-revealed.wram".into()
        });
        let rom = std::env::var_os("SM_ITEM_FIXTURE_ROM").unwrap_or_else(|| "/home/jim/sm-decomp/rom/sm.sfc".into());
        (
            sealed_input(seed, 131072, "7adc5568e4239e479fcef325fef05769da8f6399e5909174d5f6f00c2f436459"),
            sealed_input(rom, 3145728, "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72"),
        )
    })
}

struct ServiceRecord {
    service: Service,
    argument: u16,
    before: Vec<u8>,
}
struct AuditServices {
    enabled: bool,
    calls: Vec<ServiceRecord>,
    write_markers: bool,
    fail_at: Option<Service>,
    unavailable: Option<Service>,
}
impl AuditServices {
    fn new(enabled: bool) -> Self {
        Self { enabled, calls: vec![], write_markers: false, fail_at: None, unavailable: None }
    }
    fn call(&mut self, ram: &mut [u8], service: Service, argument: u16) -> Result<(), String> {
        self.calls.push(ServiceRecord { service, argument, before: ram.to_vec() });
        if self.write_markers {
            ram[0x1fff0 + service as usize] = 0xa0 + service as u8;
        }
        if self.fail_at == Some(service) {
            return Err("intentional native service failure".into());
        }
        Ok(())
    }
}
impl Services for AuditServices {
    fn available(&self, service: Service) -> bool {
        self.enabled && self.unavailable != Some(service)
    }
    fn add_missiles_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.call(ram, Service::AddMissilesToHud, 0)
    }
    fn play_room_music_after(&mut self, ram: &mut [u8], frames: u16) -> Result<(), String> {
        self.call(ram, Service::PlayRoomMusicAfter, frames)
    }
    fn queue_music_delayed8(&mut self, ram: &mut [u8], track: u16) -> Result<(), String> {
        self.call(ram, Service::QueueMusicDelayed8, track)
    }
    fn process_plm_draw_instruction(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
        self.call(ram, Service::ProcessPlmDrawInstruction, slot)
    }
    fn calculate_plm_block_coords(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
        self.call(ram, Service::CalculatePlmBlockCoords, slot)
    }
    fn draw_plm(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
        self.call(ram, Service::DrawPlm, slot)
    }
    fn add_super_missiles_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.call(ram, Service::AddSuperMissilesToHud, 0)
    }
    fn add_power_bombs_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.call(ram, Service::AddPowerBombsToHud, 0)
    }
}

fn body(id: &str) -> Fixture {
    let (seed, rom) = inputs();
    body_fixtures(seed, rom).unwrap().into_iter().find(|f| f.id == id).unwrap()
}

#[cfg_attr(test, test)]
fn frozen_body_matrix_preserves_all_unlisted_bytes_and_exact_typed_returns() {
    let (seed, rom) = inputs();
    let cases = body_fixtures(seed, rom).unwrap();
    assert_eq!(cases.len(), 25);
    let mut covered = [false; 13];
    for (index, mut fixture) in cases.into_iter().enumerate() {
        assert_eq!(fixture.id, format!("B{:02}", index + 1));
        covered[fixture.raw_entry as usize] = true;
        let mut expected = fixture.ram.clone();
        for &write in &fixture.direct_writes {
            write.apply(&mut expected);
        }
        // These inert services isolate each body's direct effects; actual-C
        // controls use the same inputs with real HUD/music and full snapshots.
        let mut services = AuditServices::new(true);
        let result = fixture.run(&mut services).unwrap();
        assert_eq!(fixture.expected, Expectation::Handled(result.expect("body handled")), "{}", fixture.id);
        assert_eq!(fixture.ram, expected, "{} full WRAM", fixture.id);
        let calls: Vec<_> = services.calls.iter().map(|c| (c.service, c.argument)).collect();
        assert_eq!(calls, fixture.service_calls, "{} service order", fixture.id);
        for call in &services.calls {
            if fixture.entry == Some(Entry::CollectAmmoMissileTank) {
                assert_eq!(word(&call.before, 0x9c6), word(&expected, 0x9c6));
                assert_eq!(word(&call.before, 0x9c8), word(&expected, 0x9c8));
                assert_eq!(word(&call.before, 0xdc8), 0x7777, "message queues after services");
            } else {
                assert!(call.before[0x619..0x639].iter().all(|&b| b == 0), "music reset precedes queue");
                assert_eq!(word(&call.before, 0x63b), 14);
                assert_eq!(word(&call.before, 0x63d), 0);
                assert_eq!(word(&call.before, 0x63f), 0);
            }
        }
    }
    assert!(covered.into_iter().all(|entry| entry));
}

#[cfg_attr(test, test)]
fn frozen_refusal_matrix_is_atomic_and_never_requests_native_service() {
    let (seed, rom) = inputs();
    let cases = refusal_fixtures(seed, rom).unwrap();
    assert_eq!(cases.len(), 14);
    for (index, mut fixture) in cases.into_iter().enumerate() {
        assert_eq!(fixture.id, format!("R{:02}", index + 1));
        let before = fixture.ram.clone();
        let mut services = AuditServices::new(fixture.services_available);
        let actual = fixture.run(&mut services);
        match fixture.expected {
            Expectation::Refused => assert_eq!(actual.unwrap(), None, "{}", fixture.id),
            Expectation::Error => assert!(actual.is_err(), "{}", fixture.id),
            _ => panic!("refusal fixture expected handled"),
        }
        assert_eq!(fixture.ram, before, "{} changed WRAM on refusal", fixture.id);
        assert!(services.calls.is_empty(), "{} called native service", fixture.id);
    }
}

#[cfg_attr(test, test)]
fn native_service_effects_flow_into_later_service_and_final_commit() {
    let mut fixture = body("B12");
    let mut services = AuditServices::new(true);
    services.write_markers = true;
    assert_eq!(fixture.run(&mut services).unwrap(), Some(Output::Continue(0xe4a8)));
    assert_eq!(services.calls[1].before[0x1fff0], 0xa0);
    assert_eq!(fixture.ram[0x1fff0..0x1fff2], [0xa0, 0xa1]);
    assert_eq!(word(&fixture.ram, 0xdc8), 2);
}

#[cfg_attr(test, test)]
fn service_failure_stops_before_second_service_and_message_without_fallback() {
    let mut fixture = body("B12");
    let mut services = AuditServices::new(true);
    services.fail_at = Some(Service::AddMissilesToHud);
    assert!(fixture.run(&mut services).is_err());
    assert_eq!(services.calls.len(), 1);
    assert_eq!(word(&fixture.ram, 0xdc8), 0x7777);
    // The adapter owns rollback/fatal handling after a foreign call, whereas
    // all static refusal cases above remain atomic before any such effects.
    assert_eq!(word(&fixture.ram, 0x9c6), 5);
}

#[cfg_attr(test, test)]
fn every_legal_shell_and_visible_preinstruction_position_is_supported() {
    let source = body("B22");
    for (link, positions) in [(0xe484, &[0xdfb7, 0xdfbb, 0xdfbf, 0xdfc3][..]), (0xe49f, &[0xe497, 0xe49b][..])] {
        for &position in positions {
            let mut fixture = source.clone();
            set_word(&mut fixture.ram, LINK + 78, link);
            set_word(&mut fixture.ram, IP + 78, position);
            let before = fixture.ram.clone();
            let mut services = AuditServices::new(false);
            assert_eq!(fixture.run(&mut services).unwrap(), Some(Output::PreinstructionDone));
            assert_eq!(fixture.ram, before);
        }
    }
}

#[cfg_attr(test, test)]
fn fixture_builders_reject_unsealed_seed_and_entry_mapping_is_exact() {
    assert!(body_fixtures(&[], &[]).is_err());
    assert!(refusal_fixtures(&vec![0; WRAM_SIZE], &[]).is_err());
    for entry in Entry::ALL {
        assert_eq!(Entry::try_from(entry as u32).unwrap(), entry);
    }
    assert!(Entry::try_from(21).is_err());
    assert_eq!(Entry::GotoLinkIfTriggered.address(), 0x84df89);
}

#[cfg_attr(test, test)]
fn morph_body_delta_preserves_full_wram_and_orders_local_common_services() {
    let (seed, rom) = inputs();
    let cases = fixtures::morph_body_fixtures(seed, rom).unwrap();
    assert_eq!(cases.len(), 31);
    let mut covered = [false; 17];
    for (index, mut fixture) in cases.into_iter().enumerate() {
        assert_eq!(fixture.id, format!("M{:02}", index + 1));
        covered[fixture.raw_entry as usize] = true;
        let mut expected = fixture.ram.clone();
        for &write in &fixture.direct_writes {
            write.apply(&mut expected);
        }
        let mut services = AuditServices::new(true);
        let actual = fixture.run(&mut services).unwrap().unwrap();
        assert_eq!(fixture.expected, Expectation::Handled(actual), "{}", fixture.id);
        assert_eq!(fixture.ram, expected, "{} full WRAM", fixture.id);
        let calls: Vec<_> = services.calls.iter().map(|call| (call.service, call.argument)).collect();
        assert_eq!(calls, fixture.service_calls, "{} service order/arguments", fixture.id);
        if matches!(fixture.entry, Some(Entry::DrawItemFrame0 | Entry::DrawItemFrame1)) {
            assert_eq!(services.calls.len(), 3);
            for call in &services.calls {
                assert_eq!(word(&call.before, INSTRUCTION_TIMER + 68), 4);
                assert_eq!(word(&call.before, IP + 68), fixture.operand.unwrap());
                assert_eq!(word(&call.before, morph::DRAW + 68), word(&expected, morph::DRAW + 68));
            }
        }
        if fixture.entry == Some(Entry::PickupEquipmentAndShowMessage) {
            assert_eq!(word(&services.calls[0].before, 0x9a2), 0x44);
            assert_eq!(word(&services.calls[0].before, 0x9a4), 0x204);
            assert_eq!(word(&services.calls[0].before, 0xdc8), 0x7777);
        }
    }
    let roots: Vec<_> = covered.into_iter().enumerate().filter_map(|(i, set)| set.then_some(i)).collect();
    assert_eq!(roots, [0, 1, 3, 5, 6, 8, 11, 12, 13, 14, 15, 16]);
}

#[cfg_attr(test, test)]
fn morph_refusal_delta_is_atomic_before_any_native_service() {
    let (seed, rom) = inputs();
    let cases = fixtures::morph_refusal_fixtures(seed, rom).unwrap();
    assert_eq!(cases.len(), 20);
    for (index, mut fixture) in cases.into_iter().enumerate() {
        assert_eq!(fixture.id, format!("MR{:02}", index + 1));
        let before = fixture.ram.clone();
        let mut services = AuditServices::new(fixture.services_available);
        let actual = fixture.run(&mut services);
        match fixture.expected {
            Expectation::Refused => assert_eq!(actual.unwrap(), None, "{}", fixture.id),
            Expectation::Error => assert!(actual.is_err(), "{}", fixture.id),
            _ => panic!("refusal expected handled"),
        }
        assert_eq!(fixture.ram, before, "{} refusal writes", fixture.id);
        assert!(services.calls.is_empty(), "{} refusal service", fixture.id);
    }
}

#[cfg_attr(test, test)]
fn each_draw_capability_is_preflighted_and_failures_stop_without_fallback() {
    let (seed, rom) = inputs();
    let source = fixtures::morph_body_fixtures(seed, rom).unwrap().remove(4);
    for (index, service) in
        [Service::ProcessPlmDrawInstruction, Service::CalculatePlmBlockCoords, Service::DrawPlm].into_iter().enumerate()
    {
        let mut fixture = source.clone();
        let mut unavailable = AuditServices::new(true);
        unavailable.unavailable = Some(service);
        assert!(fixture.run(&mut unavailable).is_err());
        assert_eq!(fixture.ram, source.ram);
        assert!(unavailable.calls.is_empty());
        let mut failing = AuditServices::new(true);
        failing.fail_at = Some(service);
        failing.write_markers = true;
        assert!(fixture.run(&mut failing).is_err());
        assert_eq!(failing.calls.len(), index + 1);
        assert_eq!(word(&fixture.ram, INSTRUCTION_TIMER + 68), 4);
        for (prior_index, call) in failing.calls.iter().enumerate() {
            for prior in 0..prior_index {
                assert_eq!(call.before[0x1fff3 + prior], 0xa3 + prior as u8);
            }
        }
    }
}

#[cfg_attr(test, test)]
fn common_reloads_global_after_process_and_keeps_saved_id_after_coordinates() {
    struct MovingId(Vec<(Service, u16)>);
    impl Services for MovingId {
        fn available(&self, _: Service) -> bool {
            true
        }
        fn add_missiles_to_hud(&mut self, _: &mut [u8]) -> Result<(), String> {
            unreachable!()
        }
        fn play_room_music_after(&mut self, _: &mut [u8], _: u16) -> Result<(), String> {
            unreachable!()
        }
        fn queue_music_delayed8(&mut self, _: &mut [u8], _: u16) -> Result<(), String> {
            unreachable!()
        }
        fn process_plm_draw_instruction(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
            self.0.push((Service::ProcessPlmDrawInstruction, slot));
            set_word(ram, 0x1c27, 70);
            Ok(())
        }
        fn calculate_plm_block_coords(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
            self.0.push((Service::CalculatePlmBlockCoords, slot));
            set_word(ram, 0x1c27, 72);
            Ok(())
        }
        fn draw_plm(&mut self, _: &mut [u8], slot: u16) -> Result<(), String> {
            self.0.push((Service::DrawPlm, slot));
            Ok(())
        }
        fn add_super_missiles_to_hud(&mut self, _: &mut [u8]) -> Result<(), String> {
            unreachable!()
        }
        fn add_power_bombs_to_hud(&mut self, _: &mut [u8]) -> Result<(), String> {
            unreachable!()
        }
    }
    let (seed, rom) = inputs();
    let mut fixture = fixtures::morph_body_fixtures(seed, rom).unwrap().pop().unwrap();
    set_word(&mut fixture.ram, 0x1c27, 68);
    let mut services = MovingId(vec![]);
    assert_eq!(fixture.run(&mut services).unwrap(), Some(Output::Stop));
    assert_eq!(
        services.0,
        [(Service::ProcessPlmDrawInstruction, 68), (Service::CalculatePlmBlockCoords, 70), (Service::DrawPlm, 70)]
    );
}

#[cfg_attr(test, test)]
fn tank_body_matrix_preserves_full_wram_and_inventory_before_services() {
    let (seed, rom) = inputs();
    let cases = fixtures::tank_body_fixtures(seed, rom).unwrap();
    assert_eq!(cases.len(), 36);
    let mut covered = [false; 21];
    for (index, mut fixture) in cases.into_iter().enumerate() {
        assert_eq!(fixture.id, format!("T{:02}", index + 1));
        covered[fixture.raw_entry as usize] = true;
        let before = fixture.ram.clone();
        let mut expected = before.clone();
        for &write in &fixture.direct_writes {
            write.apply(&mut expected);
        }
        let mut services = AuditServices::new(true);
        let actual = fixture.run(&mut services).unwrap().unwrap();
        assert_eq!(fixture.expected, Expectation::Handled(actual), "{}", fixture.id);
        assert_eq!(fixture.ram, expected, "{} full WRAM", fixture.id);
        let calls: Vec<_> = services.calls.iter().map(|call| (call.service, call.argument)).collect();
        assert_eq!(calls, fixture.service_calls, "{} services", fixture.id);
        if matches!(
            fixture.entry,
            Some(
                Entry::CollectHealthEnergyTank
                    | Entry::CollectHealthReserveTank
                    | Entry::CollectAmmoSuperMissileTank
                    | Entry::CollectAmmoPowerBombTank
                    | Entry::CollectAmmoMissileTank
            )
        ) {
            for call in &services.calls {
                assert_eq!(
                    &call.before[0x9c0..0x9d8],
                    &expected[0x9c0..0x9d8],
                    "{} inventory before service",
                    fixture.id
                );
                assert_eq!(word(&call.before, 0xdc8), 0x7777, "{} queue after all services", fixture.id);
            }
            assert_eq!(&fixture.ram[0xd870..0xd8b0], &before[0xd870..0xd8b0], "collector does not own persistent bit");
        }
    }
    let roots: Vec<_> = covered.into_iter().enumerate().filter_map(|(index, seen)| seen.then_some(index)).collect();
    assert_eq!(roots, [0, 1, 3, 5, 6, 7, 8, 11, 12, 13, 14, 15, 17, 18, 19, 20]);
}

#[cfg_attr(test, test)]
fn tank_changed_domain_refusals_are_atomic() {
    let (seed, rom) = inputs();
    let cases = fixtures::tank_refusal_fixtures(seed, rom).unwrap();
    assert_eq!(cases.len(), 21);
    for (index, mut fixture) in cases.into_iter().enumerate() {
        assert_eq!(fixture.id, format!("TR{:02}", index + 1));
        let before = fixture.ram.clone();
        let mut services = AuditServices::new(fixture.services_available);
        let actual = fixture.run(&mut services);
        match fixture.expected {
            Expectation::Refused => assert_eq!(actual.unwrap(), None, "{}", fixture.id),
            Expectation::Error => assert!(actual.is_err(), "{}", fixture.id),
            _ => panic!("refusal expected handled"),
        }
        assert_eq!(fixture.ram, before, "{} writes on refusal", fixture.id);
        assert!(services.calls.is_empty(), "{} service on refusal", fixture.id);
    }
}

#[cfg_attr(test, test)]
fn tank_hud_capabilities_and_failure_order_are_explicit() {
    let (seed, rom) = inputs();
    let cases = fixtures::tank_body_fixtures(seed, rom).unwrap();
    for (id, hud) in [("T08", Service::AddSuperMissilesToHud), ("T11", Service::AddPowerBombsToHud)] {
        let source = cases.iter().find(|f| f.id == id).unwrap();
        for service in [hud, Service::PlayRoomMusicAfter] {
            let mut fixture = source.clone();
            let mut unavailable = AuditServices::new(true);
            unavailable.unavailable = Some(service);
            assert!(fixture.run(&mut unavailable).is_err());
            assert_eq!(fixture.ram, source.ram);
            assert!(unavailable.calls.is_empty());
            let mut failing = AuditServices::new(true);
            failing.fail_at = Some(service);
            failing.write_markers = true;
            assert!(fixture.run(&mut failing).is_err());
            assert_eq!(failing.calls.len(), if service == hud { 1 } else { 2 });
            assert_eq!(word(&fixture.ram, 0xdc8), 0x7777);
            if service == Service::PlayRoomMusicAfter {
                assert_eq!(failing.calls[1].before[0x1fff0 + hud as usize], 0xa0 + hud as u8);
            }
        }
    }
}

#[cfg(feature = "private-checks")]
pub(super) fn private_assertions() -> [(&'static str, fn()); 13] {
    [
        (
            "frozen_body_matrix_preserves_all_unlisted_bytes_and_exact_typed_returns",
            frozen_body_matrix_preserves_all_unlisted_bytes_and_exact_typed_returns as fn(),
        ),
        (
            "frozen_refusal_matrix_is_atomic_and_never_requests_native_service",
            frozen_refusal_matrix_is_atomic_and_never_requests_native_service as fn(),
        ),
        (
            "native_service_effects_flow_into_later_service_and_final_commit",
            native_service_effects_flow_into_later_service_and_final_commit as fn(),
        ),
        (
            "service_failure_stops_before_second_service_and_message_without_fallback",
            service_failure_stops_before_second_service_and_message_without_fallback as fn(),
        ),
        (
            "every_legal_shell_and_visible_preinstruction_position_is_supported",
            every_legal_shell_and_visible_preinstruction_position_is_supported as fn(),
        ),
        (
            "fixture_builders_reject_unsealed_seed_and_entry_mapping_is_exact",
            fixture_builders_reject_unsealed_seed_and_entry_mapping_is_exact as fn(),
        ),
        (
            "morph_body_delta_preserves_full_wram_and_orders_local_common_services",
            morph_body_delta_preserves_full_wram_and_orders_local_common_services as fn(),
        ),
        (
            "morph_refusal_delta_is_atomic_before_any_native_service",
            morph_refusal_delta_is_atomic_before_any_native_service as fn(),
        ),
        (
            "each_draw_capability_is_preflighted_and_failures_stop_without_fallback",
            each_draw_capability_is_preflighted_and_failures_stop_without_fallback as fn(),
        ),
        (
            "common_reloads_global_after_process_and_keeps_saved_id_after_coordinates",
            common_reloads_global_after_process_and_keeps_saved_id_after_coordinates as fn(),
        ),
        (
            "tank_body_matrix_preserves_full_wram_and_inventory_before_services",
            tank_body_matrix_preserves_full_wram_and_inventory_before_services as fn(),
        ),
        ("tank_changed_domain_refusals_are_atomic", tank_changed_domain_refusals_are_atomic as fn()),
        (
            "tank_hud_capabilities_and_failure_order_are_explicit",
            tank_hud_capabilities_and_failure_order_are_explicit as fn(),
        ),
    ]
}
