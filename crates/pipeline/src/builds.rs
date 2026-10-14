//! Translation-unit reuse; generated candidate units are always rebuilt.
use crate::build_cache::Preparation;
use crate::inputs::bounded;
use crate::{CacheRequest, CacheResult, acquire_lease, atomic_toml, cached_artifact};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use translation_execution::{
    OperationContext,
    confinement::{self, Program, ReadOnlyBind, RunLimits},
};

/// Internal installed execution context. Every compiler and fixture receives a
/// separate bounded /work; no writable verifier directory is mounted into
/// candidate execution.
pub(crate) struct StrictExecution {
    pub helper: PathBuf,
    pub rustc: PathBuf,
    pub c_compiler: PathBuf,
    pub compiler_runtime: Vec<ReadOnlyBind>,
    pub operation_root: PathBuf,
    pub context: OperationContext,
    pub root: PathBuf,
    pub workers: usize,
    pub compiler_limits: RunLimits,
    pub fixture_limits: RunLimits,
    pub rust_inputs: Vec<PathBuf>,
    pub c_inputs: Vec<PathBuf>,
    pub deadline_unix_ms: u64,
    /// Entire gate reservation includes live quotas, durable results and the
    /// parent copies. Reserving before launch bounds peak production bytes.
    pub max_output_bytes: u64,
    pub reserved_bytes: Mutex<u64>,
    pub reservation_group: Mutex<Option<ReservationGroup>>,
}

/// Every producer in a parallel wave uses one admission anchor. Its full
/// worst-case reservation remains held until every worker has joined.
pub(crate) struct ReservationGroup {
    before_bytes: u64,
    reserved_bytes: u64,
    completed_reserved_bytes: u64,
    retained_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StrictInvocation {
    pub context: OperationContext,
    pub request_sha256: String,
    pub program: Program,
    pub requested_limits: RunLimits,
    pub limits: RunLimits,
    pub reserved_bytes: u64,
    pub reservation_before_bytes: u64,
}

#[derive(Clone, Copy)]
pub(crate) enum StrictFixture {
    Family,
    Callback,
    Plm,
    Ridley,
    PlmAbi,
    RidleyAbi,
}

/// Recipe argv is constructed by the trusted family adapter. Only named input
/// files enter a fixture, and every material output has an explicit file name.
pub(crate) fn strict_fixture_program(helper: &Path, command: &[String], kind: StrictFixture) -> Result<Program> {
    let mut program = Program {
        executable: PathBuf::from(command.first().context("empty fixture command")?),
        arguments: command[1..].to_vec(),
        inputs: vec![],
        runtime: confinement::helper_runtime()?,
        helper: helper.into(),
        outputs: vec![],
    };
    let (lengths, inputs, outputs): (&[usize], Vec<(usize, &str)>, Vec<(usize, &str)>) = match kind {
        StrictFixture::Family => (
            &[6, 7, 9],
            [(4, "before.wram"), (6, "rom.bin"), (7, "host-before.bin")]
                .into_iter()
                .filter(|(i, _)| *i < command.len())
                .collect(),
            [(5, "after.wram"), (8, "host-after.bin")].into_iter().filter(|(i, _)| *i < command.len()).collect(),
        ),
        StrictFixture::Callback => (&[10], vec![(5, "before.wram"), (8, "rom.bin")], vec![(6, "after.wram")]),
        StrictFixture::Plm => (&[8], vec![(4, "before.wram"), (6, "rom.bin")], vec![(5, "after.wram")]),
        StrictFixture::Ridley => (
            &[15],
            vec![(5, "before.wram"), (8, "rom.bin"), (9, "host-before.bin"), (14, "ppu-before.bin")],
            vec![(6, "after.wram"), (10, "host-after.bin"), (11, "effects.log"), (12, "ppu.bin"), (13, "services.bin")],
        ),
        StrictFixture::PlmAbi => (&[8], vec![(6, "rom.bin")], vec![]),
        StrictFixture::RidleyAbi => (&[2], vec![], vec![]),
    };
    ensure!(lengths.contains(&command.len()), "confined fixture argument contract differs");
    for (index, name) in inputs {
        let destination = Path::new("/inputs").join(name);
        program.inputs.push(ReadOnlyBind { source: PathBuf::from(&command[index]), destination: destination.clone() });
        program.arguments[index - 1] = destination.display().to_string();
    }
    for (index, name) in outputs {
        program.arguments[index - 1] = Path::new("/work").join(name).display().to_string();
        program.outputs.push(name.into());
    }
    if matches!(kind, StrictFixture::Callback) {
        program.arguments[8] = "/work".into();
        program.outputs.extend(["callbacks.txt".into(), "callback-states.bin".into()]);
    }
    Ok(program)
}

pub(crate) fn strict_output_path(run: &Path, label: &str, name: &Path) -> PathBuf {
    if name == Path::new("callbacks.txt") {
        run.join(label).join(name)
    } else if ["after.wram", "host-after.bin", "effects.log", "ppu.bin", "services.bin", "callback-states.bin"]
        .iter()
        .any(|n| name == Path::new(n))
    {
        run.join(format!("{label}-{}", name.display()))
    } else {
        run.join(name)
    }
}

fn has_callback_snapshots(program: &Program) -> bool {
    program.outputs.iter().any(|path| path == Path::new("callback-states.bin"))
}

fn producer_reservation_bytes(program: &Program, limits: &RunLimits, deadline_unix_ms: u64) -> Result<u64> {
    let operation = translation_execution::operation::reserve_bytes_until(program, limits, 0, deadline_unix_ms)?;
    // Invocation TOML contains the Program; timing TOML repeats its argv.
    // Keep the fixed allowance for context, limits, diagnostic and table
    // prefixes, and account for arbitrary admitted paths/arguments too.
    let framing = u64::try_from(toml::to_string(program)?.len())?
        .checked_mul(2)
        .context("native producer framing reservation overflow")?;
    operation
        .checked_add(limits.writable_bytes)
        .and_then(|n| n.checked_add(if has_callback_snapshots(program) { limits.writable_bytes } else { 0 }))
        .and_then(|n| n.checked_add(framing))
        .and_then(|n| n.checked_add(64 * 1024))
        .context("native producer reservation overflow")
}

pub(crate) fn reservation_total(before: u64, bytes: u64, maximum: u64) -> Result<u64> {
    let next = before.checked_add(bytes).context("native gate reservation overflow")?;
    ensure!(next <= maximum, "native gate aggregate output reservation exhausted ({next} > {maximum})");
    Ok(next)
}

#[derive(Clone, Debug)]
pub(crate) struct ProducerReservation {
    pub before_bytes: u64,
    pub reserved_bytes: u64,
    pub retained_bytes: u64,
}

pub(crate) fn authenticate_reservations(
    parent_bytes: u64,
    reservations: &mut [ProducerReservation],
    maximum: u64,
) -> Result<u64> {
    let mut held = reservation_total(0, parent_bytes, maximum)?;
    reservations.sort_by_key(|reservation| reservation.before_bytes);
    let mut remaining = reservations;
    while let Some(first) = remaining.first() {
        ensure!(first.before_bytes == held, "native completed reservation chain changed");
        let count = remaining.iter().take_while(|reservation| reservation.before_bytes == held).count();
        let (group, rest) = remaining.split_at_mut(count);
        let mut peak = held;
        let mut retained = held;
        for reservation in group {
            peak = reservation_total(peak, reservation.reserved_bytes, maximum)?;
            ensure!(
                reservation.retained_bytes <= reservation.reserved_bytes,
                "native retained producer exceeded its prelaunch reservation"
            );
            retained = reservation_total(retained, reservation.retained_bytes, maximum)?;
        }
        held = retained;
        remaining = rest;
    }
    Ok(held)
}

/// The caller first authenticates the exact operation request and references.
/// Charge its retained closure, the immutable parent copies, and framing. A
/// shared identity is conservatively charged in each producer's closure.
pub(crate) fn retained_producer_bytes(
    invocation: &StrictInvocation,
    output: &translation_execution::operation::OperationResult,
    references: &[translation_execution::retention::Reference],
    run: &Path,
    label: &str,
) -> Result<u64> {
    let mut unique = std::collections::BTreeMap::new();
    for reference in references {
        if let Some(previous) = unique.insert(&reference.path, reference) {
            ensure!(previous == reference, "native retained producer reference changed");
        }
    }
    let mut bytes = 0u64;
    let mut add = |amount: u64| -> Result<()> {
        bytes = bytes.checked_add(amount).context("native retained producer extent overflow")?;
        Ok(())
    };
    for reference in unique.values() {
        add(reference.bytes)?;
    }
    for amount in [output.stdout.len(), output.stderr.len(), output.diagnostic.len()] {
        add(u64::try_from(amount)?)?;
    }
    for content in output.outputs.values() {
        add(u64::try_from(content.len())?)?;
    }
    if has_callback_snapshots(&invocation.program)
        && let Some(packed) = output.outputs.get(Path::new("callback-states.bin"))
    {
        // The callback adapter expands one full WRAM image per packed record
        // before the next producer. Charge those future parent files now. An
        // invalid stream cannot produce more images than this bounded count.
        let count = u64::try_from(packed.len() / (9 + 0x20000))?;
        add(count.checked_mul(0x20000).context("native callback expansion extent overflow")?)?;
    }
    for suffix in ["invocation.toml", "timing.toml"] {
        let path = run.join(format!("{label}.{suffix}"));
        bounded::safe(&path)?;
        let metadata = fs::symlink_metadata(&path)?;
        ensure!(metadata.is_file(), "native retained producer framing is not a regular file");
        add(metadata.len())?;
    }
    Ok(bytes)
}

impl StrictExecution {
    pub(crate) fn begin_fixture_group(&self) -> Result<()> {
        let mut group = self.reservation_group.lock().unwrap();
        ensure!(group.is_none(), "native reservation group already active");
        *group = Some(ReservationGroup {
            before_bytes: *self.reserved_bytes.lock().unwrap(),
            reserved_bytes: 0,
            completed_reserved_bytes: 0,
            retained_bytes: 0,
        });
        Ok(())
    }

