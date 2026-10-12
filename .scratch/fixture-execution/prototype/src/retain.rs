//! Retain and authenticate the bounded experiment; no production admission.
use anyhow::{Result,ensure};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,fs,path::{Path,PathBuf}};
const ROOT:&str="/tmp/sm-fixture-execution-prototype";
fn hash(p:&Path)->Result<String>{Ok(format!("{:x}",Sha256::digest(fs::read(p)?)))}
fn walk(root:&Path,files:&mut Vec<PathBuf>)->Result<()>{for entry in fs::read_dir(root)?{let p=entry?.path();if p.is_dir(){walk(&p,files)?;}else{files.push(p);}}Ok(())}
fn main()->Result<()> {
    let root=Path::new(ROOT);let operation=std::env::args().nth(1).unwrap_or_else(||"retain".into());
    if operation=="authenticate" {
        let inventory:BTreeMap<String,String>=toml::from_str(&fs::read_to_string(root.join("retained-sha256.toml"))?)?;
        for (path,expected) in &inventory{ensure!(hash(Path::new(path))?==*expected,"changed retained file: {path}");}
        println!("Authenticated {} retained prototype files",inventory.len());return Ok(());
    }
    let mut inventory=BTreeMap::new();let mut shared=BTreeMap::new();let mut reclaimed=0u64;let mut linked=0;
    let mut samples=vec![];
    for entry in fs::read_dir(root)? {
        let run=entry?.path();if !run.is_dir() || !run.file_name().unwrap().to_string_lossy().contains("run-"){continue;}
        let mut files=vec![];walk(&run,&mut files)?;
        for path in files {
            let digest=hash(&path)?;
            if path.file_name().unwrap()=="sample.toml" {
                let value:toml::Value=toml::from_str(&fs::read_to_string(&path)?)?;
                for (fixture,signatures) in value["signatures"].as_table().unwrap(){for (name,expected) in signatures.as_table().unwrap(){ensure!(hash(&path.parent().unwrap().join(fixture).join(name))?==expected.as_str().unwrap(),"sample output changed");}}
                let mut summary=value.as_table().unwrap().clone();summary.remove("signatures");summary.insert("path".into(),path.display().to_string().into());samples.push(toml::Value::Table(summary));
            }
            let name=path.file_name().unwrap().to_string_lossy();
            if name.ends_with(".wram") || name.ends_with(".bin") || name.ends_with(".stdout") || name.ends_with(".stderr") {
                if let Some(first)=shared.get(&digest){let pending=path.with_extension("dedup-pending");fs::hard_link(first,&pending)?;fs::rename(pending,&path)?;ensure!(hash(&path)?==digest,"dedup changed bytes");reclaimed+=fs::metadata(&path)?.len();linked+=1;}
                else{shared.insert(digest.clone(),path.clone());}
            }
            inventory.insert(path.display().to_string(),digest);
        }
    }
    let source=Path::new("/home/jim/sm-decomp/.scratch/fixture-execution/prototype");fs::create_dir_all(source.join("src"))?;fs::create_dir_all(source.join("native"))?;
    for name in ["Cargo.toml","Cargo.lock","build.rs","prepare.rs","src/main.rs","src/helper.rs","src/retain.rs","native/oracle-preloaded.c"]{let src=root.join(name);let dest=source.join(name);fs::copy(&src,&dest)?;inventory.insert(src.display().to_string(),hash(&src)?);inventory.insert(dest.display().to_string(),hash(&dest)?);}
    fs::create_dir_all(root.join("bin"))?;
    for name in ["fixture-prototype","fixture-helper"]{let src=root.join("target/release").join(name);let dest=root.join("bin").join(name);fs::copy(&src,&dest)?;inventory.insert(dest.display().to_string(),hash(&dest)?);}
    for name in ["fixture-prototype","fixture-helper"]{let dep=fs::read_to_string(root.join(format!("target/release/{name}.d")))?;for path in dep.split_once(": ").unwrap().1.split_whitespace(){inventory.insert(path.into(),hash(Path::new(path))?);}}
    for name in ["original-authenticate.toml","native/libfixture_oracle.a"]{let path=root.join(name);inventory.insert(path.display().to_string(),hash(&path)?);}
    let summary=toml::Value::Table(toml::Table::from_iter([("samples".into(),samples.into()),("dedup_logical_bytes".into(),(reclaimed as i64).into()),("dedup_links".into(),linked.into()),("contract".into(),"prototype signatures and source hashes; no production acceptance verdict".into())]));
    fs::write(root.join("summary.toml"),toml::to_string_pretty(&summary)?)?;
    inventory.insert(root.join("summary.toml").display().to_string(),hash(&root.join("summary.toml"))?);
    fs::write(root.join("retained-sha256.toml"),toml::to_string_pretty(&inventory)?)?;
    println!("Retained {} files; shared {} identical fixture files / {} logical bytes",inventory.len(),linked,reclaimed);Ok(())
}
