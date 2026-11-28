use std::ops::Range;

const HEADER_BYTES: usize = 64;
pub const SECTION_HEADER_BYTES: u16 = 64;
const SYMBOL_BYTES: usize = 24;
const IDENT: [u8; 6] = [0x7f, b'E', b'L', b'F', 2, 1];
const SHT_SYMTAB: u32 = 2;
const SHT_NOBITS: u32 = 8;
const SHF_ALLOC: u64 = 2;
const STT_FUNC: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Symbols,
    NoBits,
    Other,
}

impl SectionKind {
    const fn of(code: u32) -> Self {
        match code {
            SHT_SYMTAB => Self::Symbols,
            SHT_NOBITS => Self::NoBits,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub kind: SectionKind,
    pub loaded: bool,
    pub size: u64,
    file: Range<usize>,
    link: usize,
}

impl Section {
    pub fn file_bytes(&self) -> u64 {
        if self.kind == SectionKind::NoBits {
            0
        } else {
            self.size
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub size: u64,
}

#[derive(Debug)]
pub struct Elf {
    pub file_size: u64,
    pub sections: Vec<Section>,
    pub functions: Vec<Function>,
}

fn field<const N: usize>(bytes: &[u8], at: usize) -> Result<[u8; N], String> {
    at.checked_add(N)
        .and_then(|end| bytes.get(at..end))
        .and_then(|slice| <[u8; N]>::try_from(slice).ok())
        .ok_or_else(|| format!("truncated at byte {at}"))
}

fn u16_at(bytes: &[u8], at: usize) -> Result<u16, String> {
    field(bytes, at).map(u16::from_le_bytes)
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, String> {
    field(bytes, at).map(u32::from_le_bytes)
}

fn u64_at(bytes: &[u8], at: usize) -> Result<u64, String> {
    field(bytes, at).map(u64::from_le_bytes)
}

fn usize_at(bytes: &[u8], at: usize) -> Result<usize, String> {
    usize::try_from(u64_at(bytes, at)?).map_err(|error| error.to_string())
}

fn index(value: u32) -> Result<usize, String> {
    usize::try_from(value).map_err(|error| error.to_string())
}

fn string_at(table: &[u8], at: usize) -> Result<String, String> {
    let tail = table
        .get(at..)
        .ok_or_else(|| format!("string {at} outside its table"))?;
    let text = tail.split(|&byte| byte == 0).next().unwrap_or_default();
    Ok(String::from_utf8_lossy(text).into_owned())
}

fn span(start: usize, length: usize) -> Result<Range<usize>, String> {
    start
        .checked_add(length)
        .map(|end| start..end)
        .ok_or_else(|| format!("span at {start} overflows"))
}

fn contents<'a>(bytes: &'a [u8], section: &Section) -> Result<&'a [u8], String> {
    bytes
        .get(section.file.clone())
        .ok_or_else(|| format!("{} lies outside the file", section.name))
}

struct RawSection {
    name: usize,
    kind: SectionKind,
    loaded: bool,
    size: u64,
    file: Range<usize>,
    link: usize,
}

fn raw_section(header: &[u8]) -> Result<RawSection, String> {
    let kind = SectionKind::of(u32_at(header, 4)?);
    let offset = usize_at(header, 24)?;
    let size = u64_at(header, 32)?;
    let length = if kind == SectionKind::NoBits {
        0
    } else {
        usize::try_from(size).map_err(|error| error.to_string())?
    };
    Ok(RawSection {
        name: index(u32_at(header, 0)?)?,
        kind,
        loaded: u64_at(header, 8)? & SHF_ALLOC != 0,
        size,
        file: span(offset, length)?,
        link: index(u32_at(header, 40)?)?,
    })
}

fn sections(bytes: &[u8]) -> Result<Vec<Section>, String> {
    let entry = u16_at(bytes, 0x3a)?;
    if entry != SECTION_HEADER_BYTES {
        return Err(format!("section headers of {entry} bytes"));
    }
    let entry = usize::from(entry);
    let count = usize::from(u16_at(bytes, 0x3c)?);
    let length = count
        .checked_mul(entry)
        .ok_or_else(|| "section table overflows".to_owned())?;
    let table = bytes
        .get(span(usize_at(bytes, 0x28)?, length)?)
        .ok_or_else(|| "section table lies outside the file".to_owned())?;
    let names = usize::from(u16_at(bytes, 0x3e)?);
    let raw = table
        .chunks_exact(entry)
        .map(raw_section)
        .collect::<Result<Vec<_>, _>>()?;
    let name_table = raw
        .get(names)
        .and_then(|section| bytes.get(section.file.clone()))
        .ok_or_else(|| "no section name table".to_owned())?;
    raw.into_iter()
        .map(|section| {
            Ok(Section {
                name: string_at(name_table, section.name)?,
                kind: section.kind,
                loaded: section.loaded,
                size: section.size,
                file: section.file,
                link: section.link,
            })
        })
        .collect()
}

fn functions(bytes: &[u8], sections: &[Section]) -> Result<Vec<Function>, String> {
    let Some(symbols) = sections
        .iter()
        .find(|section| section.kind == SectionKind::Symbols)
    else {
        return Ok(Vec::new());
    };
    let names = sections
        .get(symbols.link)
        .ok_or_else(|| "symbol names missing".to_owned())?;
    let names = contents(bytes, names)?;
    let table = contents(bytes, symbols)?;
    let mut functions = Vec::new();
    for entry in table.as_chunks::<SYMBOL_BYTES>().0 {
        let size = u64_at(entry, 16)?;
        let [.., info] = field::<5>(entry, 0)?;
        if info & 0xf == STT_FUNC && size > 0 {
            functions.push(Function {
                name: string_at(names, index(u32_at(entry, 0)?)?)?,
                size,
            });
        }
    }
    Ok(functions)
}

impl Elf {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.get(..IDENT.len()) != Some(&IDENT[..]) || bytes.len() < HEADER_BYTES {
            return Err("not a 64-bit little-endian ELF file".to_owned());
        }
        let sections = sections(bytes)?;
        Ok(Self {
            file_size: u64::try_from(bytes.len()).map_err(|error| error.to_string())?,
            functions: functions(bytes, &sections)?,
            sections,
        })
    }

    pub fn section_size(&self, name: &str) -> u64 {
        self.sections
            .iter()
            .filter(|section| section.name == name)
            .map(|section| section.size)
            .sum()
    }
}
