//! Passive, source-authenticated actual-C inventory of the complete Ridley subsystem.
//!
//! The one replay retains the attack inventory and observes whole native lifecycle,
//! AI, script, contact and death boundaries. Diagnostic receipts grant no Rust admission.
use crate::{atomic_toml, builds, digest, enemy_instruction_inventory, family};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use toml::Value;

const SUPPORT: &str = include_str!("ridley_subsystem_inventory.c.in");
const HOOKS: &str = include_str!("ridley_subsystem_inventory_hooks.c.in");
const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
// Pinned SnesInit applies its native emulator patches and pads the ROM to 4 MiB.
const ROM_VIEW_SHA256: &str = "40aaebb25b9646bea2dffdc5e1ae680768a39dd122de9353885e1c7456ceae27";
const INPUT_SHA256: &str = "cc643d751554ad6d30b5d7e3a41a4259525517ca7614cf73abec9fac30176c0e";
const UNITS: &[&str] = &["sm_a0.c", "sm_a6.c", "sm_86.c", "sm_80.c", "sm_90.c", "snes/snes.c"];
const HEADER: &str = "kind\taddress\theader\tslot\tstate\tpc\torientation\tarea\twindow\tlifecycle\thits\tfirst_frame\tlast_frame\tcaptures\n";
const WRAM: usize = 131_072;
const SERVICE_RECORD: usize = 20 + WRAM + 16;
const KINDS: &[&str] =
    &["ai", "touch", "shot", "direct", "state", "baby-state", "baby", "script", "opcode", "helper", "tail"];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    checkout: Option<PathBuf>,
    source: Option<PathBuf>,
    asset_root: Option<PathBuf>,
    output: Option<PathBuf>,
    #[serde(default = "default_frames")]
    frames: u64,
    #[serde(default = "default_timeout")]
    timeout: u64,
    #[serde(default = "default_capture_limit")]
    capture_limit: usize,
    #[serde(default = "default_captures_per_key")]
    captures_per_key: usize,
    #[serde(default)]
    prepare_only: bool,
    #[serde(default)]
    observation_control: bool,
    pipeline_root: Option<PathBuf>,
    workers: Option<usize>,
}
fn default_frames() -> u64 {
    129_712
}
fn default_timeout() -> u64 {
    300
}
fn default_capture_limit() -> usize {
    2_048
}
fn default_captures_per_key() -> usize {
    2
}

fn rename_definition(code: &mut String, signature: &str, name: &str, replacement: &str) -> Result<()> {
    let anchor = format!("{signature} {{");
    ensure!(code.matches(&anchor).count() == 1, "Ridley inventory definition anchor must be unique: {signature}");
    let renamed = signature.replacen(name, replacement, 1);
    *code = code.replace(&anchor, &format!("{renamed} {{"));
    Ok(())
}
fn wrap_void(code: &mut String, symbol: &str, kind: usize, address: u32, slot: &str, condition: &str) -> Result<()> {
    let original = format!("RidleyInventoryC_{symbol}");
    rename_definition(code, &format!("void {symbol}(void)"), symbol, &original)?;
    code.push_str(&format!(
        "\nvoid {symbol}(void) {{\n int observe=({condition});\n if(observe)RidleySubsystemBoundaryBegin({kind},0x{address:06x},{slot},0);\n {original}();\n if(observe)RidleySubsystemBoundaryEnd(0,0);\n}}\n"
    ));
    Ok(())
}
fn wrap_dispatch(code: &mut String, symbol: &str, kind: usize, condition: &str) -> Result<()> {
    let original = format!("RidleyInventoryC_{symbol}");
    rename_definition(code, &format!("void {symbol}(uint32 ea)"), symbol, &original)?;
    code.push_str(&format!(
        "\nvoid {symbol}(uint32 ea) {{\n int observe=({condition});\n if(observe)RidleySubsystemBoundaryBegin({kind},ea,cur_enemy_index,0);\n {original}(ea);\n if(observe)RidleySubsystemBoundaryEnd(0,0);\n}}\n"
    ));
    Ok(())
}
fn wrap_service(
    code: &mut String,
    symbol: &str,
    signature: &str,
    arguments: &str,
    kind: usize,
    context: (&str, &str),
    returns: bool,
) -> Result<()> {
    let (a, b) = context;
    let original = format!("RidleyInventoryC_{symbol}");
    rename_definition(code, signature, symbol, &original)?;
    let call = if returns {
        format!(
            "uint16 result={original}({arguments});\n RidleySubsystemServiceEnd({kind},{a},{b},result);\n return result;"
        )
    } else {
        format!("{original}({arguments});\n RidleySubsystemServiceEnd({kind},{a},{b},0);")
    };
    code.push_str(&format!("\n{signature} {{\n RidleySubsystemServiceBegin({kind},{a},{b});\n {call}\n}}\n"));
    Ok(())
}

