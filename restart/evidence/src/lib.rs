//! Bounded machine evidence values, encoded independently of Rust memory layout.
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

pub type Map = BTreeMap<String, Value>;
const MAGIC: &[u8; 8] = b"SMEVID01";
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_ITEMS: usize = 65_536;
const MAX_DEPTH: usize = 32;
// Bound decoded Value/container overhead independently of the wire byte limit.
const MAX_NODES: usize = 262_144;

struct Budget(usize);
impl Budget {
    fn consume(&mut self) -> Result<(), String> {
        if self.0 == 0 {
            return Err("evidence aggregate node budget exceeded".into());
        }
        self.0 -= 1;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<Value>),
    Table(Map),
}

impl Value {
    pub fn as_integer(&self) -> Option<i64> {
        if let Self::Integer(v) = self { Some(*v) } else { None }
    }
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Boolean(v) = self { Some(*v) } else { None }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(v) = self { Some(v) } else { None }
    }
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Self::Array(v) = self { Some(v) } else { None }
    }
    pub fn as_table(&self) -> Option<&Map> {
        if let Self::Table(v) = self { Some(v) } else { None }
    }
    pub fn as_table_mut(&mut self) -> Option<&mut Map> {
        if let Self::Table(v) = self { Some(v) } else { None }
    }
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        if let Self::Array(v) = self { Some(v) } else { None }
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_table()?.get(key)
    }
}
impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}
impl std::ops::Index<&str> for Value {
    type Output = Value;
    fn index(&self, key: &str) -> &Value {
        &self.as_table().expect("table index")[key]
    }
}
impl std::ops::IndexMut<&str> for Value {
    fn index_mut(&mut self, key: &str) -> &mut Value {
        self.as_table_mut().expect("table index").get_mut(key).expect("table key")
    }
}
impl std::ops::Index<usize> for Value {
    type Output = Value;
    fn index(&self, key: usize) -> &Value {
        &self.as_array().expect("array index")[key]
    }
}
impl std::ops::IndexMut<usize> for Value {
    fn index_mut(&mut self, key: usize) -> &mut Value {
        &mut self.as_array_mut().expect("array index")[key]
    }
}
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.into())
    }
}
impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Self::Array(v)
    }
}
impl From<Vec<String>> for Value {
    fn from(v: Vec<String>) -> Self {
        Self::Array(v.into_iter().map(Self::String).collect())
    }
}
impl From<Map> for Value {
    fn from(v: Map) -> Self {
        Self::Table(v)
    }
}

/// Explicit typed report conversion; integers exceeding the signed wire range refuse.
pub trait ToEvidence {
    fn to_evidence(&self) -> Result<Value, String>;
}
impl ToEvidence for u64 {
    fn to_evidence(&self) -> Result<Value, String> {
        i64::try_from(*self).map(Value::Integer).map_err(|_| "evidence integer exceeds i64".into())
    }
}
impl<T: ToEvidence> ToEvidence for [T] {
    fn to_evidence(&self) -> Result<Value, String> {
        self.iter().map(ToEvidence::to_evidence).collect::<Result<Vec<_>, _>>().map(Value::Array)
    }
}
impl<T: ToEvidence, const N: usize> ToEvidence for [T; N] {
    fn to_evidence(&self) -> Result<Value, String> {
        self.as_slice().to_evidence()
    }
}

pub fn encode(value: &Value) -> Result<Vec<u8>, String> {
    let mut out = MAGIC.to_vec();
    write_value(value, &mut out, 0, &mut Budget(MAX_NODES))?;
    Ok(out)
}

