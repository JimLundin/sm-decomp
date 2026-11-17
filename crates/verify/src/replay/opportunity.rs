//! Reviewed passive capture admission, independent of expected observations.
use super::{data_failure, movement, readers::record, result::Outcome};
use crate::{cache::Cache, lock::Sha256};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

const PATH: [u32; 21] = [
    0x82_8b58, 0x90_e692, 0x90_e6b3, 0x90_e6bc, 0x90_e6c3, 0x82_8b69, 0x90_e722, 0x90_e8aa,
    0x90_e8b0, 0x90_e725, 0x90_e94b, 0x90_a337, 0x90_e734, 0x90_e738, 0x90_e73b, 0x90_e73e,
    0x90_e741, 0x90_e745, 0x90_e749, 0x90_e8b4, 0x82_8b6d,
];
const MANIFEST: &str =
    include_str!("../../../../scenarios/replay/movement-opportunities-capture-manifest.toml");
const PINS: &str =
    include_str!("../../../../scenarios/replay/movement-opportunities-capture-pins.toml");
const FIRST: &str =
    include_str!("../../../../scenarios/replay/movement-opportunities-first-record-manifest.toml");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    file: Vec<File>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    name: String,
    size: usize,
    sha256: String,
}

/// Four complete reviewed opportunities; no default on missing or incompatible evidence.
#[derive(Clone)]
pub(super) struct Evidence {
    completed: [u64; 4],
}
impl Evidence {
    pub fn require(&self, cut: u64) -> Result<(), Outcome> {
        premise(
            self.completed.contains(&cut),
            "reviewed opportunity interval",
        )
    }
}
fn missing() -> Outcome {
    data_failure(None, "movement opportunity capture integrity or structure")
}
fn premise(ok: bool, dependency: &str) -> Result<(), Outcome> {
    if ok {
        Ok(())
    } else {
        Err(Outcome::Unsupported {
            cut: None,
            dependency: dependency.to_owned(),
        })
    }
}
fn read(root: &Path, name: &str) -> Result<Vec<u8>, Outcome> {
    std::fs::read(root.join(name)).map_err(|_| missing())
}
fn pinned(root: &Path, name: &str, expected: &str) -> Result<(), Outcome> {
    if read(root, name)? == expected.as_bytes() {
        Ok(())
    } else {
        Err(missing())
    }
}
fn files(root: &Path, manifest: &str) -> Result<BTreeMap<String, Vec<u8>>, Outcome> {
    let manifest: Manifest = toml::from_str(manifest).map_err(|_| missing())?;
    let mut files = BTreeMap::new();
    for file in manifest.file {
        if std::path::Path::new(&file.name)
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(missing());
        }
        let raw = read(root, &file.name)?;
        if raw.len() != file.size
            || Sha256::of(&raw).to_string() != file.sha256
            || files.insert(file.name, raw).is_some()
        {
            return Err(missing());
        }
    }
    Ok(files)
}
#[derive(Clone, Copy)]
struct Event {
    pc: u32,
    v: u16,
    sample: u32,
}
fn trace(text: &str, raw: &BTreeMap<String, Vec<u8>>) -> Result<Vec<Vec<Event>>, Outcome> {
    let mut segments = Vec::new();
    let mut events = Vec::new();
    for line in text.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.first() == Some(&"cut") {
            segments.push(std::mem::take(&mut events));
        } else {
            let [_, pc, vertical, horizontal, sample] = fields.as_slice() else {
                return Err(missing());
            };
            let pc = u32::from_str_radix(pc, 16).map_err(|_| missing())?;
            let v = vertical.parse().map_err(|_| missing())?;
            let _: u16 = horizontal.parse().map_err(|_| missing())?;
            let sample = sample.parse().map_err(|_| missing())?;
            events.push(Event { pc, v, sample });
        }
    }
    if segments.len() != 11 || !events.is_empty() {
        return Err(missing());
    }
    let rti = segments
        .iter()
        .flatten()
        .find(|event| event.sample == 0)
        .ok_or_else(missing)?
        .pc;
    let mut carried = Vec::new();
    let mut dangling = true;
    let mut result = Vec::new();
    for (cut, mut events) in segments.into_iter().enumerate() {
        if dangling && let Some(at) = events.iter().position(|event| event.pc == rti) {
            events.drain(..=at);
            dangling = false;
        }
        if events
            .first()
            .is_some_and(|event| event.pc == rti && event.v >= 241)
        {
            events.remove(0);
        }
        let mut selected = std::mem::take(&mut carried);
        let bytes = raw
            .get(&format!("cut-{cut:02}.record"))
            .ok_or_else(missing)?;
        if bytes.is_empty() {
            return Err(missing());
        }
        if bytes.get(4) == Some(&0)
            && let Some(at) = events.iter().rposition(|event| event.pc == rti)
        {
            carried = events.split_off(at.saturating_add(1));
        }
        selected.extend(events.into_iter().filter(|event| event.pc != rti));
        result.push(selected);
    }
    Ok(result)
}
fn word(raw: &[u8], at: usize) -> Result<u16, Outcome> {
    let bytes = raw.get(at..at.saturating_add(2)).ok_or_else(missing)?;
    Ok(u16::from_le_bytes(bytes.try_into().map_err(|_| missing())?))
}
fn equal(raw: &[u8], prior: &[u8], fields: &[usize]) -> Result<bool, Outcome> {
    for at in fields {
        if word(raw, *at)? != word(prior, *at)? {
            return Ok(false);
        }
    }
    Ok(true)
}
fn zeros(raw: &[u8], fields: &[usize]) -> Result<bool, Outcome> {
    for at in fields {
        if word(raw, *at)? != 0 {
            return Ok(false);
        }
    }
    Ok(true)
}
fn snapshot<'a>(
    events: &[Event],
    files: &'a BTreeMap<String, Vec<u8>>,
    pc: u32,
) -> Result<&'a [u8], Outcome> {
    let event = events
        .iter()
        .find(|event| event.pc == pc)
        .ok_or_else(missing)?;
    let bytes = files
        .get(&format!("sample-{:06}.wram", event.sample))
        .ok_or_else(missing)?;
    if bytes.len() != 0x2_0000 {
        return Err(missing());
    }
    Ok(bytes)
}
fn predicates(
    events: &[Event],
    files: &BTreeMap<String, Vec<u8>>,
    prior: &[u8],
    content: &movement::Content,
) -> Result<(), Outcome> {
    let snap = |pc| snapshot(events, files, pc);
    for pc in [0x82_8b58, 0x90_e692] {
        premise(word(snap(pc)?, 0x0a42)? == 0xe695, "normal current target")?;
    }
    premise(
        word(snap(0x90_e6b3)?, 0x0a60)? == 0xe913,
        "normal pose input",
    )?;
    for pc in [0x82_8b69, 0x90_e722] {
        premise(
            [0xe725, 0xe8aa].contains(&word(snap(pc)?, 0x0a44)?),
            "normal new target",
        )?;
    }
    premise(
        word(snap(0x90_e94b)?, 0x0a58)? == 0xa337,
        "ordinary movement target",
    )?;
    let moving = snap(0x90_a337)?;
    premise(word(moving, 0x0a78)? == 0, "unfrozen movement")?;
    premise(
        movement::ordinary_body(moving, content)?,
        "imported ordinary body action",
    )?;
    premise(
        zeros(moving, &[0x0b56, 0x0b58, 0x0b5a, 0x0b5c])?,
        "unforced movement",
    )?;
    premise(
        word(snap(0x90_e738)?, 0x0a5a)? == 0xe0e6,
        "timer only posthandler",
    )?;
    premise(
        equal(snap(0x90_e6b3)?, prior, &[0x008b, 0x008f])?,
        "prior controls at input",
    )?;
    premise(
        equal(moving, prior, &[0x008b, 0x008f])?,
        "prior controls at movement",
    )?;
    premise(
        equal(moving, prior, &[0x0a1c, 0x0a1e, 0x0afe, 0x0b00])?,
        "prior body at movement",
    )?;
    premise(
        zeros(snap(0x90_e73e)?, &[0x18aa, 0x0a52, 0x0a56])?,
        "no hit knockback or bomb jump intervention",
    )?;
    premise(
        equal(
            moving,
            prior,
            &[
                0x0af6, 0x0af8, 0x0afa, 0x0afc, 0x0b46, 0x0b48, 0x0b4a, 0x0b42, 0x0b44, 0x0b3c,
                0x0b2e, 0x0b2c, 0x0b36, 0x0b20, 0x0dba, 0x0af4,
            ],
        )?,
        "prior selected state at movement read",
    )?;
    premise(
        word(moving, 0x0a2a)? == u16::MAX && word(moving, 0x0a2c)? == u16::MAX,
        "no preselected special pose",
    )?;
    premise(
        equal(moving, prior, &[0x09a2])?,
        "prior equipment at movement",
    )?;
    premise(
        word(snap(0x90_e73b)?, 0x0a5a)? == 0xe0e6,
        "posthandler remains timer only",
    )?;
    for pc in [0x90_e6b3, 0x90_a337, 0x90_e73e, 0x90_e749] {
        premise(
            word(snap(pc)?, 0x0a60)? == 0xe913,
            "normal input through post",
        )?;
    }
    Ok(())
}
fn interval(
    events: &[Event],
    files: &BTreeMap<String, Vec<u8>>,
    prior: &[u8],
    content: &movement::Content,
) -> Result<(), Outcome> {
    premise(
        events.iter().map(|event| event.pc).eq(PATH),
        "exact one complete ordered opportunity",
    )?;
    predicates(events, files, prior, content)
}

