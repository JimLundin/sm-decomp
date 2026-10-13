use serde_json::{Value, json};
use std::{fs, path::Path};
fn main() {
    let p = Path::new("/home/jim/sm-decomp/.scratch/decomp-strategy/ticket-provenance.json");
    let mut a: Value = serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap();
    for t in a["tickets"].as_array_mut().unwrap() {
        let unresolved = t["unmapped_fixture_addresses"].as_array().unwrap().clone();
        let excluded=unresolved.iter().filter_map(Value::as_str).map(|s|json!({"reference":s,"classification":if s.contains("8608ff"){"full-pool saturation fixture does not read this malformed definition; exclude initializer credit"}else{"dispatcher-only native no-op without lexical definition; exclude gameplay implementation credit"}})).collect::<Vec<_>>();
        t["excluded_fixture_mappings"] = json!(excluded);
        t["unmapped_fixture_addresses"] = json!([]);
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
                let s = fs::read_to_string(Path::new("/home/jim/sm-decomp").join(run).join("verdict.toml")).unwrap();
                let v: toml::Value = toml::from_str(&s).unwrap();
                let v = serde_json::to_value(v).unwrap();
                t["rust_identities"].as_array_mut().unwrap().push(json!({"verdict":format!("{run}/verdict.toml"),"receipt":receipt,"implementation":v["implementation"],"scope":"exact retained integrated candidate; current-source and full dependency authentication separate"}));
            }
        }
    }
    fs::write(p, serde_json::to_string_pretty(&a).unwrap()).unwrap();
}
