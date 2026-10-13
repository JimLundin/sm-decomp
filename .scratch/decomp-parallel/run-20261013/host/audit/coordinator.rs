use std::{collections::{BTreeMap, BTreeSet}, fs, path::Path, process::{Command, Stdio}};

fn document(path: impl AsRef<Path>) -> toml::Value {
    toml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base = Path::new("/home/jim/sm-decomp");
    match args[1].as_str() {
        "compare-inputs" => {
            let inventory = document(&args[2]);
            let declarations = document(&args[3]);
            let artifacts = inventory["artifacts"].as_table().unwrap();
            let mut compared = Vec::new();
            let mut matching = 0;
            let mut changed = 0;
            for entry in declarations["entries"].as_array().unwrap() {
                let path = entry["path"].as_str().unwrap();
                let key = format!("members/{}", path.strip_prefix('/').unwrap());
                let actual = artifacts.get(&key).expect("declared member missing").as_str().unwrap();
                let historical = entry["sha256"].as_str().unwrap();
                let matches = actual == historical;
                if matches { matching += 1; } else { changed += 1; }
                compared.push(toml::Value::Table(toml::Table::from_iter([
                    ("path".into(), path.into()), ("historical_sha256".into(), historical.into()),
                    ("captured_sha256".into(), actual.into()), ("matches".into(), matches.into()),
                    ("mutable_current_input".into(), entry.get("mutable_current_input").cloned().unwrap_or(false.into())),
                ])));
            }
            let output = toml::Value::Table(toml::Table::from_iter([
                ("schema".into(), "decomp-current-input-comparison-v1".into()),
                ("matching".into(), matching.into()), ("changed".into(), changed.into()),
                ("entries".into(), toml::Value::Array(compared)),
                ("limitations".into(), toml::Value::Array(vec!["Current capture authenticates observed bytes. Changed mutable records do not rewrite historical research claims.".into()])),
            ]));
            fs::write(&args[4], toml::to_string_pretty(&output).unwrap()).unwrap();
            println!("matching={matching} changed={changed}");
        }
        "batch-audit" => {
            let batch = document(&args[2]);
            let registry: serde_json::Value = serde_json::from_str(&fs::read_to_string(&args[3]).unwrap()).unwrap();
            let known: BTreeSet<&str> = registry["routines"].as_array().unwrap().iter().filter_map(|r| r["symbol"].as_str()).collect();
            let mut counts = BTreeMap::<String, usize>::new();
            let mut symbols = BTreeSet::new();
            let mut pairs = 0;
            let mut passed = Vec::new();
            for (name, record) in batch["families"].as_table().unwrap() {
                *counts.entry(record["status"].as_str().unwrap().into()).or_default() += 1;
                if record["status"].as_str() != Some("offline-verified") { continue; }
                let verdict = document(record["verdict"].as_str().unwrap());
                assert_eq!(verdict["status"].as_str(), Some("offline-verified"));
                let mut roots = BTreeSet::new();
                for case in verdict["cases"].as_array().unwrap() {
                    if case.get("status").and_then(toml::Value::as_str).is_none_or(|v| v == "passed") {
                        let symbol = case["routine"].as_str().unwrap();
                        if known.contains(symbol) { roots.insert(symbol.to_owned()); symbols.insert(symbol.to_owned()); }
                        pairs += 1;
                    }
                }
                passed.push(toml::Value::Table(toml::Table::from_iter([
                    ("family".into(), name.clone().into()), ("verdict".into(), record["verdict"].clone()),
                    ("routines".into(), toml::Value::Array(roots.into_iter().map(Into::into).collect())),
                    ("verified_cases".into(), verdict["verified_cases"].clone()),
                ])));
            }
            println!("statuses={counts:?} unique_actual_c_roots={} fixture_pairs={pairs}", symbols.len());
            let output = toml::Value::Table(toml::Table::from_iter([
                ("schema".into(), "decomp-stopped-batch-audit-v1".into()),
                ("counts".into(), toml::Value::try_from(counts).unwrap()),
                ("actual_c_roots".into(), toml::Value::Array(symbols.into_iter().map(Into::into).collect())),
                ("passed".into(), toml::Value::Array(passed)),
            ]));
            fs::write(&args[4], toml::to_string_pretty(&output).unwrap()).unwrap();
        }
        "authenticate" => {
            let audit = document(&args[2]);
            let output = Path::new(&args[3]);
            fs::create_dir_all(output).unwrap();
            for entry in audit["passed"].as_array().unwrap() {
                let family = entry["family"].as_str().unwrap();
                let stdout = fs::File::create(output.join(format!("{family}.stdout.toml"))).unwrap();
                let stderr = fs::File::create(output.join(format!("{family}.stderr.txt"))).unwrap();
                let result = Command::new(&args[4]).current_dir(base)
                    .env("SM_PIPELINE_ROOT", base.join("out/pipeline")).env("SM_PIPELINE_WORKERS", "4")
                    .args(["family-authenticate", "--verdict", entry["verdict"].as_str().unwrap()])
                    .stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr)).status().unwrap();
                println!("{family}: {}", if result.success() { "authenticated" } else { "rejected" });
            }
        }
        "minimize" => {
            let mut manifest = document(&args[2]);
            manifest["routines"].as_array_mut().unwrap().truncate(1);
            manifest["routines"][0]["cases"].as_array_mut().unwrap().truncate(1);
            fs::write(&args[3], toml::to_string_pretty(&manifest).unwrap()).unwrap();
        }
        "probe-alias" => {
            let run = Path::new(&args[2]);
            fs::create_dir_all(run).unwrap();
            let original = fs::read_to_string(&args[3]).unwrap();
            let declaration = "#[path=\"/home/jim/sm-decomp/crates/port/src/enemy_projectiles.rs\"] mod family;";
            assert_eq!(original.matches(declaration).count(), 1);
            let patched = original.replace(declaration, "use memory::enemy_projectiles as family;");
            fs::write(run.join("wrapper.rs"), patched).unwrap();
            let request = toml::Value::Table(toml::Table::from_iter([
                ("command".into(), toml::Value::Array(vec!["rustc".into(), "--edition=2024".into(), "-O".into(),
                    run.join("wrapper.rs").display().to_string().into(), "-o".into(), run.join("rust").display().to_string().into()])),
                ("run".into(), run.display().to_string().into()), ("label".into(), "rust-alias-probe".into()),
                ("root".into(), "/home/jim/sm-decomp/out/pipeline".into()), ("workers".into(), 4.into()), ("timeout".into(), 60.into()),
            ]));
            fs::write(&args[4], toml::to_string_pretty(&request).unwrap()).unwrap();
        }
        "probe-support" => {
            let mut manifest = document(&args[2]);
            manifest["routines"].as_array_mut().unwrap().truncate(1);
            manifest["routines"][0]["cases"].as_array_mut().unwrap().truncate(1);
            let support = manifest["c_support"].as_array_mut().unwrap();
            for file in &args[4..] {
                if !support.iter().any(|entry| entry.as_str() == Some(file)) { support.push(file.clone().into()); }
            }
            fs::write(&args[3], toml::to_string_pretty(&manifest).unwrap()).unwrap();
        }
        _ => panic!("unsupported action"),
    }
}