    pub(crate) fn finish_fixture_group(&self) -> Result<()> {
        let mut group = self.reservation_group.lock().unwrap();
        let group = group.take().context("native reservation group missing")?;
        let mut held = self.reserved_bytes.lock().unwrap();
        ensure!(
            *held == reservation_total(group.before_bytes, group.reserved_bytes, self.max_output_bytes)?,
            "native reservation group ownership changed"
        );
        // Interrupted producers keep their original worst-case charge. A
        // failed wave still accounts for every producer it actually launched.
        let unresolved = group
            .reserved_bytes
            .checked_sub(group.completed_reserved_bytes)
            .context("native reservation group completion overflow")?;
        *held = reservation_total(
            reservation_total(group.before_bytes, group.retained_bytes, self.max_output_bytes)?,
            unresolved,
            self.max_output_bytes,
        )?;
        Ok(())
    }

    fn reservation_before(&self) -> u64 {
        let group = self.reservation_group.lock().unwrap();
        group.as_ref().map_or_else(|| *self.reserved_bytes.lock().unwrap(), |group| group.before_bytes)
    }

    pub(crate) fn compile(
        &self,
        command: &[String],
        run: &Path,
        label: &str,
        checkout: &Path,
        source: &Path,
    ) -> Result<ExecutionRecord> {
        let rust = command.first().is_some_and(|name| name == "rustc");
        self.compile_recipe(
            command,
            run,
            label,
            checkout,
            source,
            if rust { &["wrapper.rs", "memory.rs"] } else { &["oracle.c"] },
            if rust { &["rust", "compiler.d"] } else { &["c"] },
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compile_recipe(
        &self,
        command: &[String],
        run: &Path,
        label: &str,
        checkout: &Path,
        source: &Path,
        generated: &[&str],
        outputs: &[&str],
        headless: bool,
    ) -> Result<ExecutionRecord> {
        let rust = command.first().is_some_and(|name| name == "rustc");
        let executable = if rust { self.rustc.clone() } else { self.c_compiler.clone() };
        let mut inputs = Vec::new();
        for path in if rust { &self.rust_inputs } else { &self.c_inputs } {
            let relative = path.strip_prefix(if rust { checkout } else { source })?;
            inputs.push(ReadOnlyBind {
                source: path.clone(),
                destination: Path::new(if rust { "/inputs/rust" } else { "/inputs/c" }).join(relative),
            });
        }
        for name in generated {
            inputs
                .push(ReadOnlyBind { source: run.join(name), destination: Path::new("/inputs/generated").join(name) });
        }
        if headless {
            for name in ["headless.c", "headless_frame.h"] {
                inputs.push(ReadOnlyBind {
                    source: checkout.join("tools/decomp").join(name),
                    destination: Path::new("/inputs/rust/tools/decomp").join(name),
                });
            }
        }
        let mut arguments = Vec::new();
        for argument in &command[1..] {
            let mapped = if argument.starts_with("--emit=link,dep-info=") {
                "--emit=link,dep-info=/work/compiler.d".to_owned()
            } else {
                let path = Path::new(argument);
                if let Ok(relative) = path.strip_prefix(run) {
                    if outputs.iter().any(|name| relative == Path::new(name)) {
                        Path::new("/work").join(relative).display().to_string()
                    } else {
                        Path::new("/inputs/generated").join(relative).display().to_string()
                    }
                } else if let Ok(relative) = path.strip_prefix(source) {
                    Path::new("/inputs/c").join(relative).display().to_string()
                } else if let Ok(relative) = path.strip_prefix(checkout) {
                    Path::new("/inputs/rust").join(relative).display().to_string()
                } else {
                    argument.clone()
                }
            };
            arguments.push(mapped);
        }
        if rust {
            arguments.extend([
                "--sysroot".into(),
                "/toolchain".into(),
                "-C".into(),
                "linker=/usr/bin/cc".into(),
                "-C".into(),
                "link-arg=-B/usr/lib/gcc/x86_64-linux-gnu/9/".into(),
                "-C".into(),
                "link-arg=-B/usr/bin/".into(),
                "-C".into(),
                "linker-features=-lld".into(),
            ]);
        } else {
            // GCC normally locates subprocesses relative to argv[0] or PATH.
            // /program is deliberately not the host compiler path.
            arguments.extend(["-B/usr/lib/gcc/x86_64-linux-gnu/9/".into(), "-B/usr/bin/".into()]);
        }
        self.execute(
            &Program {
                executable,
                arguments,
                inputs,
                runtime: self.compiler_runtime.clone(),
                helper: self.helper.clone(),
                outputs: outputs.iter().map(PathBuf::from).collect(),
            },
            &self.compiler_limits,
            run,
            label,
        )
    }

    pub(crate) fn fixture(&self, command: &[String], directory: &Path, subject: &str) -> Result<ExecutionRecord> {
        self.fixture_recipe(command, directory, subject, StrictFixture::Family)
    }

    pub(crate) fn fixture_recipe(
        &self,
        command: &[String],
        directory: &Path,
        subject: &str,
        kind: StrictFixture,
    ) -> Result<ExecutionRecord> {
        self.execute(&strict_fixture_program(&self.helper, command, kind)?, &self.fixture_limits, directory, subject)
    }

    fn execute(&self, program: &Program, limits: &RunLimits, run: &Path, label: &str) -> Result<ExecutionRecord> {
        let started = Instant::now();
        let mut context = self.context.clone();
        context.operation_key = translation_execution::model::sealed(&(&self.context.operation_key, run, label))?;
        let request_sha256 = translation_execution::model::sealed(&(program, limits))?;
        let invocation_path = run.join(format!("{label}.invocation.toml"));
        let resumed = invocation_path.is_file();
        if !resumed {
            self.check_deadline()?;
        }
        let (_held, wait) = if resumed {
            (None, 0.0)
        } else {
            let (held, wait) = self.lease()?;
            (Some(held), wait)
        };
        let invocation = if resumed {
            let saved: StrictInvocation = crate::family::object(&invocation_path)?;
            ensure!(
                saved.context == context
                    && saved.request_sha256 == request_sha256
                    && saved.program == *program
                    && saved.requested_limits == *limits,
                "native invocation retried with changed inputs"
            );
            saved
        } else {
            self.check_deadline()?;
            let reserved_bytes = producer_reservation_bytes(program, limits, self.deadline_unix_ms)?;
            let remaining = self
                .deadline_unix_ms
                .saturating_sub(u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?);
            ensure!(remaining > 0, "native family aggregate time budget exhausted");
            let mut actual_limits = limits.clone();
            actual_limits.timeout = actual_limits.timeout.min(Duration::from_millis(remaining));
            let invocation = StrictInvocation {
                context: context.clone(),
                request_sha256,
                program: program.clone(),
                requested_limits: limits.clone(),
                limits: actual_limits,
                reserved_bytes,
                reservation_before_bytes: self.reservation_before(),
            };
            self.reserve(invocation.reserved_bytes)?;
            atomic_toml(&invocation_path, &invocation)?;
            invocation
        };
        let operation_directory = translation_execution::operation::directory(&self.operation_root, &context)?;
        let completed = operation_directory.join("result.json").is_file();
        let in_group = self.reservation_group.lock().unwrap().is_some();
        let held_reservation = !resumed || !completed || in_group;
        if resumed {
            let before = invocation.reservation_before_bytes;
            ensure!(before == self.reservation_before(), "native reservation prefix changed on replay");
            reservation_total(before, invocation.reserved_bytes, self.max_output_bytes)?;
            // Existing completed producers never launch again. Their original
            // peak is checked above, and publication reconciles their exact
            // retained charge before a later producer can be admitted.
            if held_reservation {
                self.reserve(invocation.reserved_bytes)?;
            }
        }
        if resumed && !operation_directory.join("intent.json").exists() {
            bail!("native invocation was interrupted before durable launch; explicit operator retry required");
        }
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        let output = if now >= self.deadline_unix_ms {
            ensure!(
                operation_directory.join("intent.json").is_file(),
                "native family aggregate time budget exhausted before launch"
            );
            if !operation_directory.join("result.json").exists() {
                translation_execution::operation::stop(&self.operation_root, &context)?;
            }
            translation_execution::operation::authenticate_request(
                &self.operation_root,
                &context,
                program,
                &invocation.limits,
                &[],
            )?
        } else {
            translation_execution::operation::run_until(
                &self.operation_root,
                &context,
                program,
                &invocation.limits,
                &[],
                self.deadline_unix_ms,
            )?
        };
        let publish = || -> Result<ExecutionRecord> {
            immutable_output(&run.join(format!("{label}.stdout")), &output.stdout)?;
            immutable_output(&run.join(format!("{label}.stderr")), &output.stderr)?;
            if !output.diagnostic.is_empty() {
                immutable_output(&run.join(format!("{label}.diagnostic.txt")), output.diagnostic.as_bytes())?;
            }
            for (name, bytes) in &output.outputs {
                let destination = strict_output_path(run, label, &name);
                fs::create_dir_all(destination.parent().context("strict output parent missing")?)?;
                immutable_output(&destination, bytes)?;
                #[cfg(unix)]
                if destination
                    .file_name()
                    .is_some_and(|name| ["rust", "c", "collision", "spawn", "ridley"].iter().any(|n| name == *n))
                {
                    use std::os::unix::fs::PermissionsExt;
                    if fs::metadata(&destination)?.permissions().mode() & 0o777 != 0o500 {
                        fs::set_permissions(&destination, fs::Permissions::from_mode(0o500))?;
                    }
                }
            }
            let record = ExecutionRecord {
                argv: std::iter::once(program.executable.display().to_string())
                    .chain(program.arguments.clone())
                    .collect(),
                seconds: started.elapsed().as_secs_f64(),
                worker_wait_seconds: wait,
                exit_code: output.status_raw.and_then(|raw| {
                    use std::os::unix::process::ExitStatusExt;
                    std::process::ExitStatus::from_raw(raw).code()
                }),
                error: (!output.passed).then(|| output.diagnostic.clone()),
            };
            let timing = run.join(format!("{label}.timing.toml"));
            if timing.is_file() {
                let saved: ExecutionRecord = crate::family::object(&timing)?;
                ensure!(
                    saved.argv == record.argv && saved.exit_code == record.exit_code && saved.error == record.error,
                    "native retained execution record differs"
                );
            } else {
                atomic_toml(&timing, &record)?;
            }
            Ok(record)
        };
        let record =
            if u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())? >= self.deadline_unix_ms {
                bounded::reconcile(8 * bounded::DEFAULT_BYTES, publish)?
            } else {
                publish()?
            };
        {
            let before = invocation.reservation_before_bytes;
            let reconcile = || -> Result<()> {
                let references = translation_execution::operation::references(&self.operation_root, &context)?;
                let retained = retained_producer_bytes(&invocation, &output, &references, run, label)?;
                ensure!(retained <= invocation.reserved_bytes, "native producer exceeded its prelaunch reservation");
                let mut group = self.reservation_group.lock().unwrap();
                if let Some(group) = group.as_mut() {
                    ensure!(held_reservation && before == group.before_bytes, "native reservation group changed");
                    group.completed_reserved_bytes = group
                        .completed_reserved_bytes
                        .checked_add(invocation.reserved_bytes)
                        .context("native completed group reservation overflow")?;
                    group.retained_bytes = group
                        .retained_bytes
                        .checked_add(retained)
                        .context("native retained group reservation overflow")?;
                    ensure!(group.completed_reserved_bytes <= group.reserved_bytes, "native group completed twice");
                    return Ok(());
                }
                let expected = if held_reservation {
                    reservation_total(before, invocation.reserved_bytes, self.max_output_bytes)?
                } else {
                    before
                };
                let mut held = self.reserved_bytes.lock().unwrap();
                ensure!(*held == expected, "native producer reservation ownership changed");
                *held = reservation_total(before, retained, self.max_output_bytes)?;
                Ok(())
            };
            if u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())? >= self.deadline_unix_ms {
                bounded::reconcile(8 * bounded::DEFAULT_BYTES, reconcile)?;
            } else {
                reconcile()?;
            }
        }
        ensure!(output.passed, "confined {label} failed: {}", output.diagnostic);
        Ok(record)
    }

