//! Nonaccepting original-ROM prefix diagnostics and independently changed C controls.
use crate::digest;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use tool_execution::builds;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
const FIELDS: [(&str, u32); 19] = [
    ("game_state", 0x998),
    ("room", 0x79b),
    ("samus_x", 0xaf6),
    ("samus_y", 0xafa),
    ("health", 0x9c2),
    ("items", 0x9a4),
    ("equipped_items", 0x9a2),
    ("clock_frames", 0x9da),
    ("clock_seconds", 0x9dc),
    ("clock_minutes", 0x9de),
    ("clock_hours", 0x9e0),
    ("rng", 0x5e5),
    ("held", 0x8b),
    ("pressed", 0x8f),
    ("previous", 0x97),
    ("accepted_nmi", 0x5b6),
    ("all_nmi", 0x5b8),
    ("upload", 0x617),
    ("waiting", 0x5b4),
];
const HOOKS: [(&str, u32); 14] = [
    ("reset", 0x80841c),
    ("boot-clear", 0x808482),
    ("controller-entry", 0x809459),
    ("controller-return", 0x809496),
    ("dispatch-entry", 0x828948),
    ("dispatch-complete", 0x82897a),
    ("explicit-wait", 0x808338),
    ("nmi-entry", 0x809583),
    ("message-entry", 0x858080),
    ("message-return", 0x8580ba),
    ("upload-entry", 0x808028),
    ("upload-return", 0x8080f1),
    ("upload-return-short", 0x80802e),
    ("upload-return-abort", 0x8080ff),
];
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Event {
    source_frame: u64,
    source_event: u64,
    kind: String,
    address: u32,
    value: u16,
    state: BTreeMap<String, u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hardware_pad: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    controller_entry_event: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    host_step: Option<u64>,
    #[serde(default)]
    classification: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CausalEvent {
    source_frame: u64,
    source_event: u64,
    kind: String,
    phase: String,
    preceding_poll: Option<u64>,
    cleared_poll: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Difference {
    original: u16,
    candidate: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Comparison {
    ordinal: u64,
    source_frame: u64,
    source_event: u64,
    host_step: u64,
    candidate_event: u64,
    differences: BTreeMap<String, Difference>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Native {
    frames: u64,
    movie_sha256: String,
    rom_sha256: String,
    executable_sha256: String,
    observer_sha256: String,
    observer_contract_sha256: String,
    artifacts: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Subject {
    command: Vec<String>,
    artifacts: BTreeMap<String, String>,
    completed_updates: u64,
    first_poll_failure: Option<u64>,
    vblank_wait_hits: u64,
    screen_wait_hits: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Report {
    schema_version: u32,
    status: String,
    accepted: bool,
    requested_frames: u64,
    scope: String,
    error: Option<String>,
    native: Option<Native>,
    subjects: BTreeMap<String, Subject>,
    first_unsupported_frame: Option<u64>,
    poll_cases: u64,
    poll_primitive_gameplay_match: bool,
    poll_primitive_full_match: bool,
    changed_sample_detected: bool,
    changed_state_detected: bool,
    changed_rng_detected: bool,
    changed_causal_rng_detected: bool,
    prelude_polls: u64,
    matching_causal_prefix: u64,
    native_completed_updates: u64,
    causal_completed_updates: u64,
    comparisons: Vec<Comparison>,
    causal_comparisons: Vec<Comparison>,
    future_poll_assignments: Vec<(u64, u64, u64)>,
    explicit_wait_events: Vec<u64>,
    reused_native: Option<PathBuf>,
    artifacts: BTreeMap<String, String>,
}
fn observer(output: &Path) -> Result<String> {
    let path = output.to_str().context("observer path must be UTF-8")?;
    ensure!(!path.contains("]]"), "unsupported Lua output path");
    let mut hooks = String::new();
    for (kind, address) in HOOKS.into_iter().filter(|(_, a)| *a != 0) {
        hooks.push_str(&format!(
            "memory.registerexec('BUS',{address},function(a,v) observe('{kind}',{address},v) end)\n"
        ));
    }
    for (kind, addresses) in [
        ("controller-register", vec![0x4218, 0x4219, 0x421a, 0x421b, 0x804218, 0x804219, 0x80421a, 0x80421b]),
        ("apu-register", vec![0x2140, 0x2141, 0x2142, 0x2143, 0x802140, 0x802141, 0x802142, 0x802143]),
    ] {
        for address in addresses {
            hooks.push_str(&format!(
                "memory.registerread('BUS',{address},function(a,v) observe('{kind}',{address},v) end)\n"
            ));
        }
    }
    Ok(include_str!("prefix_observer.lua.in")
        .replace("__OUTPUT__", path)
        .replace("__FIELDS__", &FIELDS.map(|(n, _)| n).join(","))
        .replace("__OFFSETS__", &FIELDS.map(|(_, a)| a.to_string()).join(","))
        .replace("__HOOKS__", &hooks))
}
fn hashes(directory: &Path) -> Result<BTreeMap<String, String>> {
    fn visit(root: &Path, path: &Path, found: &mut BTreeMap<String, String>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            ensure!(!path.is_symlink(), "prefix evidence symlink");
            if path.is_dir() {
                visit(root, &path, found)?;
            } else {
                found.insert(path.strip_prefix(root)?.to_string_lossy().into_owned(), digest(&path)?);
            }
        }
        Ok(())
    }
    let mut found = BTreeMap::new();
    visit(directory, directory, &mut found)?;
    Ok(found)
}
fn read_native(path: &Path) -> Result<Vec<Event>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    ensure!(
        lines.next() == Some(format!("frame,event,kind,address,value,{}", FIELDS.map(|(n, _)| n).join(",")).as_str()),
        "native event columns differ"
    );
    lines
        .enumerate()
        .map(|(index, line)| {
            let columns = line.split(',').collect::<Vec<_>>();
            ensure!(columns.len() == 24, "native event extent differs");
            let source_event = columns[1].parse()?;
            ensure!(source_event == index as u64 + 1, "noncontiguous native event order");
            Ok(Event {
                source_frame: columns[0].parse()?,
                source_event,
                kind: columns[2].into(),
                address: columns[3].parse()?,
                value: columns[4].parse()?,
                state: FIELDS
                    .iter()
                    .zip(&columns[5..])
                    .map(|((name, _), value)| Ok(((*name).into(), value.parse()?)))
                    .collect::<Result<_>>()?,
                hardware_pad: None,
                controller_entry_event: None,
                host_step: None,
                classification: "retained-boundary".into(),
            })
        })
        .collect()
}
fn causal(events: &[Event]) -> Vec<CausalEvent> {
    let mut poll = None;
    let mut started = false;
    let mut rows = Vec::new();
    for event in events {
        let mut row = CausalEvent {
            source_frame: event.source_frame,
            source_event: event.source_event,
            kind: event.kind.clone(),
            phase: String::new(),
            preceding_poll: None,
            cleared_poll: None,
        };
        match event.kind.as_str() {
            "controller-return" => {
                poll = Some(event.source_event);
                row.phase = if started { "dispatch" } else { "prelude" }.into();
            }
            "boot-clear" => {
                row.cleared_poll = poll;
                poll = None;
            }
            "dispatch-entry" | "dispatch-complete" => {
                started = true;
                row.preceding_poll = poll;
            }
            _ => continue,
        }
        rows.push(row);
    }
    rows
}
fn map(events: &mut [Event]) -> Result<Option<u64>> {
    let mut entry = None;
    let mut registers = Vec::new();
    let mut started = false;
    let mut first = None;
    let mut polls = 0;
    for event in events {
        match event.kind.as_str() {
            "dispatch-entry" => started = true,
            "controller-entry" => {
                ensure!(entry.is_none(), "nested controller poll");
                entry = Some(event.source_event);
                registers.clear();
            }
            "controller-register" if entry.is_some() => registers.push((event.address & 0xffff, event.value)),
            "controller-return" => {
                let values =
                    |address| registers.iter().filter(|(a, _)| *a == address).map(|(_, v)| *v).collect::<Vec<_>>();
                let low = values(0x4218);
                let high = values(0x4219);
                ensure!(
                    entry.is_some() && low.len() == 1 && high.len() == 1 && low[0] < 256 && high[0] < 256,
                    "controller read pair incomplete"
                );
                let second =
                    registers.iter().filter(|(a, _)| [0x421a, 0x421b].contains(a)).copied().collect::<Vec<_>>();
                ensure!(
                    second.is_empty() || (values(0x421a) == [0] && values(0x421b) == [0]),
                    "nonzero/incomplete second controller"
                );
                event.hardware_pad = Some(low[0] | high[0] << 8);
                event.controller_entry_event = entry.take();
                polls += 1;
                event.host_step = Some(polls);
                event.classification = "retimed-controller-poll; mapping unaccepted".into();
                if !started && first.is_none() {
                    first = Some(event.source_frame);
                }
            }
            "frame-complete" => event.classification = "skipped-no-c-step; boundaries retained".into(),
            "explicit-wait" => event.classification = "retained-deliberate-wait; timing unmapped".into(),
            _ => {}
        }
    }
    Ok(first)
}
#[derive(Clone, Debug)]
struct CandidateEvent {
    event: u64,
    host_step: u64,
    kind: String,
    state: BTreeMap<String, u16>,
}
fn candidate_events(path: &Path) -> Result<Vec<CandidateEvent>> {
    fs::read_to_string(path)?
        .lines()
        .map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            ensure!(parts.len() == 23, "candidate event extent");
            let state = FIELDS
                .iter()
                .enumerate()
                .map(|(i, (name, _))| Ok(((*name).into(), parts[i + 4].parse::<u16>()?)))
                .collect::<Result<_>>()?;
            Ok(CandidateEvent { event: parts[0].parse()?, host_step: parts[1].parse()?, kind: parts[2].into(), state })
        })
        .collect()
}
fn poll_results(path: &Path) -> Result<Vec<(bool, bool)>> {
    fs::read_to_string(path)?
        .lines()
        .filter(|line| line.starts_with("poll "))
        .map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            ensure!(parts.len() == 11, "poll result extent");
            Ok((parts[5] == "true", parts[6] == "true"))
        })
        .collect()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Request {
    checkout: Option<PathBuf>,
    asset_root: Option<PathBuf>,
    frames: u64,
    timeout: u64,
    pipeline_root: Option<PathBuf>,
    workers: usize,
    movie: Option<PathBuf>,
    reuse_native: Option<PathBuf>,
    output: Option<PathBuf>,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            checkout: None,
            asset_root: None,
            frames: 320,
            timeout: 120,
            pipeline_root: None,
            workers: 4,
            movie: None,
            reuse_native: None,
            output: None,
        }
    }
}
fn compare(native: &[Event], candidate: &[CandidateEvent]) -> Result<Vec<Comparison>> {
    native
        .iter()
        .zip(candidate)
        .enumerate()
        .map(|(index, (a, b))| {
            let mut differences = BTreeMap::new();
            for (key, _) in FIELDS {
                if ["accepted_nmi", "all_nmi", "waiting"].contains(&key) {
                    continue;
                }
                let original = a.state[key];
                let value = u16::try_from(u64::from(b.state[key]))?;
                if original != value {
                    differences.insert(key.into(), Difference { original, candidate: value });
                }
            }
            Ok(Comparison {
                ordinal: index as u64 + 1,
                source_frame: a.source_frame,
                source_event: a.source_event,
                host_step: b.host_step,
                candidate_event: b.event,
                differences,
            })
        })
        .collect()
}
fn changed(a: &[CandidateEvent], b: &[CandidateEvent]) -> bool {
    a.iter().zip(b).any(|(a, b)| {
        FIELDS
            .iter()
            .any(|(key, _)| !["accepted_nmi", "all_nmi", "waiting"].contains(key) && a.state[*key] != b.state[*key])
    })
}
fn execute(command: &[String], directory: &Path, shared: &Path, workers: usize, timeout: u64) -> Result<()> {
    builds::execute_timeout(command, directory, "replay", shared, workers, timeout)?;
    Ok(())
}
fn investigate(root: &Path, assets: &Path, run: &Path, request: &Request, report: &mut Report) -> Result<()> {
    let frames = report.requested_frames;
    let timeout = request.timeout;
    let shared = request.pipeline_root.clone().unwrap_or(root.join("out/pipeline"));
    let workers = request.workers;
    let movie = request.movie.clone().unwrap_or(assets.join("movies/super-metroid-any-3653.lsmv"));
    let movie_hash = digest(&movie)?;
    let rom = assets.join("rom/sm.sfc");
    ensure!(digest(&rom)? == ROM_SHA256, "unsupported ROM identity");
    #[derive(Deserialize)]
    struct Corpus {
        movies: Vec<MovieIdentity>,
    }
    #[derive(Deserialize)]
    struct MovieIdentity {
        sha256: String,
        category: String,
    }
    let corpus: Corpus = toml::from_str(&fs::read_to_string(assets.join("movies/manifest.toml"))?)?;
    ensure!(corpus.movies.iter().any(|r| r.sha256 == movie_hash && r.category == "any%"), "unsupported movie identity");
    movie::load(&movie)?;
    decomp_source::verify_source(root, &assets.join("deps/snesrev-sm"))?;
    let binary = assets.join("deps/lsnes/source/src/util/lsnes-dumpavi.util");
    let native = run.join("native");
    fs::create_dir(&native)?;
    fs::write(native.join("observe.lua"), observer(&native)?)?;
    if let Some(prior) = &request.reuse_native {
        let prior = fs::canonicalize(prior)?;
        authenticate(&prior)?;
        let retained: Report = pipeline_model::read(&prior.join("verdict.toml"))?;
        ensure!(
            retained.schema_version == 1 && retained.status == "nonaccepting" && !retained.accepted,
            "native reuse requires a completed nonaccepting diagnostic"
        );
        let seal = retained.native.context("retained native capture missing")?;
        ensure!(
            seal.frames == frames
                && seal.movie_sha256 == movie_hash
                && seal.rom_sha256 == ROM_SHA256
                && seal.executable_sha256 == digest(&binary)?
                && seal.observer_contract_sha256 == crate::readiness_run::identity(&observer(Path::new("native"))?)?
                && fs::read_to_string(prior.join("native/observe.lua"))? == observer(&prior.join("native"))?,
            "reused native identities differ"
        );
        ensure!(hashes(&prior.join("native"))? == seal.artifacts, "reused native artifacts changed");
        fs::remove_file(native.join("observe.lua"))?;
        for name in seal.artifacts.keys() {
            let source = prior.join("native").join(name);
            let target = native.join(name);
            fs::create_dir_all(target.parent().unwrap())?;
            fs::copy(source, target)?;
        }
        report.reused_native = Some(prior);
    } else {
        let command = vec![
            "env".into(),
            format!("LD_LIBRARY_PATH={}", assets.join("deps/lsnes-build-deps/usr/lib/x86_64-linux-gnu").display()),
            format!("XDG_CONFIG_HOME={}", native.join("config").display()),
            binary.to_string_lossy().into_owned(),
            "--dumper=INTERNAL-NULL".into(),
            format!("--rom-a={}", rom.display()),
            format!("--length={frames}"),
            format!("--lua={}", native.join("observe.lua").display()),
            movie.to_string_lossy().into_owned(),
        ];
        execute(&command, &native, &shared, workers, timeout)?;
    }
    let log = fs::read_to_string(native.join("replay.stderr"))? + &fs::read_to_string(native.join("replay.stdout"))?;
    ensure!(
        ["Finished!", "bsnes v085 (Compatibility core)", "snes_ntsc"].iter().all(|s| log.contains(s)),
        "native completion/core/region missing"
    );
    let mut events = read_native(&native.join("events.csv"))?;
    ensure!(events.iter().filter(|r| r.kind == "frame-complete").count() as u64 == frames, "truncated native frames");
    report.native = Some(Native {
        frames,
        movie_sha256: movie_hash,
        rom_sha256: ROM_SHA256.into(),
        executable_sha256: digest(&binary)?,
        observer_sha256: digest(&native.join("observe.lua"))?,
        observer_contract_sha256: crate::readiness_run::identity(&observer(Path::new("native"))?)?,
        artifacts: hashes(&native)?,
    });
    report.first_unsupported_frame = map(&mut events)?;
    ensure!(report.first_unsupported_frame.is_some(), "expected prelude seam absent");
    // Retain large ordered timelines as a versioned binary envelope, alongside
    // the external observer CSV. Small causal metadata remains readable.
    let mut timeline = b"SMPT\x01\x00".to_vec();
    timeline.extend_from_slice(&(events.len() as u64).to_le_bytes());
    for event in &events {
        timeline.extend_from_slice(&event.source_frame.to_le_bytes());
        timeline.extend_from_slice(&event.source_event.to_le_bytes());
        timeline.extend_from_slice(&event.address.to_le_bytes());
        timeline.extend_from_slice(&event.value.to_le_bytes());
        timeline.push(u8::try_from(event.kind.len())?);
        timeline.extend_from_slice(event.kind.as_bytes());
        for (name, _) in FIELDS {
            timeline.extend_from_slice(&event.state[name].to_le_bytes());
        }
        for value in [event.hardware_pad.map(u64::from), event.controller_entry_event, event.host_step] {
            timeline.push(u8::from(value.is_some()));
            timeline.extend_from_slice(&value.unwrap_or(0).to_le_bytes());
        }
    }
    fs::write(run.join("timeline.bin"), timeline)?;
    #[derive(Serialize)]
    struct Causality {
        schema_version: u32,
        accepted: bool,
        events: Vec<CausalEvent>,
    }
    pipeline_model::write(
        &run.join("controller-causality.toml"),
        &Causality { schema_version: 1, accepted: false, events: causal(&events) },
    )?;
    let polls = events.iter().filter(|r| r.kind == "controller-return").collect::<Vec<_>>();
    ensure!(!polls.is_empty(), "no native polls");
    let primitive = polls.iter().filter(|r| r.source_frame <= 320).collect::<Vec<_>>();
    let mut cases = String::new();
    for row in &primitive {
        let entry = row.controller_entry_event.context("poll entry missing")?;
        let before = fs::read(native.join(format!("state-{entry}.wram")))?;
        let after = fs::read(native.join(format!("state-{}.wram", row.source_event)))?;
        ensure!(before.len() == 0x20000 && after.len() == 0x20000, "native WRAM extent differs");
        let stack = (0x1f00..0x2000).filter(|i| before[*i] != after[*i]).collect::<Vec<_>>();
        ensure!(
            stack.is_empty() || (stack.len() == 1 && before[stack[0]] == 0x94 && after[stack[0]] == 0x16),
            "native call stack transition outside bounded primitive"
        );
        cases.push_str(&format!(
            "{entry} {} {:04x} {:04x}\n",
            row.source_event,
            row.hardware_pad.unwrap(),
            stack.first().copied().unwrap_or(0xffff)
        ));
    }
    fs::write(run.join("poll-cases.txt"), cases)?;
    fs::write(
        run.join("derived.inputs"),
        polls.iter().map(|r| format!("{:04x}\n", r.state["held"])).collect::<String>(),
    )?;
    fs::write(
        run.join("causal.inputs"),
        polls.iter().map(|r| format!("{:04x}\n", r.hardware_pad.unwrap())).collect::<String>(),
    )?;
    let source = assets.join("deps/snesrev-sm");
    let mut command = vec![
        "cc".into(),
        "-std=gnu11".into(),
        "-O2".into(),
        "-fno-strict-aliasing".into(),
        "-ffunction-sections".into(),
        "-fdata-sections".into(),
        "-iquote".into(),
        source.join("src").to_string_lossy().into_owned(),
    ];
    command.extend(decomp_source::c_units(&source)?.iter().map(|p| p.to_string_lossy().into_owned()));
    command.push(root.join("tools/decomp/prefix_host.c").to_string_lossy().into_owned());
    command.push("-Wl,--gc-sections".into());
    for symbol in [
        "Call",
        "cpu_runOpcode",
        "DecompressToMem",
        "ScreenOn",
        "WaitUntilEndOfVblankAndClearHdma",
        "RtlApuUpload",
        "ReadJoypadInputs",
        "Vector_NMI",
        "NextRandom",
    ] {
        command.push(format!("-Wl,--wrap={symbol}"));
    }
    command.extend(["-lm".into(), "-o".into(), run.join("prefix-host").to_string_lossy().into_owned()]);
    builds::c_build(builds::CBuildRequest {
        checkout: root.into(),
        root: shared.clone(),
        workers,
        run: run.into(),
        command,
    })?;
    for name in ["prefix_host.c", "headless.c", "headless_frame.h"] {
        fs::copy(root.join("tools/decomp").join(name), run.join(name))?;
    }
    fs::copy(source.join("LICENSE.txt"), run.join("LICENSE.txt"))?;
    for (name, mode) in [
        ("poll-primitive", "poll-fixtures"),
        ("changed-poll-primitive", "changed-poll-fixtures"),
        ("changed-state-poll-primitive", "changed-state-poll-fixtures"),
    ] {
        let directory = run.join(name);
        fs::create_dir(&directory)?;
        let command = [run.join("prefix-host"), rom.clone(), native.clone(), run.join("poll-cases.txt")]
            .map(|p| p.to_string_lossy().into_owned());
        let mut command = command.to_vec();
        command.push(mode.into());
        execute(&command, &directory, &shared, workers, timeout)?;
        let rows = poll_results(&directory.join("replay.stdout"))?;
        ensure!(rows.len() == primitive.len(), "poll primitive count differs");
        if name == "poll-primitive" {
            report.poll_cases = rows.len() as u64;
            report.poll_primitive_gameplay_match = rows.iter().all(|r| r.1);
            report.poll_primitive_full_match = rows.iter().all(|r| r.0);
            ensure!(report.poll_primitive_gameplay_match, "native controller primitive differs from actual C");
        } else if name == "changed-poll-primitive" {
            report.changed_sample_detected = rows.iter().any(|r| !r.1);
            ensure!(report.changed_sample_detected, "changed sample escaped detection");
        } else {
            report.changed_state_detected = rows.iter().all(|r| !r.1);
            ensure!(report.changed_state_detected, "changed state escaped detection");
        }
        report.subjects.insert(
            name.into(),
            Subject {
                command,
                artifacts: hashes(&directory)?,
                completed_updates: 0,
                first_poll_failure: None,
                vblank_wait_hits: 0,
                screen_wait_hits: 0,
            },
        );
    }
    let original = events.iter().filter(|r| r.kind == "dispatch-complete").cloned().collect::<Vec<_>>();
    ensure!(!original.is_empty(), "native dispatch completion absent");
    report.native_completed_updates = original.len() as u64;
    report.prelude_polls = polls.iter().filter(|p| p.source_event < original[0].source_event).count() as u64;
    let mut subjects = BTreeMap::new();
    let mut modes = vec![
        ("candidate", "unchanged", false),
        ("changed-gameplay", "changed-rng", false),
        ("causal-candidate", "causal-polling", true),
        ("causal-control", "changed-causal-rng", true),
    ];
    if frames > 320 {
        modes.extend([
            ("omitted-wait-control", "omitted-causal-wait", true),
            ("omitted-screen-wait-control", "omitted-causal-screen-wait", true),
        ]);
    }
    for (name, mode, is_causal) in modes {
        let directory = run.join(name);
        fs::create_dir(&directory)?;
        fs::create_dir(directory.join("fixtures"))?;
        fs::create_dir(directory.join("saves"))?;
        let mut command = vec![
            run.join("prefix-host").to_string_lossy().into_owned(),
            rom.to_string_lossy().into_owned(),
            run.join(if is_causal { "causal.inputs" } else { "derived.inputs" }).to_string_lossy().into_owned(),
            polls.len().to_string(),
            mode.into(),
        ];
        if is_causal {
            command.push(original.len().to_string());
        }
        execute(&command, &directory, &shared, workers, timeout)?;
        let rows = candidate_events(&directory.join("events.txt"))?;
        let complete = rows.iter().filter(|r| r.kind == "dispatch-complete").cloned().collect::<Vec<_>>();
        ensure!(!complete.is_empty(), "candidate completed no dispatches");
        let mut failure = None;
        if is_causal {
            let prior = rows
                .iter()
                .filter(|r| r.kind == "controller-return" && r.event < complete[0].event)
                .map(|r| u64::from(r.state["held"]))
                .collect::<Vec<_>>();
            ensure!(
                prior
                    == polls
                        .iter()
                        .take(report.prelude_polls as usize)
                        .map(|r| u64::from(r.state["held"]))
                        .collect::<Vec<_>>(),
                "causal prelude order differs"
            );
            let actual = rows
                .iter()
                .filter(|r| r.kind == "controller-return")
                .map(|r| u64::from(r.state["held"]))
                .collect::<Vec<_>>();
            ensure!(
                actual == polls.iter().map(|r| u64::from(r.state["held"])).collect::<Vec<_>>(),
                "causal controller samples changed or reordered"
            );
            for (index, (boundary, native)) in complete.iter().zip(&original).enumerate() {
                let count = rows.iter().filter(|r| r.kind == "controller-return" && r.event < boundary.event).count();
                let expected = polls.iter().filter(|r| r.source_event < native.source_event).count();
                if count != expected {
                    failure = Some(index as u64 + 1);
                    break;
                }
            }
            if frames == 320 {
                ensure!(
                    failure.is_none() && complete.len() == original.len(),
                    "bounded causal provenance/completion differs"
                );
            }
        }
        report.subjects.insert(
            name.into(),
            Subject {
                command,
                artifacts: hashes(&directory)?,
                completed_updates: complete.len() as u64,
                first_poll_failure: failure,
                vblank_wait_hits: rows.iter().filter(|r| r.kind == "within-dispatch-vblank-wait").count() as u64,
                screen_wait_hits: rows.iter().filter(|r| r.kind == "within-dispatch-screen-on-wait").count() as u64,
            },
        );
        subjects.insert(name, complete);
    }
    report.comparisons = compare(&original, &subjects["candidate"])?;
    report.causal_comparisons = compare(&original, &subjects["causal-candidate"])?;
    report.changed_rng_detected = changed(&subjects["candidate"], &subjects["changed-gameplay"]);
    report.changed_causal_rng_detected = changed(&subjects["causal-candidate"], &subjects["causal-control"]);
    ensure!(
        report.changed_rng_detected && report.changed_causal_rng_detected,
        "changed RNG escaped independent comparison"
    );
    ensure!(report.causal_comparisons[0].differences.is_empty(), "causal first completion differs");
    if frames == 320 {
        ensure!(report.causal_comparisons.iter().all(|r| r.differences.is_empty()), "bounded causal semantic mismatch");
    } else {
        for name in ["omitted-wait-control", "omitted-screen-wait-control"] {
            ensure!(report.subjects[name].first_poll_failure.is_some(), "omitted wait escaped provenance control");
        }
    }
    report.causal_completed_updates = subjects["causal-candidate"].len() as u64;
    report.matching_causal_prefix = (report
        .causal_comparisons
        .iter()
        .position(|r| !r.differences.is_empty())
        .unwrap_or(report.causal_comparisons.len()) as u64)
        .min(
            report.subjects["causal-candidate"]
                .first_poll_failure
                .unwrap_or(report.causal_comparisons.len() as u64 + 1)
                - 1,
        );
    for comparison in &report.comparisons {
        if let Some(assigned) = polls.iter().find(|r| r.host_step == Some(comparison.host_step))
            && assigned.source_event > comparison.source_event
        {
            report.future_poll_assignments.push((
                comparison.source_event,
                comparison.source_frame,
                assigned.source_event,
            ));
        }
    }
    report.explicit_wait_events = events.iter().filter(|r| r.kind == "explicit-wait").map(|r| r.source_event).collect();
    // Authenticate mutable inputs again after the original and all C controls.
    decomp_source::verify_source(root, &source)?;
    ensure!(digest(&rom)? == ROM_SHA256, "ROM changed during investigation");
    ensure!(digest(&movie)? == report.native.as_ref().unwrap().movie_sha256, "movie changed during investigation");
    report.status = "nonaccepting".into();
    Ok(())
}
pub(crate) fn operate(request: Request) -> Result<(PathBuf, bool)> {
    let root = fs::canonicalize(request.checkout.clone().unwrap_or(std::env::current_dir()?))?;
    let assets = fs::canonicalize(request.asset_root.clone().unwrap_or(root.clone()))?;
    let frames = request.frames;
    ensure!((320..=2000).contains(&frames), "bounded prefix requires 320..2000 frames");
    let output = request.output.clone().unwrap_or(root.join("out/decomp/prefix"));
    fs::create_dir_all(&output)?;
    let directory =
        output.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id()));
    fs::create_dir(&directory)?;
    let directory = fs::canonicalize(directory)?;
    let mut report = Report {
        schema_version: 1,
        status: "error".into(),
        accepted: false,
        requested_frames: frames,
        scope:
            "source-ordered controller and completed-update semantics; original-ROM timing/full WRAM parity unproven"
                .into(),
        error: None,
        native: None,
        subjects: BTreeMap::new(),
        first_unsupported_frame: None,
        poll_cases: 0,
        poll_primitive_gameplay_match: false,
        poll_primitive_full_match: false,
        changed_sample_detected: false,
        changed_state_detected: false,
        changed_rng_detected: false,
        changed_causal_rng_detected: false,
        prelude_polls: 0,
        matching_causal_prefix: 0,
        native_completed_updates: 0,
        causal_completed_updates: 0,
        comparisons: vec![],
        causal_comparisons: vec![],
        future_poll_assignments: vec![],
        explicit_wait_events: vec![],
        reused_native: None,
        artifacts: BTreeMap::new(),
    };
    if let Err(error) = investigate(&root, &assets, &directory, &request, &mut report) {
        report.error = Some(format!("{error:#}"));
    }
    report.artifacts = hashes(&directory)?;
    pipeline_model::write(&directory.join("verdict.toml"), &report)?;
    #[derive(Serialize)]
    struct Receipt {
        status: String,
        evidence: PathBuf,
        accepted: bool,
        error: Option<String>,
    }
    println!(
        "{}",
        toml::to_string_pretty(&Receipt {
            status: report.status.clone(),
            evidence: directory.clone(),
            accepted: false,
            error: report.error.clone()
        })?
    );
    Ok((directory, report.status == "nonaccepting"))
}
pub(crate) fn run(args: &[String]) -> Result<()> {
    let mut request = Request::default();
    for pair in args.chunks(2) {
        ensure!(pair.len() == 2, "prefix option value missing");
        match pair[0].as_str() {
            "--root" => request.checkout = Some(pair[1].clone().into()),
            "--asset-root" => request.asset_root = Some(pair[1].clone().into()),
            "--movie" => request.movie = Some(pair[1].clone().into()),
            "--reuse-native" => request.reuse_native = Some(pair[1].clone().into()),
            "--frames" => request.frames = pair[1].parse()?,
            "--timeout" => request.timeout = pair[1].parse()?,
            "--output" => request.output = Some(pair[1].clone().into()),
            other => anyhow::bail!("unknown prefix option {other}"),
        }
    }
    let (_, complete) = operate(request)?;
    std::process::exit(if complete { 2 } else { 1 });
}

