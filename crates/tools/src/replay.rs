//! CLI selection and reporting over the shared verifier; no production advancement loop.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;
use verify::replay::{
    self, Sources,
    index::{Case, CaseId, CaseKind},
    result::CaseResult,
};

mod map;

pub const USAGE: &str = "tools replay map INDEX [--mapping FILE] [--baseline FILE] [--repo ROOT] [--case ID ...] [--rule RULE] [--path FILE]\n       tools replay list INDEX [--rule RULE] [--scope scoped|integrated|movie]\n       tools replay run INDEX [--all | --case ID ... | --rule RULE] [SOURCES] [--write RECEIPT]\n       tools replay check INDEX --baseline BASELINE [--all | --case ID ... | --rule RULE] [SOURCES] [--shard I/N] [--write RECEIPT]\n       tools replay merge INDEX --baseline BASELINE [--all | --case ID ... | --rule RULE] --receipt SHARD ... [--write RECEIPT]\n       tools replay benchmark INDEX [--all | --case ID ... | --rule RULE] [SOURCES]\n       tools replay played ROW START END --cache CACHE --rom ROM --lock LOCK (END may pass ROW into the rows that continue it)\n       SOURCES: --cache CACHE --rom ROM --native NATIVE [--bundle BUNDLE]";

struct Options {
    ids: Vec<CaseId>,
    rule: Option<String>,
    kind: Option<CaseKind>,
    all: bool,
    sources: Sources,
    write: Option<PathBuf>,
    baseline: Option<PathBuf>,
    shard: Option<(usize, usize)>,
    receipts: Vec<PathBuf>,
}

fn options(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        ids: Vec::new(),
        rule: None,
        kind: None,
        all: false,
        sources: Sources::default(),
        write: None,
        baseline: None,
        shard: None,
        receipts: Vec::new(),
    };
    let mut remaining = args;
    while let Some((flag, tail)) = remaining.split_first() {
        if flag == "--all" {
            if options.all {
                return Err("duplicate --all".to_owned());
            }
            options.all = true;
            remaining = tail;
            continue;
        }
        let Some((value, tail)) = tail.split_first() else {
            return Err(USAGE.to_owned());
        };
        match flag.as_str() {
            "--case" => options.ids.push(CaseId(value.clone())),
            "--rule" if options.rule.is_none() => options.rule = Some(value.clone()),
            "--scope" if options.kind.is_none() => {
                options.kind = Some(match value.as_str() {
                    "scoped" => CaseKind::Scoped,
                    "integrated" => CaseKind::Integrated,
                    "movie" => CaseKind::Movie,
                    _ => return Err("unknown scope".to_owned()),
                });
            }
            "--cache" if options.sources.cache.as_os_str().is_empty() => {
                options.sources.cache = PathBuf::from(value);
            }
            "--bundle" if options.sources.bundle.as_os_str().is_empty() => {
                options.sources.bundle = PathBuf::from(value);
            }
            "--rom" if options.sources.rom.as_os_str().is_empty() => {
                options.sources.rom = PathBuf::from(value);
            }
            "--native" if options.sources.native.as_os_str().is_empty() => {
                options.sources.native = PathBuf::from(value);
            }
            "--receipt" => options.receipts.push(PathBuf::from(value)),
            "--write" if options.write.is_none() => options.write = Some(PathBuf::from(value)),
            "--baseline" if options.baseline.is_none() => {
                options.baseline = Some(PathBuf::from(value));
            }
            "--shard" if options.shard.is_none() => {
                let Some((index, count)) = value.split_once('/') else {
                    return Err("shard requires I/N".to_owned());
                };
                let index = index
                    .parse::<usize>()
                    .map_err(|_| "shard index".to_owned())?;
                let count = count
                    .parse::<usize>()
                    .map_err(|_| "shard count".to_owned())?;
                if count == 0 || index >= count {
                    return Err("shard outside partition".to_owned());
                }
                options.shard = Some((index, count));
            }
            _ => return Err(USAGE.to_owned()),
        }
        remaining = tail;
    }
    if options.all && (!options.ids.is_empty() || options.rule.is_some() || options.kind.is_some())
    {
        return Err("--all conflicts with filters".to_owned());
    }
    Ok(options)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operation {
    List,
    Run,
    Check,
    Merge,
    Benchmark,
}

