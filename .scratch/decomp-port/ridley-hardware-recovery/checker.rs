//! Independent std-only natural-state and public-admission checker.
#![allow(dead_code)]
#[path = "memory-wrapper.rs"]
mod gameplay;
pub use gameplay::*;
#[path = "source/crates/port/src/ridley_subsystem.rs"]
mod family;

use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};
type CheckResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Event {
    capture: u64,
    boundary: u64,
    frame: u64,
    kind: String,
    address: u32,
    argument: u32,
    slot: u16,
    returned: u32,
    after: bool,
}

fn field<'a>(line: &'a str, key: &str) -> CheckResult<&'a str> {
    let marker = format!("\"{key}\":");
    let (_, rest) = line.split_once(&marker).ok_or_else(|| format!("event key missing: {key}"))?;
    let rest = rest.trim_start();
    if let Some(quoted) = rest.strip_prefix('"') {
        Ok(quoted.split_once('"').ok_or("unterminated event string")?.0)
    } else {
        Ok(rest.split([',', '}']).next().ok_or("event value missing")?.trim())
    }
}

fn event(line: &str) -> CheckResult<Event> {
    let after = match field(line, "phase")? {
        "before" => false,
        "after" => true,
        _ => return Err("native phase differs".into()),
    };
    Ok(Event {
        capture: field(line, "capture")?.parse()?,
        boundary: field(line, "boundary_id")?.parse()?,
        frame: field(line, "frame")?.parse()?,
        kind: field(line, "kind")?.into(),
        address: field(line, "address")?.parse()?,
        argument: field(line, "argument")?.parse()?,
        slot: field(line, "slot")?.parse()?,
        returned: field(line, "returned")?.parse()?,
        after,
    })
}

fn capture_path(root: &Path, id: u64, suffix: &str) -> PathBuf {
    root.join(format!("fixtures/ridley-{id:010}-{suffix}"))
}

fn words(bytes: &[u8]) -> CheckResult<[u16; 8]> {
    if bytes.len() != 16 {
        return Err("host extent differs".into());
    }
    Ok(std::array::from_fn(|i| u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]])))
}

fn definitions(source: &str) -> CheckResult<BTreeMap<u32, String>> {
    let mut values = BTreeMap::new();
    for line in source.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() == 3 && fields[0] == "#define" && fields[1].starts_with("fn") {
            if let Some(value) = fields[2].strip_prefix("0x") {
                values.insert(u32::from_str_radix(value, 16)?, fields[1][2..].into());
            }
        }
    }
    Ok(values)
}

fn symbol<'a>(e: &Event, defs: &'a BTreeMap<u32, String>) -> CheckResult<Option<&'a str>> {
    Ok(match e.kind.as_str() {
        "baby" | "baby-state" | "helper" => None,
        "state" => Some("CallRidleyFunc"),
        "script" if e.address == 0xa0c26a => Some("ProcessEnemyInstructions"),
        "tail" if e.address == 0xa6caf5 => Some("Ridley_Func_70"),
        "direct" if e.address == 0xa6aaaf => Some("HandleCeresRidleyGetawayCutscene"),
        "direct" if e.address == 0xa6df60 => Some("sub_A6DF60"),
        "opcode" if e.address == 0xa6812f => Some("EnemyInstr_Sleep"),
        "ai" | "touch" | "shot" | "direct" | "opcode" => {
            Some(defs.get(&e.address).ok_or_else(|| format!("unknown native address {:06x}", e.address))?)
        }
        _ => return Err(format!("unsupported native kind {}", e.kind).into()),
    })
}

struct CaptureHost {
    effects: String,
    services: Vec<u8>,
    service_cursor: usize,
    ppu: [u8; 40],
    calls: usize,
}

