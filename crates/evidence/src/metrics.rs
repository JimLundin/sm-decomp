//! Optional command-side measurements, never part of stored evidence.
use anyhow::{Context, Result};
use pipeline_model::metrics::{ContextReceipt, Receipt, Resources, Stage, Transport};
use std::{fs, io::Write, path::Path};
pub struct Metrics {
    receipt: Option<Receipt>,
    path: Option<std::path::PathBuf>,
    finished: bool,
}
impl Metrics {
    pub fn start(role: &str) -> Result<Self> {
        let context = ContextReceipt::from_env()?;
        let Some(context) = context else {
            return Ok(Self { receipt: None, path: None, finished: false });
        };
        let start = context.clock.offset()?;
        Ok(Self {
            path: Some(context.path),
            receipt: Some(Receipt {
                format: "pipeline-metrics-v1".into(),
                run_id: context.run_id,
                scope_id: context.scope_id,
                parent_scope: context.parent_scope,
                role: role.into(),
                clock: context.clock,
                enabled: context.enabled,
                start_ns: start,
                end_ns: start,
                elapsed_ns: 0,
                outcome: "incomplete".into(),
                error: None,
                last_completed_stage: None,
                stages: vec![],
                processes: vec![],
                resources: Resources::default(),
                transport: Transport::default(),
                configuration: Default::default(),
            }),
            finished: false,
        })
    }
    pub fn measure<T>(&mut self, name: &str, operation: impl FnOnce() -> Result<T>) -> Result<T> {
        let start = self.receipt.as_ref().map(|r| r.clock.offset()).transpose()?;
        let result = operation();
        if let Some(r) = &mut self.receipt {
            let end = r.clock.offset()?;
            let start = start.unwrap();
            if result.is_ok() {
                r.last_completed_stage = Some(name.into());
            }
            r.stages.push(Stage {
                id: format!("{}-{}", r.scope_id, r.stages.len()),
                parent: Some(r.scope_id.clone()),
                name: name.into(),
                start_ns: start,
                end_ns: end,
                elapsed_ns: end - start,
                outcome: if result.is_ok() { "success" } else { "failed" }.into(),
            });
        }
        result
    }
    pub fn finish(&mut self, outcome: &str, error: Option<String>) -> Result<()> {
        if let Some(r) = &mut self.receipt {
            r.end_ns = r.clock.offset()?;
            r.elapsed_ns = r.end_ns - r.start_ns;
            r.outcome = outcome.into();
            r.error = error;
            let path = self.path.as_ref().unwrap();
            let pending = path.with_extension("metrics.pending");
            let mut out = fs::OpenOptions::new().create_new(true).write(true).open(&pending)?;
            out.write_all(toml::to_string_pretty(r)?.as_bytes())?;
            out.sync_all()?;
            fs::rename(&pending, path)?;
            fs::File::open(path.parent().context("metrics parent missing")?)?.sync_all()?;
        }
        self.finished = true;
        Ok(())
    }
}
impl Drop for Metrics {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.finish("incomplete", Some("evidence command failed before completion".into()));
        }
    }
}
