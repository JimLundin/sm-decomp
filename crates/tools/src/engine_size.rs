use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

mod elf;

use elf::{Elf, Function, SECTION_HEADER_BYTES};

pub const USAGE: &str = "tools engine-size BINARY BASELINE [--write]";

const LISTED: usize = 10;
const NAME_CHARACTERS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Sizes {
    file: u64,
    stripped: u64,
    text: u64,
    rodata: u64,
    data: u64,
    bss: u64,
}

impl Sizes {
    fn of(elf: &Elf) -> Self {
        let removed = elf.sections.iter().filter(|section| {
            !section.loaded
                && (section.kind == elf::SectionKind::Symbols
                    || section.name == ".strtab"
                    || section.name.starts_with(".debug"))
        });
        let (count, bytes) = removed.fold((0_u64, 0_u64), |(count, bytes), section| {
            (
                count.saturating_add(1),
                bytes.saturating_add(section.file_bytes()),
            )
        });
        let headers = count.saturating_mul(u64::from(SECTION_HEADER_BYTES));
        Self {
            file: elf.file_size,
            stripped: elf.file_size.saturating_sub(bytes).saturating_sub(headers),
            text: elf.section_size(".text"),
            rodata: elf.section_size(".rodata"),
            data: elf.section_size(".data"),
            bss: elf.section_size(".bss"),
        }
    }

