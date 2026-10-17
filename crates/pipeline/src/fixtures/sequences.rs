//! Actual-C sequence capture for native glass, spark and Maridia tube fixtures.
use super::*;
use anyhow::Context;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SequenceKind {
    Glass,
    Sparks,
    N00bTube,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceRequest {
    pub checkout: PathBuf,
    pub kind: SequenceKind,
    pub source: PathBuf,
    pub caller_source: Option<PathBuf>,
    pub output: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
}

fn patch(state: &mut [u8], values: impl IntoIterator<Item = (usize, i64)>) {
    for write in words(values) {
        state[write.offset..write.offset + write.data.len()].copy_from_slice(&write.data);
    }
}

struct Capture<'a> {
    request: &'a SequenceRequest,
    source: PathBuf,
    directory: PathBuf,
    host: Vec<u16>,
}

impl Capture<'_> {
    fn call(&self, name: &str, symbol: &str, state: &[u8], a: i64, b: i64) -> Result<(Case, Vec<u8>)> {
        ensure!(state.len() == 0x20000, "sequence state extent differs");
        let directory = self.directory.join(name);
        fs::create_dir(&directory)?;
        let before = directory.join("before.wram");
        let after = directory.join("c-after.wram");
        let host = directory.join("host-before.bin");
        let host_after = directory.join("c-host-after.bin");
        let host_bytes: Vec<u8> = self.host.iter().flat_map(|word| word.to_le_bytes()).collect();
        fs::write(&before, state)?;
        fs::write(&host, &host_bytes)?;
        let command = vec![
            self.source.join("c").to_string_lossy().into_owned(),
            symbol.into(),
            a.to_string(),
            b.to_string(),
            before.to_string_lossy().into_owned(),
            after.to_string_lossy().into_owned(),
            self.source.join("rom.bin").to_string_lossy().into_owned(),
            host.to_string_lossy().into_owned(),
            host_after.to_string_lossy().into_owned(),
        ];
        tool_execution::builds::execute_timeout(
            &command,
            &directory,
            "c",
            &self.request.root,
            self.request.workers,
            30,
        )?;
        let stdout = fs::read_to_string(directory.join("c.stdout"))?;
        let returned = stdout.trim().lines().last().context("sequence return missing")?.parse()?;
        let next = fs::read(&after)?;
        ensure!(next.len() == 0x20000, "sequence output extent differs");
        ensure!(fs::read(host_after)? == host_bytes, "sequence unexpectedly changed external host");
        Ok((
            Case {
                name: name.into(),
                a,
                b,
                before_sha256: Some(crate::digest(&before)?),
                before_file: Some(before),
                expected_after_sha256: Some(crate::digest(&after)?),
                expected_after_file: Some(after),
                expected_return: Some(returned),
                host_words: Some(self.host.clone()),
                ..Default::default()
            },
            next,
        ))
    }
}