    fn check_deadline(&self) -> Result<()> {
        ensure!(
            u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())? < self.deadline_unix_ms,
            "native family aggregate time budget exhausted before producer preparation"
        );
        Ok(())
    }

    fn reserve(&self, bytes: u64) -> Result<()> {
        let mut group = self.reservation_group.lock().unwrap();
        let mut used = self.reserved_bytes.lock().unwrap();
        *used = reservation_total(*used, bytes, self.max_output_bytes)?;
        if let Some(group) = group.as_mut() {
            group.reserved_bytes =
                group.reserved_bytes.checked_add(bytes).context("native group reservation overflow")?;
        }
        Ok(())
    }

    fn lease(&self) -> Result<(File, f64)> {
        ensure!((1..=64).contains(&self.workers), "native worker count missing");
        let started = Instant::now();
        let workers = self.root.join("workers");
        fs::create_dir_all(&workers)?;
        let guard = fs::OpenOptions::new().create(true).append(true).open(workers.join("config.lock"))?;
        loop {
            if guard.try_lock().is_ok() {
                break;
            }
            let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
            ensure!(now < self.deadline_unix_ms, "native gate deadline exhausted waiting for shared configuration");
            std::thread::sleep(Duration::from_millis(10));
        }
        let config = workers.join("capacity.toml");
        #[derive(Serialize, Deserialize)]
        struct Capacity {
            workers: usize,
        }
        if config.exists() {
            ensure!(
                crate::family::object::<Capacity>(&config)?.workers == self.workers,
                "shared native worker budget differs"
            );
        } else {
            atomic_toml(&config, &Capacity { workers: self.workers })?;
        }
        drop(guard);
        loop {
            crate::metrics::check_cancelled()?;
            for slot in 0..self.workers {
                let held =
                    fs::OpenOptions::new().create(true).append(true).open(workers.join(format!("{slot}.lock")))?;
                if held.try_lock().is_ok() {
                    return Ok((held, started.elapsed().as_secs_f64()));
                }
            }
            let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
            ensure!(now < self.deadline_unix_ms, "native gate deadline exhausted waiting for shared worker");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(test)]
mod strict_tests {
    use super::*;

    #[test]
    fn parallel_wave_reserves_the_sum_and_reconciles_only_at_the_join() -> Result<()> {
        let original = [
            ProducerReservation { before_bytes: 20, reserved_bytes: 100, retained_bytes: 10 },
            ProducerReservation { before_bytes: 20, reserved_bytes: 80, retained_bytes: 20 },
            ProducerReservation { before_bytes: 50, reserved_bytes: 100, retained_bytes: 5 },
        ];
        let mut records = original.clone();
        records.reverse();
        assert_eq!(authenticate_reservations(20, &mut records, 200)?, 55);
        // Both individual reservations fit 199; their simultaneous sum does not.
        assert!(authenticate_reservations(20, &mut original.clone(), 199).is_err());
        let mut early_reconciliation = original.clone();
        early_reconciliation[2].before_bytes = 30;
        assert!(authenticate_reservations(20, &mut early_reconciliation, 200).is_err());
        let mut retained_overflow = original.clone();
        retained_overflow[1].retained_bytes = 81;
        assert!(authenticate_reservations(20, &mut retained_overflow, 200).is_err());
        Ok(())
    }

    #[test]
    fn reconciled_producers_share_capacity_without_forgiving_retained_bytes() -> Result<()> {
        let original = [
            ProducerReservation { before_bytes: 20, reserved_bytes: 100, retained_bytes: 10 },
            ProducerReservation { before_bytes: 30, reserved_bytes: 100, retained_bytes: 25 },
        ];
        let mut records = original.clone();
        records.reverse();
        assert_eq!(authenticate_reservations(20, &mut records, 130)?, 55);
        assert!(reservation_total(120, 100, 130).is_err());
        let mut forged = original.clone();
        forged[1].before_bytes = 29;
        assert!(authenticate_reservations(20, &mut forged, 130).is_err());
        let mut excess = original.clone();
        excess[0].retained_bytes = 101;
        assert!(authenticate_reservations(20, &mut excess, 130).is_err());
        let mut peak = original.clone();
        peak[1].reserved_bytes = 101;
        assert!(authenticate_reservations(20, &mut peak, 130).is_err());
        assert!(reservation_total(u64::MAX, 1, u64::MAX).is_err());
        Ok(())
    }

    #[test]
    #[ignore = "real bounded native producer accounting and replay proof; retains every result"]
    fn real_producers_reconcile_unused_reservations_and_replay() -> Result<()> {
        let root = PathBuf::from(std::env::var("SM_NATIVE_RESERVATION_PROOF")?);
        fs::create_dir_all(root.join("run"))?;
        let helper = PathBuf::from(std::env::var("SM_NATIVE_RECIPE_HELPER")?);
        let program = Program {
            executable: fs::canonicalize("/usr/bin/printf")?,
            arguments: vec!["native reservation proof\\n".into()],
            inputs: vec![],
            runtime: confinement::helper_runtime()?,
            helper: helper.clone(),
            outputs: vec![],
        };
        let limits = RunLimits {
            timeout: Duration::from_secs(5),
            writable_bytes: 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        };
        let worst = producer_reservation_bytes(&program, &limits, bounded::now_unix_ms()? + 60_000)?;
        let maximum = worst.checked_add(96 * 1024).context("proof budget overflow")?;
        let context = OperationContext {
            run_id: translation_execution::model::sha256(root.display().to_string()),
            attempt: 1,
            policy_sha256: translation_execution::model::sha256("native producer reservation proof"),
            operation_key: translation_execution::model::sha256("native producer reservation proof"),
            phase: translation_execution::model::ExecutionPhase::Candidate,
        };
        let make = |deadline_unix_ms, max_output_bytes| StrictExecution {
            helper: helper.clone(),
            rustc: root.join("unused-rustc"),
            c_compiler: root.join("unused-cc"),
            compiler_runtime: vec![],
            operation_root: root.join("operations"),
            context: context.clone(),
            root: "/home/jim/sm-decomp/out/pipeline".into(),
            workers: 4,
            compiler_limits: limits.clone(),
            fixture_limits: limits.clone(),
            rust_inputs: vec![],
            c_inputs: vec![],
            deadline_unix_ms,
            max_output_bytes,
            reserved_bytes: Mutex::new(0),
            reservation_group: Mutex::new(None),
        };
        let execution = make(bounded::now_unix_ms()? + 60_000, maximum);
        for label in ["first", "second"] {
            execution.execute(&program, &limits, &root.join("run"), label)?;
        }
        let mut failed_program = program.clone();
        failed_program.arguments = vec!["%".into()];
        let before_failure = *execution.reserved_bytes.lock().unwrap();
        let failure = execution.execute(&failed_program, &limits, &root.join("run"), "failed").unwrap_err();
        ensure!(failure.to_string().contains("confined failed failed"), "unexpected producer failure: {failure}");
        let retained = *execution.reserved_bytes.lock().unwrap();
        ensure!(retained > before_failure && retained < worst, "completed failed producer was not charged precisely");
        let mut results = Vec::new();
        for label in ["first", "second", "failed"] {
            let invocation: StrictInvocation =
                crate::family::object(&root.join("run").join(format!("{label}.invocation.toml")))?;
            let result = translation_execution::operation::reference(&root.join("operations"), &invocation.context)?;
            results.push((result.path.clone(), fs::metadata(&result.path)?.modified()?));
        }
        let retry = make(0, maximum);
        for label in ["first", "second"] {
            retry.execute(&program, &limits, &root.join("run"), label)?;
        }
        assert!(retry.execute(&failed_program, &limits, &root.join("run"), "failed").is_err());
        ensure!(*retry.reserved_bytes.lock().unwrap() == retained, "replay changed native retained reservation");
        for (path, modified) in results {
            ensure!(fs::metadata(path)?.modified()? == modified, "reservation replay rewrote a producer");
        }
        let denied = make(bounded::now_unix_ms()? + 60_000, worst - 1);
        assert!(denied.execute(&program, &limits, &root.join("run"), "denied").is_err());
        ensure!(!root.join("run/denied.invocation.toml").exists(), "over-budget producer was declared");
        atomic_toml(
            &root.join("summary.toml"),
            &std::collections::BTreeMap::from([
                ("producer_worst_case_bytes", worst),
                ("gate_bytes", maximum),
                ("retained_charge_bytes", retained),
            ]),
        )?;
        for deduplicate in [false, true] {
            let cleanup = crate::cleanup::cleanup(crate::cleanup::CleanupRequest {
                root: root.clone(),
                apply: true,
                deduplicate,
            })?;
            atomic_toml(&root.join(if deduplicate { "cleanup-deduplicated.toml" } else { "cleanup.toml" }), &cleanup)?;
        }
        let after_cleanup = make(0, maximum);
        for label in ["first", "second"] {
            after_cleanup.execute(&program, &limits, &root.join("run"), label)?;
        }
        assert!(after_cleanup.execute(&failed_program, &limits, &root.join("run"), "failed").is_err());
        ensure!(
            *after_cleanup.reserved_bytes.lock().unwrap() == retained,
            "cleanup changed native reservation authentication"
        );
        Ok(())
    }

    #[test]
    fn expired_gate_cannot_reserve_runtime_or_publish_a_producer_invocation() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "sm-expired-producer-{}-{}",
            std::process::id(),
            crate::family::unique_id()
        ));
        let limits = RunLimits {
            timeout: Duration::from_secs(1),
            writable_bytes: 4096,
            stdout_bytes: 0,
            stderr_bytes: 0,
            stdin_bytes: 0,
        };
        let execution = StrictExecution {
            helper: root.join("helper"),
            rustc: root.join("rustc"),
            c_compiler: root.join("cc"),
            compiler_runtime: Vec::new(),
            operation_root: root.join("operations"),
            context: OperationContext {
                run_id: translation_execution::model::sha256("expired"),
                attempt: 1,
                policy_sha256: translation_execution::model::sha256("policy"),
                operation_key: translation_execution::model::sha256("operation"),
                phase: translation_execution::model::ExecutionPhase::Candidate,
            },
            root: root.join("pipeline"),
            workers: 4,
            compiler_limits: limits.clone(),
            fixture_limits: limits.clone(),
            rust_inputs: Vec::new(),
            c_inputs: Vec::new(),
            deadline_unix_ms: 0,
            max_output_bytes: 4096,
            reserved_bytes: Mutex::new(0),
            reservation_group: Mutex::new(None),
        };
        let program = Program {
            executable: root.join("program"),
            arguments: Vec::new(),
            inputs: Vec::new(),
            runtime: Vec::new(),
            helper: root.join("helper"),
            outputs: Vec::new(),
        };
        let error = execution.execute(&program, &limits, &root.join("run"), "c-build").unwrap_err();
        ensure!(
            error.to_string().contains("time budget exhausted"),
            "expired gate attempted runtime reservation: {error}"
        );
        ensure!(
            !root.exists() && *execution.reserved_bytes.lock().unwrap() == 0,
            "expired gate created a producer or resource reservation"
        );
        Ok(())
    }

    #[test]
    fn compiler_runtime_and_source_discovery_share_inherited_bounds() -> Result<()> {
        let directory = bounded::tests::Directory::new()?;
        let error = strict_compiler_runtime_until(&directory.0.join("unavailable-rustc"), 0, 1024).unwrap_err();
        assert!(error.to_string().contains("deadline"));
        fs::write(directory.0.join("source.rs"), b"12345")?;
        let error =
            bounded::until(bounded::now_unix_ms()? + 1000, 4, || source_files(&directory.0, "rs", &mut Vec::new()))
                .unwrap_err();
        assert!(error.to_string().contains("budget"));
        let mut path = directory.0.clone();
        for _ in 0..34 {
            path.push("nested");
            fs::create_dir(&path)?;
        }
        assert!(source_files(&directory.0, "h", &mut Vec::new()).unwrap_err().to_string().contains("depth"));
        Ok(())
    }
}

