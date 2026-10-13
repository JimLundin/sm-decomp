//! Audits existing live verdicts and historical authentication receipts.
//! Does not run or unpack gameplay; historical receipts remain the storage contract.
use anyhow::{Result,ensure,Context};
use serde_json::{Value,json};
use sha2::{Sha256,Digest};
use std::{collections::{BTreeSet,BTreeMap},fs,io::Read,path::Path,os::unix::fs::MetadataExt};
fn object(p:&Path)->Result<Value>{let b=fs::read(p)?;if p.extension().is_some_and(|e|e=="json"){Ok(serde_json::from_slice(&b)?)}else{Ok(serde_json::to_value(toml::from_str::<toml::Value>(std::str::from_utf8(&b)?)?)?)}}
fn s(v:&Value)->&str{v.as_str().unwrap_or("")}
fn hash(p:&Path)->Result<String>{let mut f=fs::File::open(p)?;let mut h=Sha256::new();let mut b=[0;65536];loop{let n=f.read(&mut b)?;if n==0{break}h.update(&b[..n]);}Ok(format!("{:x}",h.finalize()))}
fn main()->Result<()> {
 let root=Path::new("/home/jim/sm-decomp");let index=object(&root.join(".scratch/decomp-strategy/retained-coverage-inventory.json"))?;let registry=object(&root.join(".scratch/decomp-progress/registry.json"))?;let known:BTreeSet<_>=registry["routines"].as_array().unwrap().iter().map(|r|s(&r["symbol"])).collect();
 let mut runs=vec![];let mut union=BTreeSet::new();let mut hashed_inodes:BTreeMap<(u64,u64,u64),String>=BTreeMap::new();
 for entry in index["verdicts"].as_array().unwrap().iter().filter(|v|s(&v["status"])=="mixed-C-Rust-integration") {
  let name=s(&entry["path"]);let path=root.join(name);let v=object(&path)?;let run=path.parent().unwrap();
  let audited=(||->Result<Value>{
   ensure!(v["source"]["revision"]==registry["source"]["revision"]&&s(&v["source"]["archive_sha256"])=="69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193","C pin differs");
   ensure!(v["state_effects_match"]==true && v["remaining_CPU_fallback"].as_u64()==Some(0),"live state/effects or CPU fallback rejected");
   let binary=run.join("live");ensure!(hash(&binary)?==s(&v["binary_sha256"]),"retained live binary differs");
   let mut routines=BTreeSet::new();let mut counters=BTreeMap::new();
   for field in ["hits","dispatcher_hits"] {
    let a=v[field]["mixed-0"].as_object();let b=v[field]["mixed-1"].as_object();
    for (symbol,c) in a.into_iter().flatten(){if !known.contains(symbol.as_str()){continue}let n=c["rust"].as_u64().unwrap_or(0);let other=b.and_then(|b|b.get(symbol)).and_then(|v|v["rust"].as_u64()).unwrap_or(0);if n>0&&other>0&&c["c"].as_u64()==Some(0){routines.insert(symbol.clone());counters.insert(symbol.clone(),json!({"mixed0":n,"mixed1":other,"field":field}));}}
   }
   ensure!(!routines.is_empty(),"no explicitly measured native Rust roots");
   let artifacts=v["artifacts"].as_object().context("artifact inventory missing")?;
   let receipt_path=run.join("storage-authentication.json");let receipt=object(&receipt_path).ok().filter(|r|r["retained_authenticated"]==true&&r["status"]==v["status"]);
   let mut checked_artifacts=0usize;
   if receipt.is_none(){
    // Unpacked older runs can be rehashed read-only with hard-link deduplication.
    for subject in ["c-0","c-1","mixed-0","mixed-1"]{for(name,expected)in artifacts{let p=run.join(subject).join(name);let m=fs::metadata(&p).with_context(||format!("artifact absent: {}",p.display()))?;let key=(m.dev(),m.ino(),m.len());let h=if let Some(h)=hashed_inodes.get(&key){h.clone()}else{let h=hash(&p)?;hashed_inodes.insert(key,h.clone());h};ensure!(h==s(expected),"artifact differs: {}",p.display());checked_artifacts+=1;}}
   }
   let mut evidence=vec![path.clone(),binary];if receipt.is_some(){evidence.push(receipt_path.clone())}
   for p in [run.join("families.rs"),run.join("host.c"),run.join("sm_86.c"),run.join("sm_84.c")]{if p.is_file(){evidence.push(p)}}
   Ok(json!({"path":name,"sha256":hash(&path)?,"status":v["status"],"source_revision":v["source"]["revision"],"routines":routines,"counters":counters,"evidence":evidence,"binary_matches":true,"retained_authentication":receipt,"read_only_artifacts_hashed":checked_artifacts,"accepted":true,"limitations":["Historical immutable live binary only. Counters report native Rust roots positively measured in both mixed subjects; zero-hit wrappers and unmeasured internal calls are excluded.","Existing storage-authentication receipt is retained for compressed runs; no new decompression or full gameplay execution performed. Historical tested replay is not current-source authentication or credits/movie completion."]}))
  })();
  let record=match audited{Ok(v)=>{union.extend(v["routines"].as_array().unwrap().iter().map(|v|s(v).to_owned()));v},Err(e)=>json!({"path":name,"accepted":false,"error":format!("{e:#}")})};
  eprintln!("{}: {}",name,record.get("error").map_or("audited".to_owned(),|v|v.to_string()));runs.push(record);
  fs::write(root.join(".scratch/decomp-strategy/retained-live-coverage-audit.json"),serde_json::to_vec_pretty(&json!({"schema_version":1,"source_revision":registry["source"]["revision"],"inventory_sha256":registry["source"]["inventory_sha256"],"runs":runs,"live_routines":union}))?)?;
 }
 println!("{} live roots in {} accepted retained runs",union.len(),runs.iter().filter(|v|v["accepted"]==true).count());Ok(())
}
