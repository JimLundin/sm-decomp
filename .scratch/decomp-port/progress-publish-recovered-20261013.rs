use std::{fs, path::Path, process::Command};

const REVISION: &str = "578f90b3cc49557bb70060ad033bb90b8cf8ac50";
const INVENTORY: &str = "7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25";

fn publish(repo: &Path, id: &str, scope: &str, agent: &str, stage: &str, routines: &[String], commit: Option<&str>, limitations: &[&str], evidence: &[&str]) {
    let base = repo.join(".scratch/decomp-port");
    let path = base.join(format!("progress-{id}.toml"));
    let mut update = format!("id = {id:?}\nscope = {scope:?}\nagent = {agent:?}\nstage = {stage:?}\nsource_revision = {REVISION:?}\ninventory_sha256 = {INVENTORY:?}\nroutines = {routines:?}\nlimitations = {limitations:?}\n");
    if let Some(commit) = commit { update.push_str(&format!("commit = {commit:?}\n")); }
    for artifact in evidence { let artifact=repo.join(artifact);assert!(artifact.is_file(),"missing evidence {}",artifact.display());update.push_str(&format!("\n[[evidence]]\npath = {:?}\n",artifact.to_string_lossy())); }
    fs::write(&path, update).unwrap();
    let result=Command::new(repo.join("target/debug/sm-pipeline")).args(["progress","update","--root"]).arg(repo).arg("--update").arg(&path).output().unwrap();
    fs::write(base.join(format!("progress-{id}.stdout")), &result.stdout).unwrap();
    fs::write(base.join(format!("progress-{id}.stderr")), &result.stderr).unwrap();
    assert!(result.status.success(),"{id}: {}",String::from_utf8_lossy(&result.stderr));
    println!("published {stage} for {} routines: {id}",routines.len());
}

fn main() {
    let repo=Path::new("/home/jim/sm-decomp");
    let identity=Command::new("jq").args(["-r","[.source.revision,.source.inventory_sha256,.source.definition_count|tostring]|join(\"\\n\")"]).arg(repo.join(".scratch/decomp-progress/registry.json")).output().unwrap();
    assert!(identity.status.success());
    assert_eq!(String::from_utf8(identity.stdout).unwrap(),format!("{REVISION}\n{INVENTORY}\n6053\n"));
    let closure=fs::read_to_string(repo.join(".scratch/decomp-port/ridley-whole-source-closure.txt")).unwrap();
    let whole=closure.split_once("Native closure:\n").unwrap().1.split_once("External dependencies:\n").unwrap().0.lines().map(str::to_owned).collect::<Vec<_>>();
    assert_eq!(whole.len(),220);
    let common=[".scratch/decomp-port/ridley-whole-source-closure.txt",".scratch/decomp-port/progress-recovered-stage-evidence-20261013.md",".scratch/decomp-port/progress-ridley-recovery-completion-20261013.md",".scratch/decomp-port/progress-ridley-recovered-source-20261013.sha256","out/port/ridley-subsystem/recovery-inventory/1791858734336387661-2/verdict.toml",".scratch/decomp-port/ridley-recovery-inventory-final-authentication.toml"];
    let limitations=["Recovered source covers the complete 220-definition Ridley/Ceres closure but is uncommitted; exact pending source hashes and baseline are retained.","The authenticated actual-C inventory is diagnostic provenance with no Rust live acceptance. Controlled 6,457 pairs cover 33 native entries and do not establish verified status for all 220 definitions.","Natural captured supplement, merged current-source family gates, the whole lifecycle live seam and final commit/cleanup remain the integration coordinator's work.","External C services remain explicit dependencies; public admission refuses SpawnEnemy/EnemyItemDrop lifecycle continuations pending modeled footprints. Original-ROM parity is not tested."];
    publish(repo,"ridley-whole-6053-recovered-translating-20261013","Whole Ridley and Ceres lifecycle, AI, tail, rendering, cutscene and native scripts","/root/ridley_recovery","translating",&whole,None,&limitations,&common);
    let mut translated=common.to_vec();translated.extend(["out/port/ridley-subsystem/recovery-controlled/1791858747222202-2-0/verdict.toml",".scratch/decomp-port/ridley-recovery-controlled-authentication.toml"]);
    publish(repo,"ridley-whole-6053-recovered-translated-20261013","Whole Ridley and Ceres lifecycle, AI, tail, rendering, cutscene and native scripts","/root/ridley_recovery","translated",&whole,None,&limitations,&translated);
    let trails=["HandleProjectileTrails","ProjInstr_MoveLeftProjectileTrailDown","ProjInstr_MoveRightProjectileTrailDown","ProjInstr_MoveLeftProjectileTrailUp"].map(str::to_owned);
    let trails_limits=["Reissued after source inventory expansion; prior observations remain historical. Main commit has the same candidate SHA-256 as the recovered frozen actual-C contexts.","Frozen-context verification authenticates 3,316 controlled cases across four routines and 14 naturally active whole-frame cases after cleanup; it does not reseal all current checkout dependencies.","Movement opcodes were verified offline and had no hits in the 9,000-input natural prefix. Creation/placement reuse projectile_flight.","Current merged admission, combined four-process live evidence and integrated worktree cleanup remain pending coordinator receipts. No integrated/live stage is claimed here."];
    let trails_evidence=[".scratch/decomp-port/progress-recovered-stage-evidence-20261013.md","out/port/projectile-trails/offline/1791838049353069-2-0/verdict.toml","out/port/projectile-trails/natural-offline/1791838495334191-2-0/verdict.toml",".scratch/decomp-port/recovery-trails-controlled-after-cleanup-authentication.toml",".scratch/decomp-port/recovery-trails-natural-after-cleanup-authentication.toml"];
    for stage in ["translated","verified"] {publish(repo,&format!("projectile-trails-6053-recovered-{stage}-20261013"),"Complete native projectile trail script and render frame","/root/independent_gameplay",stage,&trails,Some("2b62134bfbee7c5dcd4c81616edc5d91e781d44f"),&trails_limits,&trails_evidence);}
    let samus=["HandleProjectile","DrawPlayerExplosions2","sub_9382FD","DrawBombAndProjectileExplosions"].map(str::to_owned);
    let samus_limits=["Reissued after source inventory expansion; prior observations remain historical. Main commit has the same candidate SHA-256 as the recovered frozen actual-C context.","Frozen-context verification authenticates 476 whole-frame cases across four scheduled boundaries after cleanup; it does not reseal all current checkout dependencies.","PLM/HDMA services remain explicit C dependencies; the pinned C uninitialized-local Ceres drawing branches conservatively refuse. Diagnostic failures remain retained.","Current merged admission, combined four-process live evidence and integrated worktree cleanup remain pending coordinator receipts. No integrated/live stage is claimed here."];
    let samus_evidence=[".scratch/decomp-port/progress-recovered-stage-evidence-20261013.md","out/port/samus-projectile-frame/offline/1791841155185563-2-0/verdict.toml",".scratch/decomp-port/recovery-samus-after-cleanup-authentication.toml"];
    for stage in ["translated","verified"] {publish(repo,&format!("samus-projectile-frame-6053-recovered-{stage}-20261013"),"Complete scheduled Samus projectile update and render passes","/root/independent_gameplay",stage,&samus,Some("c711faea56f20d42113eb63393cc4a76d4940557"),&samus_limits,&samus_evidence);}
    fs::remove_file(std::env::current_exe().unwrap()).unwrap();
}
