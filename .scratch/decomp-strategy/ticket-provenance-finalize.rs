use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
const ROOT: &str = "/home/jim/sm-decomp";
fn parse(p: &Path) -> Value {
    let s = fs::read_to_string(p).unwrap();
    if p.extension().is_some_and(|e| e == "toml") {
        let t: toml::Value = toml::from_str(&s).unwrap();
        serde_json::to_value(t).unwrap()
    } else {
        serde_json::from_str(&s).unwrap()
    }
}
fn add(a: &mut Value, s: &str) {
    if !a.as_array().unwrap().iter().any(|v| v == s) {
        a.as_array_mut().unwrap().push(json!(s));
    }
}
fn uniq(a: &mut Value) {
    let mut v = a.as_array().unwrap().clone();
    v.sort_by_key(Value::to_string);
    v.dedup();
    *a = json!(v);
}
fn pointer(t: &mut Value, p: &str, kind: &str) {
    let full = Path::new(ROOT).join(p);
    let e = json!({"path":p,"exists":full.exists(),"kind":kind});
    t["evidence"].as_array_mut().unwrap().push(e);
}
fn family(t: &mut Value, run: &str, receipt: &str) {
    let root = Path::new(ROOT);
    let p = root.join(run).join("verdict.toml");
    let v = parse(&p);
    let m = parse(&root.join(run).join("manifest.toml"));
    pointer(t, &format!("{run}/verdict.toml"), "verdict");
    pointer(t, &format!("{run}/manifest.toml"), "manifest");
    pointer(t, receipt, "authentication");
    add(&mut t["passing_verdicts"], &format!("{run}/verdict.toml"));
    for r in m["routines"].as_array().unwrap() {
        if let Some(s) = r["symbol"].as_str() {
            if t["routines"].as_array().unwrap().iter().any(|x| x == s) {
                for f in ["verified_routines", "authenticated_verified_routines"] {
                    add(&mut t[f], s);
                }
            }
        }
    }
    t["rust_identities"].as_array_mut().unwrap().push(json!({"verdict":format!("{run}/verdict.toml"),"receipt":receipt,"implementation":v["implementation"],"source_pin":v["source"]["revision"],"scope":"retained post-cleanup family receipt; exact candidate identity, current checkout dependency identity remains separate"}));
}
fn main() {
    let root = Path::new(ROOT);
    let p = root.join(".scratch/decomp-strategy/ticket-provenance.json");
    let mut audit = parse(&p);
    let source = parse(&root.join(".scratch/decomp-strategy/source-coverage-map.json"));
    let source_symbols = source["modules"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| {
            m["native_symbols"].as_array().unwrap().iter().filter_map(|s| s["symbol"].as_str()).map(str::to_owned)
        })
        .collect::<BTreeSet<_>>();
    let auth = parse(&root.join(".scratch/decomp-strategy/retained-coverage-authentication.json"));
    let authenticated = auth["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["artifact_hashes_match"] == true)
        .filter_map(|v| v["path"].as_str())
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let index = parse(&root.join(".scratch/decomp-strategy/retained-coverage-inventory.json"));
    let base_rom = fs::read(root.join("rom/sm.sfc")).unwrap();
    let composed_path = "out/recovery-session/composed-live-release/1791861180321696848-129445/verdict.toml";
    let composed = parse(&root.join(composed_path));
    let trails_path = "out/recovery-session/trails-live/1791860457650339558-98848/verdict.toml";
    let trails = parse(&root.join(trails_path));
    let mut records = vec![];
    for t in audit["tickets"].as_array_mut().unwrap() {
        let id = t["id"].as_u64().unwrap();
        let mut aggregate = vec![];
        for field in ["synthetic_dispatch_mappings", "authenticated_synthetic_dispatch_mappings"] {
            let mut groups = BTreeMap::<String, Value>::new();
            for m in t[field].as_array().unwrap() {
                let mut key = m.clone();
                key.as_object_mut().unwrap().remove("fixture_case");
                let k = key.to_string();
                let e = groups.entry(k).or_insert_with(|| {
                    let mut v = key;
                    v["fixture_case_count"] = json!(0);
                    v["fixture_case_examples"] = json!([]);
                    v
                });
                let n = e["fixture_case_count"].as_u64().unwrap() + 1;
                e["fixture_case_count"] = json!(n);
                if e["fixture_case_examples"].as_array().unwrap().len() < 3 {
                    e["fixture_case_examples"].as_array_mut().unwrap().push(m["fixture_case"].clone());
                }
            }
            t[field] = json!(groups.into_values().collect::<Vec<_>>());
        }
        if id != 141 {
            let eligible = t["routines"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .filter(|s| source_symbols.contains(*s))
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            for f in ["routines", "verified_routines", "authenticated_verified_routines", "live_routines"] {
                t[f].as_array_mut().unwrap().retain(|s| s.as_str().is_some_and(|s| eligible.contains(s)));
            }
        }
        // Infrastructure wrappers have no implementation credit; shared code credit is assigned to actual gameplay owner tickets.
        if [133, 134, 135, 139, 140].contains(&id) {
            let module = match id {
                133 => "enemy_projectile_rendering",
                134 => "enemy_projectile_contacts",
                135 => "enemy_projectile_sparks",
                139 => "projectile_trails",
                _ => "camera_scroll",
            };
            let mut eligible = source["modules"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["path"] == format!("crates/port/src/{module}.rs"))
                .unwrap()["native_symbols"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|s| s["symbol"].as_str())
                .map(str::to_owned)
                .collect::<BTreeSet<_>>();
            if id == 134 {
                eligible.remove("Samus_DealDamage");
                eligible.remove("SuitDamageDivision");
            }
            for f in ["routines", "verified_routines", "authenticated_verified_routines", "live_routines"] {
                t[f].as_array_mut().unwrap().retain(|s| s.as_str().is_some_and(|s| eligible.contains(s)));
            }
            t["review_notes"].as_array_mut().unwrap().push(json!("Combined receipt contains dependency/sibling domains; this ticket routine set is restricted to its declared native ownership."));
        }
        for f in ["verified_routines", "authenticated_verified_routines", "live_routines"] {
            let r = t["routines"].as_array().unwrap().clone();
            t[f].as_array_mut().unwrap().retain(|s| r.contains(s));
        }
        let passes = t["passing_verdicts"].as_array().unwrap().iter().filter_map(Value::as_str).collect::<Vec<_>>();
        for path in passes {
            let Some(v) = index["verdicts"].as_array().unwrap().iter().find(|v| v["path"] == path) else { continue };
            if v["status"] != "mixed-C-Rust-integration" {
                continue;
            }
            let w = parse(&root.join(path));
            let mut observed = BTreeMap::new();
            for section in ["hits", "dispatcher_hits"] {
                if let Some(hits) = w[section]["mixed-0"].as_object() {
                    for (s, h) in hits {
                        if h["rust"].as_u64().unwrap_or(0) > 0
                            && h["c"].as_u64().unwrap_or(0) == 0
                            && t["live_routines"].as_array().unwrap().iter().any(|x| x == s)
                        {
                            observed.insert(s.clone(), h.clone());
                        }
                    }
                }
            }
            if !observed.is_empty() {
                aggregate.push(json!({"verdict":path,"subject":"mixed-0","observation_kind":"direct_native_wrapper","counts":observed,"requires_retained_live_authentication":"binary/storage/observation receipt must be checked separately"}));
            }
        }
        if id == 133 || id == 134 || id == 136 {
            pointer(t, ".scratch/decomp-port/contact-render-live-family-authentication.json", "authentication");
        }
        if id == 137 {
            pointer(t, composed_path, "verdict");
            pointer(t, ".scratch/decomp-port/recovery-composed-post-cleanup-authentication.toml", "authentication");
            let mut counts = BTreeMap::new();
            for (k, s) in [("0", "SpawnEprojWithRoomGfx"), ("1", "EprojRunAll"), ("2", "SpawnEprojWithGfx")] {
                let h = &composed["counters"]["mixed-0"]["projectile_hits"][k];
                if h["rust"].as_u64().unwrap_or(0) > 0 {
                    add(&mut t["routines"], s);
                    add(&mut t["live_routines"], s);
                    counts.insert(s.to_owned(), h.clone());
                }
            }
            aggregate.push(json!({"verdict":composed_path,"authentication":".scratch/decomp-port/recovery-composed-post-cleanup-authentication.toml","subject":"mixed-0","observation_kind":"direct_native_wrapper_with_live_projectile_contract","counts":counts}));
            let mut initcounts = BTreeMap::new();
            for d in [0x9642u32, 0xa387, 0xa395] {
                let h = &composed["counters"]["mixed-0"]["projectile_allocation_hits"][d.to_string()];
                if h["rust"].as_u64().unwrap_or(0) > 0 {
                    let offset = (0x6 * 0x8000 + (d & 0x7fff)) as usize;
                    let a = 0x860000 | u32::from(u16::from_le_bytes(base_rom[offset..offset + 2].try_into().unwrap()));
                    let funcs = fs::read_to_string(root.join("deps/snesrev-sm/src/funcs.h")).unwrap();
                    if let Some(s) = funcs.lines().find_map(|l| {
                        let w = l.split_whitespace().collect::<Vec<_>>();
                        if w.len() > 2
                            && w[0] == "#define"
                            && u32::from_str_radix(w[2].trim_start_matches("0x"), 16).ok() == Some(a)
                        {
                            w[1].strip_prefix("fn")
                        } else {
                            None
                        }
                    }) {
                        if source_symbols.contains(s) {
                            add(&mut t["routines"], s);
                            initcounts.insert(s.to_owned(),json!({"definition":format!("0x{d:04x}"),"initializer":format!("0x{a:06x}"),"allocation_counter":h,"stage":"live-composed; initializer executes inside accepted native allocator, separate direct-wrapper count absent"}));
                        }
                    }
                }
            }
            t["composed_internal_live_observations"] = json!(initcounts);
            t["unmapped_fixture_addresses"].as_array_mut().unwrap().retain(|s| !s.as_str().unwrap().contains("950c"));
            t["review_notes"].as_array_mut().unwrap().push(json!("86950C is dispatcher-only no-op; no lexical implementation or stage credit. 8608FF from malformed/full-pool fixture definitions is never dereferenced on saturation; exclude as initializer mapping."));
        }
        if id == 140 {
            family(
                t,
                "out/recovery-session/integrated/families/camera-scroll/evidence/1791860292776643-47409-76",
                ".scratch/decomp-port/recovery-integrated-camera-scroll-post-cleanup-authentication.toml",
            );
            pointer(t, composed_path, "verdict");
            pointer(t, ".scratch/decomp-port/recovery-composed-post-cleanup-authentication.toml", "authentication");
            add(&mut t["live_routines"], "MainScrollingRoutine");
            aggregate.push(json!({"verdict":composed_path,"authentication":".scratch/decomp-port/recovery-composed-post-cleanup-authentication.toml","subject":"mixed-0","observation_kind":"direct_native_wrapper","counts":{"MainScrollingRoutine":composed["counters"]["mixed-0"]["frame_hits"]["MainScrollingRoutine"]}}));
        }
        if id == 139 {
            family(
                t,
                "out/recovery-session/integrated/families/projectile-trails/evidence/1791860275994273-47409-65",
                ".scratch/decomp-port/recovery-integrated-projectile-trails-post-cleanup-authentication.toml",
            );
            pointer(t, trails_path, "verdict");
            pointer(t, ".scratch/decomp-port/recovery-trails-live-post-cleanup-authentication.toml", "authentication");
            add(&mut t["live_routines"], "HandleProjectileTrails");
            let h = &trails["counters"]["mixed-0"]["frame_hits"]["HandleProjectileTrails"];
            aggregate.push(json!({"verdict":trails_path,"authentication":".scratch/decomp-port/recovery-trails-live-post-cleanup-authentication.toml","subject":"mixed-0","observation_kind":"direct_native_wrapper","counts":{"HandleProjectileTrails":h},"recorded_total":3824,"recorded_active":167,"limitations":"three trail movement opcodes have no natural hits"}));
        }
        if id == 98 {
            t["composed_internal_live_observations"] = json!({"PalFx_ProcessOne":{"verdict":"out/port/palette-effect-frame-replay/20261011T092352.241110Z/verdict.json","native_c_object_calls":9905,"rust_wrapper_calls":0,"basis":"Rust PaletteFxHandler invokes process_palette internally; retained native header samples show active scripts/color writes","stage":"live-composed behavior, exact internal Rust call count unmeasured"}});
        }
        if id == 141 {
            t["supported_stages"] = json!(["translated", "verified", "integrated"]);
            t["limitations"].as_array_mut().unwrap().push(json!("Whole recovered native closure220 is source translation/integration proof. Direct controlled33-root6457pair evidence is authenticated historical equivalence; do not credit all internal definitions as directly verified or formally whole-live."));
        }
        t["live_observations"] = json!(aggregate);
        for m in t["authenticated_synthetic_dispatch_mappings"].as_array().unwrap() {
            if t["routines"].as_array().unwrap().contains(&m["symbol"]) {
                records.push(m.clone());
            }
        }
        t["verified_routines"] = t["authenticated_verified_routines"].clone();
        let mut stages = vec![];
        if !t["routines"].as_array().unwrap().is_empty() {
            stages.push("translated");
            if !t["verified_routines"].as_array().unwrap().is_empty() {
                stages.push("verified");
            }
            if !t["commits"].as_array().unwrap().is_empty()
                || t["supported_stages"].as_array().unwrap().iter().any(|s| s == "integrated")
            {
                stages.push("integrated");
            }
            if !t["live_routines"].as_array().unwrap().is_empty() {
                stages.push("live-exercised");
            }
        }
        t["supported_stages"] = json!(stages);
        for f in [
            "routines",
            "verified_routines",
            "authenticated_verified_routines",
            "live_routines",
            "evidence",
            "passing_verdicts",
        ] {
            uniq(&mut t[f]);
        }
        t["review_notes"].as_array_mut().unwrap().push(json!("The implemented set is the cumulative current module lineage; old ticket module breadth may include later additions. Historical source aliases do not prove every current dependency. verified_routines equals authenticated_verified_routines; un-authenticated passing fixture claims remain historical_equivalence_claim_routines."));
    }
    let mut compact = BTreeMap::<String, Value>::new();
    for m in records {
        let mut key = m.clone();
        key.as_object_mut().unwrap().remove("fixture_case_count");
        key.as_object_mut().unwrap().remove("fixture_case_examples");
        compact.insert(key.to_string(), m);
    }
    fs::write(root.join(".scratch/decomp-strategy/ticket-authenticated-internal-mappings.json"),serde_json::to_string_pretty(&json!({"schema_version":1,"source_revision":"578f90b3cc49557bb70060ad033bb90b8cf8ac50","basis":"exact synthetic dispatch opcode/initializer/native rendering adapter mapping joined to artifact_hashes_match=true retained receipts","mappings":compact.into_values().collect::<Vec<_>>()})).unwrap()).unwrap();
    audit["finalized_at"] = json!("2026-10-13");
    audit["audit_scope"] = json!(
        "All141tickets and spec; declared ticket source ownership plus cumulative maintained implementation lineage, exact authenticated retained fixture mapping, direct versus composed live observations; no new gameplay runs. Use per-routine stages and limitations, not blanket ticket stage promotion."
    );
    fs::write(p, serde_json::to_string_pretty(&audit).unwrap()).unwrap();
    println!("finalized141tickets;authenticatedruns={}", authenticated.len());
}