fn immutable_output(path: &Path, bytes: &[u8]) -> Result<()> {
    crate::inputs::immutable_bytes(path, bytes)
}

/// Read-only files needed by the host's installed compiler/linker. This exposes
/// neither a host checkout nor a controller executable. Runtime imports are
/// frozen by the durable operation runner before executing rewritten Rust.
pub(crate) fn strict_compiler_runtime(rustc: &Path) -> Result<Vec<ReadOnlyBind>> {
    bounded::limited(|| strict_compiler_runtime_bounded(rustc))
}

pub(crate) fn strict_compiler_runtime_until(rustc: &Path, deadline: u64, max_bytes: u64) -> Result<Vec<ReadOnlyBind>> {
    bounded::until(deadline, max_bytes, || strict_compiler_runtime_bounded(rustc))
}

fn strict_compiler_runtime_bounded(rustc: &Path) -> Result<Vec<ReadOnlyBind>> {
    let rustc = fs::canonicalize(rustc)?;
    let toolchain = rustc.parent().and_then(Path::parent).context("compiler toolchain root missing")?;
    ensure!(toolchain.join("bin/rustc") == rustc && toolchain.join("lib/rustlib").is_dir(), "installed rustc required");
    let mut bindings = confinement::helper_runtime()?;
    for (source, destination) in [
        ("/usr/bin/x86_64-linux-gnu-gcc-9", "/usr/bin/cc"),
        ("/usr/bin/x86_64-linux-gnu-as", "/usr/bin/as"),
        ("/usr/bin/x86_64-linux-gnu-ld.bfd", "/usr/bin/ld"),
        ("/usr/lib/gcc/x86_64-linux-gnu/9/cc1", "/usr/lib/gcc/x86_64-linux-gnu/9/cc1"),
        ("/usr/lib/gcc/x86_64-linux-gnu/9/collect2", "/usr/lib/gcc/x86_64-linux-gnu/9/collect2"),
        ("/usr/lib/gcc/x86_64-linux-gnu/9/include", "/usr/lib/gcc/x86_64-linux-gnu/9/include"),
        ("/usr/include", "/usr/include"),
    ] {
        bounded::check()?;
        bindings.push(ReadOnlyBind { source: fs::canonicalize(source)?, destination: destination.into() });
    }
    // GCC's non-LTO link closure; enumerate files so unrelated installed shared
    // libraries, system programs and package caches are inaccessible.
    for directory in ["/usr/lib/gcc/x86_64-linux-gnu/9", "/usr/lib/x86_64-linux-gnu"] {
        for path in bounded::entries(Path::new(directory))? {
            let name = path.file_name().context("runtime filename missing")?.to_string_lossy();
            if (name.starts_with("crt") && name.ends_with(".o"))
                || ["Scrt1.o", "rcrt1.o"].contains(&name.as_ref())
                || [
                    "libgcc.a",
                    "libgcc_eh.a",
                    "libgcc_s.so",
                    "liblto_plugin.so",
                    "libc.so",
                    "libc.a",
                    "libc_nonshared.a",
                    "libm.so",
                    "libm.a",
                    "libmvec.so",
                    "libmvec.a",
                    "libpthread.so",
                    "libpthread.a",
                    "libpthread_nonshared.a",
                    "libdl.so",
                    "libdl.a",
                    "librt.so",
                    "librt.a",
                    "libutil.so",
                    "libutil.a",
                ]
                .contains(&name.as_ref())
            {
                bindings.push(ReadOnlyBind { source: fs::canonicalize(&path)?, destination: path });
            }
        }
    }
    // Dynamic linker/search aliases referenced by libc and gcc's link scripts.
    for path in [
        "/usr/lib/x86_64-linux-gnu/libgcc_s.so.1",
        "/lib/x86_64-linux-gnu/libc.so.6",
        "/lib/x86_64-linux-gnu/libm.so.6",
        "/lib/x86_64-linux-gnu/libmvec.so.1",
        "/lib/x86_64-linux-gnu/libpthread.so.0",
        "/lib/x86_64-linux-gnu/libdl.so.2",
        "/lib/x86_64-linux-gnu/librt.so.1",
        "/lib/x86_64-linux-gnu/libutil.so.1",
        "/lib/x86_64-linux-gnu/libz.so.1",
        "/lib/x86_64-linux-gnu/libmpfr.so.6",
        "/lib/x86_64-linux-gnu/libmpc.so.3",
        "/lib/x86_64-linux-gnu/libgmp.so.10",
        "/lib/x86_64-linux-gnu/libisl.so.22",
        "/lib/x86_64-linux-gnu/libexpat.so.1",
        "/lib/x86_64-linux-gnu/libzstd.so.1",
        "/lib/x86_64-linux-gnu/libstdc++.so.6",
        "/lib/x86_64-linux-gnu/libtinfo.so.6",
        "/lib/x86_64-linux-gnu/libxml2.so.2",
        "/lib/x86_64-linux-gnu/liblzma.so.5",
    ] {
        bounded::check()?;
        if Path::new(path).exists() && !bindings.iter().any(|binding| binding.destination == Path::new(path)) {
            bindings.push(ReadOnlyBind { source: fs::canonicalize(path)?, destination: path.into() });
        }
    }
    for path in [
        "/lib/x86_64-linux-gnu/libopcodes-2.34-system.so",
        "/lib/x86_64-linux-gnu/libbfd-2.34-system.so",
        "/lib/x86_64-linux-gnu/libctf.so.0",
    ] {
        bounded::check()?;
        bindings.push(ReadOnlyBind { source: fs::canonicalize(path)?, destination: path.into() });
    }
    bindings.push(ReadOnlyBind {
        source: fs::canonicalize("/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")?,
        destination: "/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2".into(),
    });
    bindings.push(ReadOnlyBind { source: toolchain.join("lib"), destination: "/toolchain/lib".into() });
    ensure!(bindings.len() <= 1024, "installed compiler runtime binding inventory exceeded");
    bounded::count(bindings.len())?;
    Ok(bindings)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExecRequest {
    pub command: Vec<String>,
    pub run: PathBuf,
    pub label: String,
    pub root: PathBuf,
    pub workers: usize,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub sampler_paths: Vec<SamplerPath>,
}
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SamplerPath {
    path: PathBuf,
    kind: crate::resource_sampler::PathKind,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CBuildRequest {
    pub command: Vec<String>,
    pub run: PathBuf,
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
}
fn default_timeout() -> u64 {
    240
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoBuildRequest {
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub run: PathBuf,
    pub package: String,
    pub member: String,
    pub destination: PathBuf,
    pub workers: usize,
}
/// Fixed standalone helper recipes. Generated Live replay families contain
/// run-specific source paths and deliberately do not use this cache.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Helper {
    FixtureWorker,
    RandomLibrary,
    RandomCli,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HelperBuildRequest {
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub run: PathBuf,
    pub workers: usize,
    pub helpers: Vec<Helper>,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct HelperBuildResult {
    pub artifacts: Vec<CacheResult>,
}

pub(crate) fn helper_builds(request: HelperBuildRequest) -> Result<HelperBuildResult> {
    fs::create_dir_all(&request.run)?;
    let mut metrics = crate::metrics::Run::start(request.run.join("helper-build.metrics.toml"), "helper-build")?;
    let result = helper_builds_prepared(request);
    let error = result.as_ref().err().map(|error| format!("{error:#}"));
    let finished = metrics.finish(crate::metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finished?;
    }
    result
}

fn helper_builds_prepared(request: HelperBuildRequest) -> Result<HelperBuildResult> {
    let HelperBuildRequest { checkout, root, run, workers, helpers } = request;
    ensure!(!helpers.is_empty() && helpers.len() <= 3, "invalid helper build extent");
    fs::create_dir_all(&run)?;
    let checkout = fs::canonicalize(checkout)?;
    // Resolve the project's selected toolchain once. A run outside checkout
    // must not silently choose a different rustup override for scan or build.
    execute_at(
        &["rustc".into(), "--print".into(), "sysroot".into()],
        &run,
        &checkout,
        "helper-toolchain",
        &root,
        workers,
        240,
    )?;
    let sysroot = fs::read_to_string(run.join("helper-toolchain.stdout"))?;
    let compiler = fs::canonicalize(Path::new(sysroot.trim()).join("bin/rustc"))?;
    let compiler = compiler.display().to_string();
    let preparation = Preparation::new(&root, &checkout, &compiler)?;
    let mut artifacts = Vec::new();
    for helper in helpers {
        let (source, name, label, flags): (_, _, _, &[&str]) = match helper {
            Helper::FixtureWorker => (
                "crates/pipeline/src/fixture_worker.rs",
                "libfixture_worker.a",
                "fixture-worker-build",
                &["--crate-type=staticlib"],
            ),
            Helper::RandomLibrary => {
                ("tools/decomp/next_random.rs", "librng.a", "rng-build", &["--crate-type=staticlib"])
            }
            Helper::RandomCli => ("tools/decomp/next_random.rs", "rng-cli", "rng-cli-build", &["--cfg", "cli"]),
        };
        let source = checkout.join(source);
        preparation.freeze(std::slice::from_ref(&source))?;
        let mut command = vec![compiler.clone(), "--edition=2024".into(), "-O".into()];
        command.extend(flags.iter().map(|s| (*s).to_owned()));
        command.push(source.display().to_string());
        // Always rediscover the actual compiler closure, including include! and
        // cfg-dependent modules. A cache hit must not hide a newly added input.
        let dep_file = run.join(format!("{label}.compiler.d"));
        let mut dependency = command.clone();
        dependency.push(format!("--emit=dep-info={}", dep_file.display()));
        execute(&dependency, &run, &format!("{label}-dependencies"), &root, workers)?;
        let dependency_text = fs::read_to_string(&dep_file)?;
        // These standalone recipes currently use no env!/option_env! input.
        // Fail closed if that changes instead of hiding an arbitrary variable
        // behind the cache's established compiler-environment allowlist.
        ensure!(
            !dependency_text.lines().any(|line| line.starts_with("# env-dep:")),
            "standalone helper has an unsupported build-time environment dependency"
        );
        let mut inputs: Vec<_> = crate::inputs::dep_paths(&dependency_text)?
            .into_iter()
            .map(|path| {
                let path = PathBuf::from(path);
                if path.is_absolute() { path } else { run.join(path) }
            })
            .collect();
        inputs.push(source);
        command.extend(["-o".into(), "{artifact}".into()]);
        let result = preparation.artifact(&command, &inputs, &run.join(name), |artifact| {
            let mut actual = command.clone();
            *actual.last_mut().unwrap() = artifact.display().to_string();
            let actual_dependencies = artifact.with_extension("d");
            actual.push(format!("--emit=link,dep-info={}", actual_dependencies.display()));
            execute(&actual, &run, label, &root, workers)?;
            let actual_text = fs::read_to_string(&actual_dependencies)?;
            ensure!(
                !actual_text.lines().any(|line| line.starts_with("# env-dep:")),
                "standalone helper has an unsupported build-time environment dependency"
            );
            authenticate_dependency_closure(&inputs, &actual_text, &run)?;
            fs::remove_file(actual_dependencies)?;
            Ok(())
        })?;
        atomic_toml(&run.join(format!("{label}.cache.toml")), &result)?;
        artifacts.push(result);
    }
    preparation.finish()?;
    Ok(HelperBuildResult { artifacts })
}

fn dependency_paths(text: &str, run: &Path) -> Result<Vec<PathBuf>> {
    crate::inputs::dep_paths(text)?
        .into_iter()
        .map(|name| {
            let path = PathBuf::from(name);
            Ok(fs::canonicalize(if path.is_absolute() { path } else { run.join(path) })?)
        })
        .collect()
}

fn authenticate_dependency_closure(expected: &[PathBuf], actual: &str, run: &Path) -> Result<()> {
    let expected: std::collections::BTreeSet<_> =
        expected.iter().map(fs::canonicalize).collect::<std::io::Result<_>>()?;
    let actual: std::collections::BTreeSet<_> = dependency_paths(actual, run)?.into_iter().collect();
    ensure!(expected == actual, "compiler dependency closure changed during build");
    Ok(())
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ExecutionRecord {
    pub argv: Vec<String>,
    pub seconds: f64,
    pub worker_wait_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn headers(directory: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    source_files(directory, "h", result)
}

pub(crate) fn execute(
    command: &[String],
    run: &Path,
    label: &str,
    root: &Path,
    workers: usize,
) -> Result<ExecutionRecord> {
    execute_timeout(command, run, label, root, workers, 240)
}
pub(crate) fn execute_timeout(
    command: &[String],
    run: &Path,
    label: &str,
    root: &Path,
    workers: usize,
    timeout: u64,
) -> Result<ExecutionRecord> {
    execute_at(command, run, run, label, root, workers, timeout)
}

fn execute_at(
    command: &[String],
    run: &Path,
    cwd: &Path,
    label: &str,
    root: &Path,
    workers: usize,
    timeout: u64,
) -> Result<ExecutionRecord> {
    ensure!(!command.is_empty(), "empty process command");
    ensure!(label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'), "unsafe process label");
    fs::create_dir_all(run)?;
    let mut metrics = crate::metrics::Run::start(run.join(format!("{label}.metrics.toml")), label)?;
    let result = (|| -> Result<ExecutionRecord> {
        let started = Instant::now();
        let waiting = metrics.context.stage("worker-wait")?;
        let (_held, wait) = acquire_lease(root, workers)?;
        waiting.finish("success")?;
        let building = metrics.context.stage("execute")?;
        let active = Instant::now();
        let mut actual = Command::new(&command[0]);
        actual
            .args(&command[1..])
            .current_dir(cwd)
            .stdout(Stdio::from(File::create(run.join(format!("{label}.stdout")))?))
            .stderr(Stdio::from(File::create(run.join(format!("{label}.stderr")))?));
        let mut child = crate::metrics::OwnedChild::spawn(&mut actual, &metrics.context, label)?;
        let status = loop {
            if let Some(status) = child.poll()? {
                break status;
            }
            if active.elapsed() > std::time::Duration::from_secs(timeout) {
                child.kill_wait()?;
                atomic_toml(
                    &run.join(format!("{label}.timing.toml")),
                    &ExecutionRecord {
                        argv: command.to_vec(),
                        seconds: started.elapsed().as_secs_f64(),
                        worker_wait_seconds: wait,
                        exit_code: None,
                        error: Some("compiler timeout".into()),
                    },
                )?;
                bail!("{label} timeout after {timeout}s");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        let record = ExecutionRecord {
            argv: command.to_vec(),
            seconds: started.elapsed().as_secs_f64(),
            worker_wait_seconds: wait,
            exit_code: status.code(),
            error: None,
        };
        atomic_toml(&run.join(format!("{label}.timing.toml")), &record)?;
        ensure!(status.success(), "{label} failed: {status}");
        building.finish("success")?;
        Ok(record)
    })();
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    metrics.finish(crate::metrics::outcome(error.as_deref()), error)?;
    result
}

pub fn exec(request: ExecRequest) -> Result<ExecutionRecord> {
    let mut metrics = crate::metrics::Run::start(request.run.join(format!("{}.metrics.toml", request.label)), "exec")?;
    for path in request.sampler_paths {
        metrics.context.add_path(path.path, path.kind);
    }
    metrics.context.sample_disk();
    let result = execute_timeout(
        &request.command,
        &request.run,
        &request.label,
        &request.root,
        request.workers,
        request.timeout,
    );
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    let finish = metrics.finish(crate::metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finish?;
    }
    result
}

fn rust_sources(directory: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    source_files(directory, "rs", result)
}

fn source_files(directory: &Path, extension: &str, result: &mut Vec<PathBuf>) -> Result<()> {
    bounded::limited(|| {
        let mut pending = vec![(directory.to_path_buf(), 0usize)];
        while let Some((directory, depth)) = pending.pop() {
            bounded::check()?;
            ensure!(depth <= 32, "compiler source discovery depth exceeded");
            for path in bounded::entries(&directory)? {
                bounded::safe(&path)?;
                let metadata = path.symlink_metadata()?;
                if metadata.is_dir() {
                    pending.push((path, depth + 1));
                } else if path.extension().is_some_and(|value| value == extension) {
                    ensure!(metadata.is_file(), "compiler source is not a regular file");
                    ensure!(result.len() < 4096, "compiler source discovery file count exceeded");
                    bounded::account(metadata.len())?;
                    result.push(path);
                }
            }
        }
        Ok(())
    })
}

pub fn cargo_build(request: CargoBuildRequest) -> Result<CacheResult> {
    fs::create_dir_all(&request.run)?;
    let mut metrics =
        crate::metrics::Run::start(request.run.join(format!("{}-cache.metrics.toml", request.package)), "cargo-cache")?;
    let result = (|| -> Result<CacheResult> {
        let checkout = fs::canonicalize(request.checkout)?;
        let root = request.root;
        let run = request.run;
        let package = request.package.as_str();
        let member = request.member.as_str();
        ensure!(
            (package == "evidence" && member == "sm-evidence") || (package == "decomp" && member == "libdecomp.a"),
            "Cargo input closure is defined only for evidence and decomp"
        );
        let mut inputs = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"].map(|p| checkout.join(p)).to_vec();
        inputs.push(checkout.join(format!("crates/{package}/Cargo.toml")));
        rust_sources(&checkout.join(format!("crates/{package}/src")), &mut inputs)?;
        if package == "evidence" {
            rust_sources(&checkout.join("crates/pipeline-model/src"), &mut inputs)?;
        }
        for path in bounded::entries(&checkout.join("crates"))? {
            let manifest = path.join("Cargo.toml");
            if manifest.is_file() {
                inputs.push(manifest);
            }
        }
        let config = checkout.join(".cargo");
        if config.is_dir() {
            for p in bounded::entries(&config)? {
                if p.is_file() {
                    inputs.push(p);
                }
            }
        }
        let external = checkout.join("deps/rustysnes/crates/rustysnes-core/Cargo.toml");
        if external.exists() {
            inputs.push(external);
        }
        let command = vec![
            "cargo".to_owned(),
            "build".into(),
            "--locked".into(),
            "--offline".into(),
            "--release".into(),
            "-p".into(),
            package.into(),
        ];
        let workers = request.workers;
        cached_artifact(
            &CacheRequest {
                root: root.clone(),
                checkout: checkout.clone(),
                command: command.clone(),
                inputs,
                destination: request.destination,
            },
            |artifact| {
                let target = artifact.parent().unwrap().join("target");
                let mut actual = command.clone();
                actual.extend([
                    "--jobs".into(),
                    "1".into(),
                    "--target-dir".into(),
                    target.to_string_lossy().into_owned(),
                ]);
                // Cargo's working directory must select this checkout, while logs stay in the run.
                fs::create_dir_all(&run)?;
                let waiting = metrics.context.stage("cargo-worker-wait")?;
                let (_held, _) = acquire_lease(&root, workers)?;
                waiting.finish("success")?;
                let mut command = Command::new(&actual[0]);
                command
                    .args(&actual[1..])
                    .current_dir(&checkout)
                    .stdout(File::create(run.join(format!("{package}-build.stdout")))?)
                    .stderr(File::create(run.join(format!("{package}-build.stderr")))?);
                let status = metrics.context.status(&mut command, "cargo-build")?;
                ensure!(status.success(), "{package} build failed: {status}");
                fs::copy(target.join("release").join(member), artifact)?;
                fs::remove_dir_all(target)?;
                Ok(())
            },
        )
    })();
    if let Ok(cache) = &result {
        metrics.context.configuration("cache-key", &cache.key);
        metrics.context.configuration("cache-hit", cache.hit);
    }
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    let finished = metrics.finish(crate::metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finished?;
    }
    result
}

pub fn c_build(request: CBuildRequest) -> Result<ExecutionRecord> {
    fs::create_dir_all(&request.run)?;
    // Install cancellation once on the parent. Thread-local child scopes then
    // inherit this context rather than racing to replace signal handlers.
    let mut metrics = crate::metrics::Run::start(request.run.join("c-build.metrics.toml"), "c-build")?;
    metrics.context.configuration("workers", request.workers);
    let result = c_build_prepared(request);
    let error = result.as_ref().err().map(|error| format!("{error:#}"));
    let finished = metrics.finish(crate::metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finished?;
    }
    result
}

fn c_build_prepared(request: CBuildRequest) -> Result<ExecutionRecord> {
    let CBuildRequest { run, checkout, root, workers, command } = request;
    ensure!(!command.is_empty(), "empty C command");
    ensure!((1..=64).contains(&workers), "invalid C worker count");
    let mut flags = Vec::new();
    let mut sources = Vec::new();
    let mut inputs = Vec::new();
    let mut i = 1;
    while i < command.len() {
        let arg = &command[i];
        if arg.ends_with(".c") {
            sources.push(PathBuf::from(arg));
        } else if ["-I", "-iquote", "-isystem"].contains(&arg.as_str()) {
            let include = command.get(i + 1).context("include directory missing")?;
            headers(Path::new(include), &mut inputs)?;
            flags.extend([arg.clone(), include.clone()]);
            i += 1;
        } else if arg == "-o" {
            ensure!(i + 1 < command.len(), "output missing");
            i += 1;
        } else if arg.starts_with("-Wl,") || arg.starts_with("-l") || arg.ends_with(".a") || arg.ends_with(".o") {
        } else if arg.starts_with("-O")
            || arg.starts_with("-f")
            || arg.starts_with("-D")
            || arg.starts_with("-U")
            || arg.starts_with("-std=")
            || arg.starts_with("-m")
            || arg.starts_with("-W")
            || arg == "-pthread"
            || arg == "-g"
        {
            flags.push(arg.clone());
        } else {
            bail!("unsupported C build option: {arg}");
        }
        i += 1;
    }
    ensure!(!sources.is_empty(), "C build has no translation units");
    let pinned = fs::canonicalize(checkout.join("deps"))?;
    let preparation = Preparation::new(&root, &checkout, &command[0])?;
    preparation.freeze(&inputs)?;
    let objects: Vec<_> = (0..sources.len()).map(|index| run.join(format!("c-object-{index:03}.o"))).collect();
    let compile_unit = |index: usize| -> Result<Option<CacheResult>> {
        let source = &sources[index];
        let source = fs::canonicalize(if source.is_absolute() { source.clone() } else { run.join(source) })?;
        let destination = &objects[index];
        let mut compile = vec![command[0].clone()];
        compile.extend(flags.clone());
        compile.extend(["-c".into(), source.to_string_lossy().into_owned(), "-o".into(), "{artifact}".into()]);
        let build = |artifact: &Path, expected: Option<&[PathBuf]>| -> Result<()> {
            let mut actual = compile.clone();
            *actual.last_mut().unwrap() = artifact.to_string_lossy().into_owned();
            let dep_file = artifact.with_extension("d");
            if expected.is_some() {
                actual.extend(["-MD".into(), "-MF".into(), dep_file.display().to_string()]);
            }
            execute(&actual, &run, &format!("c-object-{index:03}"), &root, workers)?;
            if let Some(expected) = expected {
                authenticate_dependency_closure(expected, &fs::read_to_string(&dep_file)?, &run)?;
                fs::remove_file(dep_file)?;
            }
            Ok(())
        };
        if source.starts_with(&pinned) {
            preparation.freeze(std::slice::from_ref(&source))?;
            // Ask the compiler for its complete preprocessor closure, including
            // system headers and textually included sources. Header edits and
            // changed include search paths must invalidate even a warm object.
            let (_held, _) = acquire_lease(&root, workers)?;
            let dependencies = crate::metrics::command_output(
                Command::new(&command[0]).args(&flags).arg("-M").arg(&source).current_dir(&run),
                "c-dependency-scan",
            )?;
            drop(_held);
            ensure!(
                dependencies.status.success(),
                "C dependency scan failed: {}",
                String::from_utf8_lossy(&dependencies.stderr)
            );
            let discovered = dependency_paths(&String::from_utf8(dependencies.stdout)?, &run)?;
            let mut closure = inputs.clone();
            closure.extend(discovered.clone());
            closure.push(source);
            Ok(Some(
                preparation.artifact(&compile, &closure, destination, |artifact| build(artifact, Some(&discovered)))?,
            ))
        } else {
            build(destination, None)?;
            Ok(None)
        }
    };
    // A fixed number of local workers feed the same cross-process leases.
    // Waiting on one cache key cannot prevent unrelated objects progressing.
    let next = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let context = crate::metrics::active();
    let mut completed = std::thread::scope(|scope| -> Result<Vec<_>> {
        let handles: Vec<_> = (0..workers.min(sources.len()))
            .map(|_| {
                scope.spawn(|| {
                    let _active = context.as_ref().map(|context| context.enter());
                    let mut completed = Vec::new();
                    while !failed.load(Ordering::Acquire) {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        if index >= sources.len() {
                            break;
                        }
                        let result = compile_unit(index);
                        if result.is_err() {
                            preparation.abort();
                            failed.store(true, Ordering::Release);
                        }
                        completed.push((index, result));
                    }
                    completed
                })
            })
            .collect();
        let mut completed = Vec::new();
        for handle in handles {
            completed.extend(handle.join().map_err(|_| anyhow::anyhow!("C build worker panicked"))?);
        }
        Ok(completed)
    })?;
    completed.sort_by_key(|(index, _)| *index);
    // A lower-index sibling cancelled during a lock wait must not hide the
    // compiler diagnostic that actually failed. Preserve source-order errors.
    if let Some(index) = completed.iter().position(|(_, result)| {
        result.as_ref().err().is_some_and(|error| !error.is::<crate::build_cache::PreparationAborted>())
    }) {
        return Err(completed.remove(index).1.unwrap_err());
    }
    let records: Vec<_> =
        completed.into_iter().map(|(_, result)| result).collect::<Result<Vec<_>>>()?.into_iter().flatten().collect();
    preparation.finish()?;
    let source_names: Vec<_> = sources.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let mut linked: Vec<String> = command.into_iter().filter(|arg| !source_names.contains(arg)).collect();
    linked.splice(1..1, objects.iter().map(|p| p.to_string_lossy().into_owned()));
    let result = execute(&linked, &run, "c-link", &root, workers);
    preparation.finish()?;
    if result.is_ok() {
        for p in objects {
            fs::remove_file(p)?;
        }
    }
    #[derive(Serialize)]
    struct CacheRecords {
        objects: Vec<CacheResult>,
    }
    atomic_toml(&run.join("c-cache.toml"), &CacheRecords { objects: records })?;
    result
}
