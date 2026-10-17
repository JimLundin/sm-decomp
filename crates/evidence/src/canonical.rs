use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

pub type Id = [u8; 32];
pub const VERSION: u8 = 1;
pub const MAX_OBJECT: usize = 2 << 20;

pub fn identity(kind: u8, payload: &[u8]) -> Id {
    let mut hash = Sha256::new();
    hash.update(b"SM-EVIDENCE\0");
    hash.update([VERSION, kind]);
    hash.update(payload);
    hash.finalize().into()
}

pub fn digest(payload: &[u8]) -> Id {
    Sha256::digest(payload).into()
}

pub fn hex(id: &Id) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn parse_id(text: &str) -> Result<Id> {
    ensure!(text.len() == 64 && text.is_ascii(), "expected 64 ASCII hex digits");
    let mut id = [0; 32];
    for (i, byte) in id.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * i..2 * i + 2], 16)?;
    }
    Ok(id)
}

pub fn varint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        out.push(value as u8 | 128);
        value >>= 7;
    }
    out.push(value as u8);
}

pub struct Reader<'a> {
    pub bytes: &'a [u8],
    pub at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }
    pub fn done(&self) -> Result<()> {
        ensure!(self.at == self.bytes.len(), "trailing object bytes");
        Ok(())
    }
    pub fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub fn varint(&mut self) -> Result<u64> {
        let mut value = 0;
        for shift in (0..70).step_by(7) {
            let byte = self.byte()?;
            ensure!(shift != 63 || byte <= 1, "varint overflow");
            value |= u64::from(byte & 127) << shift;
            if byte < 128 {
                ensure!(shift == 0 || byte != 0, "noncanonical varint");
                return Ok(value);
            }
        }
        anyhow::bail!("varint overflow")
    }
    pub fn len(&mut self, limit: usize) -> Result<usize> {
        let len = usize::try_from(self.varint()?).context("length overflow")?;
        ensure!(len <= limit, "declared length exceeds limit");
        Ok(len)
    }
    pub fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(len).context("length overflow")?;
        let bytes = self.bytes.get(self.at..end).context("truncated object")?;
        self.at = end;
        Ok(bytes)
    }
    pub fn id(&mut self) -> Result<Id> {
        Ok(self.take(32)?.try_into()?)
    }
}