/// Source transformation is deterministic and every replaced definition is unique.
/// Definitions are renamed; all original call sites keep their public symbols.
pub(crate) fn instrument(sources: &BTreeMap<String, String>, host: &str) -> Result<BTreeMap<String, String>> {
    let native = |name: &str| sources.get(name).with_context(|| format!("missing Ridley inventory source {name}"));
    let (mut a0, mut bank86, mut headless) =
        enemy_instruction_inventory::instrument(native("sm_a0.c")?, native("sm_86.c")?, host)?;
    // The observer definitions need declarations before retained C definitions.
    let include = "#include \"enemy_types.h\"\n";
    a0 = a0.replace(
        include,
        &format!("{include}{HOOKS}\nvoid CallHitboxTouch(uint32);\nvoid CallHitboxShot(uint32,uint16);\n"),
    );
    let mut a6 = format!(
        "{HOOKS}\nvoid CallRidleyFunc(uint32_t);\nvoid CallRidleyFunc_var40(uint32_t);\nuint16_t CallBabyMetroidInstr(uint32_t,uint16_t);\n{}",
        native("sm_a6.c")?
    );
    let mut bank80 = format!("{HOOKS}\n{}", native("sm_80.c")?);
    let mut bank90 = format!("{HOOKS}\n{}", native("sm_90.c")?);
    let mut snes = format!("{HOOKS}\n{}", native("snes/snes.c")?);
    bank86 = format!("{HOOKS}\n{bank86}");

    wrap_dispatch(&mut a0, "CallEnemyAi", 0, "RidleySubsystemNativeHeader(cur_enemy_index) && (ea>>16)==0xa6")?;
    wrap_dispatch(&mut a0, "CallHitboxTouch", 1, "RidleySubsystemNativeHeader(cur_enemy_index) && (ea>>16)==0xa6")?;
    rename_definition(
        &mut a0,
        "void CallHitboxShot(uint32 ea, uint16 j)",
        "CallHitboxShot",
        "RidleyInventoryC_CallHitboxShot",
    )?;
    a0.push_str("\nvoid CallHitboxShot(uint32 ea,uint16 j){\n int observe=RidleySubsystemNativeHeader(cur_enemy_index)&&(ea>>16)==0xa6;\n if(observe)RidleySubsystemBoundaryBegin(2,ea,cur_enemy_index,j);\n RidleyInventoryC_CallHitboxShot(ea,j);\n if(observe)RidleySubsystemBoundaryEnd(0,0);\n}\n");

    // Reuse the original attack observer, adding whole native script metadata outside it.
    rename_definition(
        &mut a0,
        "void ProcessEnemyInstructions(void)",
        "ProcessEnemyInstructions",
        "RidleyInventoryObserved_ProcessEnemyInstructions",
    )?;
    a0.push_str("\nvoid ProcessEnemyInstructions(void){\n uint16 slot=cur_enemy_index;int observe=RidleySubsystemNativeHeader(slot);\n if(observe)RidleySubsystemBoundaryBegin(7,0xa0c26a,slot,0);\n RidleyInventoryObserved_ProcessEnemyInstructions();\n if(observe)RidleySubsystemBoundaryEnd(0,0);\n}\n");
    rename_definition(
        &mut a0,
        "const uint16 *CallEnemyInstr(uint32 address, uint16 slot, const uint16 *argument)",
        "CallEnemyInstr",
        "RidleyInventoryObserved_CallEnemyInstr",
    )?;
    a0.push_str("\nconst uint16 *CallEnemyInstr(uint32 address,uint16 slot,const uint16 *argument){\n int observe=RidleySubsystemNativeHeader(slot)&&(address>>16)==0xa6;\n if(observe)RidleySubsystemBoundaryBegin(8,address,slot,EnemyInventoryPointer(argument,0xa6));\n const uint16 *result=RidleyInventoryObserved_CallEnemyInstr(address,slot,argument);\n if(observe)RidleySubsystemBoundaryEnd(EnemyInventoryPointer(result,0xa6),EnemyInventoryPointerKind(result));\n return result;\n}\n");

    for (symbol, address, slot, condition) in [
        ("Ridley_A2F2", 0xa6a2f2, "0", "RidleySubsystemNativeHeader(0)"),
        ("HandleCeresRidleyGetawayCutscene", 0xa6aaaf, "0", "RidleySubsystemNativeHeader(0)"),
        ("sub_A6DF59", 0xa6df59, "cur_enemy_index", "RidleySubsystemNativeHeader(cur_enemy_index)"),
        ("sub_A6DF60", 0xa6df60, "cur_enemy_index", "RidleySubsystemNativeHeader(cur_enemy_index)"),
    ] {
        wrap_void(&mut a6, symbol, 3, address, slot, condition)?;
    }
    wrap_dispatch(&mut a6, "CallRidleyFunc", 4, "RidleySubsystemNativeHeader(0)")?;
    wrap_dispatch(&mut a6, "CallRidleyFunc_var40", 5, "RidleySubsystemNativeHeader(0)")?;
    wrap_void(&mut a6, "Ridley_Func_70", 10, 0xa6caf5, "0", "RidleySubsystemNativeHeader(0)")?;
    rename_definition(
        &mut a6,
        "uint16 CallBabyMetroidInstr(uint32 ea, uint16 k)",
        "CallBabyMetroidInstr",
        "RidleyInventoryC_CallBabyMetroidInstr",
    )?;
    a6.push_str("\nuint16 CallBabyMetroidInstr(uint32 ea,uint16 k){\n int observe=RidleySubsystemNativeHeader(0);\n if(observe)RidleySubsystemBoundaryBegin(6,ea,0,0xa60000|k);\n uint16 result=RidleyInventoryC_CallBabyMetroidInstr(ea,k);\n if(observe)RidleySubsystemBoundaryEnd(result?0xa60000|result:0,result?1:0);\n return result;\n}\n");

    // These two helpers are translated locally by the Rust subsystem, not C-owned services.
    for (symbol, address) in
        [("DisableMinimapAndMarkBossRoomAsExplored", 0x90a7e2), ("Samus_SetPushedOutOfCeresRidley", 0x90e119)]
    {
        wrap_void(&mut bank90, symbol, 9, address, "0", "RidleySubsystemNativeHeader(0)")?;
    }
    for (symbol, signature, arguments, kind, a, b, returns) in [
        ("SpawnEnemy", "uint16 SpawnEnemy(uint8 db, uint16 k)", "db,k", 0, "db", "k", true),
        ("EnemyDeathAnimation", "void EnemyDeathAnimation(uint16 k, uint16 a)", "k,a", 1, "k", "a", false),
        ("Enemy_ItemDrop_Ridley", "void Enemy_ItemDrop_Ridley(uint16 k)", "k", 2, "k", "0", false),
        (
            "NormalEnemyTouchAiSkipDeathAnim_CurEnemy",
            "void NormalEnemyTouchAiSkipDeathAnim_CurEnemy(void)",
            "",
            3,
            "0",
            "0",
            false,
        ),
        (
            "NormalEnemyShotAiSkipDeathAnim_CurEnemy",
            "void NormalEnemyShotAiSkipDeathAnim_CurEnemy(void)",
            "",
            4,
            "0",
            "0",
            false,
        ),
        (
            "NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy",
            "void NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(void)",
            "",
            5,
            "0",
            "0",
            false,
        ),
    ] {
        wrap_service(&mut a0, symbol, signature, arguments, kind, (a, b), returns)?;
    }
    wrap_service(&mut bank90, "CallSomeSamusCode", "uint16 CallSomeSamusCode(uint16 a)", "a", 6, ("a", "0"), true)?;

    // Transform definitions rather than only linker-wrap: same-unit calls must be observed too.
    for channel in 0..3 {
        for maximum in [1, 3, 6, 9, 15] {
            let symbol = format!("QueueSfx{}_Max{maximum}", channel + 1);
            let original = format!("RidleyInventoryC_{symbol}");
            rename_definition(&mut bank80, &format!("void {symbol}(uint16 a)"), &symbol, &original)?;
            bank80.push_str(&format!("\nvoid {symbol}(uint16 a){{\n RidleySubsystemEffect(\"sound\",0,{channel},a,{maximum},0);\n {original}(a);\n}}\n"));
        }
    }
    rename_definition(
        &mut bank80,
        "void QueueMusic_Delayed8(uint16 a)",
        "QueueMusic_Delayed8",
        "RidleyInventoryC_QueueMusic_Delayed8",
    )?;
    bank80.push_str("\nvoid QueueMusic_Delayed8(uint16 a){\n RidleySubsystemEffect(\"music\",0,a,8,0,0);\n RidleyInventoryC_QueueMusic_Delayed8(a);\n}\n");
    rename_definition(
        &mut bank86,
        "void SpawnEprojWithRoomGfx(uint16 j, uint16 a)",
        "SpawnEprojWithRoomGfx",
        "RidleyInventoryC_SpawnEprojWithRoomGfx",
    )?;
    bank86.push_str("\nvoid SpawnEprojWithRoomGfx(uint16 j,uint16 a){\n RidleySubsystemEffect(\"projectile\",0,j,a,0,0);\n RidleyInventoryC_SpawnEprojWithRoomGfx(j,a);\n}\n");
    rename_definition(
        &mut snes,
        "uint8_t snes_read(Snes* snes, uint32_t adr)",
        "snes_read",
        "RidleyInventoryC_snes_read",
    )?;
    rename_definition(
        &mut snes,
        "void snes_write(Snes* snes, uint32_t adr, uint8_t val)",
        "snes_write",
        "RidleyInventoryC_snes_write",
    )?;
    snes.push_str("\nuint8_t snes_read(Snes* snes,uint32_t adr){\n uint8_t result=RidleyInventoryC_snes_read(snes,adr);\n RidleySubsystemEffect(\"read\",adr,0,0,0,result);return result;\n}\nvoid snes_write(Snes* snes,uint32_t adr,uint8_t val){\n RidleyInventoryC_snes_write(snes,adr,val);\n RidleySubsystemEffect(\"write\",adr,0,0,0,val);\n}\n");
    a0.push_str(SUPPORT);
    let completion = "  EnemyInventorySave();\n  return 0;";
    ensure!(headless.matches(completion).count() == 1, "whole inventory completion anchor must be unique");
    headless = headless.replace(completion, "  EnemyInventorySave();\n  RidleySubsystemInventorySave();\n  return 0;");
    headless = format!("{HOOKS}\n{headless}");
    Ok(BTreeMap::from([
        ("sm_a0.c".into(), a0),
        ("sm_a6.c".into(), a6),
        ("sm_86.c".into(), bank86),
        ("sm_80.c".into(), bank80),
        ("sm_90.c".into(), bank90),
        ("snes/snes.c".into(), snes),
        ("headless.c".into(), headless),
    ]))
}

