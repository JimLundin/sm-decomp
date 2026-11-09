//! The HUD's vectors (`scenarios/vectors/hud.toml`) run through the model, and the catalogue
//! checked against them.

#[cfg(test)]
mod support;

#[cfg(test)]
mod tests {
    use clean_model_hud::{BehaviourId, CATALOGUE, Checked};

    use crate::support::vectors::{Vector, load};

    /// The entry's name in the spec: its variant's.
    fn name(id: BehaviourId) -> String {
        format!("{id:?}")
    }

    /// The behaviour whose output an expected event is (spec: `Switching`, `SwitchClick`).
    fn output_entry(event: &str) -> Option<&'static str> {
        if event.starts_with("Switched:") {
            Some("Switching")
        } else if event == "Click" {
            Some("SwitchClick")
        } else {
            None
        }
    }

    /// The entries a vector exercises: those it lists, and the outputs whose events it expects.
    fn exercised(vector: &Vector) -> impl Iterator<Item = &str> {
        vector
            .entries
            .iter()
            .map(String::as_str)
            .chain(vector.events.iter().filter_map(|e| output_entry(e)))
    }

    #[test]
    fn every_vector_passes() {
        let vectors = load().unwrap();
        assert!(!vectors.is_empty(), "the vector file holds no vectors");
        let failures: Vec<String> = vectors
            .iter()
            .filter_map(|vector| match vector.run() {
                Ok(problems) if problems.is_empty() => None,
                Ok(problems) => Some(format!("{}: {}", vector.id, problems.join("; "))),
                Err(error) => Some(format!("{}: unreadable: {error}", vector.id)),
            })
            .collect();
        assert!(
            failures.is_empty(),
            "{} of {} vectors fail:\n{}",
            failures.len(),
            vectors.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn vector_ids_are_unique() {
        let vectors = load().unwrap();
        for (at, vector) in vectors.iter().enumerate() {
            let again = vectors.iter().skip(at).skip(1).any(|v| v.id == vector.id);
            assert!(!again, "vector {} appears twice", vector.id);
        }
    }

    #[test]
    fn every_listed_entry_is_in_the_catalogue() {
        for vector in load().unwrap() {
            for entry in &vector.entries {
                assert!(
                    CATALOGUE.iter().any(|b| name(b.id) == *entry),
                    "vector {} lists {entry}, which the catalogue lacks",
                    vector.id
                );
            }
        }
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let vectors = load().unwrap();
        let checked = CATALOGUE.iter().filter(|b| b.checked == Checked::ByVectors);
        for behaviour in checked {
            let name = name(behaviour.id);
            assert!(
                vectors.iter().any(|v| exercised(v).any(|e| e == name)),
                "no vector exercises {name}"
            );
        }
    }

    #[test]
    fn the_catalogue_has_each_entry_once() {
        for (at, behaviour) in CATALOGUE.iter().enumerate() {
            let again = CATALOGUE
                .iter()
                .skip(at)
                .skip(1)
                .any(|b| b.id == behaviour.id);
            assert!(!again, "{:?} appears twice", behaviour.id);
        }
    }
}
