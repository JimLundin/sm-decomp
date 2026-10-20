//! Explicit execution probes for focused differential replays.
//! Put `hit("routine-or-branch")` inside the behavior being validated. Merely
//! entering a room does not count. Unrequested probes are inert.
use std::{cell::RefCell, collections::BTreeMap};

#[derive(Default)]
struct Hits {
    calls: u64,
    first_frame: usize,
    last_frame: usize,
}

#[derive(Default)]
struct Probes {
    frame: usize,
    required: BTreeMap<String, Hits>,
}

thread_local! {
    static PROBES: RefCell<Probes> = RefCell::new(Probes::default());
}

pub fn configure(names: &[String]) -> Result<(), String> {
    let mut probes = Probes::default();
    for name in names {
        if name.is_empty() || name.chars().any(char::is_whitespace) {
            return Err("probe names must be nonempty and contain no whitespace".into());
        }
        if probes
            .required
            .insert(name.clone(), Hits::default())
            .is_some()
        {
            return Err(format!("repeated required probe: {name}"));
        }
    }
    PROBES.with(|slot| *slot.borrow_mut() = probes);
    Ok(())
}

/// The verifier supplies the absolute controller-input index before each frame.
pub fn begin_frame(index: usize) {
    PROBES.with(|slot| slot.borrow_mut().frame = index);
}

/// Call at function entry or inside the particular branch being changed.
pub fn hit(name: &str) {
    PROBES.with(|slot| {
        let mut probes = slot.borrow_mut();
        let frame = probes.frame;
        if let Some(hits) = probes.required.get_mut(name) {
            if hits.calls == 0 {
                hits.first_frame = frame;
            }
            hits.calls += 1;
            hits.last_frame = frame;
        }
    });
}

/// First input frame by which every required probe has executed. Repeated hits
/// do not move this boundary; an empty requirement list never satisfies it.
pub fn all_hit_frame() -> Option<usize> {
    PROBES.with(|slot| {
        let probes = slot.borrow();
        if probes.required.values().any(|hits| hits.calls == 0) {
            return None;
        }
        probes.required.values().map(|hits| hits.first_frame).max()
    })
}

pub fn report() -> Result<(), String> {
    PROBES.with(|slot| {
        let probes = slot.borrow();
        let mut missing = Vec::new();
        for (name, hits) in &probes.required {
            if hits.calls == 0 {
                missing.push(name.as_str());
                eprintln!("Probe {name}: 0 calls");
            } else {
                eprintln!(
                    "Probe {name}: {} calls, input frames {}..={}",
                    hits.calls, hits.first_frame, hits.last_frame
                );
            }
        }
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "required probes were not executed: {}",
                missing.join(", ")
            ))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_requires_every_probe_and_stays_at_first_complete_frame() {
        configure(&["later".into(), "earlier".into()]).unwrap();
        begin_frame(1200);
        hit("unrequested");
        assert_eq!(all_hit_frame(), None);
        hit("earlier");
        assert_eq!(all_hit_frame(), None);
        begin_frame(1207);
        hit("later");
        assert_eq!(all_hit_frame(), Some(1207));
        begin_frame(1300);
        hit("earlier");
        hit("later");
        assert_eq!(all_hit_frame(), Some(1207));
    }

    #[test]
    fn configuration_discards_setup_hits_and_empty_lists_cannot_complete() {
        configure(&["setup".into()]).unwrap();
        hit("setup");
        assert_eq!(all_hit_frame(), Some(0));
        configure(&["setup".into()]).unwrap();
        assert_eq!(all_hit_frame(), None);
        assert!(report().is_err());
        configure(&[]).unwrap();
        hit("setup");
        assert_eq!(all_hit_frame(), None);
    }
}
