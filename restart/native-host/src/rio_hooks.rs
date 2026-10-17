//! Rio execution over the host's admitted synchronous native transport.
use sm_rio::{Entry, NativeService, Output, ServiceProtocol, Services, WRAM_SIZE, execute};

/// Native transport for one exclusive root invocation. The implementation must
/// bind the exact scratch allocation and invocation generation, exclude reentry,
/// and admit the complete terrain domain before returning `true`. Each service
/// publishes scratch to native WRAM, executes once, and copies complete native
/// WRAM back synchronously. Failure after native effects is terminal.
pub trait RioTransport {
    /// Bind without changing scratch or canonical WRAM and without gameplay
    /// effects, so an unchanged refusal can safely return to native dispatch.
    fn bind(&mut self, scratch: &mut [u8]) -> Result<(), String>;
    fn available(&self) -> bool;
    fn service(&mut self, service: NativeService, k: u16, amount: i32, scratch: &mut [u8]) -> Result<u16, String>;
    /// Called only after final WRAM/output validation. The native implementation
    /// must authenticate binding and generation again before publishing.
    fn commit(&mut self, scratch: &[u8]) -> Result<(), String>;
}

struct RioServices<'a, T> {
    transport: &'a mut T,
    protocol: ServiceProtocol,
    requested: bool,
    failed: bool,
}
impl<T: RioTransport> RioServices<'_, T> {
    fn request(&mut self, service: NativeService, k: u16, amount: i32, scratch: &mut [u8]) -> Result<u16, String> {
        self.requested = true;
        let result = (|| {
            if !self.transport.available() {
                return Err("Rio native admission unavailable during request".into());
            }
            self.protocol.begin(service, k, amount, scratch)?;
            let result = self.transport.service(service, k, amount, scratch)?;
            self.protocol.complete(result, scratch)?;
            Ok(result)
        })();
        if result.is_err() {
            self.failed = true;
            self.protocol.fail();
        }
        result
    }
}
impl<T: RioTransport> Services for RioServices<'_, T> {
    fn available(&self) -> bool {
        !self.failed && self.transport.available()
    }
    fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        self.request(NativeService::Horizontal, k, amount, ram).map(|result| result as u8)
    }
    fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        self.request(NativeService::Vertical, k, amount, ram).map(|result| result as u8)
    }
    fn angle(&mut self, ram: &mut [u8], k: u16) -> Result<u16, String> {
        self.request(NativeService::Angle, k, 0, ram)
    }
}

