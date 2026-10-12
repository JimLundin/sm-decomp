//! Measurement interchange is separate from canonical evidence provenance.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum Measurement {
    Measured { value: u64 },
    Unavailable { reason: String },
}
impl Measurement {
    pub fn value(value: u64) -> Self {
        Self::Measured { value }
    }
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::Unavailable { reason: reason.into() }
    }
}
impl Default for Measurement {
    fn default() -> Self {
        Self::unavailable("not collected")
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clock {
    pub name: String,
    pub domain: String,
    pub origin_ns: u64,
}
pub fn monotonic_ns() -> Result<u64> {
    #[cfg(target_os = "linux")]
    {
        let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
        // SAFETY: valid writable timespec; no retained pointers.
        if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(u64::try_from(time.tv_sec)?
            .checked_mul(1_000_000_000)
            .and_then(|n| n.checked_add(time.tv_nsec as u64))
            .context("monotonic clock overflow")?)
    }
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("CLOCK_MONOTONIC metrics unavailable on this platform")
    }
}
impl Clock {
    pub fn local_domain() -> Result<String> {
        Ok(format!(
            "{}:{}",
            fs::read_to_string("/proc/sys/kernel/random/boot_id")?.trim(),
            fs::read_link("/proc/self/ns/time").map(|p| p.to_string_lossy().into_owned()).unwrap_or_default()
        ))
    }
    pub fn new() -> Result<Self> {
        Ok(Self { name: "CLOCK_MONOTONIC".into(), domain: Self::local_domain()?, origin_ns: monotonic_ns()? })
    }
    pub fn offset(&self) -> Result<u64> {
        monotonic_ns()?.checked_sub(self.origin_ns).context("clock origin is in the future")
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextReceipt {
    pub run_id: String,
    pub scope_id: String,
    pub parent_scope: Option<String>,
    pub clock: Clock,
    pub enabled: bool,
    pub path: PathBuf,
}
impl ContextReceipt {
    pub fn from_env() -> Result<Option<Self>> {
        let Some(path) = std::env::var_os("SM_PIPELINE_METRICS_CONTEXT") else {
            return Ok(None);
        };
        let value: Self = toml::from_str(&fs::read_to_string(path)?)?;
        ensure!(value.clock.name == "CLOCK_MONOTONIC", "unrelated metrics clock");
        ensure!(value.clock.domain == Clock::local_domain()?, "unrelated metrics clock domain");
        value.clock.offset()?;
        Ok(Some(value))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stage {
    pub id: String,
    pub parent: Option<String>,
    pub name: String,
    pub start_ns: u64,
    pub end_ns: u64,
    pub elapsed_ns: u64,
    pub outcome: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Process {
    pub pid: u32,
    pub parent_pid: u32,
    pub start_ticks: Measurement,
    pub role: String,
    pub argv: Vec<String>,
    pub start_ns: u64,
    pub end_ns: u64,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub outcome: String,
    pub reap_count: u32,
    pub user_cpu_ns: Measurement,
    pub system_cpu_ns: Measurement,
    pub peak_rss_bytes: Measurement,
    pub accounting: String,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Resources {
    pub aggregate_sampled_rss_bytes: Measurement,
    pub staging_allocated_bytes: Measurement,
    pub private_store_allocated_bytes: Measurement,
    pub combined_allocated_bytes: Measurement,
    pub minimum_free_bytes: Measurement,
    pub rss_samples: u64,
    pub disk_samples: u64,
    pub incomplete_samples: u64,
    pub rss_interval_ms: u64,
    pub disk_interval_ms: u64,
    pub rss_peak_at_ns: u64,
    pub disk_peak_at_ns: u64,
    pub semantics: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Transport {
    pub logical_bytes: Measurement,
    pub transport_bytes: Measurement,
    pub pack_bytes: Measurement,
    pub flush_count: Measurement,
    pub write_calls: Measurement,
    pub write_elapsed_ns: Measurement,
    pub max_flush_ns: Measurement,
    pub short_writes: Measurement,
    pub interruptions: Measurement,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub format: String,
    pub run_id: String,
    pub scope_id: String,
    pub parent_scope: Option<String>,
    pub role: String,
    pub clock: Clock,
    pub enabled: bool,
    pub start_ns: u64,
    pub end_ns: u64,
    pub elapsed_ns: u64,
    pub outcome: String,
    pub error: Option<String>,
    pub last_completed_stage: Option<String>,
    pub stages: Vec<Stage>,
    pub processes: Vec<Process>,
    pub resources: Resources,
    pub transport: Transport,
    pub configuration: std::collections::BTreeMap<String, String>,
}
impl Receipt {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == "pipeline-metrics-v1" && !self.run_id.is_empty() && !self.scope_id.is_empty(),
            "invalid metrics identity"
        );
        ensure!(
            self.clock.name == "CLOCK_MONOTONIC" && self.clock.domain == Clock::local_domain()?,
            "unrelated metrics clock domain"
        );
        ensure!(
            self.end_ns >= self.start_ns && self.elapsed_ns == self.end_ns - self.start_ns,
            "invalid metrics outer interval"
        );
        ensure!(
            ["success", "failed", "timed-out", "interrupted", "incomplete"].contains(&self.outcome.as_str()),
            "invalid metrics outcome"
        );
        let mut ids = std::collections::BTreeSet::new();
        for stage in &self.stages {
            ensure!(ids.insert(&stage.id) && !stage.name.is_empty(), "duplicate/empty metrics stage");
            ensure!(
                stage.start_ns >= self.start_ns
                    && stage.end_ns >= stage.start_ns
                    && stage.end_ns <= self.end_ns
                    && stage.elapsed_ns == stage.end_ns - stage.start_ns,
                "invalid metrics stage interval"
            );
            ensure!(
                ["success", "failed", "timed-out", "interrupted", "incomplete"].contains(&stage.outcome.as_str()),
                "invalid metrics stage outcome"
            );
        }
        for process in &self.processes {
            ensure!(process.reap_count == 1 && process.end_ns >= process.start_ns, "invalid process reap/interval");
            for metric in [&process.user_cpu_ns, &process.system_cpu_ns, &process.peak_rss_bytes, &process.start_ticks]
            {
                if let Measurement::Unavailable { reason } = metric {
                    ensure!(!reason.is_empty(), "missing unavailable reason");
                }
            }
        }
        Ok(())
    }
}
