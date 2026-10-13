use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap,BTreeSet}, fs, io::{Read,Write}, path::{Path,PathBuf}};
use anyhow::{Result,Context,ensure};
use serde::Deserialize;

#[derive(Deserialize)]
struct Declaration { family: Option<String>, #[serde(default)] routines: Vec<DeclaredRoutine> }
#[derive(Deserialize)]
struct DeclaredRoutine { symbol: String }
#[path = "../../crates/pipeline/src/progress/declarations.rs"]
mod declarations;

fn object(path:&Path)->Result<Value> {
    let bytes=fs::read(path)?;
    if path.extension().is_some_and(|v|v=="json") { Ok(serde_json::from_slice(&bytes)?) }
    else { Ok(serde_json::to_value(toml::from_str::<toml::Value>(std::str::from_utf8(&bytes)?)?)?) }
}
fn hash(path:&Path)->Result<String> { let mut file=fs::File::open(path)?; let mut sha=Sha256::new(); let mut bytes=[0u8;65536]; loop {let n=file.read(&mut bytes)?;if n==0 {break} sha.update(&bytes[..n]);}Ok(format!("{:x}",sha.finalize())) }
fn strs(value:&Value)->BTreeSet<String> { value.as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect() }
fn s(value:&Value)->&str { value.as_str().unwrap_or("") }
fn normalized_module(value:&str)->String { if let Some(i)=value.find("crates/port/src/") {value[i..].to_owned()} else {value.to_owned()} }
fn walk(dir:&Path,paths:&mut Vec<PathBuf>)->Result<()> {for e in fs::read_dir(dir)? {let p=e?.path();if p.is_dir(){walk(&p,paths)?}else{paths.push(p)}}Ok(())}
fn main()->Result<()> {
 let root=Path::new("/home/jim/sm-decomp");
 let registry=object(&root.join(".scratch/decomp-progress/registry.json"))?;
 let known:BTreeMap<String,usize>=registry["routines"].as_array().unwrap().iter().map(|r|(s(&r["symbol"]).to_owned(),r["body_lines"].as_u64().unwrap_or(0) as usize)).collect();
 let mut manifests=Vec::new();
 for e in fs::read_dir(root.join("tools/port"))? {let path=e?.path();if path.extension().is_none_or(|v|v!="toml")||path.ends_with("source-inventory.toml"){continue}
 let text=fs::read_to_string(&path)?;
 match declarations::parse(&text) {Ok(d) if d.family.is_some()=>{
 let symbols:BTreeSet<_>=d.routines.into_iter().map(|r|r.symbol).collect();
 manifests.push(json!({"path":path.strip_prefix(root)?.to_string_lossy(),"sha256":hash(&path)?,"family":d.family,"routines":symbols,"inventory_routines":symbols.iter().filter(|s|known.contains_key(*s)).collect::<Vec<_>>(),"synthetic_or_external":symbols.iter().filter(|s|!known.contains_key(*s)).collect::<Vec<_>>() }));
 },Ok(_)=>{},Err(e)=>manifests.push(json!({"path":path.to_string_lossy(),"error":e.to_string()})) }
 }
 manifests.sort_by_key(|v|s(&v["path"]).to_owned());
 let paths=fs::read_to_string(root.join(".scratch/decomp-progress/coverage-retained-verdict-paths.txt"))?;
 let mut verdicts=Vec::new();let mut statuses=BTreeMap::<String,usize>::new();
 for (index,name) in paths.lines().enumerate() {let path=root.join(name);
 let mut record=match object(&path) {Ok(v)=>{
 let routines:BTreeSet<String>=v["cases"].as_array().into_iter().flatten().filter_map(|c|c["routine"].as_str()).map(str::to_owned).collect();
 let inventory:BTreeSet<_>=routines.iter().filter(|s|known.contains_key(*s)).cloned().collect();
 let module=normalized_module(s(&v["implementation"]["module"]));
 let current=root.join(&module);let candidate=path.parent().unwrap().join("candidate.rs");
 let expected=s(&v["implementation"]["module_sha256"]);
 let snapshot_hash=if candidate.is_file(){hash(&candidate).ok()}else{None};
 let current_hash=if current.is_file(){hash(&current).ok()}else{None};
 let status=s(&v["status"]).to_owned();*statuses.entry(status.clone()).or_default()+=1;
 let artifacts=v["artifacts"].as_object();
 let manifest_name=if path.parent().unwrap().join("manifest.toml").is_file(){"manifest.toml"}else{"manifest.json"};
 let manifest=path.parent().unwrap().join(manifest_name);
 let source_matches=s(&v["source"]["revision"])==s(&registry["source"]["revision"]);
 let source_pin_matches=s(&v["source"]["archive_sha256"])=="69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193";
 json!({"path":name,"sha256":hash(&path)?,"format":path.extension().unwrap().to_string_lossy(),"status":status,"family":v["family"],"scope":v["scope"],"live_exercised":v["live_exercised"],"source_revision":v["source"]["revision"],"source_pin_matches":source_matches&&source_pin_matches,"routines":routines,"inventory_routines":inventory,"synthetic_or_external":routines.iter().filter(|s|!known.contains_key(*s)).collect::<Vec<_>>(),"verified_cases":v["verified_cases"],"verified_routines":v["verified_routines"],"case_count":v["cases"].as_array().map(Vec::len),"module":module,"module_sha256":expected,"snapshot_sha256":snapshot_hash,"snapshot_matches":snapshot_hash.as_deref()==Some(expected)&&!expected.is_empty(),"current_module_matches":current_hash.as_deref()==Some(expected)&&!expected.is_empty(),"manifest_retained":manifest.is_file(),"manifest_file":manifest_name,"artifact_count":artifacts.map(serde_json::Map::len),"eligible_for_artifact_check":status=="offline-verified"&&source_matches&&source_pin_matches&&!inventory.is_empty()&&snapshot_hash.as_deref()==Some(expected)&&manifest.is_file()})
 },Err(e)=>json!({"path":name,"error":e.to_string()})};
 if record["eligible_for_artifact_check"]==true {
   let v=object(&path)?;
   if let Some(map)=v["implementation"]["rust_dependencies"].as_object() {
    let artifact_hashes:BTreeSet<_>=v["artifacts"].as_object().unwrap().values().filter_map(Value::as_str).collect();
    record["sealed_rust_dependency_count"]=json!(map.values().filter(|expected|artifact_hashes.contains(s(expected))).count());
    record["rust_dependency_count"]=json!(map.len());
   }
 }
 verdicts.push(record);if(index+1)%50==0 {eprintln!("metadata {}/{}, no gameplay reruns",index+1,paths.lines().count())}
 }
 let mut eligible=BTreeSet::new();for v in &verdicts{if v["eligible_for_artifact_check"]==true{eligible.extend(strs(&v["inventory_routines"]))}}
 let mut selected=Vec::new();let mut remaining=eligible.clone();
 while !remaining.is_empty() {
 let best=verdicts.iter().enumerate().filter(|(_,v)|v["eligible_for_artifact_check"]==true).max_by_key(|(_,v)|strs(&v["inventory_routines"]).intersection(&remaining).count());
 let Some((index,v))=best else{break};let routines=strs(&v["inventory_routines"]);ensure!(!routines.is_disjoint(&remaining),"no cover for eligible routines");remaining=remaining.difference(&routines).cloned().collect();selected.push(index);
 }
 let output=json!({"schema_version":1,"source_revision":registry["source"]["revision"],"inventory_sha256":registry["source"]["inventory_sha256"],"manifest_count":manifests.len(),"verdict_count":verdicts.len(),"statuses":statuses,"manifests":manifests,"verdicts":verdicts,"eligible_routines":eligible,"selected_verdict_paths":selected.iter().map(|i|&verdicts[*i]["path"]).collect::<Vec<_>>(),"limitations":["Metadata reconciliation only until selected retained artifact checks complete.","Fixture routine membership does not prove internal call closure or current Rust dependencies."]});
 fs::write(root.join(".scratch/decomp-strategy/retained-coverage-inventory.json"),serde_json::to_vec_pretty(&output)?)?;
 eprintln!("{} manifests; {} verdicts; {} eligible native symbols; {} selected frozen runs",output["manifest_count"],output["verdict_count"],eligible.len(),selected.len());
 Ok(())
}