pub(crate) fn authenticate(owner: &Path) -> Result<()> {
    let report: Report = pipeline_model::read(&owner.join("verdict.toml"))?;
    ensure!(
        report.schema_version == 1 && report.status == "nonaccepting" && !report.accepted && report.error.is_none(),
        "prefix diagnostic incomplete"
    );
    let mut actual = hashes(owner)?;
    actual.remove("verdict.toml");
    ensure!(actual == report.artifacts, "prefix evidence changed");
    let native = report.native.context("native capture missing")?;
    ensure!(
        hashes(&owner.join("native"))? == native.artifacts
            && digest(&owner.join("native/observe.lua"))? == native.observer_sha256,
        "prefix native seal differs"
    );
    for (name, subject) in report.subjects {
        ensure!(hashes(&owner.join(name))? == subject.artifacts, "prefix subject seal differs");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clearing_severs_poll_ownership_and_completion_never_borrows_future_input() {
        let event = |number, kind: &str| Event {
            source_frame: 272,
            source_event: number,
            kind: kind.into(),
            address: 0,
            value: 0,
            state: BTreeMap::new(),
            hardware_pad: None,
            controller_entry_event: None,
            host_step: None,
            classification: String::new(),
        };
        let rows = causal(&[
            event(23, "controller-return"),
            event(1005, "boot-clear"),
            event(1024, "dispatch-entry"),
            event(1025, "dispatch-complete"),
            event(1030, "controller-return"),
            event(1068, "dispatch-complete"),
        ]);
        assert_eq!(rows[0].phase, "prelude");
        assert_eq!(rows[1].cleared_poll, Some(23));
        assert_eq!(rows[3].preceding_poll, None);
        assert_eq!(rows[5].preceding_poll, Some(1030));
    }
    #[test]
    fn observer_keeps_read_addresses_passive_and_rejects_lua_delimiter() {
        let script = observer(Path::new("/tmp/prefix space")).unwrap();
        assert!(script.contains("observe('controller-register',16920,v)"));
        assert!(script.contains("memory.readbyte('WRAM',a)"));
        assert!(observer(Path::new("/tmp/]]")).is_err());
    }
}