fn source_inputs(source: &Path) -> Result<BTreeMap<String, String>> {
    UNITS.iter().map(|name| Ok(((*name).into(), fs::read_to_string(source.join("src").join(name))?))).collect()
}
fn snapshot(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "Ridley source snapshot contains a symlink");
        let target = destination.join(path.file_name().context("source filename missing")?);
        if path.is_dir() {
            snapshot(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
            ensure!(digest(&path)? == digest(&target)?, "native source changed while preserving Ridley snapshot");
        }
    }
    Ok(())
}
fn artifact_hashes(root: &Path, directory: &Path, hashes: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "Ridley inventory artifact symlink");
        if path.is_dir() {
            artifact_hashes(root, &path, hashes)?;
        } else if path != root.join("verdict.toml")
            && !path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(".metrics.toml"))
        {
            hashes.insert(path.strip_prefix(root)?.to_string_lossy().into_owned(), digest(&path)?);
        }
    }
    Ok(())
}
pub(crate) fn build_command(source: &Path, run: &Path, capture_limit: usize, per_key: usize) -> Result<Vec<String>> {
    let mut command = vec![
        "cc".into(),
        "-O2".into(),
        "-fno-strict-aliasing".into(),
        "-ffunction-sections".into(),
        "-fdata-sections".into(),
        "-iquote".into(),
        source.join("src").display().to_string(),
        "-iquote".into(),
        source.join("src/snes").display().to_string(),
        format!("-DRIDLEY_SUBSYSTEM_CAPTURE_LIMIT={capture_limit}"),
        format!("-DRIDLEY_SUBSYSTEM_CAPTURES_PER_KEY={per_key}"),
    ];
    for unit in family::c_units(source)? {
        let relative = unit.strip_prefix(source.join("src"))?.to_string_lossy().replace('\\', "/");
        command.push(if UNITS.contains(&relative.as_str()) {
            run.join(relative).display().to_string()
        } else {
            unit.display().to_string()
        });
    }
    command.extend([
        run.join("headless.c").display().to_string(),
        "-Wl,--gc-sections".into(),
        "-Wl,--wrap=Call".into(),
        "-Wl,--wrap=cpu_runOpcode".into(),
        "-Wl,--wrap=DecompressToMem".into(),
        "-lm".into(),
        "-o".into(),
        run.join("inventory").display().to_string(),
    ]);
    Ok(command)
}