pub fn run(args: &[String]) -> Result<String, String> {
    match args {
        [command, rest @ ..] if command == "map" => {
            return map::run(rest).map_err(|fault| format!("replay map: {fault:?}"));
        }
        [command, rest @ ..] if command == "played" => return played(rest),
        _ => {}
    }
    let [command, index_path, rest @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let command = match command.as_str() {
        "list" => Operation::List,
        "run" => Operation::Run,
        "check" => Operation::Check,
        "merge" => Operation::Merge,
        "benchmark" => Operation::Benchmark,
        _ => return Err(USAGE.to_owned()),
    };
    let mut options = options(rest)?;
    if command != Operation::List
        && !options.all
        && options.ids.is_empty()
        && options.rule.is_none()
    {
        return Err("select --all, --case or --rule explicitly".to_owned());
    }
    let snapshot = replay::read_index(Path::new(index_path))
        .map_err(|fault| format!("case index: {fault:?}"))?;
    let baseline = options
        .baseline
        .as_ref()
        .filter(|_| command != Operation::List)
        .map(|path| {
            replay::baseline::Baseline::read(path)
                .map_err(|fault| format!("replay baseline: {fault:?}"))
        })
        .transpose()?;
    if matches!(command, Operation::Check | Operation::Merge) && baseline.is_none() {
        return Err("check requires --baseline".to_owned());
    }
    // Only a merge reads receipts, and it rejoins every shard of its selection.
    if (command == Operation::Merge) == options.receipts.is_empty()
        || (command == Operation::Merge && options.shard.is_some())
    {
        return Err(USAGE.to_owned());
    }
    if let Some(baseline) = &baseline {
        baseline
            .validate_index(&snapshot)
            .map_err(|fault| format!("replay baseline: {fault:?}"))?;
    }
    let index = snapshot.index();
    options.sources.lock.clone_from(&index.lock);
    let selected = index
        .select(&options.ids, options.rule.as_deref(), options.kind)
        .map_err(|fault| format!("case selection: {fault:?}"))?;
    let selected: Vec<_> = selected
        .into_iter()
        .enumerate()
        .filter(|(ordinal, _)| in_shard(*ordinal, options.shard))
        .map(|(_, case)| case)
        .collect();
    if selected.is_empty() {
        return Err("shard selects zero cases".to_owned());
    }
    if command == Operation::List {
        return Ok(list(&selected));
    }
    let (results, elapsed) = if command == Operation::Merge {
        (merge(&selected, &options.receipts)?, Vec::new())
    } else {
        measure(command, &selected, &options.sources)?
    };
    let mut output = receipt(&results)?;
    if let Some(path) = &options.write {
        std::fs::write(path, &output).map_err(|_| "receipt write failed".to_owned())?;
    }
    if !matches!(command, Operation::Check | Operation::Merge)
        && results.iter().any(|result| !result.passed())
    {
        return Err(output);
    }
    if let Some(baseline) = &baseline {
        let ids: Vec<_> = selected.iter().map(|case| case.id.clone()).collect();
        let summary = baseline
            .check_selected(&snapshot, &ids, &results)
            .map_err(|fault| format!("{output}\nreplay baseline: {fault:?}"))?;
        output.push_str("\n[baseline]\n");
        output.push_str(
            &toml::to_string(&summary).map_err(|_| "baseline summary encoding".to_owned())?,
        );
    }
    if command == Operation::Benchmark {
        benchmark(&mut output, elapsed)?;
    }
    if let Some(path) = options.write {
        std::fs::write(path, &output).map_err(|_| "receipt write failed".to_owned())?;
    }
    Ok(output)
}

/// Shards deal the selection round-robin, so every shard holds cases from the whole index.
fn in_shard(ordinal: usize, shard: Option<(usize, usize)>) -> bool {
    shard.is_none_or(|(shard, count)| ordinal.checked_rem(count) == Some(shard))
}

/// Rejoin shard receipts in selection order, so the merged receipt reads as one unsharded check.
/// Missing, duplicate or foreign cases are left for the baseline check to reject.
fn merge(selected: &[&Case], paths: &[PathBuf]) -> Result<Vec<CaseResult>, String> {
    #[derive(serde::Deserialize)]
    struct ShardReceipt {
        format: u16,
        cases: Vec<CaseResult>,
    }
    let mut results = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(path).map_err(|_| "shard receipt read failed")?;
        let shard: ShardReceipt =
            toml::from_str(&text).map_err(|_| "shard receipt decoding failed".to_owned())?;
        if shard.format != 1 {
            return Err("shard receipt format".to_owned());
        }
        results.extend(shard.cases);
    }
    results.sort_by_key(|result| {
        selected
            .iter()
            .position(|case| case.id == result.id)
            .unwrap_or(usize::MAX)
    });
    Ok(results)
}

