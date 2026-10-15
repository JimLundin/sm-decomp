//! Translation-unit reuse; generated candidate units are always rebuilt.
use crate::atomic_toml;
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tool_execution::bounded;
#[cfg(test)]
use tool_execution::builds::source_files;
use tool_execution::{build_cache::CacheResult, builds::ExecutionRecord};
#[cfg(test)]
use translation_execution::confinement;
use translation_execution::{
    OperationContext,
    confinement::{Program, ReadOnlyBind, RunLimits},
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

use crate::recipe::{FixtureKind, confined_fixture_program, fixture_output_path};

pub(crate) fn ordinary_c_build(
    description: &crate::recipe::CompilerCommand,
    run: &Path,
    checkout: &Path,
    root: &Path,
    workers: usize,
) -> Result<ExecutionRecord> {
    let view = crate::input_view::current().context("ordinary C compilation requires captured inputs")?;
    let generated = description.generated.iter().map(|name| fs::canonicalize(run.join(name)))
        .collect::<std::io::Result<std::collections::BTreeSet<_>>>()?;
    tool_execution::builds::c_build_with_inputs(
        tool_execution::builds::CBuildRequest {
            command: description.command.clone(), run: run.into(), checkout: checkout.into(), root: root.into(), workers,
        },
        &|paths| view.validate_c_inputs(paths, &generated),
    )
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

    pub(crate) fn compile_description(
        &self,
        description: &crate::recipe::CompilerCommand,
        run: &Path,
        label: &str,
        checkout: &Path,
        source: &Path,
    ) -> Result<ExecutionRecord> {
        let program = compiler_program(
            description,
            run,
            CompilerContext {
                helper: &self.helper,
                rustc: &self.rustc,
                c_compiler: &self.c_compiler,
                runtime: &self.compiler_runtime,
                rust_inputs: &self.rust_inputs,
                c_inputs: &self.c_inputs,
                checkout,
                source,
            },
        )?;
        self.execute(&program, &self.compiler_limits, run, label)
    }

    pub(crate) fn fixture_recipe(
        &self,
        command: &[String],
        directory: &Path,
        subject: &str,
        kind: FixtureKind,
    ) -> Result<ExecutionRecord> {
        self.execute(&confined_fixture_program(&self.helper, command, kind)?, &self.fixture_limits, directory, subject)
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
            let (held, wait) = tool_execution::acquire_lease_until(&self.root, self.workers, self.deadline_unix_ms)?;
            (Some(held), wait)
        };
        let invocation = if resumed {
            let saved: StrictInvocation = bounded::object(&invocation_path)?;
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
                let destination = fixture_output_path(run, label, &name);
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
                let saved: ExecutionRecord = bounded::object(&timing)?;
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
                decomp_source::context::reconcile(8 * bounded::DEFAULT_BYTES, publish)?
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
                decomp_source::context::reconcile(8 * bounded::DEFAULT_BYTES, reconcile)?;
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
                bounded::object(&root.join("run").join(format!("{label}.invocation.toml")))?;
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
            tool_execution::unique_id()
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
        let directory = tool_execution::testing::Directory::new()?;
        let error =
            translation_execution::compiler_runtime::discover_until(&directory.0.join("unavailable-rustc"), 0, 1024)
                .unwrap_err();
        assert!(error.to_string().contains("deadline"));
        fs::write(directory.0.join("source.rs"), b"12345")?;
        let error = decomp_source::context::until(bounded::now_unix_ms()? + 1000, 4, || {
            source_files(&directory.0, "rs", &mut Vec::new())
        })
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
    tool_execution::bounded::immutable_bytes(path, bytes)
}

/// Prepare the fixed pristine Fixture worker through the generic compiler cache.
pub(crate) fn prepare_fixture_worker(request: crate::authoring::FixtureWorkerRequest) -> Result<CacheResult> {
    fs::create_dir_all(&request.run)?;
    let mut metrics = crate::metrics::Run::start(request.run.join("helper-build.metrics.toml"), "helper-build")?;
    let result = tool_execution::builds::standalone_builds(tool_execution::builds::StandaloneBuildRequest {
        checkout: request.checkout,
        root: request.root,
        run: request.run,
        workers: request.workers,
        artifacts: vec![tool_execution::builds::StandaloneArtifact {
            source: "crates/offline-verification/src/fixture_worker.rs".into(),
            name: "libfixture_worker.a".into(),
            label: "fixture-worker-build".into(),
            flags: std::iter::once("--crate-type=staticlib".into())
                .chain(crate::input_view::current().into_iter().flat_map(|view| view.remap_arguments(true)))
                .collect(),
        }],
    })
    .and_then(|artifacts| artifacts.into_iter().next().context("Fixture worker build returned no artifact"));
    let error = result.as_ref().err().map(|error| format!("{error:#}"));
    let finished = metrics.finish(crate::metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finished?;
    }
    result
}

pub(crate) struct CompilerContext<'a> {
    pub helper: &'a Path,
    pub rustc: &'a Path,
    pub c_compiler: &'a Path,
    pub runtime: &'a [ReadOnlyBind],
    pub rust_inputs: &'a [PathBuf],
    pub c_inputs: &'a [PathBuf],
    pub checkout: &'a Path,
    pub source: &'a Path,
}

/// Resolve the recipe's compiler description identically for execution and
/// retained authentication. All paths/capabilities come from the trusted owner.
pub(crate) fn compiler_program(
    description: &crate::recipe::CompilerCommand,
    run: &Path,
    context: CompilerContext<'_>,
) -> Result<Program> {
    let CompilerContext { helper, rustc, c_compiler, runtime, rust_inputs, c_inputs, checkout, source } = context;
    let command = &description.command;
    let generated = &description.generated;
    let outputs = &description.outputs;
    let headless = description.full_c;
    let rust = command.first().is_some_and(|name| name == "rustc");
    let executable = if rust { rustc.to_path_buf() } else { c_compiler.to_path_buf() };
    let mut inputs = Vec::new();
    for path in if rust { rust_inputs } else { c_inputs } {
        let relative = path.strip_prefix(if rust { checkout } else { source })?;
        inputs.push(ReadOnlyBind {
            source: path.clone(),
            destination: Path::new(if rust { "/inputs/rust" } else { "/inputs/c" }).join(relative),
        });
    }
    for name in generated {
        inputs.push(ReadOnlyBind { source: run.join(name), destination: Path::new("/inputs/generated").join(name) });
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

    Ok(Program {
        executable,
        arguments,
        inputs,
        runtime: runtime.to_vec(),
        helper: helper.to_path_buf(),
        outputs: outputs.iter().map(PathBuf::from).collect(),
    })
}