fn validate_counters(table: &str, frames: u64, limit: usize, per_key: usize) -> Result<(usize, u64, usize)> {
    ensure!(table.starts_with(HEADER), "whole Ridley inventory table header differs");
    let mut rows = 0;
    let mut hits = 0;
    let mut captures = 0;
    let mut keys = BTreeSet::new();
    for line in table.lines().skip(1) {
        let fields = line.split('\t').collect::<Vec<_>>();
        ensure!(fields.len() == 14, "Ridley counter extent differs");
        ensure!(KINDS.contains(&fields[0]) && fields[1].len() == 6, "Ridley counter kind/address differs");
        let address = u32::from_str_radix(fields[1], 16)?;
        ensure!((0x800000..=0xffffff).contains(&address), "Ridley counter address is not canonical");
        let header = u16::from_str_radix(fields[2], 16)?;
        let slot = u16::from_str_radix(fields[3], 16)?;
        ensure!(
            [0xe13f, 0xe17f, 0xe1bf].contains(&header) && slot < 2048 && slot % 64 == 0,
            "Ridley counter header/slot differs"
        );
        for field in &fields[4..7] {
            u16::from_str_radix(field, 16)?;
        }
        let area = fields[7].parse::<u16>()?;
        let window = fields[8].parse::<u8>()?;
        ensure!(area < 8 && window < 3, "Ridley area/window differs");
        u16::from_str_radix(fields[9], 16)?;
        let count = fields[10].parse::<u64>()?;
        let first = fields[11].parse::<u64>()?;
        let last = fields[12].parse::<u64>()?;
        let retained = fields[13].parse::<usize>()?;
        ensure!(
            count > 0 && first <= last && last < frames && retained <= per_key && retained as u64 <= count,
            "Ridley counter bounds differ"
        );
        ensure!(keys.insert(fields[..10].join("\t")), "duplicate Ridley counter identity");
        rows += 1;
        hits += count;
        captures += retained;
    }
    ensure!(captures <= limit, "Ridley capture budget exceeded");
    Ok((rows, hits, captures))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Event {
    sequence: u64,
    boundary_id: u64,
    parent_id: u64,
    capture: u64,
    frame: u64,
    window: u8,
    kind: String,
    phase: String,
    depth: usize,
    address: u32,
    argument: u32,
    returned: u32,
    return_kind: u8,
    slot: u16,
    current_slot: u16,
    entry_current_slot: u16,
    entry_header: u16,
    entry_pc: u16,
    entry_state: u16,
    header: u16,
    bank: u8,
    pc: u16,
    timer: u16,
    state_zero: u16,
    state_slot: u16,
    tail_state: u16,
    baby_state: u16,
    orientation: u16,
    area: u16,
    health_zero: u16,
    health_slot: u16,
    ai_bits: u16,
    extra: u16,
    spritemap: u16,
    host: [u16; 8],
    ppu: [u8; 20],
}
fn word(ram: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([ram[offset], ram[offset + 1]])
}
fn validate_capture(root: &Path, event: &Event) -> Result<()> {
    let stem = format!("fixtures/ridley-{:010}", event.capture);
    let ram = fs::read(root.join(format!("{stem}-{}.wram", event.phase)))?;
    ensure!(ram.len() == WRAM, "Ridley capture is not complete WRAM");
    let host = fs::read(root.join(format!("{stem}-{}.host", event.phase)))?;
    ensure!(
        host == event.host.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>(),
        "Ridley captured host metadata differs"
    );
    ensure!(
        fs::read(root.join(format!("{stem}-{}.ppu", event.phase)))? == event.ppu,
        "Ridley captured PPU metadata differs"
    );
    let base = 0xf78 + usize::from(event.slot);
    ensure!(
        word(&ram, 0xfa8) == event.state_zero
            && word(&ram, 0xfa8 + usize::from(event.slot)) == event.state_slot
            && word(&ram, 0x2000) == event.tail_state
            && word(&ram, 0x8800) == event.baby_state
            && word(&ram, 0x7820) == event.orientation
            && word(&ram, base) == event.header
            && ram[base + 0x2e] == event.bank
            && word(&ram, base + 0x1c) == event.timer
            && word(&ram, base + 0x1a) == event.pc
            && word(&ram, base + 0x12) == event.ai_bits
            && word(&ram, base + 0x10) == event.extra
            && word(&ram, base + 0x16) == event.spritemap
            && word(&ram, base + 0x14) == event.health_slot
            && word(&ram, 0xf8c) == event.health_zero,
        "Ridley WRAM metadata differs from capture"
    );
    Ok(())
}
fn validate_service_file(path: &Path) -> Result<u64> {
    let mut file = fs::File::open(path)?;
    let length = file.metadata()?.len();
    ensure!(length % SERVICE_RECORD as u64 == 0, "partial Ridley service snapshot");
    let mut stack = Vec::new();
    let mut record = vec![0; SERVICE_RECORD];
    let mut calls = 0;
    for _ in 0..length / SERVICE_RECORD as u64 {
        file.read_exact(&mut record)?;
        let fields =
            std::array::from_fn::<_, 5, _>(|i| u32::from_le_bytes(record[i * 4..i * 4 + 4].try_into().unwrap()));
        ensure!(
            fields[0] <= 1 && fields[1] < 7 && fields[2] <= 65535 && fields[3] <= 65535,
            "invalid Ridley service record"
        );
        if fields[0] == 0 {
            ensure!(fields[4] == 0, "entry service already has a return");
            stack.push(fields[1..4].to_vec());
            calls += 1;
        } else {
            ensure!(stack.pop().as_deref() == Some(&fields[1..4]), "unbalanced C service transaction");
            ensure!(
                if matches!(fields[1], 0 | 6) { fields[4] <= 65535 } else { fields[4] == 0 },
                "native C service return extent differs"
            );
        }
    }
    ensure!(stack.is_empty(), "unfinished C service transaction");
    Ok(calls)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectEvent {
    sequence: u64,
    boundary_id: u64,
    root_boundary_id: u64,
    frame: u64,
    kind: String,
    address: u32,
    a: u16,
    b: u16,
    c: u16,
    value: u32,
}
fn effect_line(event: &EffectEvent) -> Result<Option<String>> {
    Ok(match event.kind.as_str() {
        "write" | "read" => {
            ensure!(event.address <= 0xffffff && event.value <= 255, "native hardware effect extent differs");
            Some(format!("{} {} {}\n", event.kind, event.address, event.value))
        }
        "sound" => {
            ensure!(event.a < 3 && [1, 3, 6, 9, 15].contains(&event.c), "native audio effect signature differs");
            Some(format!("sound {} {} {}\n", event.a, event.b, event.c))
        }
        "music" => Some(format!("music {} {}\n", event.a, event.b)),
        "projectile" => Some(format!("projectile {} {}\n", event.a, event.b)),
        "service" => {
            ensure!(event.address < 7, "native service effect kind differs");
            Some(format!("service {} {} {}\n", event.address, event.a, event.b))
        }
        "service-return" => {
            ensure!(event.address < 7, "native service return kind differs");
            None
        }
        _ => anyhow::bail!("unknown native Ridley ordered effect"),
    })
}
fn validate_ordered_effects(root: &Path, frames: u64, expected_hits: u64) -> Result<u64> {
    let boundaries = root.join("ridley-subsystem-events.jsonl");
    if expected_hits == 0 {
        return Ok(0);
    }
    let effects = root.join("ridley-subsystem-effects.jsonl");
    let mut boundary_lines = BufReader::new(fs::File::open(boundaries)?).lines();
    let mut effect_lines =
        if effects.is_file() { Some(BufReader::new(fs::File::open(effects)?).lines()) } else { None };
    let mut next_boundary =
        boundary_lines.next().transpose()?.map(|line| serde_json::from_str::<Event>(&line)).transpose()?;
    let mut next_effect = effect_lines
        .as_mut()
        .and_then(Iterator::next)
        .transpose()?
        .map(|line| serde_json::from_str::<EffectEvent>(&line))
        .transpose()?;
    let mut stack = Vec::<(u64, u64)>::new();
    let mut captured = BTreeMap::<u64, String>::new();
    let mut sequence = 0;
    let mut effects_count = 0;
    while next_boundary.is_some() || next_effect.is_some() {
        let is_boundary =
            next_boundary.as_ref().is_some_and(|b| next_effect.as_ref().is_none_or(|e| b.sequence < e.sequence));
        if is_boundary {
            let event = next_boundary.take().unwrap();
            ensure!(event.sequence == sequence + 1, "Ridley unified boundary/effect order gap");
            sequence = event.sequence;
            match event.phase.as_str() {
                "before" => {
                    stack.push((event.boundary_id, event.capture));
                    if event.capture > 0 {
                        captured.insert(event.capture, String::new());
                    }
                }
                "after" => {
                    ensure!(
                        stack.pop() == Some((event.boundary_id, event.capture)),
                        "Ridley effects boundary nesting differs"
                    );
                    if event.capture > 0 {
                        let expected = captured.remove(&event.capture).context("captured effect scope missing")?;
                        ensure!(
                            fs::read_to_string(
                                root.join(format!("fixtures/ridley-{:010}-effects.log", event.capture))
                            )? == expected,
                            "Ridley captured ordered effects differ from unified native events"
                        );
                    }
                }
                _ => anyhow::bail!("invalid native boundary phase"),
            }
            next_boundary =
                boundary_lines.next().transpose()?.map(|line| serde_json::from_str::<Event>(&line)).transpose()?;
        } else {
            let event = next_effect.take().unwrap();
            ensure!(
                event.sequence == sequence + 1
                    && event.frame < frames
                    && stack.last().is_some_and(|(id, _)| *id == event.boundary_id)
                    && stack.first().is_some_and(|(id, _)| *id == event.root_boundary_id),
                "Ridley effect identity or unified order differs"
            );
            sequence = event.sequence;
            effects_count += 1;
            if let Some(line) = effect_line(&event)? {
                for text in captured.values_mut() {
                    text.push_str(&line);
                }
            }
            next_effect = effect_lines
                .as_mut()
                .and_then(Iterator::next)
                .transpose()?
                .map(|line| serde_json::from_str::<EffectEvent>(&line))
                .transpose()?;
        }
    }
    ensure!(stack.is_empty() && captured.is_empty(), "unfinished Ridley ordered effect scopes");
    Ok(effects_count)
}
fn validate_events(root: &Path, frames: u64, expected_hits: u64, expected_captures: usize) -> Result<(usize, u64)> {
    let path = root.join("ridley-subsystem-events.jsonl");
    if expected_hits == 0 {
        ensure!(!path.exists() || fs::metadata(&path)?.len() == 0, "unaccounted Ridley events");
        return Ok((0, 0));
    }
    let mut stack: Vec<Event> = Vec::new();
    let mut captures = BTreeSet::new();
    let mut events = 0;
    let mut hits = 0;
    let mut sequence = 0;
    let mut service_calls = 0;
    for line in BufReader::new(fs::File::open(path)?).lines() {
        let event: Event = serde_json::from_str(&line?)?;
        ensure!(
            event.sequence > sequence
                && event.frame < frames
                && event.window < 3
                && KINDS.contains(&event.kind.as_str())
                && event.depth > 0
                && event.depth <= 64
                && event.return_kind <= 2
                && event.area < 8
                && event.slot < 2048
                && event.slot.is_multiple_of(64)
                && event.address <= 0xffffff
                && event.argument <= 0xffffff
                && event.returned <= 0xffffff,
            "invalid Ridley event identity"
        );
        sequence = event.sequence;
        events += 1;
        if event.capture != 0 {
            validate_capture(root, &event)?;
        }
        match event.phase.as_str() {
            "before" => {
                ensure!(
                    event.depth == stack.len() + 1
                        && event.parent_id == stack.last().map_or(0, |v| v.boundary_id)
                        && event.entry_current_slot == event.current_slot
                        && event.entry_header == event.header
                        && event.entry_pc == event.pc
                        && event.entry_state == event.state_zero
                        && event.returned == 0
                        && event.return_kind == 0,
                    "Ridley boundary entry mismatch"
                );
                if event.capture != 0 {
                    ensure!(captures.insert(event.capture), "duplicate Ridley capture identity");
                }
                stack.push(event);
                hits += 1;
            }
            "after" => {
                let before = stack.pop().context("Ridley return without entry")?;
                ensure!(
                    event.depth == stack.len() + 1
                        && event.boundary_id == before.boundary_id
                        && event.parent_id == before.parent_id
                        && event.capture == before.capture
                        && event.frame == before.frame
                        && event.window == before.window
                        && event.kind == before.kind
                        && event.address == before.address
                        && event.argument == before.argument
                        && event.slot == before.slot
                        && event.entry_current_slot == before.entry_current_slot
                        && event.entry_header == before.entry_header
                        && event.entry_pc == before.entry_pc
                        && event.entry_state == before.entry_state,
                    "Ridley boundary return mismatch"
                );
                if event.capture != 0 {
                    let stem = format!("fixtures/ridley-{:010}", event.capture);
                    service_calls += validate_service_file(&root.join(format!("{stem}-services.bin")))?;
                    ensure!(root.join(format!("{stem}-effects.log")).is_file(), "Ridley ordered effects missing");
                }
            }
            _ => anyhow::bail!("invalid Ridley phase"),
        }
    }
    ensure!(
        stack.is_empty() && hits == expected_hits && captures.len() == expected_captures,
        "Ridley events/counters disagree"
    );
    Ok((events, service_calls))
}

fn harness_sources(checkout: &Path) -> Result<BTreeMap<String, String>> {
    let mut hashes = BTreeMap::new();
    for (name, bytes) in [
        ("crates/pipeline/src/ridley_subsystem_inventory.rs", include_str!("ridley_subsystem_inventory.rs")),
        ("crates/pipeline/src/ridley_subsystem_inventory.c.in", SUPPORT),
        ("crates/pipeline/src/ridley_subsystem_inventory_hooks.c.in", HOOKS),
        ("crates/pipeline/src/enemy_instruction_inventory.rs", include_str!("enemy_instruction_inventory.rs")),
        ("crates/pipeline/src/enemy_instruction_inventory.c.in", include_str!("enemy_instruction_inventory.c.in")),
    ] {
        let hash = digest(&checkout.join(name))?;
        ensure!(
            hash == format!("{:x}", Sha256::digest(bytes.as_bytes())),
            "stale native Ridley inventory executable: {name}"
        );
        hashes.insert(name.into(), hash);
    }
    for name in ["tools/decomp/headless.c", "tools/decomp/headless_frame.h"] {
        hashes.insert(name.into(), digest(&checkout.join(name))?);
    }
    Ok(hashes)
}
fn terminal_progress(root: &Path, frames: u64) -> Result<()> {
    let progress = fs::read_to_string(root.join("replay.stdout"))?;
    let terminal = progress
        .lines()
        .rfind(|line| line.starts_with("progress "))
        .context("native Ridley inventory has no terminal progress")?
        .split_whitespace()
        .collect::<Vec<_>>();
    ensure!(
        terminal.len() == 10 && terminal[1].parse::<u64>()? == frames,
        "whole Ridley replay did not complete requested inputs"
    );
    ensure!(terminal[7] == "0" && terminal[8] == "0", "whole Ridley native inventory used CPU fallback");
    Ok(())
}

pub fn operate(value: Value) -> Result<Value> {
    let request: Request = value.try_into()?;
    ensure!(
        (1..=129_712).contains(&request.frames) && (1..=3600).contains(&request.timeout),
        "invalid native Ridley frame/timeout budget"
    );
    ensure!(request.capture_limit <= 8192 && request.captures_per_key <= 4, "invalid native Ridley capture budget");
    let checkout = fs::canonicalize(request.checkout.unwrap_or(std::env::current_dir()?))?;
    let source = fs::canonicalize(request.source.unwrap_or_else(|| checkout.join("deps/snesrev-sm")))?;
    let assets = fs::canonicalize(request.asset_root.unwrap_or_else(|| checkout.clone()))?;
    let output = request.output.unwrap_or_else(|| assets.join("out/port/ridley-subsystem/inventory"));
    fs::create_dir_all(&output)?;
    let run =
        output.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id()));
    fs::create_dir(&run)?;
    fs::create_dir(run.join("fixtures"))?;
    fs::create_dir(run.join("snes"))?;
    let run = fs::canonicalize(run)?;
    let started = Instant::now();
    let mut verdict = crate::report!({"schema_version":1,"status":"diagnostic-failed","accepted":false,
        "live_exercised":false,"frames":request.frames,"evidence":run,"prepare_only":request.prepare_only,
        "capture_limit":request.capture_limit,"captures_per_key":request.captures_per_key,
        "scope":"passive pinned actual-C whole Ridley lifecycle/AI/script/contact/death; no Rust acceptance or original-ROM parity claim",
        "capture_windows":["prefix<36000","first-combat36000..49999","later-combat>=50000"],
        "capture_contract":"full WRAM + eight little-endian host words + 20-byte PPU + ordered effects + actual C service entry/return snapshots",
        "retained_dependencies":["original-source","rom-view.bin","producer.rs","instrumentation.c.in","instrumentation-hooks.c.in","attack-producer.rs","attack-instrumentation.c.in"]});
    let result = (|| -> Result<()> {
        let identity = family::verify_source(&checkout, &source)?;
        for unit in family::c_units(&source)? {
            ensure!(
                identity.files.get(&unit.strip_prefix(&source)?.to_string_lossy().into_owned())
                    == Some(&digest(&unit)?),
                "Ridley native C unit outside pinned source identity"
            );
        }
        verdict.as_table_mut().unwrap().insert("source".into(), Value::try_from(&identity)?);
        let harness = harness_sources(&checkout)?;
        verdict.as_table_mut().unwrap().insert("harness_sources".into(), Value::try_from(&harness)?);
        let rom = assets.join("rom/sm.sfc");
        let inputs = assets.join("movies/super-metroid-any-3653.inputs");
        ensure!(
            digest(&rom)? == ROM_SHA256 && digest(&inputs)? == INPUT_SHA256,
            "whole Ridley inventory assets differ"
        );
        fs::copy(&rom, run.join("rom-view.bin"))?;
        snapshot(&source.join("src"), &run.join("original-source"))?;
        fs::copy(source.join("LICENSE.txt"), run.join("LICENSE.txt"))?;
        let originals = source_inputs(&source)?;
        let transformed = instrument(&originals, &fs::read_to_string(checkout.join("tools/decomp/headless.c"))?)?;
        for (name, contents) in &transformed {
            fs::write(run.join(name), contents)?;
        }
        for (name, contents) in [
            ("producer.rs", include_str!("ridley_subsystem_inventory.rs")),
            ("instrumentation.c.in", SUPPORT),
            ("instrumentation-hooks.c.in", HOOKS),
            ("attack-producer.rs", include_str!("enemy_instruction_inventory.rs")),
            ("attack-instrumentation.c.in", include_str!("enemy_instruction_inventory.c.in")),
        ] {
            fs::write(run.join(name), contents)?;
        }
        fs::copy(checkout.join("tools/decomp/headless_frame.h"), run.join("headless_frame.h"))?;
        let common = std::process::Command::new("git")
            .args(["rev-parse", "--git-common-dir"])
            .current_dir(&checkout)
            .output()?;
        ensure!(common.status.success(), "Git common directory missing");
        let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
        let shared = request
            .pipeline_root
            .clone()
            .or_else(|| std::env::var_os("SM_PIPELINE_ROOT").map(PathBuf::from))
            .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
        let workers =
            request.workers.unwrap_or(std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse()?);
        ensure!((1..=64).contains(&workers), "invalid worker budget");
        let command = build_command(&source, &run, request.capture_limit, request.captures_per_key)?;
        verdict.as_table_mut().unwrap().insert("build_command".into(), crate::report!(&command));
        verdict.as_table_mut().unwrap().insert(
            "build_cache".into(),
            Value::try_from(builds::c_build(builds::CBuildRequest {
                checkout: checkout.clone(),
                root: shared.clone(),
                workers,
                run: run.clone(),
                command,
            })?)?,
        );
        if request.observation_control {
            // The deliberate CPU fallback control is separate from the native replay and all capture paths.
            let directory = run.join("observation-control");
            fs::create_dir(&directory)?;
            fs::create_dir(directory.join("fixtures"))?;
            let control = vec![
                run.join("inventory").display().to_string(),
                "--observation-control".into(),
                rom.display().to_string(),
                directory.join("unused-state.bin").display().to_string(),
            ];
            builds::execute_timeout(&control, &directory, "control", &shared, workers, request.timeout.min(60))?;
            let stdout = fs::read_to_string(directory.join("control.stdout"))?;
            let fields = stdout
                .lines()
                .find(|line| line.starts_with("control "))
                .context("Ridley observation control missing")?
                .split_whitespace()
                .collect::<Vec<_>>();
            ensure!(
                fields.len() == 4 && fields[1].parse::<u64>()? > 0 && fields[2].parse::<u64>()? > 0 && fields[3] == "1",
                "native fallback/decompression observation control failed"
            );
            verdict.as_table_mut().unwrap().insert(
                "observation_control".into(),
                "actual C fallback/decompression detected in separate control process".into(),
            );
        }
        if !request.prepare_only {
            let replay = vec![
                run.join("inventory").display().to_string(),
                rom.display().to_string(),
                inputs.display().to_string(),
                request.frames.to_string(),
                run.join("terminal.bin").display().to_string(),
            ];
            verdict.as_table_mut().unwrap().insert("replay_command".into(), crate::report!(&replay));
            verdict.as_table_mut().unwrap().insert(
                "replay".into(),
                Value::try_from(builds::execute_timeout(&replay, &run, "replay", &shared, workers, request.timeout)?)?,
            );
            terminal_progress(&run, request.frames)?;
            let (rows, hits, captures) = validate_counters(
                &fs::read_to_string(run.join("ridley-subsystem.tsv"))?,
                request.frames,
                request.capture_limit,
                request.captures_per_key,
            )?;
            let (events, services) = validate_events(&run, request.frames, hits, captures)?;
            let effects = validate_ordered_effects(&run, request.frames, hits)?;
            verdict.as_table_mut().unwrap().insert("counter_rows".into(), crate::report!(rows));
            verdict.as_table_mut().unwrap().insert("boundary_entries".into(), crate::report!(hits));
            verdict.as_table_mut().unwrap().insert("retained_captures".into(), crate::report!(captures));
            verdict.as_table_mut().unwrap().insert("boundary_events".into(), crate::report!(events));
            verdict.as_table_mut().unwrap().insert("captured_service_calls".into(), crate::report!(services));
            verdict.as_table_mut().unwrap().insert("ordered_effects".into(), crate::report!(effects));
        }
        ensure!(
            family::verify_source(&checkout, &source)? == identity && harness_sources(&checkout)? == harness,
            "whole Ridley inventory source changed during execution"
        );
        ensure!(
            digest(&rom)? == ROM_SHA256 && digest(&inputs)? == INPUT_SHA256,
            "whole Ridley inventory assets changed during execution"
        );
        let replayed = !request.prepare_only;
        verdict.as_table_mut().unwrap().insert("rom_sha256".into(), ROM_SHA256.into());
        verdict.as_table_mut().unwrap().insert("inputs_sha256".into(), INPUT_SHA256.into());
        verdict
            .as_table_mut()
            .unwrap()
            .insert("status".into(), if replayed { "diagnostic-complete" } else { "diagnostic-prepared" }.into());
        Ok(())
    })();
    if let Err(error) = result {
        verdict.as_table_mut().unwrap().insert("error".into(), format!("{error:#}").into());
    }
    // Failed or interrupted executions seal exact partial observations as diagnostic evidence.
    let mut artifacts = BTreeMap::new();
    artifact_hashes(&run, &run, &mut artifacts)?;
    verdict.as_table_mut().unwrap().insert("artifacts".into(), crate::report!(artifacts));
    verdict.as_table_mut().unwrap().insert("seconds".into(), started.elapsed().as_secs_f64().into());
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    Ok(verdict)
}