pub(super) fn admit(
    root: &Path,
    cache: &Cache,
    content: &movement::Content,
) -> Result<Evidence, Outcome> {
    pinned(root, "capture-manifest.toml", MANIFEST)?;
    pinned(root, "capture-pins.toml", PINS)?;
    pinned(root, "first-record-manifest.toml", FIRST)?;
    let first = files(root, FIRST)?;
    let files = files(&root.join("capture-private-2"), MANIFEST)?;
    let trace_text =
        std::str::from_utf8(files.get("trace.log").ok_or_else(missing)?).map_err(|_| missing())?;
    let events = trace(trace_text, &files)?;
    for cut in 6..=10 {
        let bytes = files
            .get(&format!("cut-{cut:02}.record"))
            .ok_or_else(missing)?;
        if first.get(&format!("capture-private/cut-{cut:02}.record")) != Some(bytes) {
            return Err(missing());
        }
        let mut captured = oracle_record::lsnes::blank();
        oracle_record::lsnes::decode(bytes, &mut captured).map_err(|_| missing())?;
        let input = record(cache, cut)?;
        premise(
            captured.wram == input.wram
                && captured.cpu == input.cpu
                && captured.cut == input.cut
                && captured.video_frame == input.video_frame
                && captured.pad == input.pad
                && captured.joypad_reads == input.joypad_reads,
            "independent capture input alignment",
        )?;
        premise(
            matches!(
                captured.cut,
                oracle_record::Cut::NmiReturn(oracle_record::Nmi {
                    kind: oracle_record::NmiKind::Requested,
                    polled: true,
                    ..
                })
            ),
            "requested completed cut",
        )?;
        if cut > 6 {
            let selected = events
                .get(usize::try_from(cut).map_err(|_| missing())?)
                .ok_or_else(missing)?;
            interval(
                selected,
                &files,
                &record(cache, cut.checked_sub(1).ok_or_else(missing)?)?.wram,
                content,
            )?;
        }
    }
    Ok(Evidence {
        completed: [7, 8, 9, 10],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "sm-opportunity-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn missing_or_altered_raw_files_reject_integrity_before_semantics() {
        let root = temp();
        let raw = b"invented passive capture";
        let manifest = format!(
            "[[file]]\nname = \"sample.wram\"\nsize = {}\nsha256 = \"{}\"\n",
            raw.len(),
            Sha256::of(raw)
        );
        assert!(matches!(
            files(&root, &manifest),
            Err(Outcome::DataFailure { .. })
        ));
        std::fs::write(root.join("sample.wram"), raw).unwrap();
        assert!(files(&root, &manifest).is_ok());
        let mut changed = raw.to_vec();
        changed[0] ^= 1;
        std::fs::write(root.join("sample.wram"), changed).unwrap();
        assert!(matches!(
            files(&root, &manifest),
            Err(Outcome::DataFailure { .. })
        ));
        std::fs::remove_file(root.join("sample.wram")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "private reviewed opportunity capture, cache and player ROM"]
    fn intact_incompatible_paths_and_actual_reads_are_unsupported() {
        let root = std::path::PathBuf::from(std::env::var_os("SM_AUDIO_NATIVE").unwrap())
            .join("movement-opportunities-v1");
        let content = movement::Content::load(&std::path::PathBuf::from(
            std::env::var_os("SM_ROM").unwrap(),
        ))
        .unwrap();
        let lock = crate::lock::Lock::parse(include_str!("../../../../scenarios/locks/lsnes.toml"))
            .unwrap();
        let cache = Cache::open(
            &std::path::PathBuf::from(std::env::var_os("SM_CACHE").unwrap()),
            lock.row(&crate::lock::RowId("chain-any-0003-DFD7".to_owned()))
                .unwrap(),
        )
        .unwrap();
        let evidence = admit(&root, &cache, &content).unwrap();
        assert!(evidence.require(7).is_ok());
        assert!(matches!(
            evidence.require(11),
            Err(Outcome::Unsupported { .. })
        ));
        let raw = files(&root.join("capture-private-2"), MANIFEST).unwrap();
        let events = trace(
            std::str::from_utf8(raw.get("trace.log").unwrap()).unwrap(),
            &raw,
        )
        .unwrap();
        let selected = &events[7];
        let prior = record(&cache, 6).unwrap();
        assert!(interval(selected, &raw, &prior.wram, &content).is_ok());
        for (pc, at) in [
            (0x90_e6b3, 0x0a60),
            (0x90_e94b, 0x0a58),
            (0x90_a337, 0x0a78),
            (0x90_a337, 0x0b56),
            (0x90_a337, 0x0af6),
            (0x90_a337, 0x008b),
            (0x90_e73e, 0x0a56),
        ] {
            let sample = selected.iter().find(|event| event.pc == pc).unwrap().sample;
            let mut altered = raw.clone();
            altered
                .get_mut(&format!("sample-{sample:06}.wram"))
                .unwrap()[at] ^= 1;
            assert!(matches!(
                interval(selected, &altered, &prior.wram, &content),
                Err(Outcome::Unsupported { .. })
            ));
        }
        {
            let mut changed = selected.clone();
            changed.pop();
            assert!(matches!(
                interval(&changed, &raw, &prior.wram, &content),
                Err(Outcome::Unsupported { .. })
            ));
        }
        let mut doubled = selected.clone();
        doubled.push(selected[10]);
        assert!(matches!(
            interval(&doubled, &raw, &prior.wram, &content),
            Err(Outcome::Unsupported { .. })
        ));
        let sample = selected
            .iter()
            .find(|event| event.pc == 0x90_a337)
            .unwrap()
            .sample;
        let name = format!("sample-{sample:06}.wram");
        let bytes = raw.get(&name).unwrap();
        let temporary = temp();
        std::fs::write(temporary.join(&name), bytes).unwrap();
        let manifest = format!(
            "[[file]]\nname = \"{name}\"\nsize = {}\nsha256 = \"{}\"\n",
            bytes.len(),
            Sha256::of(bytes)
        );
        assert!(files(&temporary, &manifest).is_ok());
        let mut mutated = bytes.clone();
        mutated[0x0a78] ^= 1;
        std::fs::write(temporary.join(&name), mutated).unwrap();
        assert!(matches!(
            files(&temporary, &manifest),
            Err(Outcome::DataFailure { .. })
        ));
        std::fs::remove_file(temporary.join(name)).unwrap();
        std::fs::remove_dir(temporary).unwrap();
        assert!(admit(&root, &cache, &content).is_ok());
    }
}
