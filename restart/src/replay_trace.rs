//! Complete recorded host-step observations through the maintained trace protocol.
use sm_trace::{Comparison, Identity, Reference, Writer};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn reference(path: &Path, seal: &str, identity: &Identity) -> Result<Reference, String> {
    let reference = Reference::load(path, identity).map_err(|e| e.to_string())?;
    if reference.summary().sha256.iter().map(|byte| format!("{byte:02x}")).collect::<String>() != seal {
        return Err("trace reference differs from its externally supplied SHA256".into());
    }
    Ok(reference)
}

pub(crate) struct Session<'a> {
    writer: Writer,
    comparison: Option<Comparison<'a>>,
    out: PathBuf,
    count: u64,
    failure: Option<String>,
}

impl<'a> Session<'a> {
    pub(crate) fn create(out: &Path, identity: &Identity, reference: Option<&'a Reference>) -> Result<Self, String> {
        Ok(Self {
            writer: Writer::create(out.join("trace.bin"), identity).map_err(|e| e.to_string())?,
            comparison: reference.map(Reference::comparison),
            out: out.to_owned(),
            count: 0,
            failure: None,
        })
    }

    pub(crate) fn record(&mut self, step: u64, ram: &[u8], external: &[u8], effects: &[u8]) -> Result<(), String> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let result = self.record_next(step, ram, external, effects);
        if let Err(error) = &result {
            self.failure = Some(error.clone());
        }
        result
    }

    fn record_next(&mut self, step: u64, ram: &[u8], external: &[u8], effects: &[u8]) -> Result<(), String> {
        let observed = if let Some(comparison) = &mut self.comparison {
            match comparison.check_observed(step, ram, external, effects) {
                Ok(observed) => Some(observed),
                Err(error) => {
                    // Terminal before any diagnostic I/O; retries must never
                    // replace the first divergent bytes or seal a success.
                    self.failure = Some(error.to_string());
                    // Preserve the actual first divergent bytes before returning terminally.
                    for (name, bytes) in [("wram", ram), ("external", external), ("effects", effects)] {
                        fs::write(self.out.join(format!("mismatch.{name}")), bytes).map_err(|e| e.to_string())?;
                    }
                    fs::write(self.out.join("mismatch.txt"), error.to_string()).map_err(|e| e.to_string())?;
                    return Err(error.to_string());
                }
            }
        } else {
            None
        };
        if let Some(observed) = observed {
            self.writer.record_observed(observed).map_err(|e| e.to_string())?;
        } else {
            self.writer.record(step, ram, external, effects).map_err(|e| e.to_string())?;
        }
        self.count += 1;
        Ok(())
    }

    pub(crate) fn finish(self, expected: u64, accepted: bool) -> Result<sm_trace::Summary, String> {
        if !accepted {
            return Err("trace checkpoint or ownership gate failed".into());
        }
        if self.count != expected {
            return Err(format!("trace expected {expected} steps, received {}", self.count));
        }
        if let Some(error) = self.failure {
            return Err(error);
        }
        if let Some(comparison) = self.comparison {
            comparison.finish().map_err(|e| e.to_string())?;
        }
        self.writer.finish().map_err(|e| e.to_string())
    }
}

#[cfg(feature = "private-checks")]
pub(crate) fn private_assertions() -> Vec<(&'static str, fn())> {
    vec![
        (
            "comparison_rejects_wrong_seal_and_retains_first_divergent_bytes",
            tests::comparison_rejects_wrong_seal_and_retains_first_divergent_bytes as fn(),
        ),
        (
            "session_diagnostic_failure_is_terminal_and_success_bytes_match",
            tests::session_diagnostic_failure_is_terminal_and_success_bytes_match as fn(),
        ),
    ]
}

