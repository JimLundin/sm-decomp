//! Broad /proc activity and waiting attribution; this does not identify symbols.
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::Path,
    time::Instant,
};

#[derive(Serialize)]
struct Process {
    pid: u32,
    parent: u32,
    start_ticks: u64,
    cpu_ticks: u64,
    rss_bytes: u64,
    name: String,
    state: String,
    wait_channel: String,
}
#[derive(Serialize)]
struct Snapshot {
    offset_ms: u64,
    external_cpu_fraction: f64,
    loadavg: String,
    available_memory_bytes: u64,
    cpu_stat: String,
    diskstats: String,
    processes: Vec<Process>,
}
pub(super) struct Monitor {
    file: File,
    origin: Instant,
    previous_at: Instant,
    previous: BTreeMap<(u32, u64), u64>,
    clock_ticks: f64,
}
impl Monitor {
    pub fn start(path: &Path) -> Result<Self> {
        let mut file = fs::OpenOptions::new().create_new(true).write(true).open(path)?;
        file.write_all(b"format = \"pipeline-host-profile-v1\"\ninterval_ms = 500\nexternal_cpu_limit = 0.25\nsemantics = \"rolling host /proc snapshots; PID/start identity; state/wchan are broad attribution, not formatting/parser symbols; short processes may be missed; fraction is CPU seconds per wall second outside runner descendants\"\n")?;
        // SAFETY: sysconf has no pointers or retained mutable state.
        let clock_ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        ensure!(clock_ticks > 0, "CPU tick frequency unavailable");
        let now = Instant::now();
        Ok(Self { file, origin: now, previous_at: now, previous: BTreeMap::new(), clock_ticks: clock_ticks as f64 })
    }
    pub fn preflight(&mut self) -> Result<()> {
        let init = fs::read_to_string("/proc/1/comm")?;
        ensure!(
            init.trim() == "systemd" || init.trim() == "init",
            "host PID namespace unavailable; run benchmark with host process visibility"
        );
        self.sample(false)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.sample(true)
    }
    pub fn sample(&mut self, enforce: bool) -> Result<()> {
        let started = Instant::now();
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        ensure!(page_size > 0, "page size unavailable");
        let mut processes = vec![];
        for entry in fs::read_dir("/proc")? {
            ensure!(processes.len() < 32768 && started.elapsed().as_millis() < 100, "host process scan incomplete");
            let entry = entry?;
            let Some(pid) = entry.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else { continue };
            let text = match fs::read_to_string(entry.path().join("stat")) {
                Ok(text) => text,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            let (prefix, tail) = text.rsplit_once(')').context("invalid host stat")?;
            let fields = tail.split_whitespace().collect::<Vec<_>>();
            let n = |index: usize| -> Result<u64> { Ok(fields.get(index).context("truncated host stat")?.parse()?) };
            let rss = fields.get(21).context("missing host RSS")?.parse::<i64>()?.max(0) as u64;
            let wait_channel = match fs::read_to_string(entry.path().join("wchan")) {
                Ok(text) => text.trim().into(),
                Err(_) => "unavailable".into(),
            };
            processes.push(Process {
                pid,
                parent: u32::try_from(n(1)?)?,
                start_ticks: n(19)?,
                cpu_ticks: n(11)?.checked_add(n(12)?).context("CPU overflow")?,
                rss_bytes: rss.checked_mul(page_size as u64).context("RSS overflow")?,
                name: prefix.split_once('(').context("missing host comm")?.1.into(),
                state: fields[0].into(),
                wait_channel,
            });
        }
        let mut ours = BTreeSet::from([std::process::id()]);
        loop {
            let before = ours.len();
            for p in &processes {
                if ours.contains(&p.parent) {
                    ours.insert(p.pid);
                }
            }
            if ours.len() == before {
                break;
            }
        }
        let now = Instant::now();
        let elapsed = now.duration_since(self.previous_at).as_secs_f64();
        let external = processes
            .iter()
            .filter(|p| !ours.contains(&p.pid))
            .map(|p| self.previous.get(&(p.pid, p.start_ticks)).map_or(0, |old| p.cpu_ticks.saturating_sub(*old)))
            .sum::<u64>() as f64
            / self.clock_ticks
            / elapsed.max(0.001);
        self.previous = processes.iter().map(|p| ((p.pid, p.start_ticks), p.cpu_ticks)).collect();
        self.previous_at = now;
        let memory = fs::read_to_string("/proc/meminfo")?;
        let available = memory
            .lines()
            .find_map(|line| line.strip_prefix("MemAvailable:"))
            .context("available host memory missing")?
            .split_whitespace()
            .next()
            .context("available memory empty")?
            .parse::<u64>()?
            .checked_mul(1024)
            .context("memory overflow")?;
        let snapshot = Snapshot {
            offset_ms: u64::try_from(self.origin.elapsed().as_millis())?,
            external_cpu_fraction: external,
            loadavg: fs::read_to_string("/proc/loadavg")?.trim().into(),
            available_memory_bytes: available,
            cpu_stat: fs::read_to_string("/proc/stat")?,
            diskstats: fs::read_to_string("/proc/diskstats")?,
            processes,
        };
        // A wrapper keeps the process arrays nested under this sample in TOML.
        #[derive(Serialize)]
        struct Samples {
            samples: Vec<Snapshot>,
        }
        self.file.write_all(b"\n")?;
        self.file.write_all(toml::to_string(&Samples { samples: vec![snapshot] })?.as_bytes())?;
        if enforce {
            ensure!(external <= 0.25, "host interference: external CPU fraction {external:.3} > 0.25");
        }
        Ok(())
    }
    pub fn finish(&mut self) -> Result<()> {
        self.file.flush()?;
        self.file.sync_all()?;
        Ok(())
    }
}
