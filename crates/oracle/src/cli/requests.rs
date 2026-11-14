//! Complete command requests are validated without acquiring inputs or outputs.
use super::FramePads;
use crate::Tiers;
use std::num::NonZeroUsize;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum RequestError {
    Invalid(String),
    Compatibility,
    Shard,
}
impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => f.write_str(message),
            Self::Compatibility => f.write_str("incompatible backend or capture options"),
            Self::Shard => f.write_str("--shard I/N requires N > 0 and I < N"),
        }
    }
}

pub(super) enum BackendName {
    Lsnes,
    LakeSnes(String),
}
impl From<String> for BackendName {
    fn from(name: String) -> Self {
        if name == "lsnes" {
            Self::Lsnes
        } else {
            Self::LakeSnes(name)
        }
    }
}
impl BackendName {
    pub(super) fn into_name(self) -> String {
        match self {
            Self::Lsnes => "lsnes".to_owned(),
            Self::LakeSnes(name) => name,
        }
    }
}

pub(super) struct Shard {
    index: usize,
    count: NonZeroUsize,
}
impl Default for Shard {
    fn default() -> Self {
        Self {
            index: 0,
            count: NonZeroUsize::MIN,
        }
    }
}
impl Shard {
    fn new(index: usize, count: usize) -> Result<Self, RequestError> {
        let count = NonZeroUsize::new(count).ok_or(RequestError::Shard)?;
        if index >= count.get() {
            return Err(RequestError::Shard);
        }
        Ok(Self { index, count })
    }
    pub(super) fn includes(&self, position: usize) -> bool {
        position % self.count.get() == self.index
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum MovieStart {
    PowerOn,
    Saved { state: PathBuf, frame: usize },
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum CrossStart {
    PowerOn,
    Saved { state: PathBuf, step: u64 },
}

pub(super) struct MovieRequest {
    pub rom: PathBuf,
    pub inputs: PathBuf,
    pub frames: Option<usize>,
    pub seeds: Option<(PathBuf, PathBuf)>,
    pub picture: bool,
    pub streams: Option<Tiers>,
    pub ranges: bool,
    pub lag_log: Option<PathBuf>,
    pub watch: Option<(usize, usize)>,
    pub trace: Option<(usize, usize, Vec<u32>)>,
    pub spc: bool,
    pub positions_only: bool,
    pub profile: Option<(usize, usize)>,
    pub backend: BackendName,
    pub native: Vec<(PathBuf, PathBuf)>,
    pub memories: bool,
    pub start: MovieStart,
    pub cutting: Option<(u64, PathBuf)>,
    pub from_ladder: Option<PathBuf>,
}
pub(super) struct CorpusRequest {
    pub rom: PathBuf,
    pub lock: PathBuf,
    pub data: PathBuf,
    pub only: Vec<String>,
    pub limit: usize,
    pub shard: Shard,
    pub outputs: Vec<CorpusOutput>,
    pub sidecar: Option<PathBuf>,
    pub dump_frames: Vec<(String, u64)>,
    pub backend: BackendName,
    pub native_seeds: Option<PathBuf>,
    pub watched: Vec<usize>,
    pub watched_pcs: Vec<u32>,
    pub pads: Option<FramePads>,
    pub lean: bool,
    pub cache: Option<PathBuf>,
    pub cache_pictures: crate::record::codec::PictureMode,
}
pub(super) enum CorpusOutput {
    Record(PathBuf),
    RecordV2(PathBuf),
    Directory(PathBuf),
}
pub(super) struct CrossRequest {
    pub rom: PathBuf,
    pub inputs: PathBuf,
    pub range: Option<std::ops::RangeInclusive<u64>>,
    pub first_ranges: bool,
    pub streams: bool,
    pub ranges_at: Option<std::ops::RangeInclusive<u64>>,
    pub right_start: CrossStart,
}
impl MovieRequest {
    pub(super) fn parse(arguments: &[String]) -> Result<Self, RequestError> {
        let [rom, inputs, rest @ ..] = arguments else {
            return Err(RequestError::Invalid("movie ROM INPUTS [--frames N] [--seeds CSV DIR] [--ranges] [--lag-log CSV] [--watch FROM EVERY] [--trace FROM TO PC,...] [--spc] [--no-picture] [--streams] [--ppu] [--oracle lakesnes|lsnes] [--native-seeds CSV DIR]... [--no-memories] [--from STATE FRAME] [--ladder N DIR] [--from-ladder DIR]".to_string()));
        };
        let mut frames = None;
        let mut seeds: Option<(PathBuf, PathBuf)> = None;
        let mut picture = true;
        // The record streams asked of lsnes: none, or the tiers with the PPU registers.
        let mut streams: Option<Tiers> = None;
        let mut ranges = false;
        let mut lag_log: Option<PathBuf> = None;
        let mut watch: Option<(usize, usize)> = None;
        let mut trace: Option<(usize, usize, Vec<u32>)> = None;
        let mut spc = false;
        let mut positions_only = false;
        let mut profile: Option<(usize, usize)> = None;
        let mut backend = "lakesnes".to_owned();
        let mut native: Vec<(PathBuf, PathBuf)> = Vec::new();
        let mut memories = true;
        let mut start = MovieStart::PowerOn;
        let mut cutting: Option<(u64, PathBuf)> = None;
        let mut from_ladder: Option<PathBuf> = None;
        let mut options = rest.iter();
        while let Some(option) = options.next() {
            match option.as_str() {
                "--frames" => {
                    frames = Some(
                        options
                            .next()
                            .and_then(|value| value.parse().ok())
                            .ok_or_else(|| RequestError::Invalid("--frames N".to_string()))?,
                    );
                }
                "--seeds" => {
                    let (Some(csv), Some(directory)) = (options.next(), options.next()) else {
                        return Err(RequestError::Invalid("--seeds CSV DIR".to_string()));
                    };
                    seeds = Some((PathBuf::from(csv), PathBuf::from(directory)));
                }
                "--oracle" => {
                    backend = options
                        .next()
                        .cloned()
                        .ok_or_else(|| RequestError::Invalid("--oracle NAME".to_string()))?;
                }
                "--native-seeds" => {
                    let (Some(csv), Some(directory)) = (options.next(), options.next()) else {
                        return Err(RequestError::Invalid("--native-seeds CSV DIR".to_string()));
                    };
                    native.push((PathBuf::from(csv), PathBuf::from(directory)));
                }
                "--from" => {
                    let (Some(state), Some(frame)) = (options.next(), options.next()) else {
                        return Err(RequestError::Invalid("--from STATE FRAME".to_string()));
                    };
                    let frame = frame
                        .parse()
                        .map_err(|_| RequestError::Invalid("--from STATE FRAME".to_string()))?;
                    start = MovieStart::Saved {
                        state: PathBuf::from(state),
                        frame,
                    };
                }
                "--ladder" => {
                    let (Some(every), Some(directory)) = (
                        options.next().and_then(|every| every.parse().ok()),
                        options.next(),
                    ) else {
                        return Err(RequestError::Invalid("--ladder N DIR".to_string()));
                    };
                    if every == 0 {
                        return Err(RequestError::Invalid(
                            "--ladder N DIR: N is at least 1".to_string(),
                        ));
                    }
                    cutting = Some((every, PathBuf::from(directory)));
                }
                "--from-ladder" => {
                    from_ladder =
                        Some(PathBuf::from(options.next().ok_or_else(|| {
                            RequestError::Invalid("--from-ladder DIR".to_string())
                        })?));
                }
                "--no-picture" => picture = false,
                // lsnes only: the picture, the APU port log and the PPU registers
                // with each record (docs/ORACLE.md, MEASUREMENTS (o)).
                "--streams" => {
                    streams = Some(Tiers {
                        picture: true,
                        apu_log: true,
                        ..Tiers::default()
                    });
                }
                // lsnes only: the PPU registers with each record, and no other
                // stream: a seed record the gate decodes needs them, and the rest
                // of the record stays byte for byte as without.
                "--ppu" => {
                    streams.get_or_insert(Tiers {
                        picture: false,
                        apu_log: false,
                        ..Tiers::default()
                    });
                }
                // lsnes only: records without the memories, about twice as fast;
                // for cutting native seeds, not for comparing.
                "--no-memories" => memories = false,
                "--ranges" => ranges = true,
                "--spc" => spc = true,
                "--positions-only" => positions_only = true,
                "--profile" => {
                    let mut number = || {
                        options
                            .next()
                            .and_then(|value| value.parse().ok())
                            .ok_or_else(|| RequestError::Invalid("--profile FROM TO".to_string()))
                    };
                    profile = Some((number()?, number()?));
                }
                "--watch" => {
                    let mut number = || {
                        options
                            .next()
                            .and_then(|value| value.parse().ok())
                            .ok_or_else(|| RequestError::Invalid("--watch FROM EVERY".to_string()))
                    };
                    watch = Some((number()?, number()?));
                }
                "--trace" => {
                    let mut number = || {
                        options
                            .next()
                            .and_then(|text| text.parse().ok())
                            .ok_or_else(|| {
                                RequestError::Invalid("--trace FROM TO PC,...".to_string())
                            })
                    };
                    let (from, to) = (number()?, number()?);
                    let pcs = options
                        .next()
                        .ok_or_else(|| RequestError::Invalid("--trace FROM TO PC,...".to_string()))?
                        .split(',')
                        .map(|pc| {
                            u32::from_str_radix(pc, 16)
                                .map_err(|_| RequestError::Invalid(format!("--trace: {pc}")))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    trace = Some((from, to, pcs));
                }
                "--lag-log" => {
                    lag_log =
                        Some(PathBuf::from(options.next().ok_or_else(|| {
                            RequestError::Invalid("--lag-log CSV".to_string())
                        })?));
                }
                other => return Err(RequestError::Invalid(format!("unknown option {other}"))),
            }
        }
        if (cutting.is_some() || !native.is_empty() || streams.is_some()) && backend != "lsnes" {
            return Err(RequestError::Compatibility);
        }
        if from_ladder.is_some() && native.is_empty() {
            return Err(RequestError::Compatibility);
        }
        Ok(Self {
            rom: rom.into(),
            inputs: inputs.into(),
            frames,
            seeds,
            picture,
            streams,
            ranges,
            lag_log,
            watch,
            trace,
            spc,
            positions_only,
            profile,
            backend: BackendName::from(backend),
            native,
            memories,
            start,
            cutting,
            from_ladder,
        })
    }
}
impl CorpusRequest {
    pub(super) fn parse(arguments: &[String]) -> Result<Self, RequestError> {
        let [rom, lock, data, rest @ ..] = arguments else {
            return Err(RequestError::Invalid("corpus ROM LOCK DATA [--only ID]... [--shard I/N] [--limit N] [--record FILE] [--record-v2 FILE [--sidecar DIR] [--dump-frames ROW:F,...]...] [--oracle lakesnes|lsnes] [--native-seeds DIR] [--watch-words ADDR,...] [--watch-pcs ADDR,...] [--frame-pads DIR | --from-frame-pads DIR] [--no-memories] [--cache DIR [--cache-pictures]]".to_string()));
        };
        let mut only = Vec::new();
        let mut limit = usize::MAX;
        let mut shard = Shard::default();
        let mut record_v2: Option<PathBuf> = None;
        let mut outputs = Vec::new();
        let mut sidecar: Option<PathBuf> = None;
        let mut dump_frames: Vec<(String, u64)> = Vec::new();
        let mut backend = String::new();
        let mut native_seeds: Option<PathBuf> = None;
        let mut watched: Vec<usize> = Vec::new();
        let mut watched_pcs: Vec<u32> = Vec::new();
        let mut pads: Option<FramePads> = None;
        let mut lean = false;
        let mut cache: Option<PathBuf> = None;
        let mut cache_pictures = crate::record::codec::PictureMode::Drop;
        let mut options = rest.iter();
        while let Some(option) = options.next() {
            match option.as_str() {
                "--only" => only.push(
                    options
                        .next()
                        .ok_or_else(|| RequestError::Invalid("--only ID".to_string()))?
                        .clone(),
                ),
                "--shard" => {
                    let text = options
                        .next()
                        .ok_or_else(|| RequestError::Invalid("--shard I/N".to_string()))?;
                    let (index, count) = text
                        .split_once('/')
                        .ok_or_else(|| RequestError::Invalid("--shard I/N".to_string()))?;
                    shard = Shard::new(
                        index.parse().map_err(|_| RequestError::Shard)?,
                        count.parse().map_err(|_| RequestError::Shard)?,
                    )?;
                }
                "--record" => {
                    let path = options
                        .next()
                        .ok_or_else(|| RequestError::Invalid("--record FILE".to_string()))?;
                    outputs.push(CorpusOutput::Record(PathBuf::from(path)));
                }
                "--record-v2" => {
                    let path = options
                        .next()
                        .ok_or_else(|| RequestError::Invalid("--record-v2 FILE".to_string()))?;
                    record_v2 = Some(PathBuf::from(path));
                    outputs.push(CorpusOutput::RecordV2(PathBuf::from(path)));
                }
                "--sidecar" => {
                    // Each row's per-frame stream digests, `DIR/<row>.frames`,
                    // so two recordings that disagree name their first frame.
                    let directory = options
                        .next()
                        .ok_or_else(|| RequestError::Invalid("--sidecar DIR".to_string()))?;
                    sidecar = Some(PathBuf::from(directory));
                    outputs.push(CorpusOutput::Directory(PathBuf::from(directory)));
                }
                "--dump-frames" => {
                    // The pictures of named frames, `DIR/<row>.<frame>.png` in
                    // the sidecar directory, to look at a frame a sidecar named.
                    let usage = "--dump-frames ROW:F,...";
                    let text = options
                        .next()
                        .ok_or_else(|| RequestError::Invalid(usage.to_string()))?;
                    let (row, list) = text
                        .rsplit_once(':')
                        .ok_or_else(|| RequestError::Invalid(usage.to_string()))?;
                    for number in list.split(',') {
                        let number: u64 = number
                            .parse()
                            .map_err(|_| RequestError::Invalid(usage.to_string()))?;
                        dump_frames.push((row.to_owned(), number));
                    }
                }
                "--oracle" => {
                    backend = options
                        .next()
                        .cloned()
                        .ok_or_else(|| RequestError::Invalid("--oracle NAME".to_string()))?;
                }
                "--native-seeds" => {
                    native_seeds =
                        Some(PathBuf::from(options.next().ok_or_else(|| {
                            RequestError::Invalid("--native-seeds DIR".to_string())
                        })?));
                }
                "--watch-words" => {
                    // Work-RAM words (hex) printed whenever their value changes,
                    // to read a row's outcome (game state, energy) off the oracle.
                    let list = options.next().ok_or_else(|| {
                        RequestError::Invalid("--watch-words ADDR,...".to_string())
                    })?;
                    for address in list.split(',') {
                        watched.push(
                            usize::from_str_radix(address.trim_start_matches('$'), 16).map_err(
                                |_| RequestError::Invalid(format!("--watch-words: {address}")),
                            )?,
                        );
                    }
                }
                "--watch-pcs" => {
                    // Instruction addresses (24-bit hex) whose hits each record
                    // carries, printed per frame and kept in a --cache's hits.txt
                    // (LakeSnes watches the first 32).
                    let list = options
                        .next()
                        .ok_or_else(|| RequestError::Invalid("--watch-pcs ADDR,...".to_string()))?;
                    for address in list.split(',') {
                        watched_pcs.push(
                            u32::from_str_radix(
                                address.trim_start_matches('$').replace(':', "").as_str(),
                                16,
                            )
                            .map_err(|_| {
                                RequestError::Invalid(format!("--watch-pcs: {address}"))
                            })?,
                        );
                    }
                }
                "--limit" => {
                    limit = options
                        .next()
                        .and_then(|value| value.parse().ok())
                        .ok_or_else(|| RequestError::Invalid("--limit N".to_string()))?;
                }
                "--frame-pads" => {
                    let directory = options
                        .next()
                        .ok_or_else(|| RequestError::Invalid("--frame-pads DIR".to_string()))?;
                    pads = Some(FramePads::Write(PathBuf::from(directory)));
                    outputs.push(CorpusOutput::Directory(PathBuf::from(directory)));
                }
                "--from-frame-pads" => {
                    let directory = options.next().ok_or_else(|| {
                        RequestError::Invalid("--from-frame-pads DIR".to_string())
                    })?;
                    pads = Some(FramePads::Play(PathBuf::from(directory)));
                }
                "--no-memories" => lean = true,
                "--cache" => {
                    // Every frame's whole record, for lockstep (`oracle::cache`).
                    cache =
                        Some(PathBuf::from(options.next().ok_or_else(|| {
                            RequestError::Invalid("--cache DIR".to_string())
                        })?));
                }
                "--cache-pictures" => cache_pictures = crate::record::codec::PictureMode::Keep,
                other => return Err(RequestError::Invalid(format!("unknown option {other}"))),
            }
        }
        if record_v2.is_some() && (backend != "lsnes" || !matches!(pads, Some(FramePads::Play(_))))
        {
            return Err(RequestError::Invalid(
                "--record-v2 records lsnes rows played --from-frame-pads".to_string(),
            ));
        }
        if (sidecar.is_some() || !dump_frames.is_empty()) && record_v2.is_none() {
            return Err(RequestError::Invalid(
                "--sidecar and --dump-frames go with --record-v2".to_string(),
            ));
        }
        if !dump_frames.is_empty() && sidecar.is_none() {
            return Err(RequestError::Invalid(
                "--dump-frames writes into the --sidecar DIR".to_string(),
            ));
        }
        if cache.is_some() && (lean || !matches!(pads, Some(FramePads::Play(_)))) {
            return Err(RequestError::Invalid(
                "--cache records rows played --from-frame-pads, with their memories".to_string(),
            ));
        }
        Ok(Self {
            rom: rom.into(),
            lock: lock.into(),
            data: data.into(),
            only,
            limit,
            shard,
            outputs,
            sidecar,
            dump_frames,
            backend: BackendName::from(backend),
            native_seeds,
            watched,
            watched_pcs,
            pads,
            lean,
            cache,
            cache_pictures,
        })
    }
}
impl CrossRequest {
    pub(super) fn parse(arguments: &[String]) -> Result<Self, RequestError> {
        let usage = "cross ROM INPUTS [--frames A..B] [--ranges] [--ranges-at A..B] [--streams] [--right-from STATE STEP]";
        let [rom, inputs, rest @ ..] = arguments else {
            return Err(RequestError::Invalid(usage.to_string()));
        };
        let steps = |text: Option<&String>| -> Result<std::ops::RangeInclusive<u64>, RequestError> {
            let (from, to) = text
                .and_then(|text| text.split_once(".."))
                .ok_or_else(|| RequestError::Invalid(usage.to_string()))?;
            let (Ok(from), Ok(to)) = (from.parse::<u64>(), to.parse::<u64>()) else {
                return Err(RequestError::Invalid(usage.to_string()));
            };
            Ok(from..=to)
        };
        let mut range = None;
        let mut first_ranges = false;
        let mut streams = false;
        let mut ranges_at: Option<std::ops::RangeInclusive<u64>> = None;
        let mut right_start = CrossStart::PowerOn;
        let mut options = rest.iter();
        while let Some(option) = options.next() {
            match option.as_str() {
                "--frames" => range = Some(steps(options.next())?),
                "--ranges" => first_ranges = true,
                "--streams" => streams = true,
                "--ranges-at" => ranges_at = Some(steps(options.next())?),
                "--right-from" => {
                    let (Some(state), Some(step)) = (options.next(), options.next()) else {
                        return Err(RequestError::Invalid(usage.to_string()));
                    };
                    let step = step
                        .parse()
                        .map_err(|_| RequestError::Invalid(usage.to_string()))?;
                    right_start = CrossStart::Saved {
                        state: PathBuf::from(state),
                        step,
                    };
                }
                _ => return Err(RequestError::Invalid(usage.to_string())),
            }
        }
        Ok(Self {
            rom: rom.into(),
            inputs: inputs.into(),
            range,
            first_ranges,
            streams,
            ranges_at,
            right_start,
        })
    }
}
