//! Complete command requests are validated without acquiring inputs or outputs.
use crate::Streams;
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
            Self::Compatibility => f.write_str("incompatible options"),
            Self::Shard => f.write_str("--shard I/N requires N > 0 and I < N"),
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
pub(super) struct MovieRequest {
    pub rom: PathBuf,
    pub inputs: PathBuf,
    pub frames: Option<usize>,
    pub seeds: Option<(PathBuf, PathBuf)>,
    pub streams: Option<Streams>,
    pub ranges: bool,
    pub lag_log: Option<PathBuf>,
    pub watch: Option<(usize, usize)>,
    pub trace: Option<(usize, usize, Vec<u32>)>,
    pub positions_only: bool,
    pub native: Vec<(PathBuf, PathBuf)>,
    pub memories: bool,
    pub start: MovieStart,
    pub cutting: Option<(u64, PathBuf)>,
    pub from_ladder: Option<PathBuf>,
}
pub(super) struct CorpusRequest {
    pub rom: PathBuf,
    pub lock: PathBuf,
    pub bundle: PathBuf,
    pub only: Vec<String>,
    pub limit: usize,
    pub shard: Shard,
    pub watched: Vec<usize>,
    pub watched_pcs: Vec<u32>,
    pub lean: bool,
    pub cache: Option<PathBuf>,
}
impl MovieRequest {
    pub(super) fn parse(arguments: &[String]) -> Result<Self, RequestError> {
        let [rom, inputs, rest @ ..] = arguments else {
            return Err(RequestError::Invalid("movie ROM INPUTS [--frames N] [--seeds TABLE DIR] [--ranges] [--lag-log LOG] [--watch FROM EVERY] [--trace FROM TO PC,...] [--positions-only] [--streams] [--ppu] [--native-seeds TABLE DIR]... [--no-memories] [--from STATE FRAME] [--ladder N DIR] [--from-ladder DIR]".to_string()));
        };
        let mut frames = None;
        let mut seeds: Option<(PathBuf, PathBuf)> = None;
        let mut streams: Option<Streams> = None;
        let mut ranges = false;
        let mut lag_log: Option<PathBuf> = None;
        let mut watch: Option<(usize, usize)> = None;
        let mut trace: Option<(usize, usize, Vec<u32>)> = None;
        let mut positions_only = false;
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
                    let (Some(table), Some(directory)) = (options.next(), options.next()) else {
                        return Err(RequestError::Invalid("--seeds TABLE DIR".to_string()));
                    };
                    seeds = Some((PathBuf::from(table), PathBuf::from(directory)));
                }
                "--native-seeds" => {
                    let (Some(table), Some(directory)) = (options.next(), options.next()) else {
                        return Err(RequestError::Invalid(
                            "--native-seeds TABLE DIR".to_string(),
                        ));
                    };
                    native.push((PathBuf::from(table), PathBuf::from(directory)));
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
                // The APU port log and the PPU registers with each record
                // (docs/ORACLE.md, MEASUREMENTS (o)).
                "--streams" => streams = Some(Streams::PpuAndApuLog),
                // The PPU registers with each record, and no other
                // stream: a seed record the gate decodes needs them, and the rest
                // of the record stays byte for byte as without.
                "--ppu" => {
                    streams.get_or_insert(Streams::Ppu);
                }
                // Records without the memories, about twice as fast;
                // for cutting native seeds, not for comparing.
                "--no-memories" => memories = false,
                "--ranges" => ranges = true,
                "--positions-only" => positions_only = true,
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
                            RequestError::Invalid("--lag-log LOG".to_string())
                        })?));
                }
                other => return Err(RequestError::Invalid(format!("unknown option {other}"))),
            }
        }
        if from_ladder.is_some() && native.is_empty() {
            return Err(RequestError::Compatibility);
        }
        Ok(Self {
            rom: rom.into(),
            inputs: inputs.into(),
            frames,
            seeds,
            streams,
            ranges,
            lag_log,
            watch,
            trace,
            positions_only,
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
        let [rom, lock, bundle, rest @ ..] = arguments else {
            return Err(RequestError::Invalid("corpus ROM LOCK BUNDLE [--only ID]... [--shard I/N] [--limit N] [--watch-words ADDR,...] [--watch-pcs ADDR,...] [--no-memories] [--cache DIR]".to_string()));
        };
        let mut only = Vec::new();
        let mut limit = usize::MAX;
        let mut shard = Shard::default();
        let mut watched: Vec<usize> = Vec::new();
        let mut watched_pcs: Vec<u32> = Vec::new();
        let mut lean = false;
        let mut cache: Option<PathBuf> = None;
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
                "--watch-words" => {
                    // Work-RAM words (hex) printed whenever their value changes,
                    // to read a row's outcome (game state, energy) off the oracle.
                    let list = options.next().ok_or_else(|| {
                        RequestError::Invalid("--watch-words ADDR,...".to_string())
                    })?;
                    for address in list.split(',') {
                        let offset = usize::from_str_radix(address.trim_start_matches('$'), 16)
                            .ok()
                            .filter(|&offset| offset < 0x1_ffff)
                            .ok_or_else(|| {
                                RequestError::Invalid(format!("--watch-words: {address}"))
                            })?;
                        watched.push(offset);
                    }
                }
                "--watch-pcs" => {
                    // Instruction addresses (24-bit hex) whose hits each record
                    // carries, printed per frame and kept in a --cache's hits.txt.
                    let list = options
                        .next()
                        .ok_or_else(|| RequestError::Invalid("--watch-pcs ADDR,...".to_string()))?;
                    for address in list.split(',') {
                        watched_pcs.push(
                            u32::from_str_radix(
                                address.trim_start_matches('$').replace(':', "").as_str(),
                                16,
                            )
                            .ok()
                            .filter(|&pc| pc <= 0xff_ffff)
                            .ok_or_else(|| {
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
                "--no-memories" => lean = true,
                "--cache" => {
                    // Every frame's whole record, for lockstep (`oracle::cache`).
                    cache =
                        Some(PathBuf::from(options.next().ok_or_else(|| {
                            RequestError::Invalid("--cache DIR".to_string())
                        })?));
                }
                other => return Err(RequestError::Invalid(format!("unknown option {other}"))),
            }
        }
        if lean && (cache.is_some() || !watched.is_empty()) {
            return Err(RequestError::Compatibility);
        }
        Ok(Self {
            rom: rom.into(),
            lock: lock.into(),
            bundle: bundle.into(),
            only,
            limit,
            shard,
            watched,
            watched_pcs,
            lean,
            cache,
        })
    }
}
