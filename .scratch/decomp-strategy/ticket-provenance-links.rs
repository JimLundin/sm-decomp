use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};
fn main() {
    let root = Path::new("/home/jim/sm-decomp");
    let p = root.join(".scratch/decomp-strategy/ticket-provenance.json");
    let mut a: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    for t in a["tickets"].as_array_mut().unwrap() {
        if t["id"] == 138 {
            for (run, receipt) in [
                (
                    "out/recovery-session/integrated/families/observed-ridley-instructions/evidence/1791860412408415-47409-101",
                    ".scratch/decomp-port/recovery-integrated-observed-ridley-instructions-post-cleanup-authentication.toml",
                ),
                (
                    "out/recovery-session/integrated/families/observed-ridley-instructions-captured/evidence/1791860472523604-47409-112",
                    ".scratch/decomp-port/recovery-integrated-observed-ridley-instructions-captured-post-cleanup-authentication.toml",
                ),
            ] {
                for (path, kind) in [
                    (format!("{run}/verdict.toml"), "verdict"),
                    (format!("{run}/manifest.toml"), "manifest"),
                    (receipt.into(), "authentication"),
                ] {
                    t["evidence"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"path":path,"exists":root.join(&path).exists(),"kind":kind}));
                }
                t["passing_verdicts"].as_array_mut().unwrap().push(json!(format!("{run}/verdict.toml")));
            }
            t["review_notes"].as_array_mut().unwrap().push(json!("Seven actual Ridley attack opcode roots and ProcessEnemyInstructions join integrated controlled1804pairs plus90captured pairs through named post-cleanup receipts; generic historical14 instruction roots retain their own older fixture receipt."));
        }
    }
    let mut mappings = BTreeMap::new();
    for t in a["tickets"].as_array().unwrap() {
        for m in t["authenticated_synthetic_dispatch_mappings"].as_array().unwrap() {
            if t["routines"].as_array().unwrap().contains(&m["symbol"]) {
                let mut k = m.clone();
                for f in ["fixture_case_count", "fixture_case_examples"] {
                    k.as_object_mut().unwrap().remove(f);
                }
                mappings.insert(k.to_string(), m.clone());
            }
        }
    }
    fs::write(root.join(".scratch/decomp-strategy/ticket-authenticated-internal-mappings.json"),serde_json::to_string_pretty(&json!({"schema_version":1,"source_revision":"578f90b3cc49557bb70060ad033bb90b8cf8ac50","basis":"exact synthetic native mapping joined to118passing artifact receipts or explicit separate post-cleanup ticket receipt","mappings":mappings.into_values().collect::<Vec<_>>()})).unwrap()).unwrap();
    fs::write(p, serde_json::to_string_pretty(&a).unwrap()).unwrap();
}
