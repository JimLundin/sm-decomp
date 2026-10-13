#[path="/tmp/sm-ridley-instructions/crates/pipeline/src/family_model.rs"]
mod family_model;
#[path="/tmp/sm-ridley-instructions/crates/pipeline/src/ridley_instruction_fixtures.rs"]
mod fixtures;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::Path;
fn digest(path:&Path)->Result<String>{ Ok(format!("{:x}",Sha256::digest(std::fs::read(path)?))) }
fn atomic_toml<T:serde::Serialize>(path:&Path,value:&T)->Result<()>{std::fs::write(path,toml::to_string_pretty(value)?)?;Ok(())}
fn main()->Result<()> {
    let started=std::time::Instant::now();
    let report=fixtures::generate_captured(
      Path::new("/tmp/sm-ridley-instructions"),
      Path::new("/home/jim/sm-decomp/rom/sm.sfc"),
      Path::new("/home/jim/sm-decomp/out/port/ridley-instructions/inventory/1791838371843885184-2"),
      Path::new("/tmp/sm-ridley-instructions/tools/port/observed-ridley-instructions-captured.toml"),
    )?;
    std::fs::write("/tmp/sm-ridley-captured-generation.json",serde_json::to_vec_pretty(&report)?)?;
    let rom=std::fs::read("/home/jim/sm-decomp/rom/sm.sfc")?;
    let mut results=Vec::new();
    for capture in &report.captures {
        let mut bytes=std::fs::read(&capture.before_wram)?;
        let mut host=port::ridley_enemy_instructions::ProjectileHost::from_words(capture.before.host);
        let mut ram=port::Wram::new(&mut bytes).map_err(anyhow::Error::msg)?;
        let a=if capture.before.kind=="opcode"{i32::from(capture.before.slot)}else{0};
        let b=if capture.before.kind=="opcode"{capture.before.argument as i32}else{0};
        anyhow::ensure!(port::ridley_enemy_instructions::supports_call(&ram,&rom,&capture.symbol,a,b).map_err(anyhow::Error::msg)?,"natural capture refused: {}",capture.name);
        let returned=if capture.before.kind=="frame" {
            let executed=port::ridley_enemy_instructions::process_instructions(&mut ram,&rom,&mut host,4096).map_err(anyhow::Error::msg)?;
            anyhow::ensure!(executed.opcodes.len()==capture.opcodes.len(),"opcode extent differs: {}",capture.name);
            for (rust,native) in executed.opcodes.iter().zip(&capture.opcodes){
                let kind=match rust.pointer_kind {port::ridley_enemy_instructions::PointerKind::Stop=>0,port::ridley_enemy_instructions::PointerKind::BankOffset=>1,port::ridley_enemy_instructions::PointerKind::Rom=>2};
                anyhow::ensure!((rust.address,rust.argument,rust.returned,kind)==(native.opcode,native.argument,native.returned,native.return_kind),"opcode identity/order/pointer kind differs: {}",capture.name);
            }
            0
        } else {
            port::ridley_enemy_instructions::dispatch_host(&mut ram,&rom,&mut host,&capture.symbol,a,b).map_err(anyhow::Error::msg)?
        };
        let expected=std::fs::read(&capture.after_wram)?;
        anyhow::ensure!(bytes==expected,"natural full WRAM differs: {}, first {:?}",capture.name,bytes.iter().zip(&expected).position(|(a,b)|a!=b));
        anyhow::ensure!(host.words()==capture.after.host,"natural eight-word host differs: {}",capture.name);
        anyhow::ensure!(returned==if capture.before.kind=="opcode" {capture.after.returned}else{0},"natural return differs: {}",capture.name);
        results.push(serde_json::json!({"capture":capture.name,"symbol":capture.symbol,"frame":capture.before.frame,"wram":"matches full 131072 bytes","host":"matches all eight captured words","return":returned,"frame_opcode_identity_checked":capture.before.kind=="frame","frame_opcode_count":if capture.before.kind=="frame"{capture.opcodes.len()}else{0}}));
    }
    let verification=serde_json::json!({"status":"captured-rust-matches-native","live_exercised":false,"original_rom_parity":"not tested","module":"/tmp/sm-ridley-instructions/crates/port/src/ridley_enemy_instructions.rs","module_sha256":digest(Path::new("/tmp/sm-ridley-instructions/crates/port/src/ridley_enemy_instructions.rs"))?,"main_module_sha256":digest(Path::new("/home/jim/sm-decomp/crates/port/src/ridley_enemy_instructions.rs"))?,"inventory":report.inventory,"inventory_verdict_sha256":report.inventory_verdict_sha256,"authenticated_artifacts":report.authenticated_artifacts,"manifest":report.manifest,"manifest_sha256":report.manifest_sha256,"windows":report.windows,"captures":results,"seconds":started.elapsed().as_secs_f64()});
    std::fs::write("/home/jim/sm-decomp/.scratch/decomp-port/parallel-138-scope-natural-capture-verification.json",serde_json::to_vec_pretty(&verification)?)?;
    println!("{} natural fixtures; {} artifacts authenticated",report.captures.len(),report.authenticated_artifacts);
    Ok(())
}
