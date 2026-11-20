//! First-difference identity for one complete ordered frame hardware stream.

use super::frame::Datum;

/// Decode comparison bytes only; these quantities never become production inputs.
pub(super) fn difference(expected: &[u8], actual: &[u8]) -> Option<String> {
    let expected: Vec<Datum> = canonical::from_bytes(expected).ok()?;
    let actual: Vec<Datum> = canonical::from_bytes(actual).ok()?;
    for (ordinal, (expected, actual)) in expected.iter().zip(&actual).enumerate() {
        if expected.at != actual.at {
            return Some(format!("[{ordinal}].at"));
        }
        if expected.class != actual.class {
            return Some(format!("[{ordinal}].class"));
        }
        if expected.register != actual.register {
            return Some(format!("[{ordinal}].register"));
        }
        if expected.value != actual.value {
            return Some(format!("[{ordinal}].value"));
        }
    }
    (expected.len() != actual.len()).then(|| ".count".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::compare::{CompareFault, Observation, compare};
    use data::timing::MasterTicks;

    fn observed(events: &[Datum]) -> Observation {
        Observation {
            field: "frame.outputs".to_owned(),
            bytes: canonical::to_bytes(&events.to_vec()),
        }
    }

    fn stream() -> Vec<Datum> {
        vec![
            Datum {
                at: MasterTicks(7),
                class: 9,
                register: 0,
                value: 0,
            },
            Datum {
                at: MasterTicks(7),
                class: 10,
                register: 0,
                value: 0,
            },
        ]
    }

    #[test]
    fn ordered_stream_detects_tick_fault_order_and_unexpected_output() {
        let expected = [observed(&stream())];
        let mut events = stream();
        events.first_mut().unwrap().at = MasterTicks(8);
        assert_eq!(
            compare(&expected, &[observed(&events)]),
            Err(CompareFault::Difference("frame.outputs[0].at".to_owned()))
        );
        let mut events = stream();
        events.reverse();
        assert_eq!(
            compare(&expected, &[observed(&events)]),
            Err(CompareFault::Difference(
                "frame.outputs[0].class".to_owned()
            ))
        );
        let mut events = stream();
        events.push(Datum {
            at: MasterTicks(9),
            class: 7,
            register: 0,
            value: 0,
        });
        assert_eq!(
            compare(&expected, &[observed(&events)]),
            Err(CompareFault::Difference("frame.outputs.count".to_owned()))
        );
    }
}
