//! Standalone Rust producer for bank90 camera fixtures and actual-C observations.
//! Compile with `rustc --edition=2024 tools/decomp/camera_scroll_inventory.rs -o /tmp/camera-scroll-inventory`.
//! `fixtures PIPELINE OUTPUT.smf` publishes the synthetic family. `inventory CHECKOUT SOURCE ROM INPUTS PIPELINE OUTPUT FRAMES`
//! runs bounded actual C and preserves first/last full-WRAM observations per native branch.
mod fixture_output;
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
const ARCHIVE_SHA256: &str = "69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193";
const SYMBOLS: [&str; 4] = [
    "MainScrollingRoutine",
    "Samus_HandleScroll_X",
    "Samus_HandleScroll_Y",
    "Samus_ScrollFinishedHook_SporeSpawnFight",
];
const ADDRESSES: [u32; 4] = [0x9094ec, 0x9095a0, 0x90964f, 0x909589];

fn quoted(value: impl AsRef<str>) -> String {
    let mut result = String::from("\"");
    for c in value.as_ref().chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result.push('"');
    result
}
fn path_text(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| "non-UTF8 path".into())
}
fn hash(path: &Path) -> Result<String> {
    let result = Command::new("sha256sum").arg("--").arg(path).output()?;
    if !result.status.success() {
        return Err(format!("hash failed: {}", path.display()).into());
    }
    let output = String::from_utf8(result.stdout)?;
    let value = output.split_whitespace().next().ok_or("hash output missing")?;
    if value.len() != 64 || !value.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid SHA256 output".into());
    }
    Ok(value.to_owned())
}
fn source_files(root: &Path, directory: &Path, files: &mut BTreeMap<String, String>) -> Result<()> {
    let mut entries = fs::read_dir(directory)?.map(|v| v.map(|e| e.path())).collect::<std::io::Result<Vec<_>>>()?;
    entries.sort();
    for path in entries {
        if path.is_symlink() {
            return Err(format!("evidence symlink: {}", path.display()).into());
        }
        if path.is_dir() {
            source_files(root, &path, files)?;
        } else {
            files.insert(path_text(path.strip_prefix(root)?)?.to_owned(), hash(&path)?);
        }
    }
    Ok(())
}
fn table(files: &BTreeMap<String, String>) -> String {
    files.iter().map(|(k, v)| format!("{} = {}\n", quoted(k), quoted(v))).collect()
}
fn family_header(rom: &Path) -> Result<String> {
    Ok(format!(
        "family = \"camera-scroll\"\nrust_module = \"crates/port/src/camera_scroll.rs\"\nbank = \"sm_90.c\"\nc_support = [\"sm_80.c\", \"sm_rtl.c\", \"sm_cpu_infra.c\"]\nrom = {{ file = {}, sha256 = {} }}\nscope = \"Complete MainScrollingRoutine camera frame boundary; actual-C full-WRAM parity\"\n\n",
        quoted(path_text(rom)?),
        quoted(ROM_SHA256)
    ))
}
fn routine_header(index: usize) -> String {
    format!(
        "\n[[routines]]\nsymbol = {}\nargs = 0\narg_types = []\nreturn = \"void\"\noriginal_address = {}\nlive_eligible = true\n",
        quoted(SYMBOLS[index]),
        ADDRESSES[index]
    )
}
fn write_case(
    text: &mut String,
    name: &str,
    fill: u8,
    words: &BTreeMap<usize, u16>,
    bytes: &BTreeMap<usize, u8>,
    scroll: u8,
) {
    text.push_str(&format!(
        "\n[[routines.cases]]\nname = {}\nfill = {fill}\nexpected_return = 0\nwrites = [\n",
        quoted(name)
    ));
    for (offset, word) in words {
        let b = word.to_le_bytes();
        text.push_str(&format!("  {{ offset = {offset}, data = [{}, {}] }},\n", b[0], b[1]));
    }
    for (offset, byte) in bytes {
        text.push_str(&format!("  {{ offset = {offset}, data = [{byte}] }},\n"));
    }
    text.push_str(&format!(
        "  {{ offset = 52512, data = [{}] }},\n]\n",
        std::iter::repeat_n(scroll.to_string(), 256).collect::<Vec<_>>().join(", ")
    ));
}
fn defaults() -> BTreeMap<usize, u16> {
    [
        (0x7a9, 8),
        (0x7ab, 8),
        (0x7ad, 112),
        (0x7af, 144),
        (0x7e9, 0),
        (0x90f, 0x2000),
        (0x911, 512),
        (0x913, 0x9000),
        (0x915, 512),
        (0x941, 0),
        (0xa52, 0),
        (0xa78, 0),
        (0xaf6, 672),
        (0xaf8, 0x1000),
        (0xafa, 624),
        (0xafc, 0x2000),
        (0xb0a, 0x1234),
        (0xb0e, 0xabcd),
        (0xb10, 671),
        (0xb12, 0xf000),
        (0xb14, 623),
        (0xb16, 0x8000),
        (0xb36, 0),
        (0xb4a, 0),
        (0xcf8, 0),
        (0xda2, 3),
        (0xda4, 0x4000),
        (0xda6, 2),
        (0xda8, 0x3000),
    ]
    .into_iter()
    .collect()
}
fn fixtures(pipeline: &Path, output: &Path) -> Result<usize> {
    let mut text = family_header(Path::new("/home/jim/sm-decomp/rom/sm.sfc"))?;
    let mut count = 0;
    for symbol in 0..4 {
        text.push_str(&routine_header(symbol));
        let mut case = |name: String, words: BTreeMap<usize, u16>, bytes: BTreeMap<usize, u8>, scroll: u8| {
            write_case(&mut text, &name, [0, 0xa5, 0xff][count % 3], &words, &bytes, scroll);
            count += 1;
        };
        if symbol == 0 {
            for slow in [0, 1] {
                for frozen in [0, 1] {
                    for hook in [0, 0x9589] {
                        for scroll in [0, 1, 2] {
                            for motion in [-5_i32, 0, 5] {
                                let mut words = defaults();
                                words.extend([
                                    (0xcf8, slow),
                                    (0xa78, frozen),
                                    (0x7e9, hook),
                                    (0xb10, (672 - motion) as u16),
                                    (0xb14, (624 - motion) as u16),
                                ]);
                                case(
                                    format!(
                                        "frame-slow{slow}-frozen{frozen}-hook{hook:x}-scroll{scroll}-motion{motion}"
                                    ),
                                    words,
                                    [(0xa1e, 4), (0xa1f, 0)].into(),
                                    scroll,
                                );
                            }
                        }
                    }
                }
            }
            for x in [0, 0x5f, 0x60, 0x9f, 0xa0, 0xffff, 0x8000] {
                for y in [0x6f, 0x70, 0x8f, 0x90, 0x8000] {
                    let mut words = defaults();
                    words.extend([(0xcf8, 1), (0xa78, 1), (0x911, 0), (0x915, 0), (0xaf6, x), (0xafa, y)]);
                    case(format!("slow-native-window-x{x:x}-y{y:x}"), words, [(0xa1e, 4), (0xa1f, 0)].into(), 2);
                }
            }
            for (name, x, previous, frac, prevfrac) in [
                ("coordinate-wrap", 0, 0xffff, 0, 0xffff),
                ("signed-distance", 0x8000, 0, 0, 1),
                ("fraction-borrow", 674, 672, 0, 0xffff),
                ("fraction-only", 672, 672, 0x8000, 0),
            ] {
                let mut words = defaults();
                words.extend([
                    (0xaf6, x),
                    (0xb10, previous),
                    (0xaf8, frac),
                    (0xb12, prevfrac),
                    (0xafa, 624),
                    (0xb14, 624),
                ]);
                case(name.into(), words, [(0xa1e, 4), (0xa1f, 0)].into(), 2);
            }
            let mut words = defaults();
            words.extend([(0xaf6, 672), (0xb10, 672), (0xafa, 624), (0xb14, 624), (0x941, 0xffff)]);
            case("stationary-does-not-read-camera-table".into(), words, [(0xa1e, 4), (0xa1f, 0)].into(), 2);
        } else if symbol == 1 {
            for index in 0..8 {
                for direction in [4, 8] {
                    for movement in [0, 16] {
                        for knockback in [0, 2] {
                            for accel in [0, 1] {
                                for motion in [-1_i32, 1] {
                                    let mut words = defaults();
                                    words.extend([
                                        (0x941, index),
                                        (0xa52, knockback),
                                        (0xb4a, accel),
                                        (0xb10, (672 - motion) as u16),
                                        (0xaf6, if motion < 0 { 400 } else { 800 }),
                                    ]);
                                    case(
                                        format!(
                                            "x-index{index}-pose{direction}-move{movement}-knock{knockback}-accel{accel}-sign{motion}"
                                        ),
                                        words,
                                        [(0xa1e, direction), (0xa1f, movement)].into(),
                                        [0, 1, 2][usize::from(index) % 3],
                                    );
                                }
                            }
                        }
                    }
                }
            }
            for (name, x, position, distance, fract) in [
                ("x-ideal-equal", 672, 512, 3, 0x4000),
                ("x-underflow-clamp", 0, 0, 1, 0x8000),
                ("x-maximum-room-edge", 2048, 1792, 1, 0),
                ("x-signed-wrapped-ideal", 0x8000, 512, 1, 0),
                ("x-fractional-positive", 900, 512, 1, 0x4000),
                ("x-fractional-negative", 400, 512, 1, 0x4000),
            ] {
                let mut words = defaults();
                words.extend([(0xaf6, x), (0x911, position), (0xda2, distance), (0xda4, fract)]);
                case(name.into(), words, [(0xa1e, 4), (0xa1f, 0)].into(), 2);
            }
        } else if symbol == 2 {
            for dir in [0, 1, 2, 0xffff] {
                for motion in [-5_i32, 0, 5] {
                    for scroll in [0, 1, 2] {
                        for fraction in [0, 0xffff] {
                            let mut words = defaults();
                            words.extend([
                                (0xb36, dir),
                                (0xafa, (624 + motion) as u16),
                                (0xb14, 624),
                                (0xda8, fraction),
                            ]);
                            case(
                                format!("y-dir{dir}-motion{motion}-scroll{scroll}-fraction{fraction}"),
                                words,
                                [(0xa1e, 4), (0xa1f, 0)].into(),
                                scroll,
                            );
                        }
                    }
                }
            }
            for (name, y, layer, scroller) in
                [("y-up-underflow", 0, 0, 112), ("y-down-room-edge", 2048, 1792, 144), ("y-ideal-equal", 624, 512, 112)]
            {
                let mut words = defaults();
                words.extend([(0xafa, y), (0x915, layer), (0x7ad, scroller), (0xda6, 1), (0xda8, 0)]);
                case(name.into(), words, [(0xa1e, 4), (0xa1f, 0)].into(), 2);
            }
        } else {
            for y in [0, 463, 464, 465, 0x8000, 0xffff] {
                for fraction in [0, 0xbeef, 0xffff] {
                    let mut words = defaults();
                    words.extend([(0x915, y), (0x913, fraction)]);
                    case(format!("hook-y{y}-fraction{fraction}"), words, [(0xa1e, 4), (0xa1f, 0)].into(), 2);
                }
            }
        }
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fixture_output::write(pipeline, output, &text)?;
    Ok(count)
}

const C_SUPPORT: &str = r#"
#include <stdio.h>
#include <string.h>
static const char *camera_names[4]={"MainScrollingRoutine","Samus_HandleScroll_X","Samus_HandleScroll_Y","Samus_ScrollFinishedHook_SporeSpawnFight"};
static unsigned long camera_hits[4][16];
static int camera_first[4][16],camera_last[4][16];
static uint8 camera_last_before[4][16][sizeof(g_ram)],camera_last_after[4][16][sizeof(g_ram)];
static void CameraSaveState(const char *which,unsigned kind,unsigned path,const char *phase,const uint8 *ram) {
 char filename[128];snprintf(filename,sizeof(filename),"camera-%s-%u-%u-%s.bin",which,kind,path,phase);
 FILE *f=fopen(filename,"wb");if(!f||fwrite(ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fclose(f))exit(2);
}
static void CameraRecord(unsigned kind,unsigned path,const uint8 *before) {
 if(!camera_hits[kind][path]) {
  camera_first[kind][path]=snes_frame_counter;
  CameraSaveState("first",kind,path,"before",before);CameraSaveState("first",kind,path,"after",g_ram);
 }
 camera_hits[kind][path]++;camera_last[kind][path]=snes_frame_counter;
 memcpy(camera_last_before[kind][path],before,sizeof(g_ram));memcpy(camera_last_after[kind][path],g_ram,sizeof(g_ram));
}
void CameraInventorySave(void) {
 FILE *f=fopen("camera.tsv","w");if(!f)exit(2);
 fprintf(f,"symbol\tpath\thits\tfirst_frame\tlast_frame\n");
 for(unsigned kind=0;kind<4;kind++)for(unsigned path=0;path<16;path++)if(camera_hits[kind][path]) {
  fprintf(f,"%s\t%u\t%lu\t%d\t%d\n",camera_names[kind],path,camera_hits[kind][path],camera_first[kind][path],camera_last[kind][path]);
  CameraSaveState("last",kind,path,"before",camera_last_before[kind][path]);CameraSaveState("last",kind,path,"after",camera_last_after[kind][path]);
 }
 if(fclose(f))exit(2);
}
"#;
const C_WRAPPERS: &str = r#"
void MainScrollingRoutine(void) {
 uint8 before[sizeof(g_ram)];memcpy(before,g_ram,sizeof(g_ram));
 unsigned path=(slow_grabble_scrolling_flag?1:0)|(time_is_frozen_flag?2:0)|(scrolling_finished_hook?4:0);
 CameraOriginalMainScrollingRoutine();CameraRecord(0,path,before);
}
void Samus_HandleScroll_X(void) {
 uint8 before[sizeof(g_ram)];memcpy(before,g_ram,sizeof(g_ram));
 unsigned path;
 if(samus_prev_x_pos==samus_x_pos)path=0;
 else {
  const uint16 left[4]={0xa0,0x50,0x20,0xe0},right[4]={0x60,0x40,0x20,0xe0};
  unsigned index=camera_distance_index>>1;if(index>=4)exit(2);
  bool facing=(knockback_dir||samus_movement_type==16||samus_x_accel_mode==1)^(samus_pose_x_dir!=4);
  uint16 ideal=samus_x_pos-(facing?right[index]:left[index]);
  path=ideal==layer1_x_pos?1:((int16)(ideal-layer1_x_pos)<0?2:3);
 }
 CameraOriginalSamus_HandleScroll_X();CameraRecord(1,path,before);
}
void Samus_HandleScroll_Y(void) {
 uint8 before[sizeof(g_ram)];memcpy(before,g_ram,sizeof(g_ram));
 uint16 ideal=samus_y_pos-(samus_y_dir==1?down_scroller:up_scroller);
 unsigned path=samus_prev_y_pos==samus_y_pos?0:(ideal==layer1_y_pos?1:((int16)(ideal-layer1_y_pos)<0?2:3));
 CameraOriginalSamus_HandleScroll_Y();CameraRecord(2,path,before);
}
void Samus_ScrollFinishedHook_SporeSpawnFight(void) {
 uint8 before[sizeof(g_ram)];memcpy(before,g_ram,sizeof(g_ram));unsigned path=layer1_y_pos<=0x1d0?1:0;
 CameraOriginalSamus_ScrollFinishedHook_SporeSpawnFight();CameraRecord(3,path,before);
}
"#;
fn replace_once(text: &mut String, old: &str, new: &str) -> Result<()> {
    if text.matches(old).count() != 1 {
        return Err(format!("instrumentation anchor must be unique: {old}").into());
    }
    *text = text.replace(old, new);
    Ok(())
}
fn instrument(source: &str, host: &str) -> Result<(String, String)> {
    let mut source = source.to_owned();
    for symbol in SYMBOLS {
        replace_once(
            &mut source,
            &format!("void {symbol}(void) {{"),
            &format!("void CameraOriginal{symbol}(void) {{"),
        )?;
    }
    let anchor = "void CallScrollingFinishedHook(uint32 ea) {";
    replace_once(&mut source, anchor, &format!("{C_SUPPORT}\n{anchor}"))?;
    source.push_str(C_WRAPPERS);
    let mut host = host.to_owned();
    let start = host.find("static void SaveRoutineState").ok_or("routine capture anchor missing")?;
    let end = start + host[start..].find("// Linker wrapping").ok_or("linker anchor missing")?;
    host.replace_range(start..end, "");
    let start = host.find("void __wrap_DecompressToMem").ok_or("decompress wrapper missing")?;
    let end = start + host[start..].find("// The host has no audio thread").ok_or("audio anchor missing")?;
    host.replace_range(start..end,"void __wrap_DecompressToMem(uint32 src,uint8 *dst) { decompression_calls++;__real_DecompressToMem(src,dst); }\nextern void CameraInventorySave(void);\n");
    replace_once(
        &mut host,
        "  fclose(inputs);\n  return 0;",
        "  fclose(inputs);\n  CameraInventorySave();\n  return 0;",
    )?;
    Ok((source, host))
}
fn pipeline_request(pipeline: &Path, operation: &str, request: &str, output: &Path) -> Result<()> {
    fs::write(output.with_extension("request.toml"), request)?;
    let mut child = Command::new(pipeline)
        .arg(operation)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("pipeline stdin missing")?.write_all(request.as_bytes())?;
    let result = child.wait_with_output()?;
    fs::write(output, &result.stdout)?;
    fs::write(output.with_extension("stderr.txt"), &result.stderr)?;
    if !result.status.success() {
        return Err(format!("pipeline {operation} failed; see {}", output.display()).into());
    }
    Ok(())
}
fn command_toml(command: &[String]) -> String {
    format!("[{}]", command.iter().map(quoted).collect::<Vec<_>>().join(", "))
}
fn natural_family(run: &Path, rom: &Path, frames: u64, pipeline: &Path, output: &Path) -> Result<usize> {
    let counters = fs::read_to_string(run.join("camera.tsv"))?;
    if !counters.starts_with("symbol\tpath\thits\tfirst_frame\tlast_frame\n") {
        return Err("camera counter header differs".into());
    }
    let mut cases: [Vec<(String, PathBuf, PathBuf)>; 4] = Default::default();
    for line in counters.lines().skip(1) {
        let c = line.split('\t').collect::<Vec<_>>();
        if c.len() != 5 {
            return Err("malformed camera counter".into());
        }
        let kind = SYMBOLS.iter().position(|v| *v == c[0]).ok_or("unknown camera symbol")?;
        let path: u32 = c[1].parse()?;
        let hits: u64 = c[2].parse()?;
        let first: u64 = c[3].parse()?;
        let last: u64 = c[4].parse()?;
        if path >= 16 || hits == 0 || first > last || last >= frames {
            return Err("camera counter bounds differ".into());
        }
        for (which, frame) in [("first", first), ("last", last)] {
            let before = run.join(format!("camera-{which}-{kind}-{path}-before.bin"));
            let after = run.join(format!("camera-{which}-{kind}-{path}-after.bin"));
            if fs::metadata(&before)?.len() != 0x20000 || fs::metadata(&after)?.len() != 0x20000 {
                return Err("camera snapshot is not full WRAM".into());
            }
            cases[kind].push((format!("natural-kind{kind}-{which}-path{path}-frame{frame}"), before, after));
        }
    }
    let mut text = family_header(rom)?;
    let mut count = 0;
    for (kind, cases) in cases.iter().enumerate() {
        if cases.is_empty() {
            continue;
        }
        text.push_str(&routine_header(kind));
        for (name, before, after) in cases {
            text.push_str(&format!("\n[[routines.cases]]\nname = {}\nexpected_return = 0\nbefore_file = {}\nbefore_sha256 = {}\nexpected_after_file = {}\nexpected_after_sha256 = {}\n",quoted(name),quoted(path_text(before)?),quoted(hash(before)?),quoted(path_text(after)?),quoted(hash(after)?)));
            count += 1;
        }
    }
    fixture_output::write(pipeline, output, &text)?;
    Ok(count)
}
fn inventory(args: &[String]) -> Result<()> {
    if args.len() != 7 {
        return Err("inventory CHECKOUT SOURCE ROM INPUTS PIPELINE OUTPUT FRAMES".into());
    }
    let checkout = fs::canonicalize(&args[0])?;
    let source = fs::canonicalize(&args[1])?;
    let rom = fs::canonicalize(&args[2])?;
    let inputs = fs::canonicalize(&args[3])?;
    let pipeline = fs::canonicalize(&args[4])?;
    let output = PathBuf::from(&args[5]);
    let frames: u64 = args[6].parse()?;
    if !(1..=129712).contains(&frames) {
        return Err("frames must be 1..129712".into());
    }
    if hash(&rom)? != ROM_SHA256 {
        return Err("unsupported ROM identity".into());
    }
    let archive = source.parent().ok_or("source parent missing")?.join("downloads/snesrev-sm.tar.gz");
    if hash(&archive)? != ARCHIVE_SHA256 {
        return Err("pinned archive differs".into());
    }
    fs::create_dir_all(&output)?;
    let run =
        output.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id()));
    fs::create_dir(&run)?;
    let run = fs::canonicalize(run)?;
    let started = Instant::now();
    let mut status = "diagnostic-failed";
    let mut failure = String::new();
    let mut natural_cases = 0;
    let mut identities = BTreeMap::new();
    identities.insert("rom".into(), hash(&rom)?);
    identities.insert("movie".into(), hash(&inputs)?);
    identities.insert("pipeline".into(), hash(&pipeline)?);
    identities.insert("producer".into(), hash(&checkout.join("tools/decomp/camera_scroll_inventory.rs"))?);
    identities.insert("fixture_output".into(), hash(&checkout.join("tools/decomp/fixture_output.rs"))?);
    identities.insert("host".into(), hash(&checkout.join("tools/decomp/headless.c"))?);
    identities.insert("frame_header".into(), hash(&checkout.join("tools/decomp/headless_frame.h"))?);
    let result = (|| -> Result<()> {
        fs::copy(checkout.join("tools/decomp/camera_scroll_inventory.rs"), run.join("producer.rs"))?;
        fs::copy(checkout.join("tools/decomp/fixture_output.rs"), run.join("fixture_output.rs"))?;
        let original = run.join("original-source");
        fs::create_dir(&original)?;
        let extraction = Command::new("tar")
            .args(["-xzf"])
            .arg(&archive)
            .args(["--strip-components=1", "-C"])
            .arg(&original)
            .status()?;
        if !extraction.success() {
            return Err("pinned source extraction failed".into());
        }
        let mut pinned = BTreeMap::new();
        source_files(&original, &original.join("src"), &mut pinned)?;
        for (relative, expected) in &pinned {
            if hash(&source.join(relative))? != *expected {
                return Err(format!("pinned C source changed: {relative}").into());
            }
        }
        fs::write(run.join("source-inputs.toml"), table(&pinned))?;
        fs::copy(original.join("LICENSE.txt"), run.join("LICENSE.txt"))?;
        let (bank, host) = instrument(
            &fs::read_to_string(source.join("src/sm_90.c"))?,
            &fs::read_to_string(checkout.join("tools/decomp/headless.c"))?,
        )?;
        fs::write(run.join("sm_90.c"), bank)?;
        fs::write(run.join("headless.c"), host)?;
        fs::copy(checkout.join("tools/decomp/headless_frame.h"), run.join("headless_frame.h"))?;
        let src = source.join("src");
        let mut units = pinned
            .keys()
            .filter(|p| {
                p.ends_with(".c")
                    && (p.starts_with("src/snes/")
                        || Path::new(p).file_stem().is_some_and(|n| {
                            n.to_string_lossy().starts_with("sm_")
                                || ["spc_player", "tracing", "util"].contains(&n.to_string_lossy().as_ref())
                        }))
            })
            .map(|p| source.join(p))
            .collect::<Vec<_>>();
        units.sort();
        let mut command = vec![
            "cc".into(),
            "-O2".into(),
            "-fno-strict-aliasing".into(),
            "-ffunction-sections".into(),
            "-fdata-sections".into(),
            "-iquote".into(),
            path_text(&src)?.into(),
        ];
        for unit in units {
            let unit = if unit.file_name().is_some_and(|n| n == "sm_90.c") { run.join("sm_90.c") } else { unit };
            command.push(path_text(&unit)?.into());
        }
        command.extend([
            path_text(&run.join("headless.c"))?.into(),
            "-Wl,--gc-sections".into(),
            "-Wl,--wrap=Call".into(),
            "-Wl,--wrap=cpu_runOpcode".into(),
            "-Wl,--wrap=DecompressToMem".into(),
            "-lm".into(),
            "-o".into(),
            path_text(&run.join("inventory"))?.into(),
        ]);
        let root = std::env::var("SM_PIPELINE_ROOT").unwrap_or_else(|_| "/home/jim/sm-decomp/out/pipeline".into());
        let workers: usize = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse()?;
        let build = format!(
            "checkout = {}\nroot = {}\nworkers = {workers}\nrun = {}\ncommand = {}\n",
            quoted(path_text(&checkout)?),
            quoted(&root),
            quoted(path_text(&run)?),
            command_toml(&command)
        );
        pipeline_request(&pipeline, "c-build", &build, &run.join("build.toml"))?;
        println!("camera C build finished; run={}", run.display());
        let command = vec![
            path_text(&run.join("inventory"))?.into(),
            path_text(&rom)?.into(),
            path_text(&inputs)?.into(),
            frames.to_string(),
            path_text(&run.join("terminal.bin"))?.into(),
        ];
        let replay = format!(
            "root = {}\nworkers = {workers}\nrun = {}\nlabel = \"replay\"\ntimeout = 60\ncommand = {}\n",
            quoted(&root),
            quoted(path_text(&run)?),
            command_toml(&command)
        );
        pipeline_request(&pipeline, "exec", &replay, &run.join("execute.toml"))?;
        if hash(&rom)? != identities["rom"]
            || hash(&inputs)? != identities["movie"]
            || hash(&pipeline)? != identities["pipeline"]
            || hash(&checkout.join("tools/decomp/camera_scroll_inventory.rs"))? != identities["producer"]
            || hash(&checkout.join("tools/decomp/fixture_output.rs"))? != identities["fixture_output"]
            || hash(&checkout.join("tools/decomp/headless.c"))? != identities["host"]
            || hash(&checkout.join("tools/decomp/headless_frame.h"))? != identities["frame_header"]
        {
            return Err("diagnostic inputs changed during execution".into());
        }
        natural_cases = natural_family(&run, &rom, frames, &pipeline, &run.join("camera-natural.smf"))?;
        if natural_cases == 0 {
            return Err("camera boundary not exercised".into());
        }
        Ok(())
    })();
    if let Err(e) = result {
        failure = e.to_string();
    } else {
        status = "diagnostic-passed";
    }
    let mut artifacts = BTreeMap::new();
    source_files(&run, &run, &mut artifacts)?;
    let verdict = format!(
        "schema_version = 1\nstatus = {}\naccepted = false\nframes = {frames}\nnatural_cases = {natural_cases}\nseconds = {}\nerror = {}\nscope = \"Actual-C first/last full-WRAM camera observations; no Rust admission claim\"\n[inputs]\n{}\n[artifacts]\n{}",
        quoted(status),
        started.elapsed().as_secs_f64(),
        quoted(&failure),
        table(&identities),
        table(&artifacts)
    );
    fs::write(run.join("verdict.toml"), verdict)?;
    println!("camera inventory {status}: {}", run.join("verdict.toml").display());
    if status != "diagnostic-passed" {
        return Err(failure.into());
    }
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str){
        Some("fixtures") if args.len()==3=>{let cases=fixtures(Path::new(&args[1]),Path::new(&args[2]))?;println!("camera synthetic fixtures: {cases}");Ok(())},
        Some("inventory")=>inventory(&args[1..]),
        Some("natural") if args.len() == 5 => {
            let run = fs::canonicalize(&args[1])?;
            let cases = natural_family(&run, Path::new("/home/jim/sm-decomp/rom/sm.sfc"), args[4].parse()?, Path::new(&args[2]), Path::new(&args[3]))?;
            println!("camera natural fixtures: {cases}");
            Ok(())
        },
        _=>Err("usage: camera-scroll-inventory fixtures PIPELINE OUTPUT.smf | inventory CHECKOUT SOURCE ROM INPUTS PIPELINE OUTPUT FRAMES | natural RUN PIPELINE OUTPUT.smf FRAMES".into()),
    }
}