/// Authenticate retained native observations without granting gameplay admission.
pub fn authenticate(checkout: &Path, path: &Path) -> Result<Value> {
    let checkout = fs::canonicalize(checkout)?;
    let run = fs::canonicalize(path.parent().context("Ridley verdict parent missing")?)?;
    let verdict: Value = toml::from_str(&fs::read_to_string(path)?)?;
    ensure!(
        matches!(verdict.get("status").and_then(Value::as_str), Some("diagnostic-complete" | "diagnostic-prepared")),
        "Ridley inventory did not complete"
    );
    let mut actual = BTreeMap::new();
    artifact_hashes(&run, &run, &mut actual)?;
    let expected: BTreeMap<String, String> =
        verdict.get("artifacts").context("Ridley inventory artifact seal missing")?.clone().try_into()?;
    ensure!(actual == expected, "Ridley inventory artifacts differ");
    let source = checkout.join("deps/snesrev-sm");
    let identity = family::verify_source(&checkout, &source)?;
    ensure!(
        Value::try_from(&identity)? == *verdict.get("source").context("Ridley source seal missing")?,
        "Ridley inventory source differs"
    );
    let harness: BTreeMap<String, String> =
        verdict.get("harness_sources").context("Ridley harness seal missing")?.clone().try_into()?;
    for (original, retained) in [
        ("crates/pipeline/src/ridley_subsystem_inventory.rs", "producer.rs"),
        ("crates/pipeline/src/ridley_subsystem_inventory.c.in", "instrumentation.c.in"),
        ("crates/pipeline/src/ridley_subsystem_inventory_hooks.c.in", "instrumentation-hooks.c.in"),
        ("crates/pipeline/src/enemy_instruction_inventory.rs", "attack-producer.rs"),
        ("crates/pipeline/src/enemy_instruction_inventory.c.in", "attack-instrumentation.c.in"),
    ] {
        ensure!(
            harness.get(original) == Some(&digest(&run.join(retained))?),
            "Ridley retained source seal differs: {original}"
        );
    }
    for original in ["tools/decomp/headless.c", "tools/decomp/headless_frame.h"] {
        ensure!(
            harness.get(original) == Some(&digest(&checkout.join(original))?),
            "Ridley native host source differs: {original}"
        );
    }
    for (name, hash) in &identity.files {
        if let Some(relative) = name.strip_prefix("src/") {
            ensure!(
                digest(&run.join("original-source").join(relative))? == *hash,
                "Ridley native source snapshot differs: {relative}"
            );
        }
    }
    ensure!(
        digest(&checkout.join("rom/sm.sfc"))? == ROM_SHA256 && digest(&run.join("rom-view.bin"))? == ROM_VIEW_SHA256,
        "Ridley retained native ROM view differs"
    );
    for (name, contents) in
        instrument(&source_inputs(&source)?, &fs::read_to_string(checkout.join("tools/decomp/headless.c"))?)?
    {
        ensure!(fs::read_to_string(run.join(&name))? == contents, "Ridley instrumented unit differs: {name}");
    }
    for (name, contents) in [
        ("instrumentation.c.in", SUPPORT),
        ("instrumentation-hooks.c.in", HOOKS),
        ("attack-instrumentation.c.in", include_str!("enemy_instruction_inventory.c.in")),
    ] {
        ensure!(fs::read_to_string(run.join(name))? == contents, "Ridley retained producer differs: {name}");
    }
    let frames = verdict.get("frames").and_then(Value::as_integer).context("Ridley frames missing")? as u64;
    let capture_limit =
        verdict.get("capture_limit").and_then(Value::as_integer).context("Ridley capture budget missing")? as usize;
    let per_key =
        verdict.get("captures_per_key").and_then(Value::as_integer).context("Ridley per-key budget missing")? as usize;
    if verdict.get("status").and_then(Value::as_str) == Some("diagnostic-complete") {
        terminal_progress(&run, frames)?;
        let (rows, hits, captures) =
            validate_counters(&fs::read_to_string(run.join("ridley-subsystem.tsv"))?, frames, capture_limit, per_key)?;
        let (events, services) = validate_events(&run, frames, hits, captures)?;
        let effects = validate_ordered_effects(&run, frames, hits)?;
        for (key, count) in [
            ("counter_rows", rows as u64),
            ("boundary_entries", hits),
            ("retained_captures", captures as u64),
            ("boundary_events", events as u64),
            ("captured_service_calls", services),
            ("ordered_effects", effects),
        ] {
            ensure!(
                verdict.get(key).and_then(Value::as_integer) == Some(i64::try_from(count)?),
                "Ridley authenticated count differs: {key}"
            );
        }
    }
    Ok(
        crate::report!({"status":"authenticated","evidence":run,"frames":frames,"accepted":false,"live_exercised":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counter_identity_preserves_header_slot_orientation_and_intervals() {
        let rows = "ai\ta6a288\te13f\t0000\ta923\t0000\t0000\t6\t1\ta923\t100\t36000\t36100\t2\nstate\ta6b227\te17f\t0040\tb3ec\t0000\t0002\t2\t2\tb3ec\t4\t70000\t70003\t1\n";
        assert_eq!(validate_counters(&format!("{HEADER}{rows}"), 129_712, 4, 2).unwrap(), (2, 104, 3));
        for change in [
            rows.replace("e13f", "e100"),
            rows.replace("0040", "0041"),
            rows.replace("70003", "129712"),
            rows.replace("\t2\n", "\t3\n"),
            format!("{rows}{rows}"),
        ] {
            assert!(validate_counters(&format!("{HEADER}{change}"), 129_712, 4, 2).is_err());
        }
    }
    #[test]
    fn definition_renaming_preserves_calls_and_rejects_ambiguous_sources() {
        let mut source = "void Ridley_Main(void) { Ridley_Main(); }\n".to_owned();
        rename_definition(&mut source, "void Ridley_Main(void)", "Ridley_Main", "Retained_Ridley_Main").unwrap();
        assert!(source.contains("Retained_Ridley_Main(void) { Ridley_Main();"));
        assert!(
            rename_definition(
                &mut "void Ridley_Main(void) {}\nvoid Ridley_Main(void) {}".into(),
                "void Ridley_Main(void)",
                "Ridley_Main",
                "Retained"
            )
            .is_err()
        );
    }
    #[test]
    fn one_native_command_replaces_only_owned_units_and_sets_capture_budget() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let source = root.join("deps/snesrev-sm");
        let run = Path::new("/tmp/ridley-inventory-command-test");
        let command = build_command(&source, run, 2048, 2).unwrap();
        assert!(command.contains(&"-DRIDLEY_SUBSYSTEM_CAPTURE_LIMIT=2048".into()));
        assert_eq!(command.iter().filter(|arg| arg.ends_with("sm_a6.c")).count(), 1);
        assert!(command.contains(&run.join("snes/snes.c").display().to_string()));
        assert!(!command.iter().any(|arg| arg.contains("--wrap=SpawnEnemy")));
    }
    #[test]
    fn composed_observer_retains_all_native_bodies_and_catches_same_unit_services() {
        let sources = BTreeMap::from([
            ("sm_a0.c".into(), include_str!("../../../deps/snesrev-sm/src/sm_a0.c").into()),
            ("sm_a6.c".into(), include_str!("../../../deps/snesrev-sm/src/sm_a6.c").into()),
            ("sm_86.c".into(), include_str!("../../../deps/snesrev-sm/src/sm_86.c").into()),
            ("sm_80.c".into(), include_str!("../../../deps/snesrev-sm/src/sm_80.c").into()),
            ("sm_90.c".into(), include_str!("../../../deps/snesrev-sm/src/sm_90.c").into()),
            ("snes/snes.c".into(), include_str!("../../../deps/snesrev-sm/src/snes/snes.c").into()),
        ]);
        let output = instrument(&sources, include_str!("../../../tools/decomp/headless.c")).unwrap();
        assert!(output["sm_a0.c"].contains("RidleyInventoryC_CallEnemyAi(ea);"));
        assert!(output["sm_a0.c"].contains("CInventory_ProcessEnemyInstructions();"));
        assert!(output["sm_a0.c"].contains("RidleyInventoryC_SpawnEnemy(db,k);"));
        assert!(output["sm_a0.c"].contains("CallEnemyAi(ED->bank << 16 | ED->ai_init);"));
        assert!(output["sm_a6.c"].contains("CallRidleyFunc_var40(Get_Ridley(0)->ridley_var_40 | 0xA60000);"));
        assert!(output["sm_90.c"].contains("RidleyInventoryC_Samus_SetPushedOutOfCeresRidley();"));
        assert!(output["headless.c"].contains("RidleySubsystemInventorySave();"));
        assert_eq!(output.len(), 7);
    }
}
