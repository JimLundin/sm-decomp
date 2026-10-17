//! Native adapter service sequencing over independently supplied snapshots.
use crate::{Entry, Output, ServiceWrites, Services, execute, memory::*, service_wram_matches};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeService {
    Horizontal,
    Vertical,
    Angle,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Services, execute, fixtures::*};

    struct CheckedServices {
        protocol: ServiceProtocol,
        scripted: ScriptedServices,
    }
    impl Services for CheckedServices {
        fn available(&self) -> bool {
            true
        }
        fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
            self.protocol.begin(NativeService::Horizontal, k, amount, ram)?;
            let result = self.scripted.move_horizontal(ram, k, amount)?;
            self.protocol.complete(result.into(), ram)?;
            Ok(result)
        }
        fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
            self.protocol.begin(NativeService::Vertical, k, amount, ram)?;
            let result = self.scripted.move_vertical(ram, k, amount)?;
            self.protocol.complete(result.into(), ram)?;
            Ok(result)
        }
        fn angle(&mut self, ram: &mut [u8], k: u16) -> Result<u16, String> {
            self.protocol.begin(NativeService::Angle, k, 0, ram)?;
            let result = self.scripted.angle(ram, k)?;
            self.protocol.complete(result, ram)?;
            Ok(result)
        }
    }

    #[test]
    fn actor_corpus_completes_exact_native_protocol_with_copyback() {
        let mut rom = vec![0; 0x300000];
        put(&mut rom, 0x113bbb, 0x180);
        put(&mut rom, 0x113bbf, 0x200);
        for mut fixture in body_fixtures() {
            let protocol = ServiceProtocol::new(fixture.entry, fixture.k, fixture.arg, &fixture.wram).unwrap().unwrap();
            let mut services = CheckedServices { protocol, scripted: ScriptedServices::new(fixture.steps) };
            let result = execute(fixture.entry, fixture.k, fixture.arg, &mut fixture.wram, &rom, &mut services);
            assert!(result.as_ref().is_ok_and(|v| v.is_some()), "{} rejected", fixture.name);
            assert!(services.protocol.services_complete(), "{} missing native completion", fixture.name);
            assert!(services.scripted.complete(), "{} missing scripted request", fixture.name);
            services
                .protocol
                .validate_commit(&fixture.wram, result.unwrap().unwrap(), &rom)
                .unwrap_or_else(|e| panic!("{} commit: {e}", fixture.name));
        }
    }

    fn movement() -> BodyFixture {
        body_fixtures().into_iter().find(|f| f.name == "state3-decelerate").unwrap()
    }

    #[test]
    fn wrong_order_slot_amount_and_state_edits_poison_before_native_effects() {
        let f = movement();
        let amount = i32::from(word(&f.wram, ENEMIES + usize::from(f.k) + D) as i16) * 256;
        for fault in 0..4 {
            let mut protocol = ServiceProtocol::new(f.entry, f.k, f.arg, &f.wram).unwrap().unwrap();
            let mut candidate = f.wram.clone();
            if fault == 3 {
                candidate[0x10000] ^= 1;
            }
            assert!(
                protocol
                    .begin(
                        if fault == 0 { NativeService::Vertical } else { NativeService::Horizontal },
                        if fault == 1 { f.k + 0x40 } else { f.k },
                        if fault == 2 { amount + 1 } else { amount },
                        &candidate,
                    )
                    .is_err()
            );
            assert!(!protocol.services_complete());
            assert!(protocol.begin(NativeService::Horizontal, f.k, amount, &f.wram).is_err());
        }
    }

    #[test]
    fn horizontal_collision_short_circuit_and_duplicate_completion_are_enforced() {
        let f = movement();
        let amount = i32::from(word(&f.wram, ENEMIES + usize::from(f.k) + D) as i16) * 256;
        let mut protocol = ServiceProtocol::new(f.entry, f.k, f.arg, &f.wram).unwrap().unwrap();
        protocol.begin(NativeService::Horizontal, f.k, amount, &f.wram).unwrap();
        assert!(!protocol.services_complete());
        protocol.complete(1, &f.wram).unwrap();
        assert!(protocol.services_complete());
        assert!(protocol.begin(NativeService::Vertical, f.k, 0, &f.wram).is_err());
        assert!(!protocol.services_complete());
        assert!(protocol.complete(0, &f.wram).is_err());
    }

    #[test]
    fn invalid_result_or_missing_copyback_never_completes() {
        let f = movement();
        let amount = i32::from(word(&f.wram, ENEMIES + usize::from(f.k) + D) as i16) * 256;
        for short in [true, false] {
            let mut protocol = ServiceProtocol::new(f.entry, f.k, f.arg, &f.wram).unwrap().unwrap();
            protocol.begin(NativeService::Horizontal, f.k, amount, &f.wram).unwrap();
            assert!(
                protocol
                    .complete(if short { 0 } else { 2 }, if short { &f.wram[..SIZE - 1] } else { &f.wram })
                    .is_err()
            );
            assert!(!protocol.services_complete());
        }
    }
    #[test]
    fn final_geometry_edits_wrong_output_and_duplicate_commit_are_rejected() {
        let f = body_fixtures().into_iter().find(|f| f.entry == Entry::Init).unwrap();
        let mut ram = f.wram.clone();
        let mut scripted = ScriptedServices::new(vec![]);
        let output = execute(f.entry, f.k, f.arg, &mut ram, &[], &mut scripted).unwrap().unwrap();
        for fault in 0..3 {
            let mut protocol = ServiceProtocol::new(f.entry, f.k, f.arg, &f.wram).unwrap().unwrap();
            let mut candidate = ram.clone();
            if fault == 0 {
                candidate[ENEMIES + usize::from(f.k) + 2] ^= 1;
            }
            if fault == 1 {
                candidate[0x10000] ^= 1;
            }
            let result = if fault == 2 { Output::Continue(0x8000) } else { output };
            assert!(protocol.validate_commit(&candidate, result, &[]).is_err());
            assert!(protocol.validate_commit(&ram, output, &[]).is_err());
        }
        let mut protocol = ServiceProtocol::new(f.entry, f.k, f.arg, &f.wram).unwrap().unwrap();
        protocol.validate_commit(&ram, output, &[]).unwrap();
        assert!(protocol.validate_commit(&ram, output, &[]).is_err());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Horizontal,
    Vertical,
    Angle,
    PursuitHorizontal(u16),
    Done,
    Pending(NativeService),
    Failed,
    Committed,
}

