//! The catalogue matches the spec one for one, and the stages are numbered as the spec numbers
//! them.

#[cfg(test)]
mod catalogue {
    use enum_map::{Enum, EnumMap};
    use samus_appearance::{BehaviourId, CATALOGUE};
    use types::{BoostStage, Checked, Kind, Switch};

    const SPEC: &str = include_str!("../../../../docs/behaviours/samus-appearance.md");

    /// An entry as the spec writes it: its id, and its kind, switch, summary and checked lines.
    #[derive(Default)]
    struct Entry {
        id: String,
        lines: Vec<(String, String)>,
    }

    impl Entry {
        /// The text of the field `name` ("kind", "summary", ...), its continued lines joined, with
        /// its closing full stop and the backticks around a step's name left out.
        fn field(&self, name: &str) -> String {
            let text = self
                .lines
                .iter()
                .find(|(field, _)| field == name)
                .map_or_else(|| panic!("{} has no {name}", self.id), |(_, text)| text);
            text.trim().trim_end_matches('.').replace('`', "")
        }
    }

    /// Every entry of the spec, in its order: each `### Id` heading with a `- kind:` line under it.
    fn spec_entries() -> Vec<Entry> {
        let mut entries: Vec<Entry> = Vec::new();
        for line in SPEC.lines() {
            if let Some(id) = line.strip_prefix("### ") {
                entries.push(Entry {
                    id: id.to_owned(),
                    ..Entry::default()
                });
                continue;
            }
            let Some(entry) = entries.last_mut() else {
                continue;
            };
            // Every bullet starts a line of its own; a rule's bullet names no field.
            if let Some(bullet) = line.strip_prefix("- ") {
                let (field, text) = bullet.split_once(": ").unwrap_or(("", bullet));
                entry.lines.push((field.to_owned(), text.to_owned()));
            } else if let (Some(more), Some((_, text))) =
                (line.strip_prefix("  "), entry.lines.last_mut())
            {
                text.push(' ');
                text.push_str(more.trim());
            }
        }
        entries.retain(|entry| entry.lines.iter().any(|(field, _)| field == "kind"));
        entries
    }

    fn kind(written: &str) -> Kind {
        match written {
            "mechanic" => Kind::Mechanic,
            "quirk" => Kind::Quirk,
            "timing" => Kind::Timing,
            "encoding" => Kind::Encoding,
            "output" => Kind::Output,
            other => panic!("unwritten kind {other:?}"),
        }
    }

    /// The switch line's first word: `none`, `toggle` or `parameter`, then what it names.
    fn switch(written: &str) -> Switch {
        match written.split_whitespace().next() {
            Some("none") => Switch::Fixed,
            Some("toggle") => Switch::Toggle,
            Some("parameter") => Switch::Parameter,
            other => panic!("unwritten switch {other:?}"),
        }
    }

    /// Checked by vectors where the checked line names constructed, recorded or adjusted vectors.
    fn checked(written: &str) -> Checked {
        let by_vectors = ["constructed", "recorded", "adjusted"]
            .iter()
            .any(|way| written.contains(way));
        if by_vectors {
            Checked::ByVectors
        } else {
            Checked::OriginalOnly
        }
    }

    #[test]
    fn the_catalogue_is_the_specs_entries_in_order() {
        let entries = spec_entries();
        let ids: Vec<String> = entries.iter().map(|entry| entry.id.clone()).collect();
        let catalogued: Vec<String> = CATALOGUE
            .iter()
            .map(|behaviour| format!("{:?}", behaviour.id))
            .collect();
        assert_eq!(catalogued, ids);
        for (behaviour, entry) in CATALOGUE.iter().zip(&entries) {
            assert_eq!(behaviour.kind, kind(&entry.field("kind")), "{}", entry.id);
            assert_eq!(
                behaviour.switch,
                switch(&entry.field("switch")),
                "{}",
                entry.id
            );
            assert_eq!(behaviour.summary, entry.field("summary"), "{}", entry.id);
            assert_eq!(
                behaviour.checked,
                checked(&entry.field("checked")),
                "{}",
                entry.id
            );
        }
    }

    /// Every member of the enum `K`, in its order.
    fn members<K: Enum>() -> Vec<K> {
        EnumMap::<K, ()>::default()
            .into_iter()
            .map(|(member, ())| member)
            .collect()
    }

    #[test]
    fn the_catalogue_names_every_behaviour_once() {
        for id in members::<BehaviourId>() {
            let rows = CATALOGUE
                .iter()
                .filter(|behaviour| behaviour.id == id)
                .count();
            assert_eq!(rows, 1, "{id:?} has {rows} rows in the catalogue");
        }
    }

    /// The stages numbered 0 to 4 (`BoostStage::ALL`, by which the vectors number them) name every
    /// stage once, in the order the stages go up.
    #[test]
    fn the_stages_name_every_stage_once_in_order() {
        assert_eq!(BoostStage::ALL.to_vec(), members::<BoostStage>());
    }
}
