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

pub const USAGE: &str = "tools replay map INDEX [--mapping FILE] [--baseline FILE] [--repo ROOT] [--case ID ...] [--rule RULE] [--path FILE]\n       tools replay list INDEX [--rule RULE] [--scope scoped|integrated|movie]\n       tools replay run INDEX [--all | --case ID ... | --rule RULE] [SOURCES] [--write RECEIPT]\n       tools replay check INDEX --baseline BASELINE [--all | --case ID ... | --rule RULE] [SOURCES] [--shard I/N] [--write RECEIPT]\n       tools replay benchmark INDEX [--all | --case ID ... | --rule RULE] [SOURCES]\n       SOURCES: --cache CACHE --rom ROM --native NATIVE [--bundle BUNDLE]";

struct Options {
    ids: Vec<CaseId>,
    rule: Option<String>,
    kind: Option<CaseKind>,
    all: bool,
    sources: Sources,
    write: Option<PathBuf>,
    baseline: Option<PathBuf>,
    shard: Option<(usize, usize)>,
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
    Benchmark,
}

pub fn run(args: &[String]) -> Result<String, String> {
    if let [command, rest @ ..] = args
        && command == "map"
    {
        return map::run(rest).map_err(|fault| format!("replay map: {fault:?}"));
    }
    let [command, index_path, rest @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let command = match command.as_str() {
        "list" => Operation::List,
        "run" => Operation::Run,
        "check" => Operation::Check,
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
    let index = replay::read_index(Path::new(index_path))
        .map_err(|fault| format!("case index: {fault:?}"))?;
    options.sources.lock.clone_from(&index.lock);
    let selected = index
        .select(&options.ids, options.rule.as_deref(), options.kind)
        .map_err(|fault| format!("case selection: {fault:?}"))?;
    let selected: Vec<_> = selected
        .into_iter()
        .enumerate()
        .filter(|(ordinal, _)| {
            options
                .shard
                .is_none_or(|(shard, count)| ordinal.checked_rem(count) == Some(shard))
        })
        .map(|(_, case)| case)
        .collect();
    if selected.is_empty() {
        return Err("shard selects zero cases".to_owned());
    }
    if command == Operation::List {
        return Ok(list(&selected));
    }
    let index_bytes =
        std::fs::read(index_path).map_err(|_| "index bytes unavailable".to_owned())?;
    let index_sha256 = verify::lock::Sha256::of(&index_bytes).to_string();
    let baseline = options
        .baseline
        .as_ref()
        .map(|path| {
            replay::baseline::Baseline::read(path)
                .map_err(|fault| format!("replay baseline: {fault:?}"))
        })
        .transpose()?;
    if command == Operation::Check && baseline.is_none() {
        return Err("check requires --baseline".to_owned());
    }
    if let Some(baseline) = &baseline {
        baseline
            .validate_index(&index, &index_sha256)
            .map_err(|fault| format!("replay baseline: {fault:?}"))?;
    }
    let (results, elapsed) = measure(command, &selected, &options.sources)?;
    let mut output = receipt(&results)?;
    if let Some(path) = &options.write {
        std::fs::write(path, &output).map_err(|_| "receipt write failed".to_owned())?;
    }
    if command != Operation::Check && results.iter().any(|result| !result.passed()) {
        return Err(output);
    }
    if let Some(baseline) = &baseline {
        let ids: Vec<_> = selected.iter().map(|case| case.id.clone()).collect();
        let summary = baseline
            .check_selected(&index, &index_sha256, &ids, &results)
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
