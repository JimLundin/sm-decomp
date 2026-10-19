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