struct Receipt {
    service: NativeService,
    k: u16,
    amount: i32,
    result: u16,
    copyback: Vec<u8>,
}

/// One admitted invocation. Native terrain admission, allocation/generation
/// binding and commit validation remain the host's responsibility. Keep this
/// object private to the host registration; callback code must not control it.
pub struct ServiceProtocol {
    initial: Vec<u8>,
    entry: Entry,
    input_k: u16,
    arg: u32,
    receipts: Vec<Receipt>,
    pending_amount: i32,
    canonical: Vec<u8>,
    phase: Phase,
    k: u16,
    state: u16,
    main_random: bool,
    bounce: bool,
}

impl ServiceProtocol {
    /// Construct from the native root's incoming canonical snapshot. Unsupported
    /// dynamic dispatch refuses without changing state. This never runs terrain.
    pub fn new(entry: Entry, k: u16, arg: u32, ram: &[u8]) -> Result<Option<Self>, String> {
        if ram.len() != SIZE {
            return Err("Rio protocol requires complete WRAM".into());
        }
        let current_base = current(ram)?;
        let input_k = k;
        let (state, k) = match entry {
            Entry::Main => (word(ram, current_base + B), word(ram, CURRENT)),
            Entry::CallRioFunc => {
                if arg & 0xffff0000 != 0xa20000 {
                    return Ok(None);
                }
                (arg as u16, word(ram, CURRENT))
            }
            Entry::State1 => (STATES[0], k),
            Entry::State2 => (STATES[1], k),
            Entry::State3 => (STATES[2], k),
            Entry::State4 => (STATES[3], k),
            Entry::State5 => (STATES[4], k),
            Entry::Instruction => {
                slot(k)?;
                if !(0x8000..=0xfffe).contains(&arg) {
                    return Err("invalid Rio instruction continuation".into());
                }
                (0, word(ram, CURRENT))
            }
            Entry::SelectAnimation => {
                u16::try_from(arg).map_err(|_| "invalid Rio animation argument")?;
                (0, word(ram, CURRENT))
            }
            Entry::Init => (0, word(ram, CURRENT)),
        };
        if matches!(entry, Entry::Main | Entry::CallRioFunc) && !STATES.contains(&state) {
            return Ok(None);
        }
        let base = slot(k)?;
        let phase = match state {
            0xbc48 | 0xbcb7 => Phase::Horizontal,
            0xbcff if (word(ram, base + 6).wrapping_sub(word(ram, 0xafa)) as i16) < 0 => Phase::Angle,
            _ => Phase::Done,
        };
        Ok(Some(Self {
            initial: ram.to_vec(),
            entry,
            input_k,
            arg,
            receipts: Vec::new(),
            pending_amount: 0,
            canonical: ram.to_vec(),
            phase,
            k,
            state,
            main_random: entry == Entry::Main,
            bounce: false,
        }))
    }

    /// Validate before any native effect. Any rejected request permanently
    /// poisons this invocation, including an attempted request after completion.
    pub fn begin(&mut self, service: NativeService, k: u16, amount: i32, candidate: &[u8]) -> Result<(), String> {
        let accepted = self.validate(service, k, amount, candidate);
        match accepted {
            Ok(true) => {
                self.phase = Phase::Pending(service);
                self.pending_amount = amount;
                self.main_random = false;
                self.bounce = false;
                Ok(())
            }
            result => {
                self.phase = Phase::Failed;
                Err(result.err().unwrap_or_else(|| "Rio native request violates admitted protocol".into()))
            }
        }
    }