#[cfg(any(test, feature = "private-checks"))]
mod tests {
    use super::*;
    #[cfg_attr(test, test)]
    pub(super) fn comparison_rejects_wrong_seal_and_retains_first_divergent_bytes() {
        let out = std::env::temp_dir().join(format!(
            "sm-full-trace-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&out).unwrap();
        let identity = Identity {
            reference: "source".into(),
            rom: "rom".into(),
            host: "host".into(),
            input: "input".into(),
            observation: "schema".into(),
            scenario: "one-step".into(),
        };
        let ram = vec![0u8; sm_trace::WRAM_BYTES];
        let mut writer = Writer::create(out.join("reference.bin"), &identity).unwrap();
        writer.record(1, &ram, b"external", b"effects").unwrap();
        let summary = writer.finish().unwrap();
        let seal = summary.sha256.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert!(reference(&out.join("reference.bin"), &"0".repeat(64), &identity).is_err());
        let expected = reference(&out.join("reference.bin"), &seal, &identity).unwrap();
        let mut session = Session::create(&out, &identity, Some(&expected)).unwrap();
        let error = session.record(1, &ram, b"changed", b"effects").unwrap_err();
        assert_eq!(session.record(1, &ram, b"retry", b"different"), Err(error));
        assert_eq!(fs::read(out.join("mismatch.external")).unwrap(), b"changed");
        assert_eq!(fs::read(out.join("mismatch.wram")).unwrap(), ram);
        assert_eq!(fs::read(out.join("mismatch.effects")).unwrap(), b"effects");
        assert!(session.finish(1, true).is_err());
        assert!(Reference::load(out.join("trace.bin"), &identity).is_err());
        let rejected = out.join("rejected");
        fs::create_dir(&rejected).unwrap();
        let mut session = Session::create(&rejected, &identity, None).unwrap();
        session.record(1, &ram, b"external", b"effects").unwrap();
        assert!(session.finish(1, false).is_err());
        assert!(Reference::load(rejected.join("trace.bin"), &identity).is_err());
        let missing = out.join("missing");
        fs::create_dir(&missing).unwrap();
        let mut session = Session::create(&missing, &identity, None).unwrap();
        session.record(1, &ram, b"external", b"effects").unwrap();
        assert!(session.finish(2, true).is_err());
        assert!(Reference::load(missing.join("trace.bin"), &identity).is_err());
        fs::remove_dir_all(out).unwrap();
    }

    #[cfg_attr(test, test)]
    pub(super) fn session_diagnostic_failure_is_terminal_and_success_bytes_match() {
        let out = std::env::temp_dir().join(format!(
            "sm-trace-terminal-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&out).unwrap();
        let identity = Identity {
            reference: "source".into(),
            rom: "rom".into(),
            host: "host".into(),
            input: "input".into(),
            observation: "schema".into(),
            scenario: "two-steps".into(),
        };
        let ram = vec![0; sm_trace::WRAM_BYTES];
        let mut writer = Writer::create(out.join("reference.bin"), &identity).unwrap();
        for step in 1..=2 {
            writer.record(step, &ram, &[step as u8], b"effects").unwrap();
        }
        let summary = writer.finish().unwrap();
        let reference = Reference::load(out.join("reference.bin"), &identity).unwrap();
        let mut success = Session::create(&out, &identity, Some(&reference)).unwrap();
        for step in 1..=2 {
            success.record(step, &ram, &[step as u8], b"effects").unwrap();
        }
        assert_eq!(success.finish(2, true).unwrap(), summary);
        assert_eq!(fs::read(out.join("trace.bin")).unwrap(), fs::read(out.join("reference.bin")).unwrap());

        // Fail each ordered diagnostic write deterministically via a directory
        // at its file path. Earlier raw outputs must survive retries unchanged.
        for (index, blocked) in ["wram", "external", "effects", "txt"].into_iter().enumerate() {
            let failed = out.join(format!("failed-{index}"));
            fs::create_dir(&failed).unwrap();
            fs::create_dir(failed.join(format!("mismatch.{blocked}"))).unwrap();
            let mut session = Session::create(&failed, &identity, Some(&reference)).unwrap();
            session.record(1, &ram, &[1], b"effects").unwrap();
            let error = session.record(2, &ram, b"first-change", b"first-effects").unwrap_err();
            assert_eq!(session.count, 1, "comparison must precede append");
            assert_eq!(session.record(2, &ram, b"retry-change", b"retry-effects"), Err(error.clone()));
            assert_eq!(session.finish(1, true), Err(error));
            if index > 0 {
                assert_eq!(fs::read(failed.join("mismatch.wram")).unwrap(), ram);
            }
            if index > 1 {
                assert_eq!(fs::read(failed.join("mismatch.external")).unwrap(), b"first-change");
            }
            if index > 2 {
                assert_eq!(fs::read(failed.join("mismatch.effects")).unwrap(), b"first-effects");
            }
            assert!(Reference::load(failed.join("trace.bin"), &identity).is_err());
        }
        let author = out.join("invalid-author");
        fs::create_dir(&author).unwrap();
        let mut session = Session::create(&author, &identity, None).unwrap();
        session.record(1, &ram, &[1], b"effects").unwrap();
        let error = session.record(2, &[], &[2], b"effects").unwrap_err();
        assert_eq!(session.record(2, &ram, &[2], b"effects"), Err(error.clone()));
        assert_eq!(session.finish(1, true), Err(error));
        assert!(Reference::load(author.join("trace.bin"), &identity).is_err());
        fs::remove_dir_all(out).unwrap();
    }
}
