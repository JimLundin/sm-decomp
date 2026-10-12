//! Bounded process-tree RSS and allocated-block samples, separate from evidence.
use pipeline_model::metrics::{Measurement, Resources};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{CStr, CString},
    fs, io,
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};

const PROC_ENTRIES: usize = 32_768;
const PROC_TIME: Duration = Duration::from_millis(20);
const DISK_ENTRIES: usize = 100_000;
const DISK_TIME: Duration = Duration::from_millis(50);
const MAX_DEPTH: usize = 256;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PathKind {
    Staging,
    PrivateStore,
}

pub(crate) struct ResourceSnapshot {
    pub resources: Resources,
    pub complete: bool,
    pub limitations: Vec<String>,
    pub rss_missing_processes: u64,
    pub disk_missing_paths: u64,
    pub truncated_scans: u64,
}

pub(crate) struct ResourceSampler {
    root_pid: u32,
    root_start: Option<u64>,
    registered: BTreeMap<u32, u64>,
    paths: BTreeMap<PathBuf, PathKind>,
    resources: Resources,
    complete: bool,
    limitations: BTreeSet<String>,
    rss_missing_processes: u64,
    disk_missing_paths: u64,
    truncated_scans: u64,
}

#[derive(Clone, Copy)]
struct Proc {
    parent: u32,
    start: u64,
    rss: u64,
}

fn process(pid: u32) -> io::Result<Proc> {
    let text = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let (_, fields) = text.rsplit_once(')').ok_or_else(|| io::Error::other("invalid proc stat"))?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    let number = |index: usize| -> io::Result<u64> {
        fields.get(index).ok_or_else(|| io::Error::other("truncated proc stat"))?.parse().map_err(io::Error::other)
    };
    // stat fields following comm are state (3), ppid (4), ... starttime (22), rss (24).
    let parent = u32::try_from(number(1)?).map_err(io::Error::other)?;
    let start = number(19)?;
    let pages = number(21)?;
    // SAFETY: sysconf has no pointers and does not mutate process state.
    let page_size = u64::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) }).map_err(io::Error::other)?;
    Ok(Proc { parent, start, rss: pages.checked_mul(page_size).ok_or_else(|| io::Error::other("RSS overflow"))? })
}

impl ResourceSampler {
    pub fn new(root_pid: u32) -> Self {
        let root_start = process(root_pid).ok().map(|p| p.start);
        Self {
            root_pid, root_start, registered: BTreeMap::new(), paths: BTreeMap::new(),
            resources: Resources {
                rss_interval_ms: 100, disk_interval_ms: 500,
                semantics: "aggregate concurrent root-orchestrator plus registered process trees; rolling Linux /proc PID+start-time samples; summed RSS counts shared pages repeatedly and may miss short-lived or between-sample peaks; disk uses allocated blocks*512 with per-scan dev/inode deduplication and no symlink traversal; proc scan bounded to 32768 entries/20ms, disk scan to 100000 entries/50ms and depth256; filesystem calls themselves may block beyond the scan deadline; incomplete peaks are lower bounds".into(),
                ..Resources::default()
            },
            complete: true, limitations: BTreeSet::new(), rss_missing_processes: 0,
            disk_missing_paths: 0, truncated_scans: 0,
        }
    }
    pub fn register(&mut self, pid: u32, start_ticks: u64) {
        self.registered.insert(pid, start_ticks);
    }
    pub fn unregister(&mut self, pid: u32) {
        self.registered.remove(&pid);
    }
    pub fn add_path(&mut self, path: PathBuf, kind: PathKind) {
        self.paths.insert(path, kind);
    }
    pub fn remove_path(&mut self, path: &Path) {
        self.paths.remove(path);
    }

