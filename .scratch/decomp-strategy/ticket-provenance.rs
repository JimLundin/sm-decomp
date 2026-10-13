use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
const ROOT: &str = "/home/jim/sm-decomp";
fn parse(path: &Path) -> Option<Value> {
    let s = fs::read_to_string(path).ok()?;
    if path.extension()?.to_str()? == "toml" {
        let v: toml::Value = toml::from_str(&s).ok()?;
        serde_json::to_value(v).ok()
    } else {
        serde_json::from_str(&s).ok()
    }
}
fn tokens(s: &str) -> BTreeSet<String> {
    s.split(|c: char| c.is_whitespace() || "`\"'()[]{}<>|,;".contains(c))
        .map(|s| s.trim_end_matches(['.', ':']))
        .filter(|s| {
            ["out/", ".scratch/", "../", "/tmp/", "/home/jim/sm-decomp/", "crates/port/", "tools/port/"]
                .iter()
                .any(|p| s.starts_with(p))
                && !["out", "out/", "out/port", "out/port/", "tools/port", "tools/port/", "/home/jim/sm-decomp/"]
                    .contains(s)
                && s.len() < 400
                && !s.contains('*')
                && !s.contains("...")
        })
        .map(str::to_owned)
        .collect()
}
fn resolved(s: &str, base: &Path) -> PathBuf {
    if s.starts_with('/') {
        PathBuf::from(s)
    } else if s.starts_with("../") {
        base.join(s)
    } else {
        Path::new(ROOT).join(s)
    }
}
fn pathkind(p: &Path) -> &'static str {
    let n = p.to_string_lossy();
    if n.contains("authentication") || n.contains("authenticate") || n.contains("-auth.") {
        "authentication"
    } else if n.contains("cleanup") || n.contains("dedup") {
        "cleanup"
    } else if n.contains("verdict") {
        "verdict"
    } else if n.contains("manifest") || n.contains("tools/port/") {
        "manifest"
    } else if n.contains("capture") || n.contains("inspection") {
        "capture-or-inspection"
    } else if n.ends_with(".rs") {
        "rust-source"
    } else if n.ends_with(".md") {
        "scope-or-handoff"
    } else {
        "retained-path"
    }
}
fn strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => {
            for x in a {
                strings(x, out)
            }
        }
        Value::Object(o) => {
            for x in o.values() {
                strings(x, out)
            }
        }
        _ => {}
    }
}
fn thin(v: Value) -> Value {
    let mut all = vec![];
    strings(&v, &mut all);
    let mut o = serde_json::Map::new();
    o.insert("audit_document_paths".into(), json!(all.into_iter().flat_map(|s| tokens(&s)).collect::<BTreeSet<_>>()));
    for k in ["status", "source", "implementation", "hits", "dispatcher_hits", "counters", "bank", "rom", "rust_module"]
    {
        if let Some(x) = v.get(k) {
            o.insert(k.into(), x.clone());
        }
    }
    for k in ["routines", "cases"] {
        if let Some(a) = v[k].as_array() {
            o.insert(
                k.into(),
                Value::Array(
                    a.iter()
                        .map(|x| {
                            let mut item = serde_json::Map::new();
                            for key in ["symbol", "routine", "name", "a", "b", "c", "j"] {
                                if let Some(y) = x.get(key) {
                                    item.insert(key.into(), y.clone());
                                }
                            }
                            if let Some(cs) = x["cases"].as_array() {
                                item.insert(
                                    "cases".into(),
                                    Value::Array(
                                        cs.iter()
                                            .map(|c| {
                                                let mut case = serde_json::Map::new();
                                                for key in ["name", "routine", "a", "b", "c", "j", "rom_writes"] {
                                                    if let Some(y) = c.get(key) {
                                                        case.insert(key.into(), y.clone());
                                                    }
                                                }
                                                Value::Object(case)
                                            })
                                            .collect(),
                                    ),
                                );
                            }
                            Value::Object(item)
                        })
                        .collect(),
                ),
            );
        }
    }
    Value::Object(o)
}
fn codewords(s: &str) -> BTreeSet<String> {
    s.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).filter(|s| !s.is_empty()).map(str::to_owned).collect()
}
fn hexcommits(s: &str) -> BTreeSet<String> {
    codewords(s)
        .into_iter()
        .filter(|s| {
            (7..=40).contains(&s.len())
                && s.bytes().all(|b| b.is_ascii_hexdigit())
                && s.bytes().any(|b| b.is_ascii_digit())
        })
        .filter(|s| {
            Command::new("git")
                .args(["cat-file", "-e", &format!("{s}^{{commit}}")])
                .current_dir(ROOT)
                .output()
                .is_ok_and(|o| o.status.success())
        })
        .collect()
}
fn digest(p: &Path) -> Option<String> {
    let o = Command::new("sha256sum").arg(p).output().ok()?;
    if o.status.success() { Some(String::from_utf8(o.stdout).ok()?[..64].to_string()) } else { None }
}
fn rom_word(rom: &[u8], address: u32, case: &Value) -> Option<u16> {
    let at = (((address >> 16) & 0x7f) * 0x8000 + (address & 0x7fff)) as usize;
    let mut word = rom.get(at..at + 2)?.to_vec();
    for w in case["rom_writes"].as_array().into_iter().flatten() {
        let offset = w["offset"].as_u64()? as usize;
        for (i, b) in w["data"].as_array()?.iter().enumerate() {
            if (at..at + 2).contains(&(offset + i)) {
                word[offset + i - at] = b.as_u64()? as u8;
            }
        }
    }
    Some(u16::from_le_bytes(word.try_into().ok()?))
}
fn document_paths(path: &Path, v: &Value) -> Vec<PathBuf> {
    let mut ss = vec![];
    strings(v, &mut ss);
    ss.into_iter()
        .filter(|s| s.len() < 400)
        .flat_map(|s| tokens(&s))
        .map(|s| resolved(&s, path.parent().unwrap()))
        .filter(|p| p != path)
        .collect()
}
fn verdicts_in(path: &Path, out: &mut Vec<PathBuf>, depth: u8) {
    if depth == 0 || path.is_symlink() {
        return;
    }
    let Ok(es) = fs::read_dir(path) else { return };
    for e in es.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n != "target" && n != "store" && n != "fixtures" && n != ".git") {
                verdicts_in(&p, out, depth - 1)
            }
        } else if p
            .file_name()
            .is_some_and(|n| n == "verdict.json" || n == "verdict.toml" || n == "batch.json" || n == "batch.toml")
        {
            out.push(p)
        }
    }
}
fn main() {
    let root = Path::new(ROOT);
    let registry = parse(&root.join(".scratch/decomp-progress/registry.json")).unwrap();
    let index = parse(&root.join(".scratch/decomp-strategy/retained-coverage-inventory.json")).unwrap();
    let source = parse(&root.join(".scratch/decomp-strategy/source-coverage-map.json")).unwrap();
    let authentication = parse(&root.join(".scratch/decomp-strategy/retained-coverage-authentication.json")).unwrap();
    let authenticated = authentication["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["artifact_hashes_match"] == true)
        .filter_map(|r| r["path"].as_str())
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let mut native = BTreeMap::<String, Value>::new();
    let mut by_address = BTreeMap::<u32, String>::new();
    let mut noops = BTreeSet::new();
    for r in registry["routines"].as_array().unwrap() {
        let s = r["symbol"].as_str().unwrap().to_owned();
        if let Some(a) =
            r["original_address"].as_str().and_then(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        {
            by_address.insert(a, s.clone());
        }
        let file = root.join("deps/snesrev-sm").join(r["source"].as_str().unwrap());
        let body = fs::read_to_string(file).unwrap();
        let tail = body
            .lines()
            .skip(r["line"].as_u64().unwrap() as usize - 1)
            .take(r["body_lines"].as_u64().unwrap() as usize + 2)
            .map(|l| l.split("//").next().unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        if let Some(at) = tail.find('{') {
            let after = &tail[at + 1..];
            if let Some(end) = after.find('}') {
                let b = after[..end].trim();
                if b.is_empty() || b == "return;" {
                    noops.insert(s.clone());
                }
            }
        }
        native.insert(s, r.clone());
    }
    for l in fs::read_to_string(root.join("deps/snesrev-sm/src/funcs.h")).unwrap().lines() {
        let a = l.split_whitespace().collect::<Vec<_>>();
        if a.len() >= 3 && a[0] == "#define" {
            if let Some(s) = a[1].strip_prefix("fn") {
                if native.contains_key(s) {
                    if let Ok(address) = u32::from_str_radix(a[2].trim_start_matches("0x"), 16) {
                        by_address.insert(address, s.into());
                    }
                }
            }
        }
    }
    let mut current_sources = BTreeMap::<String, Vec<String>>::new();
    for m in source["modules"].as_array().unwrap() {
        current_sources.insert(
            m["path"].as_str().unwrap().into(),
            m["native_symbols"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|r| r["symbol"].as_str())
                .map(str::to_owned)
                .collect(),
        );
    }
    let base_rom = fs::read(root.join("rom/sm.sfc")).unwrap();
    let mut manifest_cache = BTreeMap::<PathBuf, Value>::new();
    let mut live_cache = BTreeMap::<PathBuf, Vec<String>>::new();
    let mut output = vec![];
    let mut paths =
        fs::read_dir(root.join(".scratch/decomp-port/issues")).unwrap().flatten().map(|e| e.path()).collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        let body = fs::read_to_string(&path).unwrap();
        let id = path.file_name().unwrap().to_str().unwrap().split('-').next().unwrap().parse::<u32>().unwrap();
        let status = body.lines().find_map(|l| l.strip_prefix("Status:").map(str::trim)).unwrap_or("unknown");
        let complete = ["resolved", "done"].contains(&status);
        let infra = [1, 7, 10, 11, 76, 78, 88, 89, 127].contains(&id);
        let scope = body.lines().find_map(|l| l.strip_prefix("# ")).unwrap_or("");
        let mut evidence = BTreeMap::<String, Value>::new();
        let mut ptrs = BTreeSet::<PathBuf>::new();
        let mut pending =
            tokens(&body).into_iter().map(|t| (resolved(&t, path.parent().unwrap()), 0u8)).collect::<Vec<_>>();
        while let Some((p, depth)) = pending.pop() {
            if !ptrs.insert(p.clone()) {
                continue;
            }
            let s = p.to_string_lossy().to_string();
            evidence.insert(s.clone(), json!({"path":s,"exists":p.exists(),"kind":pathkind(&p)}));
            if depth < 2
                && p.extension().is_some_and(|e| ["md", "json", "toml"].contains(&e.to_str().unwrap_or("")))
                && pathkind(&p) != "verdict"
                && pathkind(&p) != "manifest"
            {
                if let Ok(t) = fs::read_to_string(&p) {
                    if t.len() < 100000 {
                        pending.extend(tokens(&t).into_iter().map(|t| (resolved(&t, p.parent().unwrap()), depth + 1)));
                        if let Some(v) = parse(&p) {
                            if let Some(r) = v["run"].as_str() {
                                pending.push((PathBuf::from(r).join("verdict.toml"), depth + 1));
                            }
                        };
                    }
                }
            }
        }
        let mut mentioned_modules = ptrs
            .iter()
            .filter_map(|p| p.strip_prefix(root).ok())
            .map(|p| p.to_string_lossy().to_string())
            .filter(|p| p.starts_with("crates/port/src/") && p.ends_with(".rs"))
            .collect::<BTreeSet<_>>();
        let owned: Vec<&str> = match id {
            2 => vec!["system"],
            3 => vec!["audio"],
            5 => vec!["samus"],
            6 => vec!["enemy"],
            8 => vec!["samus_control"],
            12 => vec!["collision"],
            13 => vec!["plm"],
            14 => vec!["projectiles"],
            15 => vec!["enemy_ai"],
            16 => vec!["projectile_flight"],
            17 => vec!["enemy_rio"],
            18 => vec!["enemy_jump", "enemy_jump_instructions"],
            19 => vec!["projectile_special"],
            20 => vec!["room"],
            21 => vec!["plm_instructions", "plm_render"],
            22 => vec!["enemy_crawler"],
            23 => vec!["power_bomb"],
            133 => vec!["enemy_projectile_rendering"],
            134 => vec!["enemy_projectile_contacts"],
            135 => vec!["enemy_projectile_sparks"],
            140 => vec!["camera_scroll"],
            _ => vec![],
        };
        for m in &owned {
            mentioned_modules.insert(format!("crates/port/src/{m}.rs"));
        }
        let directly = index["verdicts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| {
                let p = root.join(v["path"].as_str().unwrap());
                ptrs.iter().any(|q| p == *q || (q.is_dir() && p.starts_with(q)))
            })
            .collect::<Vec<_>>();
        let fallback = directly.iter().all(|v| !v["eligible_for_artifact_check"].as_bool().unwrap_or(false));
        let mut selected = directly;
        if fallback && !infra {
            selected.extend(index["verdicts"].as_array().unwrap().iter().filter(|v| {
                [Some("offline-verified"), Some("mixed-C-Rust-integration"), Some("accepted")]
                    .contains(&v["status"].as_str())
                    && v["module"].as_str().is_some_and(|m| mentioned_modules.contains(m))
            }));
        }
        selected.sort_by_key(|v| v["path"].as_str().unwrap());
        selected.dedup_by_key(|v| v["path"].as_str().unwrap());
        let mut routines = BTreeSet::new();
        let mut verified = BTreeSet::new();
        let mut live = BTreeSet::new();
        let mut mappings = vec![];
        let mut identities = vec![];
        let mut passing = vec![];
        let mut unresolved = BTreeSet::new();
        let mut omitted = BTreeSet::new();
        let mut commits = hexcommits(&body);
        let mut review = vec![];
        for v in &selected {
            let p = root.join(v["path"].as_str().unwrap());
            evidence.insert(
                p.to_string_lossy().to_string(),
                json!({"path":p,"exists":p.exists(),"kind":"verdict","status":v["status"]}),
            );
            if ![Some("offline-verified"), Some("mixed-C-Rust-integration"), Some("accepted")]
                .contains(&v["status"].as_str())
            {
                continue;
            }
            passing.push(v["path"].clone());
            for s in v["inventory_routines"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                if !noops.contains(s) {
                    routines.insert(s.to_owned());
                    verified.insert(s.to_owned());
                } else {
                    omitted.insert(s.to_owned());
                }
            }
            identities.push(json!({"verdict":v["path"],"module":v["module"],"sealed_module_sha256":v["module_sha256"],"module_matches_current":v["current_module_matches"],"retained_snapshot_matches":v["snapshot_matches"],"source_pin_matches":v["source_pin_matches"],"scope":"indexed retained module/snapshot comparison only; artifact authentication belongs to retained-coverage-authentication.json; full current dependencies not inferred"}));
            let mp = p.parent().unwrap().join(v["manifest_file"].as_str().unwrap_or("manifest.json"));
            if !manifest_cache.contains_key(&mp) {
                let mut mapped = vec![];
                let mut missing = vec![];
                if let Some(m) = parse(&mp).map(thin) {
                    let bank = m["bank"]
                        .as_str()
                        .and_then(|s| s.strip_prefix("sm_"))
                        .and_then(|s| s.strip_suffix(".c"))
                        .and_then(|s| u32::from_str_radix(s, 16).ok())
                        .unwrap_or(0x86);
                    let file = m["rom"]["file"].as_str().map(|s| resolved(s, mp.parent().unwrap()));
                    let rom = file.as_ref().and_then(|p| fs::read(p).ok());
                    let missing_rom = file.is_some() && rom.is_none();
                    let active = rom.as_deref().unwrap_or(&base_rom);
                    for r in m["routines"].as_array().into_iter().flatten() {
                        let Some(s) = r["symbol"].as_str() else { continue };
                        if ![
                            "EprojInstruction",
                            "EprojPreinstruction",
                            "EprojInitializer",
                            "SpawnEprojWithRoomGfx",
                            "SpawnEprojWithGfx",
                            "PlmInstruction",
                            "PortScreenShake",
                            "PortDrawEprojs",
                            "PortDrawEprojSpritemap",
                            "PortDrawEprojSpritemapOffscreen",
                        ]
                        .contains(&s)
                        {
                            continue;
                        }
                        for c in r["cases"].as_array().into_iter().flatten() {
                            let address = match s {
                                "EprojPreinstruction" | "EprojInitializer" => {
                                    c["b"].as_u64().map(|b| bank << 16 | b as u32)
                                }
                                "EprojInstruction" | "PlmInstruction" => c["b"]
                                    .as_u64()
                                    .and_then(|b| {
                                        rom_word(active, if b > 65535 { b as u32 } else { bank << 16 | b as u32 }, c)
                                    })
                                    .map(|w| bank << 16 | u32::from(w)),
                                "SpawnEprojWithRoomGfx" => c["a"]
                                    .as_u64()
                                    .and_then(|a| rom_word(active, bank << 16 | a as u32, c))
                                    .map(|w| bank << 16 | u32::from(w)),
                                "SpawnEprojWithGfx" => c["b"]
                                    .as_u64()
                                    .and_then(|b| rom_word(active, bank << 16 | (b >> 16) as u32, c))
                                    .map(|w| bank << 16 | u32::from(w)),
                                "PortScreenShake" => Some(0x868427),
                                "PortDrawEprojs" => Some(0x8683d6),
                                "PortDrawEprojSpritemap" => Some(0x818c0a),
                                "PortDrawEprojSpritemapOffscreen" => Some(0x818c7f),
                                _ => None,
                            };
                            if missing_rom && !["EprojPreinstruction", "EprojInitializer"].contains(&s) {
                                missing.push(format!("{s}: fixture ROM absent {}", file.as_ref().unwrap().display()));
                                continue;
                            }
                            if let Some(a) = address {
                                if let Some(n) = by_address.get(&a) {
                                    if !noops.contains(n) {
                                        mapped.push(json!({"symbol":n,"address":format!("0x{a:06x}"),"manifest":mp,"fixture_dispatch":s,"fixture_case":c["name"],"rom":file}));
                                    }
                                } else {
                                    missing.push(format!(
                                        "{s}: native address0x{a:06x} not in lexical inventory ({})",
                                        mp.display()
                                    ));
                                }
                            }
                        }
                    }
                }
                mapped.sort_by_key(Value::to_string);
                mapped.dedup();
                missing.sort();
                missing.dedup();
                manifest_cache.insert(mp.clone(), json!({"mappings":mapped,"missing":missing}));
            }
            for x in manifest_cache[&mp]["mappings"].as_array().unwrap() {
                let s = x["symbol"].as_str().unwrap().to_owned();
                routines.insert(s.clone());
                verified.insert(s);
                mappings.push(x.clone());
            }
            for x in manifest_cache[&mp]["missing"].as_array().unwrap() {
                unresolved.insert(x.as_str().unwrap().to_owned());
            }
            if !live_cache.contains_key(&p) {
                let mut actual = BTreeSet::new();
                if v["status"] == "mixed-C-Rust-integration" {
                    if let Some(w) = parse(&p) {
                        for section in ["hits", "dispatcher_hits"] {
                            if let Some(hits) = w[section]["mixed-0"].as_object() {
                                for (s, h) in hits {
                                    if h["rust"].as_u64().unwrap_or(0) > 0
                                        && h["c"].as_u64().unwrap_or(0) == 0
                                        && native.contains_key(s)
                                        && !noops.contains(s)
                                    {
                                        actual.insert(s.clone());
                                    }
                                }
                            }
                        }
                        if let Some(count) = w["counters"]["mixed-0"].as_object() {
                            for f in ["hits", "frame_hits"] {
                                if let Some(hits) = count[f].as_object() {
                                    for (s, h) in hits {
                                        if h["rust"].as_u64().unwrap_or(0) > 0
                                            && h["c"].as_u64().unwrap_or(0) == 0
                                            && native.contains_key(s)
                                            && !noops.contains(s)
                                        {
                                            actual.insert(s.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                live_cache.insert(p.clone(), actual.into_iter().collect());
            }
            for s in &live_cache[&p] {
                live.insert(s.clone());
                routines.insert(s.clone());
            }
        }
        let mut closure = vec![];
        if id == 141 {
            let cp = root.join(".scratch/decomp-port/ridley-whole-source-closure.txt");
            let text = fs::read_to_string(&cp).unwrap();
            for s in text.lines().skip(2).take_while(|l| *l != "External dependencies:") {
                if native.contains_key(s) && !noops.contains(s) {
                    routines.insert(s.to_owned());
                    closure.push(s.to_owned());
                }
            }
            commits.insert("608b04fada530265ebeaeb85fed15be6e22e9d60".into());
            review.push("Whole Ridley source closure is translated/integrated lineage evidence; only the33 declared controlled roots are directly verified. Internal closure entries require composed/test reachability before verification or live stage promotion; formal whole live remains open.".to_owned());
        }
        let mut source_correspondences = vec![];
        for m in &owned {
            let module = format!("crates/port/src/{m}.rs");
            if let Some(syms) = current_sources.get(&module) {
                for s in syms {
                    if !noops.contains(s) {
                        routines.insert(s.clone());
                        source_correspondences.push(json!({"symbol":s,"module":module,"source_map":".scratch/decomp-strategy/source-coverage-map.json","stage":"translated","basis":"reviewed executable correspondence in maintained Rust; fixture verification assessed separately"}));
                    }
                }
            }
            let p = root.join(&module);
            if p.exists() {
                if let Ok(o) =
                    Command::new("git").args(["log", "-1", "--format=%H", "--", &module]).current_dir(root).output()
                {
                    let h = String::from_utf8_lossy(&o.stdout).trim().to_owned();
                    if h.len() == 40 {
                        commits.insert(h);
                    }
                }
            }
        }
        if !owned.is_empty() {
            let relevant = owned
                .iter()
                .flat_map(|m| current_sources.get(&format!("crates/port/src/{m}.rs")).into_iter().flatten().cloned())
                .collect::<BTreeSet<_>>();
            routines.retain(|s| relevant.contains(s));
            verified.retain(|s| relevant.contains(s));
            live.retain(|s| relevant.contains(s));
        }
        let mut authenticated_routines = BTreeSet::new();
        let mut authenticated_mappings = vec![];
        let mut historic_equivalence_claims = BTreeSet::new();
        for v in &selected {
            if [Some("offline-verified"), Some("mixed-C-Rust-integration"), Some("accepted")]
                .contains(&v["status"].as_str())
            {
                for s in v["inventory_routines"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                    if routines.contains(s) {
                        historic_equivalence_claims.insert(s.to_owned());
                        if authenticated.contains(v["path"].as_str().unwrap()) {
                            authenticated_routines.insert(s.to_owned());
                        }
                    }
                }
            }
        }
        for m in &mappings {
            let mp = Path::new(m["manifest"].as_str().unwrap());
            if selected.iter().any(|v| {
                authenticated.contains(v["path"].as_str().unwrap())
                    && root.join(v["path"].as_str().unwrap()).parent() == mp.parent()
            }) {
                let s = m["symbol"].as_str().unwrap();
                if routines.contains(s) {
                    authenticated_routines.insert(s.into());
                    authenticated_mappings.push(m.clone());
                }
            }
        }
        for s in &authenticated_routines {
            verified.insert(s.clone());
        }
        let textnative =
            codewords(&body).into_iter().filter(|s| native.contains_key(s) && !noops.contains(s)).collect::<Vec<_>>();
        if fallback && !infra {
            review.push("Missing directly linked early receipt reconciled through later retained cumulative evidence for the explicitly named maintained module; scope remains the cited ticket domain and indexed lineage.".into());
        }
        let completion = body.split("## Completion evidence").skip(1).collect::<Vec<_>>().join("\n");
        let integrated = (!commits.is_empty() && !routines.is_empty())
            || completion.contains("Integration branch")
            || completion.contains("integrated")
            || completion.contains("Integrated");
        let mut stages = vec![];
        if !infra && !routines.is_empty() {
            stages.push("translated");
            if !verified.is_empty() {
                stages.push("verified");
            }
            if integrated {
                stages.push("integrated");
            }
            if !live.is_empty() {
                stages.push("live-exercised");
            }
        }
        if infra {
            routines.clear();
            verified.clear();
            live.clear();
            closure.clear();
        }
        if !omitted.is_empty() {
            review.push(format!(
                "Excluded proven empty source bodies: {}",
                omitted.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
        let mut limits = body
            .split("\n\n")
            .filter(|p| {
                [
                    "outside",
                    "unproven",
                    "unverified",
                    "remain",
                    "not-tested",
                    "not tested",
                    "offline only",
                    "historical",
                    "unsupported",
                    "exclud",
                    "synthetic",
                    "no-op",
                    "pending",
                    "zero",
                ]
                .iter()
                .any(|x| p.to_lowercase().contains(x))
            })
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if !complete {
            limits.push("Ticket remains claimed/open; substantiated partial source/fixture/live scope is included independently of ticket resolution.".into());
        }
        if infra {
            limits.push("Infrastructure or diagnostic scope; no gameplay stage credit.".into());
        }
        review.push("Passing retained evidence is historical native equivalence at the declared valid fixture domain, not automatic current checkout authentication. Source correspondence, verification and integration must be assessed per routine; ticket-level supported_stages indicates at least one supported routine.".into());
        output.push(json!({"path":path.strip_prefix(root).unwrap(),"id":id,"status":status,"scope":scope,"routines":routines,"verified_routines":verified,"commits":commits,"evidence":evidence.into_values().collect::<Vec<_>>(),"supported_stages":stages,"limitations":limits,"review_notes":review,"source_pin":{"revision":registry["source"]["revision"],"inventory_sha256":registry["source"]["inventory_sha256"]},"rust_identities":identities,"passing_verdicts":passing,"live_routines":live,"synthetic_dispatch_mappings":mappings,"text_native_symbols":textnative,"unmapped_fixture_addresses":unresolved,"source_closure_routines":closure,"source_correspondences":source_correspondences,"authenticated_verified_routines":authenticated_routines,"authenticated_synthetic_dispatch_mappings":authenticated_mappings,"historical_equivalence_claim_routines":historic_equivalence_claims,"infrastructure":infra,"completed":complete,"ticket_text":body}));
        println!("ticket{id}:{} routines", output.last().unwrap()["routines"].as_array().unwrap().len());
    }
    let union = output
        .iter()
        .flat_map(|t| t["routines"].as_array().unwrap().iter().filter_map(Value::as_str))
        .collect::<BTreeSet<_>>()
        .len();
    let result = json!({"schema_version":1,"audit_scope":"All141 decomp-port tickets plus spec, joined to retained533-verdict inventory. Read-only provenance; source internal closure may support translated only, exact passing fixture roots and opcode/initializer mappings support historical verified domains. No publication or new gameplay verification.","source_revision":registry["source"]["revision"],"inventory_sha256":registry["source"]["inventory_sha256"],"retained_inventory_path":".scratch/decomp-strategy/retained-coverage-inventory.json","source_map_path":".scratch/decomp-strategy/source-coverage-map.json","spec":{"path":".scratch/decomp-port/spec.md","text":fs::read_to_string(root.join(".scratch/decomp-port/spec.md")).unwrap()},"excluded_empty_source_symbols":noops,"tickets":output});
    fs::write(
        root.join(".scratch/decomp-strategy/ticket-provenance.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();
    println!("tickets={} substantiated_native_union={union}", result["tickets"].as_array().unwrap().len());
}