    fn validate(&self, service: NativeService, k: u16, amount: i32, candidate: &[u8]) -> Result<bool, String> {
        let expected_service = match self.phase {
            Phase::Horizontal | Phase::PursuitHorizontal(_) => NativeService::Horizontal,
            Phase::Vertical => NativeService::Vertical,
            Phase::Angle => NativeService::Angle,
            _ => return Ok(false),
        };
        if service != expected_service || k != self.k {
            return Ok(false);
        }
        let writes = if self.main_random {
            ServiceWrites::MainRandom
        } else if let Phase::PursuitHorizontal(angle) = self.phase {
            ServiceWrites::PursuitVector { k: self.k, angle }
        } else if self.bounce {
            ServiceWrites::HorizontalBounce { k: self.k }
        } else {
            ServiceWrites::Unchanged
        };
        if !service_wram_matches(&self.canonical, candidate, writes)? {
            return Ok(false);
        }
        let base = slot(self.k)?;
        let expected_amount = match service {
            NativeService::Angle => 0,
            NativeService::Horizontal => i32::from(word(candidate, base + D) as i16) * 256,
            NativeService::Vertical => i32::from(word(candidate, base + C) as i16) * 256,
        };
        Ok(amount == expected_amount)
    }

    /// Record a successful native service and its full synchronous copyback.
    /// The host must call this with the actual native result, never callback
    /// data. Native failure must instead call `fail` and terminate ownership.
    pub fn complete(&mut self, result: u16, copyback: &[u8]) -> Result<(), String> {
        let Phase::Pending(service) = self.phase else {
            self.fail();
            return Err("Rio service completion without issued request".into());
        };
        if copyback.len() != SIZE || result > if service == NativeService::Angle { 255 } else { 1 } {
            self.fail();
            return Err("invalid Rio native service completion".into());
        }
        self.canonical.copy_from_slice(copyback);
        self.receipts.push(Receipt {
            service,
            k: self.k,
            amount: self.pending_amount,
            result,
            copyback: copyback.to_vec(),
        });
        self.phase = match service {
            NativeService::Angle => Phase::PursuitHorizontal(result),
            NativeService::Horizontal if self.state == STATES[2] && result != 0 => Phase::Done,
            NativeService::Horizontal => {
                self.bounce = self.state == STATES[3] && result != 0;
                Phase::Vertical
            }
            NativeService::Vertical => Phase::Done,
        };
        Ok(())
    }

    pub fn fail(&mut self) {
        self.phase = Phase::Failed;
    }

    /// Establishes only that required foreign services completed. Final actor
    /// writes and output must still pass the host's commit gate.
    pub fn services_complete(&self) -> bool {
        self.phase == Phase::Done
    }

    /// Validate final actor writes and output using retained native receipts.
    /// No native operation is repeated. The host supplies its immutable runtime
    /// ROM and callback result, then commits only after this succeeds. Rejection
    /// poisons the invocation; a successful commit cannot be validated twice.
    /// This checks conformance to the linked Rust actor, not independent C parity.
    pub fn validate_commit(&mut self, candidate: &[u8], output: Output, rom: &[u8]) -> Result<(), String> {
        let result = self.check_commit(candidate, output, rom);
        self.phase = if result.is_ok() { Phase::Committed } else { Phase::Failed };
        result
    }

    fn check_commit(&self, candidate: &[u8], output: Output, rom: &[u8]) -> Result<(), String> {
        if !self.services_complete() {
            return Err("Rio commit before complete services or after terminal state".into());
        }
        struct Replay<'a> {
            receipts: &'a [Receipt],
            cursor: usize,
        }
        impl Replay<'_> {
            fn request(&mut self, service: NativeService, k: u16, amount: i32, ram: &mut [u8]) -> Result<u16, String> {
                let receipt =
                    self.receipts.get(self.cursor).ok_or("Rio commit replay requested an unrecorded service")?;
                if receipt.service != service || receipt.k != k || receipt.amount != amount {
                    return Err("Rio commit replay request differs from native receipt".into());
                }
                ram.copy_from_slice(&receipt.copyback);
                self.cursor += 1;
                Ok(receipt.result)
            }
        }
        impl Services for Replay<'_> {
            fn available(&self) -> bool {
                true
            }
            fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
                self.request(NativeService::Horizontal, k, amount, ram).map(|v| v as u8)
            }
            fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
                self.request(NativeService::Vertical, k, amount, ram).map(|v| v as u8)
            }
            fn angle(&mut self, ram: &mut [u8], k: u16) -> Result<u16, String> {
                self.request(NativeService::Angle, k, 0, ram)
            }
        }
        let mut expected = self.initial.clone();
        let mut replay = Replay { receipts: &self.receipts, cursor: 0 };
        let expected_output = execute(self.entry, self.input_k, self.arg, &mut expected, rom, &mut replay)?;
        if expected_output != Some(output) || replay.cursor != self.receipts.len() || candidate != expected {
            return Err("Rio final WRAM or output differs from owned execution".into());
        }
        Ok(())
    }
}
