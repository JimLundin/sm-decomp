//! Prepared content and a sealed start: what a Session opens, built by tools.
//!
//! The content's body is the validated image the engine reads by its stable
//! addresses; a start names the content it was prepared from. Neither names a
//! path or a device, and opening runs no normalization: a start is complete.
#![forbid(unsafe_code)]
use crate::digest::Digest;

const CONTENT_MAGIC: &[u8; 8] = b"SMPACK\x00\x01";
const START_MAGIC: &[u8; 8] = b"SMSTRT\x00\x01";

/// A pack or start that does not open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreparedError {
    NotPrepared,
    Damaged,
    OtherContent,
}

impl core::fmt::Display for PreparedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NotPrepared => "not a prepared file",
            Self::Damaged => "the file is damaged",
            Self::OtherContent => "the start was prepared from other content",
        })
    }
}

fn sealed(magic: &[u8; 8], id: u64, body: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(body.len() + 32);
    bytes.extend_from_slice(magic);
    bytes.extend_from_slice(&id.to_le_bytes());
    bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&Digest::of(body).to_le_bytes());
    bytes.extend_from_slice(body);
    bytes
}

fn opened<'a>(magic: &[u8; 8], bytes: &'a [u8]) -> Result<(u64, &'a [u8]), PreparedError> {
    if bytes.get(..8) != Some(magic) || bytes.len() < 32 {
        return Err(PreparedError::NotPrepared);
    }
    let word = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("eight bytes"));
    let body = &bytes[32..];
    if word(16) != body.len() as u64 || word(24) != Digest::of(body) {
        return Err(PreparedError::Damaged);
    }
    Ok((word(8), body))
}

/// The prepared content a Session plays from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedContent {
    id: u64,
    image: Vec<u8>,
}

impl PreparedContent {
    /// Prepare content from its image: a tool's step, not the game's.
    pub fn prepare(image: Vec<u8>) -> Self {
        Self {
            id: Digest::of(&image),
            image,
        }
    }
    pub fn encode(&self) -> Vec<u8> {
        sealed(CONTENT_MAGIC, self.id, &self.image)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, PreparedError> {
        let (id, body) = opened(CONTENT_MAGIC, bytes)?;
        if id != Digest::of(body) {
            return Err(PreparedError::Damaged);
        }
        Ok(Self {
            id,
            image: body.to_vec(),
        })
    }
    /// The content's identity, which its starts and saves name.
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn image(&self) -> &[u8] {
        &self.image
    }
}

/// A complete, validated start, sealed to its content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedStart {
    content: u64,
    state: Vec<u8>,
}

impl PreparedStart {
    /// Seal a complete start state to `content`: a tool's step.
    pub fn seal(content: &PreparedContent, state: Vec<u8>) -> Self {
        Self {
            content: content.id,
            state,
        }
    }
    pub fn encode(&self) -> Vec<u8> {
        sealed(START_MAGIC, self.content, &self.state)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, PreparedError> {
        let (content, body) = opened(START_MAGIC, bytes)?;
        Ok(Self {
            content,
            state: body.to_vec(),
        })
    }
    /// The state, once `content` proves to be the start's own.
    pub fn state_for(&self, content: &PreparedContent) -> Result<&[u8], PreparedError> {
        if self.content != content.id {
            return Err(PreparedError::OtherContent);
        }
        Ok(&self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_and_start_round_trip_and_seal() {
        let content = PreparedContent::prepare(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let opened = PreparedContent::decode(&content.encode()).unwrap();
        assert_eq!(opened, content);
        let start = PreparedStart::seal(&content, vec![9; 40]);
        let start = PreparedStart::decode(&start.encode()).unwrap();
        assert_eq!(start.state_for(&content).unwrap(), &[9; 40]);
        let other = PreparedContent::prepare(vec![0; 9]);
        assert_eq!(start.state_for(&other), Err(PreparedError::OtherContent));
    }

    #[test]
    fn damage_and_foreign_files_are_rejected() {
        let content = PreparedContent::prepare(vec![1; 64]);
        let mut bytes = content.encode();
        *bytes.last_mut().unwrap() ^= 1;
        assert_eq!(PreparedContent::decode(&bytes), Err(PreparedError::Damaged));
        assert_eq!(
            PreparedContent::decode(b"not a pack"),
            Err(PreparedError::NotPrepared)
        );
    }
}