/// Execute one root using a complete canonical snapshot and immutable runtime
/// ROM. `None` permits unchanged native fallback; every error is terminal and
/// must prohibit fallback. Native effects may already have occurred on error.
/// The host supplies a process-lifetime-exclusive transport for this invocation.
pub fn execute_rio(
    entry: Entry,
    k: u16,
    arg: u32,
    canonical: &[u8],
    rom: &[u8],
    transport: &mut impl RioTransport,
) -> Result<Option<Output>, String> {
    if canonical.len() != WRAM_SIZE {
        return Err("Rio host requires complete canonical WRAM".into());
    }
    let Some(protocol) = ServiceProtocol::new(entry, k, arg, canonical)? else {
        return Ok(None);
    };
    let mut scratch = canonical.to_vec();
    transport.bind(&mut scratch)?;
    let mut services = RioServices { transport, protocol, requested: false, failed: false };
    let outcome = execute(entry, k, arg, &mut scratch, rom, &mut services)?;
    if services.failed {
        return Err("Rio host invocation poisoned by native failure".into());
    }
    match outcome {
        None => {
            if services.requested || scratch != canonical {
                return Err("Rio refused after service request or WRAM change".into());
            }
        }
        Some(output) => {
            services.protocol.validate_commit(&scratch, output, rom)?;
            services.transport.commit(&scratch)?;
        }
    }
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_rio::fixtures::{ScriptedServices, ServiceKind, body_fixtures};

    struct Transport {
        scripted: ScriptedServices,
        bound: usize,
        commits: Vec<Vec<u8>>,
        fail_service: bool,
        fail_commit: bool,
    }
    impl RioTransport for Transport {
        fn bind(&mut self, scratch: &mut [u8]) -> Result<(), String> {
            self.bound = scratch.as_ptr() as usize;
            Ok(())
        }
        fn available(&self) -> bool {
            self.scripted.admitted
        }
        fn service(&mut self, service: NativeService, k: u16, amount: i32, scratch: &mut [u8]) -> Result<u16, String> {
            assert_eq!(self.bound, scratch.as_ptr() as usize);
            let result = match service {
                NativeService::Horizontal => u16::from(self.scripted.move_horizontal(scratch, k, amount)?),
                NativeService::Vertical => u16::from(self.scripted.move_vertical(scratch, k, amount)?),
                NativeService::Angle => self.scripted.angle(scratch, k)?,
            };
            if self.fail_service {
                return Err("injected native failure after copyback".into());
            }
            Ok(result)
        }
        fn commit(&mut self, scratch: &[u8]) -> Result<(), String> {
            assert_eq!(self.bound, scratch.as_ptr() as usize);
            if self.fail_commit {
                return Err("injected commit failure".into());
            }
            self.commits.push(scratch.to_vec());
            Ok(())
        }
    }
    fn transport(scripted: ScriptedServices) -> Transport {
        Transport { scripted, bound: 0, commits: vec![], fail_service: false, fail_commit: false }
    }
    fn rom() -> Vec<u8> {
        let mut rom = vec![0; 0x300000];
        rom[0x113bbb..0x113bbd].copy_from_slice(&0x180u16.to_le_bytes());
        rom[0x113bbf..0x113bc1].copy_from_slice(&0x200u16.to_le_bytes());
        rom
    }
    #[test]
    fn full_actor_corpus_commits_exactly_once_with_native_copyback() {
        let rom = rom();
        for fixture in body_fixtures() {
            let mut expected = fixture.wram.clone();
            let mut scripted = ScriptedServices::new(fixture.steps.clone());
            let output = execute(fixture.entry, fixture.k, fixture.arg, &mut expected, &rom, &mut scripted).unwrap();
            let mut native = transport(ScriptedServices::new(fixture.steps));
            assert_eq!(
                execute_rio(fixture.entry, fixture.k, fixture.arg, &fixture.wram, &rom, &mut native).unwrap(),
                output
            );
            assert_eq!(native.commits, vec![expected], "{}", fixture.name);
            assert_eq!(native.scripted.calls, scripted.calls, "{}", fixture.name);
            assert!(native.scripted.complete());
        }
    }
    #[test]
    fn admission_refusal_has_no_service_or_commit_and_preserves_rng() {
        let f = body_fixtures().into_iter().find(|f| f.name == "dispatch-Main-2").unwrap();
        let mut native = transport(ScriptedServices::new(f.steps));
        native.scripted.admitted = false;
        assert_eq!(execute_rio(f.entry, f.k, f.arg, &f.wram, &rom(), &mut native).unwrap(), None);
        assert!(native.scripted.calls.is_empty());
        assert!(native.commits.is_empty());
    }
    #[test]
    fn native_failure_or_invalid_result_stops_before_later_service_and_commit() {
        for invalid in [false, true] {
            let mut f = body_fixtures().into_iter().find(|f| f.name == "state3-decelerate").unwrap();
            assert_eq!(f.steps[0].kind, ServiceKind::Horizontal);
            if invalid {
                f.steps[0].result = 2;
            }
            let mut native = transport(ScriptedServices::new(f.steps));
            native.fail_service = !invalid;
            assert!(execute_rio(f.entry, f.k, f.arg, &f.wram, &rom(), &mut native).is_err());
            assert_eq!(native.scripted.calls.len(), 1);
            assert!(native.commits.is_empty());
        }
    }
    #[test]
    fn commit_failure_is_terminal_without_repeating_native_requests() {
        let f = body_fixtures().into_iter().find(|f| f.name == "state3-decelerate").unwrap();
        let count = f.steps.len();
        let mut native = transport(ScriptedServices::new(f.steps));
        native.fail_commit = true;
        assert!(execute_rio(f.entry, f.k, f.arg, &f.wram, &rom(), &mut native).is_err());
        assert_eq!(native.scripted.calls.len(), count);
        assert!(native.commits.is_empty());
    }
}
