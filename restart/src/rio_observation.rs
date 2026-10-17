//! Bounded runtime observations; no gameplay acceptance authority.
//! Call once after every host_step, before Game's borrowed effects are cleared.
//! Counters are checked deltas since the previous call (first call needs a
//! separately captured pre-step baseline). Retain complete WRAM and effects in
//! the caller's authenticated trace; these extracted facts cannot replace them.
#![forbid(unsafe_code)]

pub const WRAM_SIZE: usize = 0x20000;
pub const SLOT: u16 = 0xc0;
const BASE: usize = 0xf78 + SLOT as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effect {
    pub kind: u32,
    pub a: u32,
    pub b: u32,
    pub c: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Actor {
    pub definition: u16,
    pub bank: u8,
    pub x: u16,
    pub x_fraction: u16,
    pub y: u16,
    pub y_fraction: u16,
    pub instruction: u16,
    pub state: u16,
    pub saved_horizontal_velocity: i16,
    pub vertical_velocity: i16,
    pub horizontal_velocity: i16,
    pub animation_event: u16,
    pub selected_animation: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub step: u64,
    pub room: u16,
    pub area: u16,
    pub game_state: u16,
    pub health: u16,
    /// End-of-step current actor only; never callback-time attribution.
    pub current_slot_at_sample: u16,
    pub actor: Actor,
    pub expected_identity: bool,
    /// Signed wrapped Samus-minus-Rio horizontal distance, as in recovered C.
    pub samus_horizontal_distance: i16,
    pub samus_at_or_above: bool,
    pub position_changed_between_samples: bool,
    pub sampled_state_change: Option<(u16, u16)>,
    /// BC32/E!=0 -> BBED/E==0 in adjacent valid samples. A correlation,
    /// not proof of instruction origin, callback-time actor or intermediate path.
    pub sampled_event_consumption: bool,
    pub waiting_with_positive_health: bool,
    pub completed_roots: [u64; 10],
    /// These root totals cover ALL actors, including Init/Main with no slot.
    pub counter_delta: [u64; 67],
    /// Complete ordered effects, including foreign consumers; owned copy.
    pub effects: Vec<Effect>,
    pub services_for_expected_slot: [u64; 3],
}

#[derive(Default)]
pub struct Observer {
    previous: Option<Sample>,
}

struct Sample {
    step: u64,
    expected_identity: bool,
    actor: Actor,
}

impl Observer {
    pub fn observe(
        &mut self,
        step: u64,
        ram: &[u8],
        effects: &[u8],
        counter_delta: [u64; 67],
    ) -> Result<Observation, String> {
        if ram.len() != WRAM_SIZE {
            return Err("Rio observation requires complete 128 KiB WRAM".into());
        }
        if self.previous.as_ref().is_some_and(|p| p.step.checked_add(1) != Some(step)) {
            return Err("Rio observation missed or repeated a host step".into());
        }
        let effects = decode(effects)?;
        let (completed_roots, services_for_expected_slot) = validate(&effects, &counter_delta)?;
        let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
        let actor = Actor {
            definition: word(BASE),
            bank: ram[BASE + 0x2e],
            x: word(BASE + 2),
            x_fraction: word(BASE + 4),
            y: word(BASE + 6),
            y_fraction: word(BASE + 8),
            instruction: word(BASE + 0x1a),
            state: word(BASE + 0x32),
            saved_horizontal_velocity: word(BASE + 0x30) as i16,
            vertical_velocity: word(BASE + 0x34) as i16,
            horizontal_velocity: word(BASE + 0x36) as i16,
            animation_event: word(BASE + 0x38),
            selected_animation: word(BASE + 0x3a),
        };
        let room = word(0x79b);
        let area = word(0x79f);
        let game_state = word(0x998);
        let health = word(0x9c2);
        let expected_identity =
            room == 0x98e2 && area == 0 && game_state == 8 && actor.definition == 0xd27f && actor.bank == 0xa2;
        // A room/definition loss severs continuity; later slot reuse starts fresh.
        // Same-definition despawn/reload entirely within one host step is invisible.
        let previous = self.previous.as_ref().filter(|p| p.expected_identity && expected_identity);
        let observation = Observation {
            step,
            room,
            area,
            game_state,
            health,
            current_slot_at_sample: word(0xe54),
            expected_identity,
            samus_horizontal_distance: word(0xaf6).wrapping_sub(actor.x) as i16,
            samus_at_or_above: (actor.y.wrapping_sub(word(0xafa)) as i16) >= 0,
            position_changed_between_samples: previous.is_some_and(|p| {
                (p.actor.x, p.actor.x_fraction, p.actor.y, p.actor.y_fraction)
                    != (actor.x, actor.x_fraction, actor.y, actor.y_fraction)
            }),
            sampled_state_change: previous
                .and_then(|p| (p.actor.state != actor.state).then_some((p.actor.state, actor.state))),
            sampled_event_consumption: previous.is_some_and(|p| {
                p.actor.state == 0xbc32
                    && p.actor.animation_event != 0
                    && actor.state == 0xbbed
                    && actor.animation_event == 0
            }),
            waiting_with_positive_health: expected_identity && health > 0 && actor.state == 0xbbed,
            actor,
            completed_roots,
            counter_delta,
            effects,
            services_for_expected_slot,
        };
        // Retain only a sample for continuity, not the potentially large trace.
        self.previous = Some(Sample { step, expected_identity, actor });
        Ok(observation)
    }
}

fn decode(bytes: &[u8]) -> Result<Vec<Effect>, String> {
    if !bytes.len().is_multiple_of(16) {
        return Err("native effects end in a truncated 16-byte record".into());
    }
    Ok(bytes
        .as_chunks::<16>()
        .0
        .iter()
        .map(|r| {
            let word = |at| u32::from_le_bytes([r[at], r[at + 1], r[at + 2], r[at + 3]]);
            Effect { kind: word(0), a: word(4), b: word(8), c: word(12) }
        })
        .collect())
}

fn slot(k: u32) -> bool {
    k < 0x800 && k.is_multiple_of(64)
}

fn validate(effects: &[Effect], counts: &[u64; 67]) -> Result<([u64; 10], [u64; 3]), String> {
    let mut root: Option<Effect> = None;
    let mut service: Option<Effect> = None;
    let mut completed = [0u64; 10];
    let mut services = [0u64; 3];
    let mut expected_slot_services = [0u64; 3];
    for e in effects {
        match e.kind {
            68 => {
                if root.is_some()
                    || e.a >= 10
                    || !slot(e.b)
                    || (e.a <= 1 && e.b != 0)
                    || (e.a == 2 && !(0x8000..=0xfffe).contains(&e.c))
                    || (e.a == 8 && e.c > 0xffff)
                {
                    return Err("invalid or nested Rio root effect".into());
                }
                root = Some(*e);
            }
            69 => {
                let r = root.ok_or("Rio completion without root")?;
                if service.is_some() || e.a != r.a || e.b != r.b || e.c != if r.a == 2 { r.c } else { 0 } {
                    return Err("Rio root completion mismatch".into());
                }
                completed[e.a as usize] += 1;
                root = None;
            }
            70 => {
                let r = root.ok_or("Rio service outside root")?;
                if service.is_some()
                    || e.a >> 16 != r.a
                    || !slot(e.a & 0xffff)
                    || e.b >= 3
                    || (e.b == 2 && e.c != 0)
                    || ((3..=7).contains(&r.a) && e.a & 0xffff != r.b)
                {
                    return Err("invalid Rio service request".into());
                }
                service = Some(*e);
            }
            71 => {
                let s = service.ok_or("Rio service completion without request")?;
                if e.a != s.a || e.b != s.b || e.c > if s.b == 2 { 255 } else { 1 } {
                    return Err("Rio service completion mismatch".into());
                }
                services[e.b as usize] += 1;
                if e.a & 0xffff == u32::from(SLOT) {
                    expected_slot_services[e.b as usize] += 1;
                }
                service = None;
            }
            _ => {} // Foreign ordered effects remain in the returned trace.
        }
    }
    if root.is_some() || service.is_some() {
        return Err("unfinished Rio transaction at host-step boundary".into());
    }
    for i in 0..10 {
        let add = |a: u64, b: u64| a.checked_add(b).ok_or("Rio counter delta overflow");
        if counts[i] != completed[i]
            || counts[i] != add(counts[10 + i], counts[40 + i])?
            || counts[20 + i] != add(counts[30 + i], counts[40 + i])?
            || counts[30 + i] > counts[10 + i]
            || counts[20 + i] > counts[i]
            || counts[50 + i] > counts[i]
        {
            return Err(format!("Rio root {i} counter delta disagrees with effects/ownership"));
        }
    }
    for i in 0..3 {
        if counts[60 + i] != services[i] || counts[63 + i] != services[i] {
            return Err(format!("Rio service {i} counter delta disagrees with ordered effects"));
        }
    }
    if counts[66] != 0 {
        return Err("Rio invalid/poisoned native request observed".into());
    }
    Ok((completed, expected_slot_services))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn put(r: &mut [u8], at: usize, n: u16) {
        r[at..at + 2].copy_from_slice(&n.to_le_bytes());
    }
    fn ram(state: u16, event: u16) -> Vec<u8> {
        let mut r = vec![0; WRAM_SIZE];
        for (at, n) in [
            (0x79b, 0x98e2),
            (0x998, 8),
            (0x9c2, 99),
            (BASE, 0xd27f),
            (BASE + 2, 331),
            (BASE + 6, 69),
            (BASE + 0x32, state),
            (BASE + 0x38, event),
        ] {
            put(&mut r, at, n);
        }
        r[BASE + 0x2e] = 0xa2;
        r
    }
    fn encode(e: &[[u32; 4]]) -> Vec<u8> {
        e.iter().flatten().flat_map(|n| n.to_le_bytes()).collect()
    }
    fn main_counts() -> [u64; 67] {
        let mut c = [0; 67];
        c[1] = 1;
        c[21] = 1;
        c[41] = 1;
        c
    }

    #[test]
    fn signed_fields_fractional_motion_and_sampled_recovery() {
        let mut o = Observer::default();
        let mut r = ram(0xbc32, 1);
        put(&mut r, BASE + 0x36, 0xfe80);
        put(&mut r, BASE + 0x34, 0xffff);
        put(&mut r, 0xaf6, 330);
        let a = o.observe(10, &r, &[], [0; 67]).unwrap();
        assert_eq!(
            (a.actor.horizontal_velocity, a.actor.vertical_velocity, a.samus_horizontal_distance),
            (-384, -1, -1)
        );
        put(&mut r, BASE + 4, 1);
        put(&mut r, BASE + 0x32, 0xbbed);
        put(&mut r, BASE + 0x38, 0);
        let b = o.observe(11, &r, &[], [0; 67]).unwrap();
        assert!(b.position_changed_between_samples && b.sampled_event_consumption && b.waiting_with_positive_health);
        assert_eq!(b.sampled_state_change, Some((0xbc32, 0xbbed)));
        assert_eq!(b.completed_roots, [0; 10]); // No instruction execution is invented.
    }
    #[test]
    fn transient_event_is_not_invented() {
        let mut o = Observer::default();
        o.observe(1, &ram(0xbc32, 0), &[], [0; 67]).unwrap();
        let b = o.observe(2, &ram(0xbbed, 0), &[], [0; 67]).unwrap();
        assert!(!b.sampled_event_consumption);
    }
    #[test]
    fn identity_loss_severs_continuity_and_does_not_accept_dead_player() {
        let mut o = Observer::default();
        o.observe(1, &ram(0xbc32, 1), &[], [0; 67]).unwrap();
        let mut r = ram(0xbc32, 1);
        put(&mut r, BASE, 0xd0ff);
        assert!(!o.observe(2, &r, &[], [0; 67]).unwrap().expected_identity);
        r = ram(0xbbed, 0);
        put(&mut r, 0x9c2, 0);
        let b = o.observe(3, &r, &[], [0; 67]).unwrap();
        assert!(!b.sampled_event_consumption && !b.waiting_with_positive_health);
        r[BASE + 0x2e] = 0xa3;
        assert!(!o.observe(4, &r, &[], [0; 67]).unwrap().expected_identity);
    }
    #[test]
    fn ordered_foreign_effects_and_services_are_owned_copies() {
        let e =
            encode(&[[68, 1, 0, 0], [70, 0x100c0, 0, 0xffff0000], [7, 1, 2, 3], [71, 0x100c0, 0, 1], [69, 1, 0, 0]]);
        let mut c = main_counts();
        c[60] = 1;
        c[63] = 1;
        let b = Observer::default().observe(1, &ram(0xbc48, 0), &e, c).unwrap();
        assert_eq!(b.services_for_expected_slot, [1, 0, 0]);
        assert_eq!(b.effects[2].kind, 7);
        assert_eq!(b.effects[1].c as i32, -65536);
        assert_eq!(b.completed_roots[1], 1); // Global Main total, not actor attribution.
    }
    #[test]
    fn foreign_slot_main_services_do_not_count_as_target_services() {
        let e = encode(&[[68, 1, 0, 0], [70, 0x10040, 1, 0], [71, 0x10040, 1, 0], [69, 1, 0, 0]]);
        let mut c = main_counts();
        c[61] = 1;
        c[64] = 1;
        let b = Observer::default().observe(1, &ram(0xbc48, 0), &e, c).unwrap();
        assert_eq!(b.services_for_expected_slot, [0; 3]);
    }
    #[test]
    fn refusal_remains_visible_and_counter_mismatch_rejected() {
        let e = encode(&[[68, 1, 0, 0], [69, 1, 0, 0]]);
        let mut c = main_counts();
        c[41] = 0;
        c[11] = 1;
        c[31] = 1;
        c[51] = 1;
        let b = Observer::default().observe(1, &ram(0xbbed, 0), &e, c).unwrap();
        assert_eq!((b.counter_delta[11], b.counter_delta[31], b.counter_delta[51]), (1, 1, 1));
        c[41] = 1;
        assert!(Observer::default().observe(1, &ram(0xbbed, 0), &e, c).is_err());
    }
    #[test]
    fn failed_observation_does_not_advance_sample() {
        let mut o = Observer::default();
        let r = ram(0xbc32, 1);
        o.observe(1, &r, &[], [0; 67]).unwrap();
        assert!(o.observe(2, &r[..100], &[], [0; 67]).is_err());
        assert!(o.observe(2, &r, &[0], [0; 67]).is_err());
        assert!(o.observe(3, &r, &[], [0; 67]).is_err());
        assert!(o.observe(1, &r, &[], [0; 67]).is_err());
        assert!(o.observe(2, &r, &[], [0; 67]).is_ok());
    }
    #[test]
    fn malformed_or_unfinished_transactions_fail() {
        for e in [
            vec![[68, 1, 0, 0]],
            vec![[69, 1, 0, 0]],
            vec![[68, 10, 0, 0]],
            vec![[68, 1, 0, 0], [70, 0x100c0, 3, 0]],
            vec![[68, 1, 0, 0], [70, 0x100c0, 0, 0], [71, 0x100c0, 0, 2]],
            vec![[68, 1, 0, 0], [69, 1, 0, 1]],
            vec![[68, 5, 0xc0, 0], [70, 0x50040, 0, 0]],
            vec![[68, 1, 0, 0], [68, 1, 0, 0]],
            vec![[68, 2, 0xc0, 0x9000], [69, 2, 0xc0, 0x9002]],
        ] {
            assert!(Observer::default().observe(1, &ram(0xbc48, 0), &encode(&e), [0; 67]).is_err());
        }
        let mut c = [0; 67];
        c[66] = 1;
        assert!(Observer::default().observe(1, &ram(0xbc48, 0), &[], c).is_err());
    }
}