    pub fn sample(&mut self, now_ns: u64, disk: bool) -> ResourceSnapshot {
        self.sample_rss(now_ns);
        if disk {
            self.sample_disk(now_ns);
        }
        ResourceSnapshot {
            resources: self.resources.clone(),
            complete: self.complete,
            limitations: self.limitations.iter().cloned().collect(),
            rss_missing_processes: self.rss_missing_processes,
            disk_missing_paths: self.disk_missing_paths,
            truncated_scans: self.truncated_scans,
        }
    }
    fn incomplete(&mut self, reason: impl Into<String>) {
        self.complete = false;
        self.resources.incomplete_samples += 1;
        self.limitations.insert(reason.into());
    }
    fn sample_rss(&mut self, now_ns: u64) {
        let started = Instant::now();
        let mut roots = self.registered.clone();
        if let Some(start) = self.root_start {
            roots.insert(self.root_pid, start);
        } else {
            self.incomplete("root orchestrator process identity unavailable");
        }
        let mut included = BTreeSet::new();
        let mut pending: Vec<_> = roots.into_iter().map(|(pid, start)| (pid, Some(start), None)).collect();
        let mut count = 0usize;
        let mut total = Some(0u64);
        while let Some((pid, expected, parent)) = pending.pop() {
            if count >= PROC_ENTRIES || started.elapsed() >= PROC_TIME {
                self.truncated_scans += 1;
                self.incomplete("process-tree entry/time bound reached");
                break;
            }
            count += 1;
            let record = match process(pid) {
                Ok(record)
                    if expected.is_none_or(|start| start == record.start)
                        && parent.is_none_or(|(ppid, start)| record.parent == ppid && record.start >= start) =>
                {
                    record
                }
                Err(error) if pid != self.root_pid && error.kind() == io::ErrorKind::NotFound => {
                    // A child may exit/reap between registration/enumeration and stat.
                    self.rss_missing_processes += 1;
                    self.limitations.insert("descendants that exit during a rolling sample may be missed".into());
                    continue;
                }
                _ => {
                    self.rss_missing_processes += 1;
                    self.incomplete("registered/root process missing, inaccessible or PID identity changed");
                    continue;
                }
            };
            if !included.insert((pid, record.start)) {
                continue;
            }
            total = total.and_then(|total| total.checked_add(record.rss));
            // Children belong to the spawning thread, so inspect every thread's children list.
            let tasks = match fs::read_dir(format!("/proc/{pid}/task")) {
                Ok(tasks) => tasks,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(_) => {
                    self.incomplete("process task enumeration inaccessible");
                    continue;
                }
            };
            for task in tasks {
                if count >= PROC_ENTRIES || started.elapsed() >= PROC_TIME {
                    self.truncated_scans += 1;
                    self.incomplete("process-tree task entry/time bound reached");
                    break;
                }
                count += 1;
                let task = match task {
                    Ok(task) => task,
                    Err(_) => {
                        self.incomplete("process task directory enumeration failed");
                        continue;
                    }
                };
                match fs::read_to_string(task.path().join("children")) {
                    Ok(children) => {
                        for child in children.split_whitespace() {
                            if pending.len() >= PROC_ENTRIES {
                                self.truncated_scans += 1;
                                self.incomplete("process-tree pending entry bound reached");
                                break;
                            }
                            match child.parse::<u32>() {
                                Ok(child) => pending.push((child, None, Some((pid, record.start)))),
                                Err(_) => self.incomplete("invalid process children list"),
                            }
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(_) => self.incomplete("process children list inaccessible"),
                }
            }
        }
        self.resources.rss_samples += 1;
        if let Some(total) = total {
            if peak(&mut self.resources.aggregate_sampled_rss_bytes, total) {
                self.resources.rss_peak_at_ns = now_ns;
            }
        } else {
            self.incomplete("aggregate RSS overflow");
        }
    }
    fn sample_disk(&mut self, now_ns: u64) {
        if self.paths.is_empty() {
            unavailable_if_unmeasured(&mut self.resources.staging_allocated_bytes, "no staging paths registered");
            unavailable_if_unmeasured(
                &mut self.resources.private_store_allocated_bytes,
                "no private-store paths registered",
            );
            unavailable_if_unmeasured(
                &mut self.resources.combined_allocated_bytes,
                "no staging/private-store paths registered",
            );
            unavailable_if_unmeasured(
                &mut self.resources.minimum_free_bytes,
                "no resource filesystem paths registered",
            );
            return;
        }
        self.resources.disk_samples += 1;
        let mut scan = DiskScan::new();
        let mut totals = [0u64; 2];
        let mut kinds = [false; 2];
        let mut free = None::<u64>;
        for (path, kind) in &self.paths {
            let index = if *kind == PathKind::Staging { 0 } else { 1 };
            match scan.path(path) {
                Ok(bytes) => {
                    kinds[index] = true;
                    match totals[index].checked_add(bytes) {
                        Some(value) => totals[index] = value,
                        None => scan.errors.push("disk allocated byte count overflow".into()),
                    }
                }
                Err(error) => {
                    scan.missing += 1;
                    scan.errors.push(format!("resource path {}: {error}", path.display()));
                }
            }
            match available_bytes(path) {
                Ok(bytes) => free = Some(free.map_or(bytes, |previous| previous.min(bytes))),
                Err(error) => {
                    scan.errors.push(format!("free filesystem bytes {} unavailable: {error}", path.display()))
                }
            }
        }
        self.disk_missing_paths += scan.missing;
        if scan.truncated {
            self.truncated_scans += 1;
        }
        for error in scan.errors {
            self.incomplete(error);
        }
        if kinds[0] {
            peak(&mut self.resources.staging_allocated_bytes, totals[0]);
        } else {
            unavailable_if_unmeasured(
                &mut self.resources.staging_allocated_bytes,
                "no readable staging paths registered",
            );
        }
        if kinds[1] {
            peak(&mut self.resources.private_store_allocated_bytes, totals[1]);
        } else {
            unavailable_if_unmeasured(
                &mut self.resources.private_store_allocated_bytes,
                "no readable private-store paths registered",
            );
        }
        if !kinds[0] && !kinds[1] {
            unavailable_if_unmeasured(
                &mut self.resources.combined_allocated_bytes,
                "no readable staging/private-store paths registered",
            );
        } else if let Some(total) = totals[0].checked_add(totals[1]) {
            if peak(&mut self.resources.combined_allocated_bytes, total) {
                self.resources.disk_peak_at_ns = now_ns;
            }
        } else {
            self.incomplete("combined allocated byte count overflow");
        }
        if let Some(free) = free {
            match self.resources.minimum_free_bytes {
                Measurement::Measured { value } if value <= free => {}
                _ => self.resources.minimum_free_bytes = Measurement::value(free),
            }
        }
    }
}

fn peak(measurement: &mut Measurement, value: u64) -> bool {
    match *measurement {
        Measurement::Measured { value: previous } if previous > value => false,
        _ => {
            *measurement = Measurement::value(value);
            true
        }
    }
}
fn unavailable_if_unmeasured(measurement: &mut Measurement, reason: &str) {
    if !matches!(measurement, Measurement::Measured { .. }) {
        *measurement = Measurement::unavailable(reason);
    }
}

struct DiskScan {
    started: Instant,
    entries: usize,
    seen: BTreeSet<(u64, u64)>,
    errors: Vec<String>,
    missing: u64,
    truncated: bool,
}
impl DiskScan {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            entries: 0,
            seen: BTreeSet::new(),
            errors: vec![],
            missing: 0,
            truncated: false,
        }
    }
    fn bounded(&mut self, depth: usize) -> bool {
        if self.entries >= DISK_ENTRIES || self.started.elapsed() >= DISK_TIME || depth >= MAX_DEPTH {
            if !self.truncated {
                self.errors.push("allocated-disk scan entry/time/depth bound reached".into());
            }
            self.truncated = true;
            return true;
        }
        self.entries += 1;
        false
    }
    fn path(&mut self, path: &Path) -> io::Result<u64> {
        let directory = open_directory(path)?;
        self.entry(directory.0, c".", 0)
    }
    fn entry(&mut self, parent: i32, name: &CStr, depth: usize) -> io::Result<u64> {
        if self.bounded(depth) {
            return Ok(0);
        }
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: name is NUL-terminated; metadata is valid output storage. Never follows the final symlink.
        if unsafe { libc::fstatat(parent, name.as_ptr(), metadata.as_mut_ptr(), libc::AT_SYMLINK_NOFOLLOW) } != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful fstatat initialized the stat.
        let metadata = unsafe { metadata.assume_init() };
        let kind = metadata.st_mode & libc::S_IFMT;
        if kind == libc::S_IFLNK {
            return Ok(0);
        }
        if !self.seen.insert((metadata.st_dev, metadata.st_ino)) {
            return Ok(0);
        }
        let mut bytes = u64::try_from(metadata.st_blocks)
            .map_err(io::Error::other)?
            .checked_mul(512)
            .ok_or_else(|| io::Error::other("allocated block count overflow"))?;
        if kind != libc::S_IFDIR {
            return Ok(bytes);
        }
        // SAFETY: openat consumes no pointers; O_NOFOLLOW prevents a raced directory symlink.
        let fd = unsafe {
            libc::openat(parent, name.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let owned = Descriptor(fd);
        let mut opened = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: fd is live and opened is valid writable output.
        if unsafe { libc::fstat(fd, opened.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful fstat initialized opened metadata.
        let opened = unsafe { opened.assume_init() };
        if opened.st_dev != metadata.st_dev || opened.st_ino != metadata.st_ino {
            return Err(io::Error::other("resource directory identity changed during scan"));
        }
        // SAFETY: fd is newly owned. fdopendir takes ownership on success.
        let directory = unsafe { libc::fdopendir(fd) };
        if directory.is_null() {
            let error = io::Error::last_os_error();
            return Err(error);
        }
        std::mem::forget(owned);
        let directory = Directory(directory);
        loop {
            if self.truncated || self.started.elapsed() >= DISK_TIME {
                self.bounded(MAX_DEPTH);
                break;
            }
            // SAFETY: this thread owns DIR. errno distinguishes end-of-directory from an error.
            unsafe {
                *libc::__errno_location() = 0;
            }
            let entry = unsafe { libc::readdir(directory.0) };
            if entry.is_null() {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(0) {
                    self.errors.push(format!("directory enumeration failed: {error}"));
                }
                break;
            }
            // SAFETY: readdir provided a valid NUL-terminated d_name, used before the next readdir.
            let child = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
            if child.to_bytes() == b"." || child.to_bytes() == b".." {
                continue;
            }
            match self.entry(fd, child, depth + 1) {
                Ok(child_bytes) => {
                    bytes =
                        bytes.checked_add(child_bytes).ok_or_else(|| io::Error::other("allocated bytes overflow"))?
                }
                Err(error) => {
                    self.missing += 1;
                    self.errors.push(format!("resource entry missing/unreadable during scan: {error}"));
                }
            }
        }
        Ok(bytes)
    }
}
struct Directory(*mut libc::DIR);
impl Drop for Directory {
    fn drop(&mut self) {
        // SAFETY: this guard uniquely owns the DIR and closes it once.
        unsafe {
            libc::closedir(self.0);
        }
    }
}
fn available_bytes(path: &Path) -> io::Result<u64> {
    let directory = open_directory(path)?;
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: live owned descriptor and writable statvfs output.
    if unsafe { libc::fstatvfs(directory.0, stat.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: statvfs succeeded and initialized output.
    let stat = unsafe { stat.assume_init() };
    stat.f_bavail.checked_mul(stat.f_frsize).ok_or_else(|| io::Error::other("free filesystem byte count overflow"))
}
struct Descriptor(i32);
impl Drop for Descriptor {
    fn drop(&mut self) {
        // SAFETY: descriptor is uniquely owned.
        unsafe {
            libc::close(self.0);
        }
    }
}
fn open_directory(path: &Path) -> io::Result<Descriptor> {
    let origin = if path.is_absolute() { c"/" } else { c"." };
    // SAFETY: NUL-terminated origin and no retained pointers.
    let fd = unsafe { libc::open(origin.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut directory = Descriptor(fd);
    for component in path.components() {
        let name = match component {
            Component::Normal(name) => CString::new(name.as_bytes()).map_err(io::Error::other)?,
            Component::RootDir | Component::CurDir => continue,
            _ => return Err(io::Error::other("resource path may not contain parent/prefix components")),
        };
        // SAFETY: directory fd is owned and name valid. Every component refuses symlink traversal.
        let fd = unsafe {
            libc::openat(
                directory.0,
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        directory = Descriptor(fd);
    }
    Ok(directory)
}
