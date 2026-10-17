//! Typed independent timing vectors and compact, versioned interchange.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub pc: u16,
    pub pbr: u8,
    pub s: u16,
    pub p: u8,
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub dbr: u8,
    pub d: u16,
    pub e: u8,
    pub ram: Vec<(u32, u8)>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cycle {
    pub address: u32,
    pub value: Option<u8>,
    pub pins: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Case {
    pub name: String,
    pub initial: State,
    #[serde(rename = "final")]
    pub fin: State,
    pub cycles: Vec<Cycle>,
    pub case_sha256: String,
    #[serde(default)]
    pub opcode: String,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub independent_fixture: bool,
    #[serde(default)]
    pub body: usize,
    #[serde(default)]
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimedResult {
    pub name: String,
    pub case_sha256: String,
    pub status: String,
    pub differences: Vec<String>,
    pub opcode: String,
    pub mx: u8,
    pub decimal: bool,
    pub expected_cycles: usize,
}
struct Writer(Vec<u8>);
impl Writer {
    fn byte(&mut self, x: u8) {
        self.0.push(x)
    }
    fn uint(&mut self, x: u32) {
        self.0.extend(x.to_le_bytes())
    }
    fn bytes(&mut self, x: &[u8]) -> Result<()> {
        self.uint(u32::try_from(x.len())?);
        self.0.extend(x);
        Ok(())
    }
    fn text(&mut self, x: &str) -> Result<()> {
        self.bytes(x.as_bytes())
    }
    fn list<T>(&mut self, x: &[T], f: impl Fn(&mut Self, &T) -> Result<()>) -> Result<()> {
        self.uint(u32::try_from(x.len())?);
        for y in x {
            f(self, y)?
        }
        Ok(())
    }
    fn state(&mut self, s: &State) -> Result<()> {
        for n in [s.pc, s.s, s.a, s.x, s.y, s.d] {
            self.uint(u32::from(n))
        }
        for n in [s.pbr, s.p, s.dbr, s.e] {
            self.byte(n)
        }
        self.list(&s.ram, |w, (a, v)| {
            w.uint(*a);
            w.byte(*v);
            Ok(())
        })
    }
}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        ensure!(n <= self.0.len(), "truncated timing binary");
        let (x, r) = self.0.split_at(n);
        self.0 = r;
        Ok(x)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn uint(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn bytes(&mut self) -> Result<Vec<u8>> {
        let n = self.uint()? as usize;
        ensure!(n <= 64 * 1024 * 1024, "timing field too large");
        Ok(self.take(n)?.to_vec())
    }
    fn text(&mut self) -> Result<String> {
        Ok(String::from_utf8(self.bytes()?)?)
    }
    fn list<T>(&mut self, f: impl Fn(&mut Self) -> Result<T>) -> Result<Vec<T>> {
        let n = self.uint()? as usize;
        ensure!(n <= 1_000_000 && n <= self.0.len(), "timing list too large");
        (0..n).map(|_| f(self)).collect()
    }
    fn state(&mut self) -> Result<State> {
        let mut word = || -> Result<u16> { u16::try_from(self.uint()?).context("timing register exceeds 16 bits") };
        let pc = word()?;
        let s = word()?;
        let a = word()?;
        let x = word()?;
        let y = word()?;
        let d = word()?;
        let pbr = self.byte()?;
        let p = self.byte()?;
        let dbr = self.byte()?;
        let e = self.byte()?;
        let ram = self.list(|r| Ok((r.uint()?, r.byte()?)))?;
        Ok(State { pc, pbr, s, p, a, x, y, dbr, d, e, ram })
    }
}
pub fn encode_cases(cases: &[Case]) -> Result<Vec<u8>> {
    let mut w = Writer(b"SMTC\x01\x00".to_vec());
    w.list(cases, |w, c| {
        w.text(&c.name)?;
        w.state(&c.initial)?;
        w.state(&c.fin)?;
        w.list(&c.cycles, |w, c| {
            w.uint(c.address);
            w.byte(u8::from(c.value.is_some()));
            if let Some(v) = c.value {
                w.byte(v)
            }
            w.text(&c.pins)
        })?;
        w.text(&c.case_sha256)?;
        w.text(&c.opcode)?;
        w.list(&c.conditions, |w, x| w.text(x))?;
        w.byte(u8::from(c.independent_fixture));
        w.uint(u32::try_from(c.body)?);
        w.bytes(&c.bytes)
    })?;
    Ok(w.0)
}
pub fn decode_cases(bytes: &[u8]) -> Result<Vec<Case>> {
    ensure!(bytes.starts_with(b"SMTC\x01\x00"), "unknown timing case format");
    let mut r = Reader(&bytes[6..]);
    let cases = r.list(|r| {
        let name = r.text()?;
        let initial = r.state()?;
        let fin = r.state()?;
        let cycles = r.list(|r| {
            let address = r.uint()?;
            let value = match r.byte()? {
                0 => None,
                1 => Some(r.byte()?),
                _ => anyhow::bail!("invalid optional cycle byte"),
            };
            Ok(Cycle { address, value, pins: r.text()? })
        })?;
        let case_sha256 = r.text()?;
        let opcode = r.text()?;
        let conditions = r.list(|r| r.text())?;
        let independent_fixture = match r.byte()? {
            0 => false,
            1 => true,
            _ => anyhow::bail!("invalid fixture flag"),
        };
        let body = r.uint()? as usize;
        let bytes = r.bytes()?;
        Ok(Case { name, initial, fin, cycles, case_sha256, opcode, conditions, independent_fixture, body, bytes })
    })?;
    ensure!(r.0.is_empty(), "trailing timing binary data");
    Ok(cases)
}
pub fn encode_ledger(rows: &[TimedResult]) -> Result<Vec<u8>> {
    let mut w = Writer(b"SMTL\x01\x00".to_vec());
    w.list(rows, |w, x| {
        w.text(&x.name)?;
        w.text(&x.case_sha256)?;
        w.text(&x.status)?;
        w.list(&x.differences, |w, x| w.text(x))?;
        w.text(&x.opcode)?;
        w.byte(x.mx);
        w.byte(u8::from(x.decimal));
        w.uint(u32::try_from(x.expected_cycles)?);
        Ok(())
    })?;
    Ok(w.0)
}
pub fn decode_ledger(bytes: &[u8]) -> Result<Vec<TimedResult>> {
    ensure!(bytes.starts_with(b"SMTL\x01\x00"), "unknown timing ledger format");
    let mut r = Reader(&bytes[6..]);
    let rows = r.list(|r| {
        let name = r.text()?;
        let case_sha256 = r.text()?;
        let status = r.text()?;
        let differences = r.list(|r| r.text())?;
        let opcode = r.text()?;
        let mx = r.byte()?;
        let decimal = match r.byte()? {
            0 => false,
            1 => true,
            _ => anyhow::bail!("invalid decimal flag"),
        };
        let expected_cycles = r.uint()? as usize;
        Ok(TimedResult { name, case_sha256, status, differences, opcode, mx, decimal, expected_cycles })
    })?;
    ensure!(r.0.is_empty(), "trailing timing ledger data");
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary_roundtrip_preserves_sparse_state_optional_cycles_and_result_identity() {
        let state = State {
            pc: 0xfffe,
            pbr: 0x80,
            s: 0x1fff,
            p: 0x38,
            a: 0xabcd,
            x: 0xffff,
            y: 3,
            dbr: 0x7e,
            d: 0x101,
            e: 0,
            ram: vec![(0x80fffe, 0xad), (0x7e0020, 0xff)],
        };
        let case = Case {
            name: "AD decimal width".into(),
            initial: state.clone(),
            fin: state,
            cycles: vec![
                Cycle { address: 0x80fffe, value: Some(0xad), pins: "r".into() },
                Cycle { address: 0xffffff, value: None, pins: "idle".into() },
            ],
            case_sha256: "a".repeat(64),
            opcode: "ad".into(),
            conditions: vec!["m8".into(), "decimal".into()],
            independent_fixture: true,
            body: 257,
            bytes: vec![0xad, 0x20, 0],
        };
        let bytes = encode_cases(std::slice::from_ref(&case)).unwrap();
        assert_eq!(decode_cases(&bytes).unwrap(), vec![case]);
        for n in 0..bytes.len() {
            assert!(decode_cases(&bytes[..n]).is_err(), "truncation {n}");
        }
        let row = TimedResult {
            name: "AD decimal width".into(),
            case_sha256: "a".repeat(64),
            status: "divergence".into(),
            differences: vec!["complete sparse memory differs".into()],
            opcode: "ad".into(),
            mx: 3,
            decimal: true,
            expected_cycles: 2,
        };
        assert_eq!(decode_ledger(&encode_ledger(std::slice::from_ref(&row)).unwrap()).unwrap(), vec![row]);
    }
    #[test]
    fn framed_formats_reject_truncation_unknown_versions_and_trailing_data() {
        let empty = encode_cases(&[]).unwrap();
        assert_eq!(decode_cases(&empty).unwrap(), vec![]);
        for n in 0..empty.len() {
            assert!(decode_cases(&empty[..n]).is_err())
        }
        let mut extra = empty.clone();
        extra.push(0);
        assert!(decode_cases(&extra).is_err());
        extra[4] = 2;
        assert!(decode_cases(&extra).is_err());
        let rows = encode_ledger(&[]).unwrap();
        assert!(decode_cases(&rows).is_err());
        assert!(decode_ledger(&rows).unwrap().is_empty());
    }
}