impl CaptureHost {
    const SERVICE_BYTES: usize = 20 + WRAM_SIZE + 16;
    fn new(ppu: &[u8], services: Vec<u8>) -> CheckResult<Self> {
        if services.len() % Self::SERVICE_BYTES != 0 {
            return Err("partial captured service".into());
        }
        if ppu.len() != 40 || ppu[0] > 1 {
            return Err("hardware v2 extent or discriminator differs".into());
        }
        Ok(Self { effects: String::new(), services, service_cursor: 0, ppu: ppu.try_into()?, calls: 0 })
    }
    fn active_base(&self) -> usize {
        if self.ppu[0] == 0 { 1 } else { 20 }
    }
    fn matrix(&self, i: usize) -> i16 {
        let base = self.active_base();
        i16::from_le_bytes([self.ppu[base + i * 2], self.ppu[base + i * 2 + 1]])
    }
    fn write_mode7(&mut self, address: u16, value: u8) {
        let i = usize::from(address - 0x211b);
        let base = self.active_base();
        self.ppu[base + i * 2] = self.ppu[base + 16];
        self.ppu[base + i * 2 + 1] = value;
        self.ppu[base + 16] = value;
        self.effects.push_str(&format!("write {address} {value}\n"));
    }
    fn read_multiply(&mut self, address: u16) -> u8 {
        let product = i32::from(self.matrix(0)) * i32::from(self.matrix(1) >> 8);
        let value = (product >> (8 * u32::from(address - 0x2134))) as u8;
        let base = self.active_base();
        self.ppu[base + 17] = value;
        self.effects.push_str(&format!("read {address} {value}\n"));
        value
    }
    fn service_fields(record: &[u8]) -> [u32; 5] {
        std::array::from_fn(|i| u32::from_le_bytes(record[i * 4..i * 4 + 4].try_into().unwrap()))
    }
}

impl family::Host for CaptureHost {
    fn service(
        &mut self,
        ram: &mut Wram<'_>,
        _: &[u8],
        projectile: &mut family::ProjectileHost,
        service: family::Service,
    ) -> family::Result<u32> {
        let (kind, a, b) = match service {
            family::Service::SpawnEnemy { bank, population } => (0, u16::from(bank), population),
            family::Service::EnemyDeath { slot, animation } => (1, slot, animation),
            family::Service::EnemyItemDrop { slot } => (2, slot, 0),
            family::Service::NormalTouch => (3, 0, 0),
            family::Service::NormalShot => (4, 0, 0),
            family::Service::NormalPowerbomb => (5, 0, 0),
            family::Service::SamusCode { code } => (6, code, 0),
        };
        let end = self.service_cursor.checked_add(Self::SERVICE_BYTES * 2).ok_or("service extent overflow")?;
        let pair = self.services.get(self.service_cursor..end).ok_or("captured service missing")?;
        let (before, after) = pair.split_at(Self::SERVICE_BYTES);
        if Self::service_fields(before) != [0, kind, u32::from(a), u32::from(b), 0] {
            return Err("captured service request differs");
        }
        if before[20..20 + WRAM_SIZE].iter().enumerate().any(|(i, v)| *v != ram.byte(i)) {
            return Err("full WRAM before captured service differs");
        }
        let before_words = words(&before[20 + WRAM_SIZE..]).map_err(|_| "captured service host extent")?;
        if projectile.words() != before_words {
            return Err("eight host words before service differ");
        }
        let [phase, returned_kind, returned_a, returned_b, value] = Self::service_fields(after);
        if [phase, returned_kind, returned_a, returned_b] != [1, kind, u32::from(a), u32::from(b)] {
            return Err("captured service continuation signature differs");
        }
        self.effects.push_str(&format!("service {kind} {a} {b}\n"));
        for (i, v) in after[20..20 + WRAM_SIZE].iter().enumerate() {
            ram.set_byte(i, *v);
        }
        *projectile = family::ProjectileHost::from_words(
            words(&after[20 + WRAM_SIZE..]).map_err(|_| "captured service host extent")?,
        );
        self.service_cursor = end;
        self.calls += 1;
        Ok(value)
    }
    fn mode7_multiply(&mut self, sine: i16, factor: u8) -> family::Result<u16> {
        self.write_mode7(0x211b, sine as u8);
        self.write_mode7(0x211b, (sine as u16 >> 8) as u8);
        self.write_mode7(0x211c, factor);
        let low = self.read_multiply(0x2135);
        let high = self.read_multiply(0x2136);
        Ok(u16::from_le_bytes([low, high]))
    }
    fn ordered_audio(&mut self, channel: u8, sound: u16, maximum: u8) -> family::Result<()> {
        self.effects.push_str(&format!("sound {channel} {sound} {maximum}\n"));
        Ok(())
    }
    fn ordered_music(&mut self, track: u16, delay: u16) -> family::Result<()> {
        self.effects.push_str(&format!("music {track} {delay}\n"));
        Ok(())
    }
    fn ordered_projectile(&mut self, definition: u16, parameter: u16, _: Option<u16>) -> family::Result<()> {
        self.effects.push_str(&format!("projectile {definition} {parameter}\n"));
        Ok(())
    }
}