fn list(cases: &[&Case]) -> String {
    let mut output = String::new();
    for case in cases {
        let _ = writeln!(
            output,
            "{} {:?} {} {}",
            case.id.0,
            case.kind,
            case.row,
            case.rules.join(",")
        );
    }
    output
}

fn measure(
    command: Operation,
    cases: &[&Case],
    sources: &Sources,
) -> Result<(Vec<CaseResult>, Vec<u128>), String> {
    let run = || {
        cases
            .iter()
            .map(|case| replay::run(case, sources))
            .collect::<Vec<_>>()
    };
    let mut frozen = if command == Operation::Benchmark {
        let warmup = run();
        if warmup.iter().any(|result| !result.passed()) {
            return Err(receipt(&warmup)?);
        }
        Some(receipt(&warmup)?)
    } else {
        None
    };
    let repetitions = if command == Operation::Benchmark {
        5
    } else {
        1
    };
    let mut elapsed = Vec::new();
    let mut results = Vec::new();
    for _ in 0..repetitions {
        let started = Instant::now();
        results = run();
        elapsed.push(started.elapsed().as_micros());
        let measured = receipt(&results)?;
        if frozen
            .as_ref()
            .is_some_and(|previous| previous != &measured)
        {
            return Err("benchmark exact receipt changed".to_owned());
        }
        frozen = Some(measured);
    }
    Ok((results, elapsed))
}

fn benchmark(output: &mut String, mut elapsed: Vec<u128>) -> Result<(), String> {
    elapsed.sort_unstable();
    let [minimum, _, median, _, maximum] = elapsed.as_slice() else {
        return Err("benchmark repetition count".to_owned());
    };
    let _ = write!(
        output,
        "\n[benchmark]\nwarmups = 1\nrepetitions = 5\nmedian_microseconds = {median}\nminimum_microseconds = {minimum}\nmaximum_microseconds = {maximum}\n"
    );
    Ok(())
}

fn receipt(results: &[CaseResult]) -> Result<String, String> {
    #[derive(serde::Serialize)]
    struct Receipt<'a> {
        format: u16,
        cases: &'a [CaseResult],
    }
    toml::to_string(&Receipt {
        format: 1,
        cases: results,
    })
    .map_err(|_| "receipt encoding failed".to_owned())
}

