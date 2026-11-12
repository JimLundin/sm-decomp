//! Checks the catalogue against `docs/behaviours/world-camera.md`: one entry per spec entry, in the
//! spec's order, each with its spec entry's summary, kind and switch.

#[cfg(test)]
mod tests {
    use types::{Kind, Switch};
    use world_camera::{BehaviourId, CATALOGUE};

    const SPEC: &str = include_str!("../../../../docs/behaviours/world-camera.md");

    /// A spec entry: its name, and its lines up to the next heading.
    struct SpecEntry {
        name: &'static str,
        lines: Vec<&'static str>,
    }

    impl SpecEntry {
        /// The text of one of its fields, `- field: ...`, its continuation lines joined on.
        fn field(&self, field: &str) -> String {
            let start = format!("- {field}: ");
            let mut lines = self.lines.iter().skip_while(|l| !l.starts_with(&start));
            let first = lines
                .next()
                .unwrap_or_else(|| panic!("{} has no {field}", self.name));
            let rest = lines.take_while(|l| l.starts_with("  "));
            core::iter::once(&first[start.len()..])
                .chain(rest.map(|l| l.trim()))
                .collect::<Vec<_>>()
                .join(" ")
        }
    }

    /// The spec's behaviour entries, in its order: the `###` headings not numbered.
    fn spec_entries() -> Vec<SpecEntry> {
        let mut entries: Vec<SpecEntry> = Vec::new();
        for line in SPEC.lines() {
            if let Some(heading) = line.strip_prefix('#') {
                let name = heading.strip_prefix("## ");
                entries.extend(
                    name.filter(|n| !n.starts_with(|c: char| c.is_ascii_digit()))
                        .map(|name| SpecEntry {
                            name,
                            lines: Vec::new(),
                        }),
                );
                continue;
            }
            if let Some(entry) = entries.last_mut() {
                entry.lines.push(line);
            }
        }
        entries
    }

    #[test]
    fn the_catalogue_has_one_entry_per_spec_entry() {
        let entries: Vec<&str> = spec_entries().iter().map(|e| e.name).collect();
        let ids: Vec<String> = CATALOGUE.iter().map(|b| format!("{:?}", b.id)).collect();
        assert_eq!(
            entries, ids,
            "one entry per spec entry, in the spec's order"
        );
        let first = CATALOGUE.first().map(|b| b.id);
        assert_eq!(first, Some(BehaviourId::FollowingHer));
    }

    #[test]
    fn each_entry_has_its_spec_entry_summary_and_kind() {
        for (entry, behaviour) in spec_entries().iter().zip(&CATALOGUE) {
            assert_eq!(
                entry.field("summary"),
                behaviour.summary,
                "{}'s summary",
                entry.name
            );
            let kind = entry.field("kind");
            let agrees = match behaviour.kind {
                Kind::Mechanic => kind.starts_with("mechanic"),
                Kind::Quirk => kind.starts_with("quirk"),
                Kind::Timing => kind.starts_with("timing"),
                Kind::Encoding => kind.starts_with("encoding"),
                Kind::Output => kind.starts_with("output"),
            };
            assert!(agrees, "{}'s kind is {kind}", entry.name);
        }
    }

    /// The spec's `switch:` is `**Parameter**` or `**Toggle**` where the entry has one, and
    /// `none` otherwise, which the catalogue holds as `Fixed`.
    #[test]
    fn each_entry_has_its_spec_entry_switch() {
        for (entry, behaviour) in spec_entries().iter().zip(&CATALOGUE) {
            let switch = entry.field("switch");
            let agrees = match behaviour.switch {
                Switch::Parameter => switch.starts_with("**Parameter**"),
                Switch::Toggle => switch.starts_with("**Toggle**"),
                Switch::Fixed => switch.starts_with("none"),
            };
            assert!(agrees, "{}'s switch is {switch}", entry.name);
        }
    }
}
