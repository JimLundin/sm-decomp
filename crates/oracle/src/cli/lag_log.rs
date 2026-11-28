//! The lag log `tools/lsnes_lag_log.lua` writes: after a magic and a version, one record a movie
//! frame, in order from frame 1, of little-endian words. A log runs to hundreds of thousands of
//! frames, too many for a TOML table a frame. A diagnostic names a record, never its contents.

const MAGIC: &[u8; 8] = b"SMLAGLOG";
const VERSION: u16 = 1;
const RECORD: usize = 18;

/// What lsnes observed after one movie frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LagFrame {
    /// The game's frame counter `$05B6`.
    pub frame_counter: u16,
    /// The game's NMI counter `$05B8`.
    pub nmi_counter: u16,
    /// Whether the emulator flagged the frame as lag.
    pub lagged: bool,
    /// The game state `$0998`.
    pub game_state: u16,
    /// The room `$079B`.
    pub room: u16,
    /// Samus's X `$0AF6`.
    pub x: u16,
    /// Samus's Y `$0AFA`.
    pub y: u16,
}

pub(super) fn parse(bytes: &[u8]) -> Result<Vec<LagFrame>, String> {
    let body = bytes
        .strip_prefix(MAGIC.as_slice())
        .ok_or("not an lsnes lag log")?;
    let (version, body) = body.split_first_chunk::<2>().ok_or("no version")?;
    if u16::from_le_bytes(*version) != VERSION {
        return Err(format!(
            "lag log version {}, this oracle reads {VERSION}",
            u16::from_le_bytes(*version)
        ));
    }
    let (records, rest) = body.as_chunks::<RECORD>();
    if !rest.is_empty() {
        return Err(format!("{} bytes after the last whole record", rest.len()));
    }
    records
        .iter()
        .zip(1u32..)
        .map(|(record, frame)| decode(record, frame))
        .collect()
}

fn decode(record: &[u8; RECORD], expected: u32) -> Result<LagFrame, String> {
    let (frame, words) = record.split_first_chunk::<4>().ok_or("short record")?;
    if u32::from_le_bytes(*frame) != expected {
        return Err(format!("record {expected}: not movie frame {expected}"));
    }
    let word = |index: usize| {
        words
            .as_chunks::<2>()
            .0
            .get(index)
            .map_or(0, |pair| u16::from_le_bytes(*pair))
    };
    let lagged = match word(2) {
        0 => false,
        1 => true,
        _ => return Err(format!("record {expected}: lag flag is neither 0 nor 1")),
    };
    Ok(LagFrame {
        frame_counter: word(0),
        nmi_counter: word(1),
        lagged,
        game_state: word(3),
        room: word(4),
        x: word(5),
        y: word(6),
    })
}
