//! Exact first-difference comparison over declared independent observations.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub field: String,
    pub bytes: Vec<u8>,
}

impl Observation {
    pub fn quantity(field: impl Into<String>, value: i64) -> Self {
        Self {
            field: field.into(),
            bytes: value.to_le_bytes().to_vec(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompareFault {
    EmptyExpected,
    DuplicateField(String),
    MissingActual(String),
    Difference(String),
}

pub fn compare(expected: &[Observation], actual: &[Observation]) -> Result<(), CompareFault> {
    if expected.is_empty() {
        return Err(CompareFault::EmptyExpected);
    }
    let mut seen = BTreeSet::new();
    let mut indexed = BTreeMap::new();
    let mut duplicates = BTreeSet::new();
    for observed in actual {
        if indexed.insert(observed.field.as_str(), observed).is_some() {
            duplicates.insert(observed.field.as_str());
        }
    }
    for observed in expected {
        if !seen.insert(&observed.field) {
            return Err(CompareFault::DuplicateField(observed.field.clone()));
        }
        if duplicates.contains(observed.field.as_str()) {
            return Err(CompareFault::DuplicateField(observed.field.clone()));
        }
        match indexed.get(observed.field.as_str()) {
            None => return Err(CompareFault::MissingActual(observed.field.clone())),
            Some(actual) if actual.bytes == observed.bytes => {}
            Some(actual) => {
                let suffix = if [
                    "movement.work",
                    "movement.collision_tests",
                    "movement.reached.complete",
                    "movement.inside",
                    "movement.skip",
                ]
                .contains(&observed.field.as_str())
                {
                    super::full_movement::difference(&observed.bytes, &actual.bytes)
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                return Err(CompareFault::Difference(format!(
                    "{}{suffix}",
                    observed.field
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CompareFault, Observation, compare};
    #[test]
    fn first_difference_and_absent_observation_fail_by_identity() {
        let expected = [
            Observation::quantity("position.x", 4),
            Observation::quantity("position.y", 8),
        ];
        let actual = [
            Observation::quantity("position.x", 5),
            Observation::quantity("position.y", 9),
        ];
        assert_eq!(
            compare(&expected, &actual),
            Err(CompareFault::Difference("position.x".to_owned()))
        );
        assert_eq!(
            compare(&expected, &[]),
            Err(CompareFault::MissingActual("position.x".to_owned()))
        );
        assert_eq!(compare(&[], &actual), Err(CompareFault::EmptyExpected));
    }
}
