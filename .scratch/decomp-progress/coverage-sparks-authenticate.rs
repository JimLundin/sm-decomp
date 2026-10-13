//! Read-only reconciliation of retained fixture evidence; never executes gameplay.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap, BTreeSet, HashMap}, fs, io::Read, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

fn object(path: &Path) -> Result<Value> {
    let bytes = fs::read(path)?;
    if path.extension().is_some_and(|e| e == "json") { Ok(serde_json::from_slice(&bytes)?) }
    else { Ok(serde_json::to_value(toml::from_str::<toml::Value>(std::str::from_utf8(&bytes)?)?)?) }
}
fn s(v: &Value) -> &str { v.as_str().unwrap_or("") }
fn strings(v: &Value) -> BTreeSet<String> { v.as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect() }
type HashCache = HashMap<(u64, u64, u64, i64, i64), String>;
fn hash(path: &Path, cache: &mut HashCache) -> Result<String> {
    let meta = fs::metadata(path)?;
    let key = (meta.dev(), meta.ino(), meta.len(), meta.mtime(), meta.mtime_nsec());
    if let Some(h) = cache.get(&key) { return Ok(h.clone()); }
    let mut file = fs::File::open(path)?; let mut sha = Sha256::new(); let mut bytes = [0; 65536];
    loop { let n = file.read(&mut bytes)?; if n == 0 { break; } sha.update(&bytes[..n]); }
    let value = format!("{:x}", sha.finalize()); cache.insert(key, value.clone()); Ok(value)
}
fn check(root: &Path, path: &Path, known: &BTreeSet<String>, source: &Value, cache: &mut HashCache) -> Result<Value> {
    let v = object(path)?; let run = path.parent().context("missing run")?;
    ensure!(s(&v["status"]) == "offline-verified", "not a passing Offline family");
    ensure!(v["source"]["revision"] == source["revision"] && s(&v["source"]["archive_sha256"]) == "69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193", "C pin differs");
    let artifacts = v["artifacts"].as_object().context("missing artifact inventory")?;
    ensure!(!artifacts.is_empty(), "empty artifact inventory");
    for (name, expected) in artifacts {
        let rel = Path::new(name); ensure!(!rel.is_absolute() && !rel.components().any(|c| matches!(c, std::path::Component::ParentDir)), "unsafe artifact path");
        ensure!(hash(&run.join(rel), cache)? == s(expected), "artifact differs: {name}");
    }
    let implementation = &v["implementation"];
    let expected = s(&implementation["module_sha256"]); ensure!(!expected.is_empty(), "missing module digest");
    let candidates: Vec<_> = artifacts.iter().filter(|(name, h)| name.ends_with(".rs") && s(h) == expected).map(|(name, _)| name.clone()).collect();
    ensure!(!candidates.is_empty(), "module source not retained in sealed artifacts");
    let sealed: BTreeSet<_> = artifacts.values().map(s).collect();
    let mut missing_dependencies = Vec::new();
    if let Some(deps) = implementation["rust_dependencies"].as_object() { for (name, digest) in deps { if !sealed.contains(s(digest)) { missing_dependencies.push(name.clone()); } } }
    ensure!(missing_dependencies.is_empty(), "Rust dependency sources absent from sealed artifacts: {}", missing_dependencies.join(", "));
    let manifest_name = if artifacts.contains_key("manifest.toml") { "manifest.toml" } else { "manifest.json" };
    ensure!(artifacts.contains_key(manifest_name), "manifest absent from seal");
    let manifest = object(&run.join(manifest_name))?;
    ensure!(manifest["family"] == v["family"], "manifest family differs");
    let declared: BTreeSet<String> = manifest["routines"].as_array().into_iter().flatten().filter_map(|r|r["symbol"].as_str()).map(str::to_owned).collect();
    let cases = v["cases"].as_array().context("missing cases")?; ensure!(!cases.is_empty(), "empty passing cases");
    let mut routines = BTreeSet::new();
    for c in cases {
        let routine = s(&c["routine"]); ensure!(declared.contains(routine), "case routine not declared: {routine}");
        if let Some(values) = c["return"].as_object() { ensure!(values.get("c") == values.get("rust"), "return mismatch: {routine}"); }
        if let Some(values) = c["after_sha256"].as_object() { ensure!(values.get("c") == values.get("rust"), "WRAM mismatch: {routine}"); }
        if known.contains(routine) { routines.insert(routine.to_owned()); }
    }
    ensure!(v["verified_cases"].as_u64() == Some(cases.len() as u64), "verified case count differs");
    let current = PathBuf::from(s(&implementation["module"]));
    let current_module_matches = current.is_file() && hash(&current, cache)? == expected;
    Ok(json!({"path":path.strip_prefix(root)?.to_string_lossy(),"sha256":hash(path, cache)?,"family":v["family"],"scope":v["scope"],"verified_cases":cases.len(),"routines":routines,"synthetic_or_external":cases.iter().map(|c|s(&c["routine"])).filter(|r|!known.contains(*r)).collect::<BTreeSet<_>>(),"module":implementation["module"],"module_sha256":expected,"candidate_artifacts":candidates,"rust_dependencies":implementation["rust_dependencies"].as_object().map_or(0, |d|d.len()),"authenticated_artifacts":artifacts.len(),"current_module_matches":current_module_matches,"artifact_hashes_match":true,"retained_source_closure":true,"limitations":["Historical valid fixture domain only; no current checkout dependency identity or full-game completion inferred.","Only explicit fixture roots are counted here; internal calls and synthetic dispatch need independent native mapping."]}))
}
fn main() -> Result<()> {
    let root = Path::new("/home/jim/sm-decomp");
    let selected = object(&root.join(".scratch/decomp-port/sparks-integration-authentication.json"))?;
    let registry = object(&root.join(".scratch/decomp-progress/registry.json"))?;
    let known: BTreeSet<String> = registry["routines"].as_array().unwrap().iter().map(|r|s(&r["symbol"]).to_owned()).collect();
    let paths: Vec<_> = selected.as_array().unwrap().iter().map(|v|s(&v["verdict"]).to_owned()).collect();
    let mut cache = HashCache::new(); let mut records = Vec::new(); let mut admitted = BTreeSet::new();
    for (i, name) in paths.iter().enumerate() {
        let record = match check(root, &root.join(name), &known, &registry["source"], &mut cache) { Ok(v) => { admitted.extend(strings(&v["routines"])); v }, Err(e) => json!({"path":name,"error":format!("{e:#}"),"artifact_hashes_match":false}) };
        eprintln!("retained artifact check {}/{}: {}{}", i+1,paths.len(),name,if record.get("error").is_some(){format!(" -> {}",record["error"])}else{" -> authenticated".into()});
        records.push(record);
        fs::write(root.join(".scratch/decomp-progress/sparks-retained-coverage-authentication.json"), serde_json::to_vec_pretty(&json!({"schema_version":1,"source_revision":registry["source"]["revision"],"inventory_sha256":registry["source"]["inventory_sha256"],"checked_runs":records.len(),"expected_runs":paths.len(),"unique_hashed_files":cache.len(),"admitted_native_roots":admitted,"runs":records}))?)?;
    }
    let counts: BTreeMap<_,_> = [("checked_runs",records.len()),("authenticated_runs",records.iter().filter(|v|v["artifact_hashes_match"]==true).count()),("admitted_native_roots",admitted.len()),("unique_hashed_files",cache.len())].into();
    println!("{}",serde_json::to_string_pretty(&counts)?); Ok(())
}
