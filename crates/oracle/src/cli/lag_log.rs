//! The lag log `tools/lsnes_lag_log.lua` writes: after a magic and a version, one record a movie
//! frame, at least one, in order from frame 1, of little-endian words. A log runs to hundreds of thousands of
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
    if records.is_empty() {
        return Err("no frames".to_owned());
    }
    records
        .iter()
        .zip(1u32..)
        .map(|(record, frame)| decode(record, frame))
        .collect()
}

fn decode(record: &[u8; RECORD], expected: u32) -> Result<LagFrame, String> {
    let [
        f0,
        f1,
        f2,
        f3,
        c0,
        c1,
        n0,
        n1,
        l0,
        l1,
        g0,
        g1,
        r0,
        r1,
        x0,
        x1,
        y0,
        y1,
    ] = *record;
    if u32::from_le_bytes([f0, f1, f2, f3]) != expected {
        return Err(format!("record {expected}: not movie frame {expected}"));
    }
    let lagged = match u16::from_le_bytes([l0, l1]) {
        0 => false,
        1 => true,
        _ => return Err(format!("record {expected}: lag flag is neither 0 nor 1")),
    };
    Ok(LagFrame {
        frame_counter: u16::from_le_bytes([c0, c1]),
        nmi_counter: u16::from_le_bytes([n0, n1]),
        lagged,
        game_state: u16::from_le_bytes([g0, g1]),
        room: u16::from_le_bytes([r0, r1]),
        x: u16::from_le_bytes([x0, x1]),
        y: u16::from_le_bytes([y0, y1]),
    })
}
