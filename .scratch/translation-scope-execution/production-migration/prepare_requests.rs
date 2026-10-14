use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
fn reference(p: &Path) -> Value {
    let out = Command::new("sha256sum").arg(p).output().unwrap();
    assert!(out.status.success());
    let hash = String::from_utf8(out.stdout).unwrap().split_whitespace().next().unwrap().to_owned();
    json!({"path":p,"sha256":hash,"bytes":fs::metadata(p).unwrap().len()})
}
fn main() {
    let architecture = Path::new("/home/jim/sm-decomp-translation-execution");
    let effort = architecture.join(".scratch/translation-scope-execution");
    let root = effort.join("production-migration");
    let old: Value = serde_json::from_slice(
        &fs::read(effort.join("production-preparation/ceres-actual-c-02.request.json")).unwrap(),
    )
    .unwrap();
    for (name, ledger, claims) in [
        (
            "ceres-production-01",
            "ceres-continuation",
            vec!["compact-controller", "shared-game-host", "integrate-shared-game-host"],
        ),
        ("canonical-production-01", "decomp-parallel", vec!["pause-equipment-render-offline"]),
    ] {
        let request = json!({"schema":"translation-migration-build-v1","checkout":"/home/jim/sm-decomp","effort":ledger,"claims":claims.iter().map(|id|if *id=="integrate-shared-game-host" {json!({"id":id,"source_commit":"c2cae9ac145f9343cd345e6f978bff0774510b56"})}else{json!({"id":id})}).collect::<Vec<_>>(),"stopped_owners":claims.iter().map(|id|reference(&root.join("inspections").join(format!("{id}-stop.json")))).collect::<Vec<_>>(),"proof_installation":old["proof_installation"],"proof_completion":old["proof_completion"],"output_root":effort.join("production-preparation").join(name),"limits":old["limits"]});
        let target = root.join(format!("{name}.request.json"));
        let data = serde_json::to_vec_pretty(&request).unwrap();
        if target.exists() { assert_eq!(fs::read(target).unwrap(), data) } else { fs::write(target, data).unwrap() }
    }
}