/// Play a lock row alone from its seed and report the cuts held. The line carries only the
/// row, cut numbers and the stop's field or dependency name, never an observed value.
fn played(args: &[String]) -> Result<String, String> {
    let [row, start, end, rest @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let cut = |text: &String| text.parse::<u64>().map_err(|_| USAGE.to_owned());
    let (start, end) = (cut(start)?, cut(end)?);
    let mut sources = Sources::default();
    let mut remaining = rest;
    while let [flag, value, tail @ ..] = remaining {
        let path = PathBuf::from(value);
        match flag.as_str() {
            "--cache" => sources.cache = path,
            "--rom" => sources.rom = path,
            "--lock" => sources.lock = path,
            _ => return Err(USAGE.to_owned()),
        }
        remaining = tail;
    }
    if !remaining.is_empty()
        || [&sources.cache, &sources.rom, &sources.lock]
            .iter()
            .any(|path| path.as_os_str().is_empty())
    {
        return Err(USAGE.to_owned());
    }
    let report = replay::played::play(&sources, row, start, end)
        .map_err(|outcome| format!("replay played: {outcome:?}"))?;
    Ok(format!(
        "PLAYED_ALONE row={} start={} held={} stop={:?} seed_held={:?} borrowed={:?}\n",
        report.row, report.start, report.held, report.stop, report.seed_held, report.borrowed,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use verify::replay::{baseline::Baseline, index::Start, result::Outcome};

    /// The receipt a check would write for a case that holds its reviewed floor exactly.
    fn reviewed(case: &Case, baseline: &Baseline) -> CaseResult {
        let floor = baseline
            .cases
            .iter()
            .find(|floor| floor.id == case.id)
            .unwrap();
        CaseResult {
            id: case.id.clone(),
            kind: case.kind,
            row: case.row.clone(),
            start_cut: match case.start {
                Start::CacheCut { frame } | Start::NativeRecord { frame, .. } => Some(frame),
                Start::NativeOpportunity { .. }
                | Start::NativeSeed { .. }
                | Start::NativeSenderOffer { .. }
                | Start::ConstructedSenderOffer { .. }
                | Start::PowerOn => None,
            },
            end_cut: case.end.cut_inclusive,
            counts: floor.counts.clone(),
            outcome: floor.expected.clone(),
            observation_sha256: floor.observation_sha256.clone(),
            limits: case.limits.clone(),
        }
    }

    #[test]
    fn merged_shard_receipts_equal_the_unsharded_check() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/replay");
        let index_path = root.join("index.toml");
        let baseline_path = root.join("baseline.toml");
        let snapshot = replay::read_index(&index_path).unwrap();
        let baseline = Baseline::read(&baseline_path).unwrap();
        let cases = snapshot.index().select(&[], None, None).unwrap();
        let results: Vec<_> = cases.iter().map(|case| reviewed(case, &baseline)).collect();
        let summary = baseline.check(&snapshot, &results).unwrap();
        let unsharded = format!(
            "{}\n[baseline]\n{}",
            receipt(&results).unwrap(),
            toml::to_string(&summary).unwrap()
        );

        let scratch = std::env::temp_dir().join(format!("replay-merge-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();
        let mut args = vec![
            "merge".to_owned(),
            index_path.to_string_lossy().into_owned(),
            "--all".to_owned(),
            "--baseline".to_owned(),
            baseline_path.to_string_lossy().into_owned(),
        ];
        // Shards are passed out of order: the merge restores index order itself.
        for shard in [2, 0, 1] {
            let part: Vec<_> = results
                .iter()
                .enumerate()
                .filter(|(ordinal, _)| in_shard(*ordinal, Some((shard, 3))))
                .map(|(_, result)| result.clone())
                .collect();
            let path = scratch.join(format!("shard-{shard}.toml"));
            std::fs::write(&path, receipt(&part).unwrap()).unwrap();
            args.extend(["--receipt".to_owned(), path.to_string_lossy().into_owned()]);
        }
        let merged = run(&args);
        let without_last_shard = run(&args[..args.len() - 2]);
        let _ = std::fs::remove_dir_all(&scratch);

        assert_eq!(merged.unwrap(), unsharded);
        assert!(without_last_shard.is_err());
    }

    #[test]
    fn check_selected_movie_preserves_the_reviewed_boundary() {
        #[derive(serde::Deserialize)]
        struct Receipt {
            cases: Vec<CaseResult>,
            baseline: BoundarySummary,
        }
        #[derive(serde::Deserialize)]
        struct BoundarySummary {
            #[serde(rename = "passed_cases")]
            passed: usize,
            #[serde(rename = "held_boundary_cases")]
            held_boundary: usize,
            #[serde(rename = "unsupported_cases")]
            unsupported: usize,
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/replay");
        let baseline_path = root.join("baseline.toml");
        let id = "movie/missing-power-on";
        let args = vec![
            "check".to_owned(),
            root.join("index.toml").to_string_lossy().into_owned(),
            "--baseline".to_owned(),
            baseline_path.to_string_lossy().into_owned(),
            "--case".to_owned(),
            id.to_owned(),
        ];
        let output: Receipt = toml::from_str(&run(&args).unwrap()).unwrap();
        let [case] = output.cases.as_slice() else {
            panic!("expected one selected receipt");
        };
        let baseline = Baseline::read(&baseline_path).unwrap();
        let floor = baseline
            .cases
            .iter()
            .find(|floor| floor.id.0 == id)
            .unwrap();
        assert_eq!(case.id, floor.id);
        assert!(matches!(case.outcome, Outcome::Unsupported { .. }));
        assert_eq!(case.outcome, floor.expected);
        assert_eq!(case.counts, floor.counts);
        assert_eq!(case.observation_sha256, floor.observation_sha256);
        assert_eq!(output.baseline.passed, 0);
        assert_eq!(output.baseline.held_boundary, 1);
        assert_eq!(output.baseline.unsupported, 1);
    }
}
