//! Cross-process worker capacity; partial multi-slot reservations never wait.
use crate::metrics;
use anyhow::{Context, Result, ensure};
use std::{
    fs::{self, File, OpenOptions},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    time::{Duration, Instant},
};
pub fn acquire_lease(root: &Path, requested: usize) -> Result<(File, f64)> {
    let now = u64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis())?;
    acquire_lease_until(root, requested, now.checked_add(3_600_000).context("worker deadline overflow")?)
}

pub fn acquire_lease_until(root: &Path, requested: usize, deadline_unix_ms: u64) -> Result<(File, f64)> {
    let (mut held, wait) = acquire_leases_until(root, requested, 1, deadline_unix_ms)?;
    Ok((held.pop().context("shared worker reservation is empty")?, wait))
}

pub fn acquire_leases_until(
    root: &Path,
    requested: usize,
    count: usize,
    deadline_unix_ms: u64,
) -> Result<(Vec<File>, f64)> {
    metrics::measure("shared-capacity-wait", || acquire_leases_inner(root, requested, count, deadline_unix_ms))
}

fn acquire_leases_inner(
    root: &Path,
    requested: usize,
    count: usize,
    deadline_unix_ms: u64,
) -> Result<(Vec<File>, f64)> {
    ensure!((1..=64).contains(&requested), "worker count must be 1..64");
    ensure!((1..=requested).contains(&count), "shared worker reservation exceeds configured capacity");
    let started = Instant::now();
    let now = u64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis())?;
    ensure!(now < deadline_unix_ms, "shared worker time budget exhausted before reservation");
    let deadline = started + Duration::from_millis(deadline_unix_ms - now).min(Duration::from_secs(3600));
    let root = root.join("workers");
    fs::create_dir_all(&root)?;
    let guard = OpenOptions::new()
        .create(true)
        .append(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(root.join("config.lock"))?;
    ensure!(guard.metadata()?.is_file(), "worker configuration lock is not regular");
    loop {
        metrics::check_cancelled()?;
        ensure!(Instant::now() < deadline, "shared worker time budget exhausted waiting for configuration");
        match guard.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(10)),
            Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
        }
    }
    let config = root.join("capacity.toml");
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Capacity {
        workers: usize,
    }
    let capacity = if config.exists() {
        pipeline_model::read::<Capacity>(&config)?.workers
    } else {
        pipeline_model::write(&config, &Capacity { workers: requested })?;
        requested
    };
    ensure!(capacity == requested, "shared worker budget already configured to {capacity}");
    drop(guard);
    let held = loop {
        metrics::check_cancelled()?;
        ensure!(Instant::now() < deadline, "shared worker time budget exhausted waiting for a slot");
        let mut selected = Vec::with_capacity(count);
        for slot in 0..capacity {
            ensure!(Instant::now() < deadline, "shared worker time budget exhausted while reserving slots");
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(root.join(format!("{slot}.lock")))?;
            ensure!(file.metadata()?.is_file(), "worker slot lock is not regular");
            match file.try_lock() {
                Ok(()) => {
                    selected.push(file);
                    if selected.len() == count {
                        break;
                    }
                }
                Err(std::fs::TryLockError::WouldBlock) => {}
                Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
            }
        }
        if selected.len() == count {
            ensure!(Instant::now() < deadline, "shared worker time budget exhausted during reservation");
            break selected;
        }
        // Never wait with a partial reservation: concurrent multi-slot callers
        // must leave enough capacity for one complete producer to make progress.
        drop(selected);
        std::thread::sleep(Duration::from_millis(10));
    };
    Ok((held, started.elapsed().as_secs_f64()))
}

#[cfg(test)]
#[path = "worker_budget_tests.rs"]
mod worker_budget_tests;
