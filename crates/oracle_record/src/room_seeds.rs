//! A movie's room seeds (`movies/room-seeds-<capture>.toml`): the room captures its seeds come
//! from, each the room the game entered and the movie frame the capture followed.

use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

/// A room-seed table.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomSeeds {
    /// The seeds, each index once.
    pub seed: Vec<RoomSeed>,
}

/// One room capture. The importer's tables also carry the capture's provenance.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomSeed {
    /// The capture's index within its table, counted from 1.
    pub room_index: u32,
    /// The room the game entered (`$7E:079B`).
    pub room: Room,
    /// The movie frame (1-based) the capture followed.
    pub movie_frame: u64,
    /// The next movie input, zero-based.
    pub next_input_index: Option<u64>,
    /// The movie subframe the capture's room began on.
    pub movie_subframe_index: Option<u64>,
    /// The SHA-256 of the captured seed.
    pub seed_sha256: Option<Sha256>,
    /// The SHA-256 of the capture's observation.
    pub observation_sha256: Option<Sha256>,
    /// The game's frame counter (`$7E:05B6`) at the capture.
    pub nmi: Option<u16>,
}

/// A room id, written as four uppercase hexadecimal digits.
pub type Room = Hex<4>;

/// A number written as exactly `DIGITS` uppercase hexadecimal digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct Hex<const DIGITS: usize>(pub u32);

impl<const DIGITS: usize> TryFrom<String> for Hex<DIGITS> {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let digits = text.len() == DIGITS
            && text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte));
        u32::from_str_radix(&text, 16)
            .ok()
            .filter(|_| digits)
            .map(Self)
            .ok_or_else(|| format!("{text:?} is not {DIGITS} uppercase hexadecimal digits"))
    }
}

/// A SHA-256, written as 64 lowercase hexadecimal digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct Sha256(pub [u8; 32]);

impl TryFrom<String> for Sha256 {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let refuse = || format!("{text:?} is not 64 lowercase hexadecimal digits");
        let digits = text
            .bytes()
            .map(|byte| match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                _ => None,
            })
            .collect::<Option<Vec<u8>>>()
            .filter(|digits| digits.len() == 64)
            .ok_or_else(refuse)?;
        let mut bytes = [0; 32];
        for (byte, [high, low]) in bytes.iter_mut().zip(digits.as_chunks::<2>().0) {
            *byte = high << 4 | low;
        }
        Ok(Self(bytes))
    }
}

impl RoomSeeds {
    /// Parse a room-seed table.
    ///
    /// # Errors
    /// Text that is no table, or an index given twice.
    pub fn parse(text: &str) -> Result<Self, String> {
        let table: Self = toml::from_str(text).map_err(|error| error.to_string())?;
        let mut indices = BTreeSet::new();
        if let Some(seed) = table
            .seed
            .iter()
            .find(|seed| !indices.insert(seed.room_index))
        {
            return Err(format!("seed {} given twice", seed.room_index));
        }
        Ok(table)
    }

    /// Read the room-seed table at `path`.
    ///
    /// # Errors
    /// An unreadable file, or the errors of [`RoomSeeds::parse`], naming the path.
    pub fn read(path: &Path) -> Result<Self, String> {
        std::fs::read_to_string(path)
            .map_err(|error| error.to_string())
            .and_then(|text| Self::parse(&text))
            .map_err(|error| format!("{}: {error}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: &str = "[[seed]]\nroom_index = 7\nroom = \"DF45\"\nmovie_frame = 40\n";

    #[test]
    fn a_seed_reads_its_room_as_hexadecimal() {
        let table = RoomSeeds::parse(SEED).unwrap();
        assert_eq!(table.seed[0].room_index, 7);
        assert_eq!(table.seed[0].room, Hex(0xDF45));
        assert_eq!(table.seed[0].movie_frame, 40);
    }

    #[test]
    fn an_unknown_field_is_refused() {
        let error = RoomSeeds::parse(&format!("{SEED}region = \"wram\"\n")).unwrap_err();
        assert!(error.contains("region"), "{error}");
    }

    #[test]
    fn an_index_given_twice_is_refused() {
        let error = RoomSeeds::parse(&format!("{SEED}\n{SEED}")).unwrap_err();
        assert_eq!(error, "seed 7 given twice");
    }

    #[test]
    fn a_seed_reads_its_hashes() {
        let digest = "0123456789abcdef".repeat(4);
        let table = RoomSeeds::parse(&format!("{SEED}seed_sha256 = \"{digest}\"\n")).unwrap();
        let mut bytes = [0; 32];
        for (byte, value) in bytes.iter_mut().zip(
            [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]
                .iter()
                .cycle(),
        ) {
            *byte = *value;
        }
        assert_eq!(table.seed[0].seed_sha256, Some(Sha256(bytes)));
    }

    #[test]
    fn a_hash_of_other_than_64_lowercase_digits_is_refused() {
        let digest = "0123456789abcdef".repeat(4);
        let upper = digest.to_uppercase();
        for hash in [
            &digest[1..],
            &format!("{digest}0"),
            &upper,
            &digest.replace('f', "g"),
            "",
        ] {
            for field in ["seed_sha256", "observation_sha256"] {
                let error = RoomSeeds::parse(&format!("{SEED}{field} = \"{hash}\"\n")).unwrap_err();
                assert!(error.contains("64 lowercase"), "{field} {hash}: {error}");
            }
        }
    }

    #[test]
    fn every_committed_table_parses() {
        let movies = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../movies");
        let mut tables = 0;
        for entry in std::fs::read_dir(&movies).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if name.starts_with("room-seeds-") && name.ends_with(".toml") {
                assert!(!RoomSeeds::read(&path).unwrap().seed.is_empty(), "{name}");
                tables += 1;
            }
        }
        assert_eq!(tables, 10);
    }

    #[test]
    fn a_room_of_other_than_four_uppercase_digits_is_refused() {
        for room in ["df45", "F45", "0DF45", "DG45"] {
            assert!(
                RoomSeeds::parse(&SEED.replace("DF45", room)).is_err(),
                "{room}"
            );
        }
    }
}