fn append(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), String> {
    if out.len().checked_add(bytes.len()).is_none_or(|n| n > MAX_BYTES) {
        return Err("evidence exceeds 16 MiB".into());
    }
    out.extend_from_slice(bytes);
    Ok(())
}
fn length(out: &mut Vec<u8>, n: usize, limit: usize) -> Result<(), String> {
    if n > limit {
        return Err("evidence extent exceeded".into());
    }
    append(out, &(n as u32).to_le_bytes())
}
fn string(out: &mut Vec<u8>, text: &str) -> Result<(), String> {
    length(out, text.len(), MAX_BYTES)?;
    append(out, text.as_bytes())
}
fn write_value(value: &Value, out: &mut Vec<u8>, depth: usize, budget: &mut Budget) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err("evidence nesting exceeded".into());
    }
    budget.consume()?;
    match value {
        Value::Integer(n) => {
            append(out, &[0])?;
            append(out, &n.to_le_bytes())
        }
        Value::Float(n) => {
            if !n.is_finite() {
                return Err("nonfinite evidence float".into());
            }
            append(out, &[1])?;
            append(out, &n.to_bits().to_le_bytes())
        }
        Value::Boolean(v) => append(out, &[2, u8::from(*v)]),
        Value::String(v) => {
            append(out, &[3])?;
            string(out, v)
        }
        Value::Array(values) => {
            append(out, &[4])?;
            length(out, values.len(), MAX_ITEMS)?;
            for v in values {
                write_value(v, out, depth + 1, budget)?;
            }
            Ok(())
        }
        Value::Table(values) => {
            append(out, &[5])?;
            length(out, values.len(), MAX_ITEMS)?;
            for (key, v) in values {
                string(out, key)?;
                write_value(v, out, depth + 1, budget)?;
            }
            Ok(())
        }
    }
}

pub fn decode(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() > MAX_BYTES || !bytes.starts_with(MAGIC) {
        return Err("unsupported evidence size/magic/version".into());
    }
    let mut input = Input { bytes: &bytes[MAGIC.len()..], budget: Budget(MAX_NODES) };
    let value = input.value(0)?;
    if !input.bytes.is_empty() {
        return Err("trailing evidence bytes".into());
    }
    Ok(value)
}

/// Refuse oversized files before allocating their payload, and recheck after reading.
pub fn read(path: impl AsRef<Path>) -> Result<Value, String> {
    decode(&read_bytes(path)?)
}

/// Bounded immutable bytes for callers that authenticate before decoding.
pub fn read_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path.as_ref()).map_err(|e| format!("{}: {e}", path.as_ref().display()))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_BYTES as u64 {
        return Err("evidence exceeds 16 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("evidence exceeds 16 MiB".into());
    }
    Ok(bytes)
}
struct Input<'a> {
    bytes: &'a [u8],
    budget: Budget,
}
impl<'a> Input<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        if count > self.bytes.len() {
            return Err("truncated evidence".into());
        }
        let (head, tail) = self.bytes.split_at(count);
        self.bytes = tail;
        Ok(head)
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn count(&mut self, limit: usize) -> Result<usize, String> {
        let n = u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize;
        if n > limit {
            return Err("evidence extent exceeded".into());
        }
        Ok(n)
    }
    fn string(&mut self) -> Result<String, String> {
        let n = self.count(MAX_BYTES)?;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|_| "invalid evidence UTF-8".into())
    }
    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > MAX_DEPTH {
            return Err("evidence nesting exceeded".into());
        }
        self.budget.consume()?;
        Ok(match self.byte()? {
            0 => Value::Integer(i64::from_le_bytes(self.take(8)?.try_into().unwrap())),
            1 => {
                let n = f64::from_bits(u64::from_le_bytes(self.take(8)?.try_into().unwrap()));
                if !n.is_finite() {
                    return Err("nonfinite evidence float".into());
                }
                Value::Float(n)
            }
            2 => Value::Boolean(match self.byte()? {
                0 => false,
                1 => true,
                _ => return Err("invalid evidence boolean".into()),
            }),
            3 => Value::String(self.string()?),
            4 => {
                let n = self.count(MAX_ITEMS)?;
                let mut values = Vec::new();
                for _ in 0..n {
                    values.push(self.value(depth + 1)?);
                }
                Value::Array(values)
            }
            5 => {
                let n = self.count(MAX_ITEMS)?;
                let mut values = Map::new();
                for _ in 0..n {
                    let key = self.string()?;
                    if values.last_key_value().is_some_and(|(p, _)| p >= &key) {
                        return Err("noncanonical evidence table keys".into());
                    }
                    values.insert(key, self.value(depth + 1)?);
                }
                Value::Table(values)
            }
            _ => return Err("unknown evidence value tag".into()),
        })
    }
}

#[cfg(any(test, feature = "private-checks"))]
mod tests {
    use super::*;

    struct OracleFiles(std::path::PathBuf);

    impl OracleFiles {
        fn cleanup(self) -> std::io::Result<()> {
            std::fs::remove_dir_all(&self.0)
                .map_err(|error| std::io::Error::new(error.kind(), format!("remove {}: {error}", self.0.display())))
        }
    }