    const fn rows(self) -> [(&'static str, u64); 6] {
        [
            ("File", self.file),
            ("Stripped", self.stripped),
            (".text", self.text),
            (".rodata", self.rodata),
            (".data", self.data),
            (".bss", self.bss),
        ]
    }
}

fn change(before: u64, after: u64) -> String {
    if after >= before {
        format!("+{}", after.saturating_sub(before))
    } else {
        format!("-{}", before.saturating_sub(after))
    }
}

fn readable(function: &Function) -> String {
    let name = format!("{:#}", rustc_demangle::demangle(&function.name));
    if name.chars().count() > NAME_CHARACTERS {
        let kept: String = name.chars().take(NAME_CHARACTERS).collect();
        format!("{kept}...")
    } else {
        name
    }
}

/// Built-in types whose inherent methods core defines, such as `<str>::trim`.
const PRIMITIVES: [&str; 17] = [
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64",
];

fn crate_of(symbol: &str) -> String {
    let Ok(demangled) = rustc_demangle::try_demangle(symbol) else {
        return "(other)".to_owned();
    };
    let demangled = format!("{demangled:#}");
    let path = demangled.trim_start_matches(['<', '&', '*']);
    let path = path
        .strip_prefix("mut ")
        .or_else(|| path.strip_prefix("const "))
        .unwrap_or(path);
    if path.starts_with(['[', '(']) {
        return "core".to_owned();
    }
    let name: String = path
        .chars()
        .take_while(|character| character.is_alphanumeric() || *character == '_')
        .collect();
    if PRIMITIVES.contains(&name.as_str()) {
        "core".to_owned()
    } else if name.is_empty() {
        "(other)".to_owned()
    } else {
        name
    }
}

fn by_crate(functions: &[Function]) -> Vec<(String, u64)> {
    let mut crates = BTreeMap::<String, u64>::new();
    for function in functions {
        let total = crates.entry(crate_of(&function.name)).or_default();
        *total = total.saturating_add(function.size);
    }
    let mut crates: Vec<_> = crates.into_iter().collect();
    crates.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    crates
}

fn largest(functions: &[Function]) -> Vec<&Function> {
    let mut sorted: Vec<_> = functions.iter().collect();
    sorted.sort_by(|left, right| {
        right
            .size
            .cmp(&left.size)
            .then_with(|| left.name.cmp(&right.name))
    });
    sorted.truncate(LISTED);
    sorted
}

fn report(baseline: Sizes, elf: &Elf) -> String {
    let current = Sizes::of(elf);
    let mut text = String::from(
        "## Engine size\n\n| Bytes | Baseline | This build | Change |\n|---|---:|---:|---:|\n",
    );
    for ((name, before), (_, after)) in baseline.rows().into_iter().zip(current.rows()) {
        let _ = writeln!(
            text,
            "| {name} | {before} | {after} | {} |",
            change(before, after)
        );
    }
    text.push_str("\n### Largest crates by code bytes\n\n| Crate | Bytes |\n|---|---:|\n");
    for (name, size) in by_crate(&elf.functions).into_iter().take(LISTED) {
        let _ = writeln!(text, "| {name} | {size} |");
    }
    text.push_str("\n### Largest functions\n\n| Function | Bytes |\n|---|---:|\n");
    for function in largest(&elf.functions) {
        let _ = writeln!(text, "| `{}` | {} |", readable(function), function.size);
    }
    text
}

pub fn run(args: &[String]) -> Result<String, String> {
    let (binary, baseline, write) = match args {
        [binary, baseline] => (binary, baseline, false),
        [binary, baseline, flag] if flag == "--write" => (binary, baseline, true),
        _ => return Err(USAGE.to_owned()),
    };
    let bytes = std::fs::read(binary).map_err(|error| format!("{binary}: {error}"))?;
    let elf = Elf::parse(&bytes).map_err(|error| format!("{binary}: {error}"))?;
    let recorded = if write {
        let current = Sizes::of(&elf);
        let text = toml::to_string(&current).map_err(|error| error.to_string())?;
        std::fs::write(baseline, text).map_err(|error| format!("{baseline}: {error}"))?;
        current
    } else {
        let text =
            std::fs::read_to_string(baseline).map_err(|error| format!("{baseline}: {error}"))?;
        toml::from_str(&text).map_err(|error| format!("{baseline}: {error}"))?
    };
    Ok(report(recorded, &elf))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAMES: &[u8] =
        b"\0.text\0.rodata\0.data\0.bss\0.symtab\0.strtab\0.shstrtab\0.debug_info\0";
    const SYMBOL_NAMES: &[u8] =
        b"\0_RNvCs1_4game4tick\0_RNvCs1_4game4draw\0_RNvCs1_7runtime3run\0table\0empty\0";

    struct Part {
        name: &'static str,
        kind: u32,
        flags: u64,
        contents: Vec<u8>,
        size: u64,
        link: u32,
    }

    fn offset_in(table: &[u8], name: &str) -> u32 {
        let needle = [b"\0", name.as_bytes(), b"\0"].concat();
        let at = table
            .windows(needle.len())
            .position(|window| window == needle)
            .unwrap();
        u32::try_from(at.checked_add(1).unwrap()).unwrap()
    }

    fn symbol(name: &str, info: u8, size: u64) -> Vec<u8> {
        let mut entry = Vec::new();
        entry.extend(offset_in(SYMBOL_NAMES, name).to_le_bytes());
        entry.extend([info, 0, 1, 0]);
        entry.extend(0_u64.to_le_bytes());
        entry.extend(size.to_le_bytes());
        entry
    }

    fn part(name: &'static str, kind: u32, flags: u64, contents: Vec<u8>) -> Part {
        let size = u64::try_from(contents.len()).unwrap();
        Part {
            name,
            kind,
            flags,
            contents,
            size,
            link: 0,
        }
    }

    fn fixture() -> Vec<u8> {
        let symbols = [
            vec![0; 24],
            symbol("_RNvCs1_4game4tick", 0x12, 20),
            symbol("_RNvCs1_4game4draw", 0x02, 5),
            symbol("_RNvCs1_7runtime3run", 0x12, 7),
            symbol("table", 0x11, 16),
            symbol("empty", 0x12, 0),
        ]
        .concat();
        let bss = Part {
            size: 64,
            ..part(".bss", 8, 3, Vec::new())
        };
        let parts = [
            part("", 0, 0, Vec::new()),
            part(".text", 1, 6, vec![0x90; 32]),
            part(".rodata", 1, 2, vec![1; 16]),
            part(".data", 1, 3, vec![2; 8]),
            bss,
            Part {
                link: 6,
                ..part(".symtab", 2, 0, symbols)
            },
            part(".strtab", 3, 0, SYMBOL_NAMES.to_vec()),
            part(".shstrtab", 3, 0, NAMES.to_vec()),
            part(".debug_info", 1, 0, vec![3; 10]),
        ];
        let mut bytes = vec![0; 64];
        let mut headers = Vec::new();
        for part in &parts {
            let name = if part.name.is_empty() {
                0
            } else {
                offset_in(NAMES, part.name)
            };
            headers.extend(name.to_le_bytes());
            headers.extend(part.kind.to_le_bytes());
            headers.extend(part.flags.to_le_bytes());
            headers.extend(0_u64.to_le_bytes());
            headers.extend(u64::try_from(bytes.len()).unwrap().to_le_bytes());
            headers.extend(part.size.to_le_bytes());
            headers.extend(part.link.to_le_bytes());
            headers.extend([0; 20]);
            bytes.extend(&part.contents);
        }
        let table = u64::try_from(bytes.len()).unwrap();
        bytes.extend(headers);
        bytes[..6].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1]);
        bytes[0x28..0x30].copy_from_slice(&table.to_le_bytes());
        bytes[0x3a..0x3c].copy_from_slice(&64_u16.to_le_bytes());
        bytes[0x3c..0x3e].copy_from_slice(&u16::try_from(parts.len()).unwrap().to_le_bytes());
        bytes[0x3e..0x40].copy_from_slice(&7_u16.to_le_bytes());
        bytes
    }

    #[test]
    fn sizes_come_from_the_named_sections_and_stripping_drops_symbols_and_debug_info() {
        let bytes = fixture();
        let sizes = Sizes::of(&Elf::parse(&bytes).unwrap());
        let file = u64::try_from(bytes.len()).unwrap();
        let symbol_names = u64::try_from(SYMBOL_NAMES.len()).unwrap();
        let removed = symbol_names.checked_add(6 * 24 + 10 + 3 * 64).unwrap();
        assert_eq!(
            sizes,
            Sizes {
                file,
                stripped: file.checked_sub(removed).unwrap(),
                text: 32,
                rodata: 16,
                data: 8,
                bss: 64,
            }
        );
    }

    #[test]
    fn code_bytes_group_by_crate_and_count_only_sized_functions() {
        let elf = Elf::parse(&fixture()).unwrap();
        assert_eq!(
            by_crate(&elf.functions),
            [("game".to_owned(), 25), ("runtime".to_owned(), 7)]
        );
        let names: Vec<_> = largest(&elf.functions).into_iter().map(readable).collect();
        assert_eq!(names, ["game::tick", "runtime::run", "game::draw"]);
    }

    #[test]
    fn built_in_type_methods_count_as_core_and_unmangled_symbols_as_other() {
        let crates = [
            "_RNvMNtCs1_4core3stre4trim",
            "_RNvMNtNtCs1_4core4char7methodsc11is_assigned",
            "_RNvMNtCs1_4core5sliceSh3len",
            "_RNvCs1_4game4tick",
            "main",
            "memcpy",
        ]
        .map(crate_of);
        assert_eq!(
            crates,
            ["core", "core", "core", "game", "(other)", "(other)"]
        );
    }

    #[test]
    fn the_report_shows_each_size_against_the_baseline() {
        let elf = Elf::parse(&fixture()).unwrap();
        let baseline = Sizes {
            text: 40,
            bss: 60,
            ..Sizes::of(&elf)
        };
        let text = report(baseline, &elf);
        assert!(text.contains("| .text | 40 | 32 | -8 |"), "{text}");
        assert!(text.contains("| .bss | 60 | 64 | +4 |"), "{text}");
        assert!(text.contains("| .rodata | 16 | 16 | +0 |"), "{text}");
    }

    #[test]
    fn a_file_that_is_not_a_whole_elf_is_refused() {
        let bytes = fixture();
        assert!(Elf::parse(b"#!/bin/sh\n").is_err());
        assert!(Elf::parse(bytes.split_last().unwrap().1).is_err());
    }
}
