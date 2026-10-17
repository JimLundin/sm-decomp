//! Exact Rust-owned changes before a synchronous native service.
//!
//! The native adapter selects a variant from its admitted root and service
//! sequence, never from callback-controlled data. `before` is the canonical
//! snapshot immediately before this service, including prior service copyback.
use crate::{memory::*, sine::SINE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceWrites {
    /// No actor writes precede this service.
    Unchanged,
    /// Rio_Main advances RNG before its first movement or angle service.
    MainRandom,
    /// Rio_4 reverses D after a horizontal collision, before vertical movement.
    HorizontalBounce { k: u16 },
    /// Rio_5 installs the vector from the completed native angle request.
    PursuitVector { k: u16, angle: u16 },
}

/// Compare complete WRAM against the exact allowed writes for one admitted
/// boundary. This does not admit terrain, validate service order or authorize
/// commit. The adapter must establish those facts independently.
pub fn service_wram_matches(before: &[u8], candidate: &[u8], writes: ServiceWrites) -> Result<bool, String> {
    if before.len() != SIZE || candidate.len() != SIZE {
        return Err("Rio service validation requires complete canonical WRAM".into());
    }
    let mut expected = before.to_vec();
    match writes {
        ServiceWrites::Unchanged => (),
        ServiceWrites::MainRandom => random(&mut expected),
        ServiceWrites::HorizontalBounce { k } => {
            let base = slot(k)?;
            let value = word(before, base + D).wrapping_neg();
            put(&mut expected, base + D, value);
        }
        ServiceWrites::PursuitVector { k, angle } => {
            let base = slot(k)?;
            if angle > 255 {
                return Err("Rio service validation received an invalid angle".into());
            }
            put(&mut expected, base + D, SINE[usize::from(angle) + 64] as u16);
            put(&mut expected, base + C, SINE[usize::from(angle)] as u16);
        }
    }
    Ok(candidate == expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(before: &[u8], candidate: &[u8], writes: ServiceWrites) -> bool {
        service_wram_matches(before, candidate, writes).unwrap()
    }

    #[test]
    fn rng_boundary_rejects_geometry_terrain_and_unrelated_actor_writes() {
        let before = vec![0; SIZE];
        let mut candidate = before.clone();
        // Recovered byte-carry algorithm maps seed 0 to 0x0111.
        put(&mut candidate, 0x5e5, 0x0111);
        assert!(check(&before, &candidate, ServiceWrites::MainRandom));
        assert!(!check(&before, &candidate, ServiceWrites::Unchanged));
        for at in [ENEMIES + 2, ENEMIES + 10, 0x7a5, 0x10002, ENEMIES + 0x40 + D] {
            candidate[at] ^= 1;
            assert!(!check(&before, &candidate, ServiceWrites::MainRandom), "accepted edit at {at:#x}");
            candidate[at] ^= 1;
        }
    }

    #[test]
    fn bounce_uses_explicit_actor_and_latest_service_copyback() {
        let mut before = vec![0; SIZE];
        put(&mut before, CURRENT, 0x80);
        put(&mut before, ENEMIES + 0x40 + D, 0x1234);
        before[0x10002] = 7; // Terrain write already completed by native service.
        let mut candidate = before.clone();
        put(&mut candidate, ENEMIES + 0x40 + D, 0xedcc);
        let writes = ServiceWrites::HorizontalBounce { k: 0x40 };
        assert!(check(&before, &candidate, writes));
        assert!(!check(&before, &candidate, ServiceWrites::HorizontalBounce { k: 0x80 }));
        candidate[0x10002] = 0;
        assert!(!check(&before, &candidate, writes));
    }

    #[test]
    fn pursuit_rejects_wrong_vector_and_preserves_geometry_after_angle_copyback() {
        let mut before = vec![0; SIZE];
        before[ENEMIES + 0x40 + 2] = 0x55;
        let mut candidate = before.clone();
        // Literal recovered sine extrema for angle 0.
        put(&mut candidate, ENEMIES + 0x40 + C, (-256i16) as u16);
        put(&mut candidate, ENEMIES + 0x40 + D, 0);
        let writes = ServiceWrites::PursuitVector { k: 0x40, angle: 0 };
        assert!(check(&before, &candidate, writes));
        candidate[ENEMIES + 0x40 + D] ^= 1;
        assert!(!check(&before, &candidate, writes));
        candidate[ENEMIES + 0x40 + D] ^= 1;
        candidate[ENEMIES + 0x40 + 2] ^= 1;
        assert!(!check(&before, &candidate, writes));
        assert!(
            service_wram_matches(&before, &candidate, ServiceWrites::PursuitVector { k: 0x40, angle: 256 }).is_err()
        );
        assert!(service_wram_matches(&before, &candidate, ServiceWrites::HorizontalBounce { k: 1 }).is_err());
        assert!(service_wram_matches(&before[..SIZE - 1], &candidate, writes).is_err());
    }

    #[test]
    fn every_actor_fixture_matches_each_pre_service_write_boundary() {
        use crate::{Entry, Services, execute, fixtures::*};
        struct CheckingServices {
            canonical: Vec<u8>,
            inner: ScriptedServices,
            main: bool,
            state4: bool,
            last_angle: Option<u16>,
        }
        impl CheckingServices {
            fn request(&mut self, ram: &mut [u8], kind: ServiceKind, k: u16, amount: i32) -> Result<u16, String> {
                let ordinal = self.inner.calls.len();
                let writes = if ordinal == 0 && self.main {
                    ServiceWrites::MainRandom
                } else if kind == ServiceKind::Horizontal && self.last_angle.is_some() {
                    ServiceWrites::PursuitVector { k, angle: self.last_angle.take().unwrap() }
                } else if kind == ServiceKind::Vertical && self.state4 && self.inner.steps[0].result != 0 {
                    ServiceWrites::HorizontalBounce { k }
                } else {
                    ServiceWrites::Unchanged
                };
                if !service_wram_matches(&self.canonical, ram, writes)? {
                    return Err(format!("unexpected pre-service WRAM at request {ordinal}"));
                }
                let result = match kind {
                    ServiceKind::Horizontal => u16::from(self.inner.move_horizontal(ram, k, amount)?),
                    ServiceKind::Vertical => u16::from(self.inner.move_vertical(ram, k, amount)?),
                    ServiceKind::Angle => self.inner.angle(ram, k)?,
                };
                self.canonical.copy_from_slice(ram);
                if kind == ServiceKind::Angle {
                    self.last_angle = Some(result);
                }
                Ok(result)
            }
        }
        impl Services for CheckingServices {
            fn available(&self) -> bool {
                true
            }
            fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
                self.request(ram, ServiceKind::Horizontal, k, amount).map(|v| v as u8)
            }
            fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
                self.request(ram, ServiceKind::Vertical, k, amount).map(|v| v as u8)
            }
            fn angle(&mut self, ram: &mut [u8], k: u16) -> Result<u16, String> {
                self.request(ram, ServiceKind::Angle, k, 0)
            }
        }
        let mut rom = vec![0; 0x300000];
        put(&mut rom, 0x113bbb, 0x180);
        put(&mut rom, 0x113bbf, 0x200);
        let mut checked = 0;
        for mut fixture in body_fixtures() {
            let main = fixture.entry == Entry::Main;
            let state4 = fixture.entry == Entry::State4
                || (matches!(fixture.entry, Entry::Main | Entry::CallRioFunc)
                    && word(&fixture.wram, current(&fixture.wram).unwrap() + B) == STATES[3]);
            let mut services = CheckingServices {
                canonical: fixture.wram.clone(),
                inner: ScriptedServices::new(fixture.steps),
                main,
                state4,
                last_angle: None,
            };
            let result = execute(fixture.entry, fixture.k, fixture.arg, &mut fixture.wram, &rom, &mut services);
            assert!(result.is_ok_and(|v| v.is_some()), "{} rejected a legitimate service boundary", fixture.name);
            assert!(services.inner.complete(), "{} missed a service", fixture.name);
            checked += services.inner.calls.len();
        }
        assert!(checked > 0);
    }
}
