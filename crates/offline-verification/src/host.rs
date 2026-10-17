//! Compiled producer inputs supplied by the trusted application composition.
use anyhow::{Result, ensure};
use std::collections::BTreeSet;

/// The application's compiled source stamps and retained producer identity.
///
/// This is configuration from the host executable, never deserialized candidate
/// input. Verification always adds the source stamps of its implementation and
/// both lower owners. Admission authenticates the retained producer identity;
/// it does not require equality with the currently running executable.
#[derive(Clone, Debug)]
pub struct HostInputs {
    sources: &'static [(&'static str, &'static str)],
    producer_sha256: String,
}

impl HostInputs {
    pub fn new(sources: &'static [(&'static str, &'static str)], producer_sha256: String) -> Result<Self> {
        ensure!(
            producer_sha256.len() == 64 && producer_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid producer identity"
        );
        ensure!(!sources.is_empty(), "compiled host source contract is empty");
        let mut names = BTreeSet::new();
        for (name, _) in sources {
            ensure!(
                !name.is_empty()
                    && std::path::Path::new(name)
                        .components()
                        .all(|part| matches!(part, std::path::Component::Normal(_)))
                    && names.insert(*name),
                "invalid or repeated compiled host source: {name}"
            );
        }
        Ok(Self { sources, producer_sha256 })
    }

    pub fn sources(&self) -> &'static [(&'static str, &'static str)] {
        self.sources
    }

    pub fn producer_sha256(&self) -> &str {
        &self.producer_sha256
    }
}