    impl Drop for OracleFiles {
        fn drop(&mut self) {
            for name in ["valid.sme", "oversized.sme", "trailing.sme"] {
                let _ = std::fs::remove_file(self.0.join(name));
            }
            let _ = std::fs::remove_dir(&self.0);
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn bounded_file_reader_handles_valid_sparse_and_malformed_files() {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let files = OracleFiles(std::env::temp_dir().join(format!(
            "sm-codec-oracle-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )));
        std::fs::create_dir(&files.0).unwrap();
        let valid = b"SMEVID01\x02\x01";
        std::fs::write(files.0.join("valid.sme"), valid).unwrap();
        assert_eq!(read(files.0.join("valid.sme")).unwrap(), Value::Boolean(true));
        let sparse = files.0.join("oversized.sme");
        std::fs::File::create(&sparse).unwrap().set_len(16 * 1024 * 1024 + 1).unwrap();
        // Exact error isolates the metadata size guard from magic/version rejection.
        assert_eq!(read(&sparse).unwrap_err(), "evidence exceeds 16 MiB");
        let mut trailing = valid.to_vec();
        trailing.push(0);
        std::fs::write(files.0.join("trailing.sme"), trailing).unwrap();
        assert_eq!(read(files.0.join("trailing.sme")).unwrap_err(), "trailing evidence bytes");
        assert!(read(files.0.join("absent.sme")).unwrap_err().contains("absent.sme"));
        // File-growth allocation remains a source invariant of the limit+1 reader;
        // this file test does not claim to schedule a race between metadata and read.
        files.cleanup().expect("remove completed codec oracle files");
    }

    #[cfg_attr(test, test)]
    pub(super) fn unsigned_counter_conversion_refuses_signed_range_overflow() {
        assert_eq!(0_u64.to_evidence().unwrap(), Value::Integer(0));
        assert_eq!((i64::MAX as u64).to_evidence().unwrap(), Value::Integer(i64::MAX));
        assert!((i64::MAX as u64 + 1).to_evidence().is_err());
        assert!(u64::MAX.to_evidence().is_err());
        let distinct = [1_u64, 258, i64::MAX as u64];
        assert_eq!(
            distinct.to_evidence().unwrap(),
            Value::Array(vec![Value::Integer(1), Value::Integer(258), Value::Integer(i64::MAX),])
        );
        assert!([1_u64, u64::MAX, 258].to_evidence().is_err());
    }

    fn frame(payload: &[u8]) -> Vec<u8> {
        let mut bytes = b"SMEVID01".to_vec();
        bytes.extend_from_slice(payload);
        bytes
    }

    #[cfg_attr(test, test)]
    pub(super) fn literal_scalar_wire_layouts_are_independent() {
        let fixtures = [
            (Value::Integer(258), &[0, 2, 1, 0, 0, 0, 0, 0, 0][..]),
            (Value::Integer(-2), &[0, 254, 255, 255, 255, 255, 255, 255, 255]),
            (Value::Float(1.5), &[1, 0, 0, 0, 0, 0, 0, 248, 63]),
            (Value::Boolean(false), &[2, 0]),
            (Value::Boolean(true), &[2, 1]),
            (Value::String("\u{e5}".into()), &[3, 2, 0, 0, 0, 195, 165]),
        ];
        for (value, payload) in fixtures {
            let literal = frame(payload);
            assert_eq!(encode(&value).unwrap(), literal);
            assert_eq!(decode(&literal).unwrap(), value);
        }
        let negative_zero = frame(&[1, 0, 0, 0, 0, 0, 0, 0, 128]);
        assert_eq!(encode(&Value::Float(-0.0)).unwrap(), negative_zero);
        let Value::Float(decoded) = decode(&negative_zero).unwrap() else { panic!("float tag") };
        assert_eq!(decoded.to_bits(), 0x8000_0000_0000_0000);
    }

    #[cfg_attr(test, test)]
    pub(super) fn literal_array_and_nested_sorted_table_layouts() {
        let literal = frame(&[4, 2, 0, 0, 0, 2, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0]);
        let value = Value::Array(vec![Value::Boolean(false), Value::Integer(258)]);
        assert_eq!(encode(&value).unwrap(), literal);
        assert_eq!(decode(&literal).unwrap(), value);
        let nested = frame(&[
            5, 2, 0, 0, 0, // Two entries, in canonical order.
            1, 0, 0, 0, b'a', 4, 1, 0, 0, 0, 2, 1, // a = [true].
            1, 0, 0, 0, b'b', 5, 0, 0, 0, 0, // b = {}.
        ]);
        let value = Value::Table(Map::from([
            ("b".into(), Value::Table(Map::new())),
            ("a".into(), Value::Array(vec![Value::Boolean(true)])),
        ]));
        assert_eq!(encode(&value).unwrap(), nested);
        assert_eq!(decode(&nested).unwrap(), value);
    }

    #[cfg_attr(test, test)]
    pub(super) fn string_lengths_count_utf8_bytes_in_little_endian() {
        let mut literal = frame(&[3, 2, 1, 0, 0]);
        literal.extend_from_slice(&[b'x'; 258]);
        assert_eq!(encode(&Value::String("x".repeat(258))).unwrap(), literal);
        assert_eq!(decode(&literal).unwrap().as_str(), Some("x".repeat(258).as_str()));
    }

    #[cfg_attr(test, test)]
    pub(super) fn malformed_literal_tags_strings_booleans_and_versions_refuse() {
        for payload in [
            &[9][..],
            &[2, 2],
            &[3, 1, 0, 0, 0, 255],
            &[3, 255, 255, 255, 255],
            &[4, 1, 0],
            &[5, 1, 0, 0, 0, 1, 0, 0, 0, b'a'],
        ] {
            assert!(decode(&frame(payload)).is_err(), "{payload:?}");
        }
        let valid = frame(&[2, 1]);
        for end in 0..valid.len() {
            assert!(decode(&valid[..end]).is_err());
        }
        let mut wrong_version = valid.clone();
        wrong_version[7] = b'2';
        assert!(decode(&wrong_version).is_err());
        let mut trailing = valid;
        trailing.push(0);
        assert!(decode(&trailing).is_err());
    }

    #[cfg_attr(test, test)]
    pub(super) fn duplicate_and_reverse_sorted_literal_keys_refuse() {
        for keys in [(b'a', b'a'), (b'b', b'a')] {
            let literal = frame(&[5, 2, 0, 0, 0, 1, 0, 0, 0, keys.0, 2, 0, 1, 0, 0, 0, keys.1, 2, 1]);
            assert!(decode(&literal).is_err());
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn nonfinite_literal_float_bits_refuse_in_both_directions() {
        for bits in [0x7ff0_0000_0000_0000_u64, 0xfff0_0000_0000_0000, 0x7ff8_0000_0000_0000] {
            let mut literal = frame(&[1]);
            literal.extend_from_slice(&bits.to_le_bytes());
            assert!(decode(&literal).is_err());
            assert!(encode(&Value::Float(f64::from_bits(bits))).is_err());
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn literal_decoder_depth_boundary_is_not_an_encoder_roundtrip() {
        for (depth, admitted) in [(32, true), (33, false)] {
            let mut literal = b"SMEVID01".to_vec();
            for _ in 0..depth {
                literal.extend_from_slice(&[4, 1, 0, 0, 0]);
            }
            literal.extend_from_slice(&[2, 0]);
            assert_eq!(decode(&literal).is_ok(), admitted, "depth {depth}");
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn per_container_count_above_limit_refuses_without_payload() {
        // 65,537, independently laid out as little-endian u32.
        for tag in [4, 5] {
            assert!(decode(&frame(&[tag, 1, 0, 1, 0])).is_err());
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn total_wire_byte_boundary_counts_magic_tag_and_length() {
        const LIMIT: usize = 16 * 1024 * 1024;
        let mut literal = frame(&[3, 243, 255, 255, 0]); // LIMIT - 13 bytes.
        literal.resize(LIMIT, b'x');
        let value = Value::String("x".repeat(LIMIT - 13));
        assert_eq!(encode(&value).unwrap(), literal);
        assert_eq!(decode(&literal).unwrap(), value);
        literal.push(b'x');
        assert!(decode(&literal).is_err());
        assert!(encode(&Value::String("x".repeat(LIMIT - 12))).is_err());
    }

    #[cfg_attr(test, test)]
    pub(super) fn aggregate_node_boundary_is_independent_of_container_limits() {
        // Root + four arrays + (3*65536 + 65531) leaves = 262144 nodes.
        // Each individual array is legal, depth is two, and wire size is <1 MiB.
        for (last, admitted) in [(65_531_usize, true), (65_532, false)] {
            let counts = [65_536, 65_536, 65_536, last];
            let mut literal = frame(&[4, 4, 0, 0, 0]);
            let mut groups = Vec::new();
            for count in counts {
                literal.push(4);
                literal.extend_from_slice(&(count as u32).to_le_bytes());
                for _ in 0..count {
                    literal.extend_from_slice(&[2, 0]);
                }
                groups.push(Value::Array(vec![Value::Boolean(false); count]));
            }
            assert_eq!(decode(&literal).is_ok(), admitted, "aggregate count, last={last}");
            let encoded = encode(&Value::Array(groups));
            assert_eq!(encoded.is_ok(), admitted, "encoder aggregate count, last={last}");
            if admitted {
                assert_eq!(encoded.unwrap(), literal);
            }
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn independent_literals_cover_all_tags() {
        for (value, bytes) in [
            (Value::Integer(258), &b"SMEVID01\x00\x02\x01\x00\x00\x00\x00\x00\x00"[..]),
            (Value::Float(1.5), &b"SMEVID01\x01\x00\x00\x00\x00\x00\x00\xf8\x3f"[..]),
            (Value::Boolean(false), &b"SMEVID01\x02\x00"[..]),
            (Value::String("\u{e5}".into()), &b"SMEVID01\x03\x02\x00\x00\x00\xc3\xa5"[..]),
            (Value::Array(vec![Value::Boolean(true)]), &b"SMEVID01\x04\x01\x00\x00\x00\x02\x01"[..]),
        ] {
            assert_eq!(encode(&value).unwrap(), bytes);
            assert_eq!(decode(bytes).unwrap(), value);
        }
        let negative_zero = b"SMEVID01\x01\x00\x00\x00\x00\x00\x00\x00\x80";
        let Value::Float(n) = decode(negative_zero).unwrap() else { panic!("float") };
        assert_eq!(n.to_bits(), (-0.0_f64).to_bits());
        assert_eq!(encode(&Value::Float(n)).unwrap(), negative_zero);
    }

    #[cfg_attr(test, test)]
    pub(super) fn decoder_rejects_noncanonical_keys_versions_and_nonfinite_bits() {
        for bytes in [
            &b"SMEVID02\x02\x00"[..],
            &b"SMEVID01\x01\x00\x00\x00\x00\x00\x00\xf0\x7f"[..],
            &b"SMEVID01\x01\x01\x00\x00\x00\x00\x00\xf0\x7f"[..],
            &b"SMEVID01\x05\x02\x00\x00\x00\x01\x00\x00\x00a\x02\x00\x01\x00\x00\x00a\x02\x01"[..],
            &b"SMEVID01\x05\x02\x00\x00\x00\x01\x00\x00\x00b\x02\x00\x01\x00\x00\x00a\x02\x01"[..],
        ] {
            assert!(decode(bytes).is_err());
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn depth_count_bytes_and_aggregate_nodes_have_exact_limits() {
        let mut nested = MAGIC.to_vec();
        for _ in 0..32 {
            nested.extend_from_slice(&[4, 1, 0, 0, 0]);
        }
        nested.extend_from_slice(&[2, 0]);
        assert!(decode(&nested).is_ok());
        nested.splice(8..8, [4, 1, 0, 0, 0]);
        assert!(decode(&nested).is_err());

        let mut array = b"SMEVID01\x04\x00\x00\x01\x00".to_vec();
        for _ in 0..65_536 {
            array.extend_from_slice(&[2, 0]);
        }
        assert!(decode(&array).is_ok());
        array[9] = 1;
        assert!(decode(&array).is_err());

        let text = Value::String("x".repeat(MAX_BYTES - 13));
        let bytes = encode(&text).unwrap();
        assert_eq!(bytes.len(), MAX_BYTES);
        assert!(decode(&bytes).is_ok());
        let mut over = bytes;
        over.push(0);
        assert!(decode(&over).is_err());
        assert!(encode(&Value::String("x".repeat(MAX_BYTES - 12))).is_err());

        let branch = Value::Array(vec![Value::Boolean(false); 65_536]);
        assert!(encode(&Value::Array(vec![branch; 4])).unwrap_err().contains("node budget"));
        let mut hostile = b"SMEVID01\x04\x04\x00\x00\x00".to_vec();
        for _ in 0..4 {
            hostile.extend_from_slice(&[4, 0, 0, 1, 0]);
            for _ in 0..65_536 {
                hostile.extend_from_slice(&[2, 0]);
            }
        }
        assert!(decode(&hostile).unwrap_err().contains("node budget"));
    }
    #[cfg_attr(test, test)]
    pub(super) fn golden_bytes_bind_signed_integer_and_table_layout() {
        let value = Value::Table(Map::from([("x".into(), Value::Integer(-2))]));
        let bytes = b"SMEVID01\x05\x01\x00\x00\x00\x01\x00\x00\x00x\x00\xfe\xff\xff\xff\xff\xff\xff\xff";
        assert_eq!(encode(&value).unwrap(), bytes);
        assert_eq!(decode(bytes).unwrap(), value);
    }
    #[cfg_attr(test, test)]
    pub(super) fn rejects_every_truncation_trailing_data_and_invalid_tags() {
        let bytes = encode(&Value::Array(vec![Value::Boolean(true), Value::String("å".into())])).unwrap();
        for end in 0..bytes.len() {
            assert!(decode(&bytes[..end]).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(decode(&trailing).is_err());
        for payload in [&[9][..], &[2, 2], &[3, 1, 0, 0, 0, 255], &[4, 255, 255, 255, 255]] {
            let mut invalid = MAGIC.to_vec();
            invalid.extend_from_slice(payload);
            assert!(decode(&invalid).is_err());
        }
    }
    #[cfg_attr(test, test)]
    pub(super) fn rejects_nonfinite_float_and_excessive_nesting() {
        assert!(encode(&Value::Float(f64::NAN)).is_err());
        let mut value = Value::Boolean(false);
        for _ in 0..34 {
            value = Value::Array(vec![value]);
        }
        assert!(encode(&value).is_err());
    }
}

#[cfg(feature = "private-checks")]
pub fn private_assertions() -> Vec<(&'static str, fn())> {
    vec![
        (
            "bounded_file_reader_handles_valid_sparse_and_malformed_files",
            tests::bounded_file_reader_handles_valid_sparse_and_malformed_files as fn(),
        ),
        (
            "unsigned_counter_conversion_refuses_signed_range_overflow",
            tests::unsigned_counter_conversion_refuses_signed_range_overflow as fn(),
        ),
        ("literal_scalar_wire_layouts_are_independent", tests::literal_scalar_wire_layouts_are_independent as fn()),
        ("literal_array_and_nested_sorted_table_layouts", tests::literal_array_and_nested_sorted_table_layouts as fn()),
        (
            "string_lengths_count_utf8_bytes_in_little_endian",
            tests::string_lengths_count_utf8_bytes_in_little_endian as fn(),
        ),
        (
            "malformed_literal_tags_strings_booleans_and_versions_refuse",
            tests::malformed_literal_tags_strings_booleans_and_versions_refuse as fn(),
        ),
        (
            "duplicate_and_reverse_sorted_literal_keys_refuse",
            tests::duplicate_and_reverse_sorted_literal_keys_refuse as fn(),
        ),
        (
            "nonfinite_literal_float_bits_refuse_in_both_directions",
            tests::nonfinite_literal_float_bits_refuse_in_both_directions as fn(),
        ),
        (
            "literal_decoder_depth_boundary_is_not_an_encoder_roundtrip",
            tests::literal_decoder_depth_boundary_is_not_an_encoder_roundtrip as fn(),
        ),
        (
            "per_container_count_above_limit_refuses_without_payload",
            tests::per_container_count_above_limit_refuses_without_payload as fn(),
        ),
        (
            "total_wire_byte_boundary_counts_magic_tag_and_length",
            tests::total_wire_byte_boundary_counts_magic_tag_and_length as fn(),
        ),
        (
            "aggregate_node_boundary_is_independent_of_container_limits",
            tests::aggregate_node_boundary_is_independent_of_container_limits as fn(),
        ),
        ("independent_literals_cover_all_tags", tests::independent_literals_cover_all_tags as fn()),
        (
            "decoder_rejects_noncanonical_keys_versions_and_nonfinite_bits",
            tests::decoder_rejects_noncanonical_keys_versions_and_nonfinite_bits as fn(),
        ),
        (
            "depth_count_bytes_and_aggregate_nodes_have_exact_limits",
            tests::depth_count_bytes_and_aggregate_nodes_have_exact_limits as fn(),
        ),
        (
            "golden_bytes_bind_signed_integer_and_table_layout",
            tests::golden_bytes_bind_signed_integer_and_table_layout as fn(),
        ),
        (
            "rejects_every_truncation_trailing_data_and_invalid_tags",
            tests::rejects_every_truncation_trailing_data_and_invalid_tags as fn(),
        ),
        ("rejects_nonfinite_float_and_excessive_nesting", tests::rejects_nonfinite_float_and_excessive_nesting as fn()),
    ]
}