fn main() -> CheckResult<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 5 {
        return Err("usage: checker INVENTORY ROM_VIEW ORIGINAL_ROM REPORT".into());
    }
    let root = Path::new(&args[1]);
    let rom = fs::read(&args[2])?;
    let admission_rom = fs::read(&args[3])?;
    let defs = definitions(&fs::read_to_string(root.join("original-source/funcs.h"))?)?;
    let mut events = BTreeMap::<u64, (Option<Event>, Option<Event>)>::new();
    for line in BufReader::new(fs::File::open(root.join("ridley-subsystem-events.jsonl"))?).lines() {
        let e = event(&line?)?;
        if e.capture == 0 {
            continue;
        }
        let pair = events.entry(e.capture).or_default();
        let target = if e.after { &mut pair.1 } else { &mut pair.0 };
        if target.replace(e).is_some() {
            return Err("duplicate capture phase".into());
        }
    }
    if events.len() != 897 {
        return Err("complete native capture count differs".into());
    }
    let mut counts = BTreeMap::<(String, String, String), usize>::new();
    let mut details = String::new();
    let mut selected = 0;
    let mut admitted = 0;
    let mut failures = 0;
    let mut services = 0;
    for (id, pair) in events {
        let before = pair.0.ok_or("before capture missing")?;
        let after = pair.1.ok_or("after capture missing")?;
        if (before.capture, before.boundary, before.frame, &before.kind, before.address, before.argument, before.slot)
            != (after.capture, after.boundary, after.frame, &after.kind, after.address, after.argument, after.slot)
        {
            return Err("capture invocation differs".into());
        }
        let Some(name) = symbol(&before, &defs)? else {
            *counts.entry((before.kind, "internal".into(), "no direct fixture ABI".into())).or_default() += 1;
            continue;
        };
        selected += 1;
        let mut bytes = fs::read(capture_path(root, id, "before.wram"))?;
        let original = bytes.clone();
        let ram = Wram::new(&mut bytes)?;
        let (a, b) = match before.kind.as_str() {
            "opcode" => (i32::from(before.slot), before.argument as i32),
            "state" => (before.address as i32, 0),
            _ => (0, 0),
        };
        let support = if before.kind == "opcode" {
            family::supports_callback(&ram, &admission_rom, name, a, b)
        } else {
            family::supports_call(&ram, &admission_rom, name)
        };
        let support_label = match support {
            Ok(true) => {
                admitted += 1;
                "admitted"
            }
            Ok(false) => "refused",
            Err(_) => "admission error",
        };
        if bytes != original {
            return Err("immutable public support changed state".into());
        }
        let mut ram = Wram::new(&mut bytes)?;
        let mut projectile =
            family::ProjectileHost::from_words(words(&fs::read(capture_path(root, id, "before.host"))?)?);
        let mut host = CaptureHost::new(
            &fs::read(capture_path(root, id, "before.ppu"))?,
            fs::read(capture_path(root, id, "services.bin"))?,
        )?;
        let result = family::dispatch_fixture(&mut ram, &rom, &mut projectile, &mut host, name, a, b);
        let expected = fs::read(capture_path(root, id, "after.wram"))?;
        let mut mismatch = Vec::new();
        if bytes != expected {
            mismatch.push(format!("WRAM first {:?}", bytes.iter().zip(&expected).position(|(a, b)| a != b)));
        }
        if projectile.words() != words(&fs::read(capture_path(root, id, "after.host"))?)? {
            mismatch.push("host".into());
        }
        let expected_ppu = fs::read(capture_path(root, id, "after.ppu"))?;
        if host.ppu.as_slice() != expected_ppu {
            mismatch.push(format!(
                "PPU {:?}",
                host.ppu.iter().zip(&expected_ppu).enumerate().filter(|(_, (a, b))| a != b).collect::<Vec<_>>()
            ));
        }
        if host.effects != fs::read_to_string(capture_path(root, id, "effects.log"))? {
            mismatch.push("ordered effects".into());
        }
        if host.service_cursor != host.services.len() {
            mismatch.push("remaining services".into());
        }
        if result != Ok(after.returned) {
            mismatch.push(format!("return {result:?} expected {}", after.returned));
        }
        services += host.calls;
        if !mismatch.is_empty() {
            failures += 1;
            eprintln!("capture {id} {name}: {mismatch:?}");
        }
        *counts.entry((before.kind.clone(), name.into(), support_label.into())).or_default() += 1;
        let header = u16::from_le_bytes([
            original[0xf78 + usize::from(before.slot)],
            original[0xf79 + usize::from(before.slot)],
        ]);
        let area = u16::from_le_bytes([original[0x79f], original[0x7a0]]);
        details.push_str(&format!(
            "{id}\t{}\t{}\t{name}\t{support_label}\t{}\t{header:04x}\t{area}\t{:?}\t{}\n",
            before.frame,
            before.kind,
            host.calls,
            host.ppu,
            if mismatch.is_empty() {
                "exact WRAM/host/effects/return; both PPU views and active discriminator".into()
            } else {
                mismatch.join("; ")
            }
        ));
    }
    let mut report = format!(
        "status = \"{}\"\naccepted = false\nlive_exercised = false\noriginal_rom_parity = \"not tested\"\nactive_ppu_state = \"exact authenticated active discriminator and both PPU views\"\nmode7_register_arithmetic = \"independently matched ordered writes/reads and final active PPU state\"\ncaptures = 897\nfixture_pairs = {selected}\npublic_admitted = {admitted}\npublic_refused = {}\nparity_failures = {failures}\nmeasured_c_service_dependencies = {services}\n\n",
        if failures == 0 { "natural-captured-state-effects-match" } else { "natural-captured-mismatch" },
        selected - admitted
    );
    for ((kind, symbol, support), count) in counts {
        report.push_str(&format!(
            "[[counts]]\nkind = {kind:?}\nsymbol = {symbol:?}\npublic_support = {support:?}\ncount = {count}\n\n"
        ));
    }
    fs::write(&args[4], report)?;
    fs::write(
        Path::new(&args[4]).with_extension("tsv"),
        format!(
            "capture\tframe\tkind\tsymbol\tpublic_support\tservices\theader\tarea\tindependent_active_ppu\tparity\n{details}"
        ),
    )?;
    println!(
        "{selected} natural fixtures, {admitted} public-admitted, {} refused, {services} C service dependencies, {failures} parity failures",
        selected - admitted
    );
    if failures != 0 {
        return Err("natural parity differs".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_service_input_is_required_before_replaying_c_continuation() {
        let mut state = vec![0; WRAM_SIZE];
        let mut p = family::ProjectileHost::default();
        let mut records = vec![0; CaptureHost::SERVICE_BYTES * 2];
        records[4..8].copy_from_slice(&3u32.to_le_bytes());
        records[20 + 0x100] = 1;
        let mut host = CaptureHost::new(&[0; 40], records).unwrap();
        use family::Host;
        assert_eq!(
            host.service(&mut Wram::new(&mut state).unwrap(), &[], &mut p, family::Service::NormalTouch),
            Err("full WRAM before captured service differs")
        );
        assert_eq!(host.service_cursor, 0);
    }
    #[test]
    fn mode7_matches_signed_native_register_order_and_preserves_other_ppu_words() {
        let mut ppu = [0x55; 40];
        ppu[0] = 1;
        ppu[20..24].fill(0);
        let mut host = CaptureHost::new(&ppu, Vec::new()).unwrap();
        use family::Host;
        assert_eq!(host.mode7_multiply(-256, 2), Ok(0xfffe));
        assert_eq!(&host.ppu[1..20], &ppu[1..20]);
        assert_eq!(&host.ppu[20..24], &[0, 255, 255, 2]);
        assert_eq!(host.ppu[36], 2);
        assert_eq!(host.ppu[37], 255);
        assert_eq!(host.ppu[38], ppu[38]);
        assert_eq!(host.ppu[39], ppu[39]);
        assert_eq!(host.effects, "write 8475 0\nwrite 8475 255\nwrite 8476 2\nread 8501 254\nread 8502 255\n");
    }
    #[test]
    fn partial_service_snapshot_is_rejected() {
        assert!(CaptureHost::new(&[0; 40], vec![0; CaptureHost::SERVICE_BYTES - 1]).is_err());
    }
}