pub fn generate_sequence(request: SequenceRequest) -> Result<Generation> {
    ensure!(request.workers > 0, "sequence worker count must be positive");
    let checkout = fs::canonicalize(&request.checkout)?;
    let source = fs::canonicalize(&request.source)?;
    ensure!(source.join("c").is_file(), "sequence C harness missing");
    let native_rom = super::rom(&checkout)?;
    ensure!(fs::read(source.join("rom.bin"))? == native_rom, "sequence harness ROM identity differs");
    let caller = request.caller_source.as_ref().map(fs::canonicalize).transpose()?;
    let (family, template, frames) = match request.kind {
        SequenceKind::Glass => ("mother-brain-glass-sequence", "mother-brain-glass-frames", 128),
        SequenceKind::Sparks => ("sparks-sequence", "sparks-frames", 96),
        SequenceKind::N00bTube => ("n00b-tube-sequence", "n00b-tube", 96),
    };
    let mut manifest = load(&checkout, template)?;
    let mut state = vec![0; 0x20000];
    let mut host = vec![0x1234, 0xabcd, 1, 2, 3, 4, 5, 6];
    match request.kind {
        SequenceKind::Glass => {
            let caller = caller.as_ref().context("glass sequence requires caller_source")?;
            let case = caller.join("plm-0-18-0-53897");
            state = fs::read(case.join("c-after.wram"))?;
            let bytes = fs::read(case.join("c-host-after.bin"))?;
            ensure!(bytes.len() == 16, "glass caller host extent differs");
            host = bytes.as_chunks::<2>().0.iter().map(|word| u16::from_le_bytes(*word)).collect();
            ensure!(host == [11, 22, 33, 44, 55, 66, 77, 88], "glass caller host identity differs");
        }
        SequenceKind::N00bTube => {
            let caller = caller.as_ref().context("tube sequence requires caller_source")?;
            state = fs::read(caller.join("caller-d507-18-0/c-after.wram"))?;
        }
        SequenceKind::Sparks => {
            patch(
                &mut state,
                [(0xe54, 0), (0xf7a, 128), (0xf7c, 0xfe37), (0xf7e, 120), (0xf80, 0xfe73), (0x5e5, 0), (0x7a5, 64)],
            );
            patch(&mut state, (0..64).map(|x| (0x10002 + (10 * 64 + x) * 2, 0x8000)));
        }
    }
    ensure!(state.len() == 0x20000, "sequence caller state extent differs");
    patch(&mut state, [(0x198d, 0x8000)]);
    if matches!(request.kind, SequenceKind::N00bTube) {
        patch(&mut state, [(0x915, 1024)]);
    }
    let output = if request.output.is_absolute() { request.output.clone() } else { checkout.join(&request.output) };
    fs::create_dir_all(output.parent().context("sequence output parent missing")?)?;
    ensure!(!output.exists(), "sequence manifest already exists");
    let directory = output.with_extension("inputs");
    fs::create_dir(&directory).context("sequence inputs must use a fresh output path")?;
    let directory = fs::canonicalize(directory)?;
    let capture = Capture { request: &request, source, directory, host: host.clone() };
    let mut routines = Vec::new();
    match request.kind {
        SequenceKind::Glass | SequenceKind::Sparks => {
            routines.push(manifest.routines.first().context("sequence control missing")?.clone());
        }
        SequenceKind::N00bTube => {
            let mut control = routine(
                "EprojInstruction",
                0,
                "rom_ptr",
                vec![Case {
                    name: "sequence-native-control".into(),
                    fill: Some(0),
                    a: 0,
                    b: 0x86d51b,
                    host_words: Some(host),
                    ..Default::default()
                }],
            );
            control.arg_types = Some(vec!["uint16".into(), "rom_ptr".into()]);
            routines.push(control);
        }
    }
    if matches!(request.kind, SequenceKind::Sparks) {
        let (allocated, next) = capture.call("sequence-allocate", "SpawnEprojWithRoomGfx", &state, 0xf498, 0)?;
        state = next;
        routines.push(routine("SpawnEprojWithRoomGfx", 2, "void", vec![allocated]));
    }
    let mut cases = Vec::new();
    for frame in 0..frames {
        match request.kind {
            SequenceKind::Sparks => state[0x5b5] = frame as u8,
            SequenceKind::N00bTube => patch(&mut state, [(0x5b6, frame as i64)]),
            SequenceKind::Glass => (),
        }
        let (case, next) = capture.call(&format!("sequence-{frame:03}"), "EprojRunAll", &state, 0, 0)?;
        cases.push(case);
        state = next;
    }
    routines.push(routine("EprojRunAll", 0, "void", cases));
    manifest.family = family.into();
    manifest.routines = routines;
    manifest.case_order = None;
    offline_verification::fixtures::write(&output, &manifest)?;
    Ok(Generation {
        family: family.into(),
        manifest: output,
        cases: manifest.routines.iter().map(|r| r.cases.len()).sum(),
    })
}
