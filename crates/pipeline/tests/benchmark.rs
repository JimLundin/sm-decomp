//! Frozen-input, seed and capacity refusals through the public benchmark CLI.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
    config: toml::Value,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn hash(path: &Path) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}
fn environment_digest() -> String {
    let values = std::env::vars()
        .filter(|(name, _)| {
            name.starts_with("CARGO_")
                || name.starts_with("RUST")
                || name.starts_with("CC_")
                || [
                    "CC",
                    "AR",
                    "CFLAGS",
                    "CPPFLAGS",
                    "LDFLAGS",
                    "PATH",
                    "CPATH",
                    "C_INCLUDE_PATH",
                    "CPLUS_INCLUDE_PATH",
                    "LIBRARY_PATH",
                    "SOURCE_DATE_EPOCH",
                    "GCC_EXEC_PREFIX",
                    "COMPILER_PATH",
                    "SDKROOT",
                    "MACOSX_DEPLOYMENT_TARGET",
                ]
                .contains(&name.as_str())
        })
        .collect::<BTreeMap<_, _>>();
    format!("{:x}", Sha256::digest(toml::to_string(&values).unwrap().as_bytes()))
}
fn save<T: Serialize>(path: &Path, value: &T) {
    fs::write(path, toml::to_string(value).unwrap()).unwrap();
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sm-benchmark-control-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        assert!(Command::new("git").args(["init", "--quiet"]).arg(&root).status().unwrap().success());
        let family = root.join("system.toml");
        save(&family, &toml::toml! {family="system"});
        let accepted = root.join("accepted.toml");
        fs::write(&accepted, "frozen predecessor").unwrap();
        let rng = root.join("rng.toml");
        fs::write(&rng, "frozen RNG").unwrap();
        let source = root.join("source.rs");
        fs::write(&source, "fn frozen() {}\n").unwrap();
        let pipeline = Path::new(env!("CARGO_BIN_EXE_sm-pipeline"));
        let mut command = Command::new(pipeline)
            .arg("cache")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let request = toml::toml! {
            root=(root.to_str().unwrap())
            checkout=(root.to_str().unwrap())
            command=["rustc","--version"]
            inputs=[(source.to_str().unwrap())]
            destination=(root.join("artifact").to_str().unwrap().to_owned())
        };
        let bytes = toml::to_string(&request).unwrap().into_bytes();
        command.stdin.as_mut().unwrap().write_all(&(bytes.len() as u32).to_le_bytes()).unwrap();
        command.stdin.as_mut().unwrap().write_all(&bytes).unwrap();
        let mut header = [0; 4];
        command.stdout.as_mut().unwrap().read_exact(&mut header).unwrap();
        let mut reply = vec![0; u32::from_le_bytes(header) as usize];
        command.stdout.as_mut().unwrap().read_exact(&mut reply).unwrap();
        let reply: toml::Value = toml::from_str(std::str::from_utf8(&reply).unwrap()).unwrap();
        fs::write(reply["artifact"].as_str().unwrap(), b"native cache control").unwrap();
        command.stdin.as_mut().unwrap().write_all(b"commit\n").unwrap();
        command.stdin.take();
        let output = command.wait_with_output().unwrap();
        assert!(output.status.success());
        let cache = root.join("build-cache");
        // Copy only immutable public cache artifacts; lock files stay outside the seed.
        let seed = root.join("cache-seed");
        fs::create_dir(&seed).unwrap();
        let mut cache_inventory = BTreeMap::new();
        for entry in fs::read_dir(cache).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                let target = seed.join(path.file_name().unwrap());
                fs::create_dir(&target).unwrap();
                for name in ["artifact", "seal.toml"] {
                    let target = target.join(name);
                    fs::copy(path.join(name), &target).unwrap();
                    cache_inventory
                        .insert(target.strip_prefix(&seed).unwrap().to_str().unwrap().to_owned(), hash(&target));
                }
            }
        }
        let store = root.join("store");
        let mut writer = evidence::Store::create(&store).unwrap();
        writer.put(b'B', b"real sealed store control").unwrap();
        writer.commit().unwrap();
        let store_inventory = fs::read_dir(&store)
            .unwrap()
            .map(|e| {
                let p = e.unwrap().path();
                (p.file_name().unwrap().to_str().unwrap().to_owned(), hash(&p))
            })
            .collect::<BTreeMap<_, _>>();
        for n in [2, 4] {
            let workers = root.join(format!("capacity-{n}/workers"));
            fs::create_dir_all(&workers).unwrap();
            save(&workers.join("capacity.toml"), &toml::toml! {workers=n});
        }
        let frozen = [pipeline.to_path_buf(), source.clone(), family.clone(), accepted.clone(), rng.clone()]
            .into_iter()
            .map(|p| (p.to_str().unwrap().to_owned(), hash(&p)))
            .collect::<BTreeMap<_, _>>();
        let config = toml::toml! {
            format="pipeline-benchmark-v1"
            pipeline=(pipeline.to_str().unwrap())
            output=(root.join("output").to_str().unwrap().to_owned())
            cache_seed=(seed.to_str().unwrap())
            store_seed=(store.to_str().unwrap())
            seed_verdicts=[(root.join("seed-1000.toml").to_str().unwrap().to_owned()),(root.join("seed-45000.toml").to_str().unwrap().to_owned())]
            source_revision="control"
            environment_sha256=(environment_digest())
            max_rss_bytes=8589934592_i64
            max_private_staging_bytes=68719476736_i64
            minimum_free_bytes=53687091200_i64
            [request]
            checkout=(root.to_str().unwrap())
            accepted=(accepted.to_str().unwrap())
            rng_predecessor=(rng.to_str().unwrap())
            family_verdicts=[(family.to_str().unwrap())]
            [capacity_roots]
            "2"=(root.join("capacity-2").to_str().unwrap().to_owned())
            "4"=(root.join("capacity-4").to_str().unwrap().to_owned())
        };
        let mut config = toml::Value::Table(config);
        config.as_table_mut().unwrap().insert("frozen".into(), toml::Value::try_from(frozen).unwrap());
        config
            .as_table_mut()
            .unwrap()
            .insert("cache_inventory".into(), toml::Value::try_from(cache_inventory).unwrap());
        config
            .as_table_mut()
            .unwrap()
            .insert("store_inventory".into(), toml::Value::try_from(store_inventory).unwrap());
        Self { root, config }
    }
    fn refusal(&self, expected: &str) {
        let path = self.root.join("config.toml");
        save(&path, &self.config);
        let output = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .args(["benchmark", "validate"])
            .arg(path)
            .env_remove("SM_PIPELINE_METRICS_CONTEXT")
            .output()
            .unwrap();
        assert!(!output.status.success(), "invalid benchmark admitted");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!self.root.join("output/summary.toml").exists());
    }
}
#[test]
fn changed_frozen_source_is_refused_before_sampling() {
    let f = Fixture::new();
    fs::write(f.root.join("source.rs"), "changed source").unwrap();
    f.refusal("frozen input changed");
}
#[test]
fn changed_store_seed_inventory_is_refused_before_sampling() {
    let f = Fixture::new();
    let p = fs::read_dir(f.root.join("store")).unwrap().next().unwrap().unwrap().path();
    let mut b = fs::read(&p).unwrap();
    b[9] ^= 1;
    fs::write(p, b).unwrap();
    f.refusal("store seed inventory changed");
}
#[test]
fn missing_native_cache_entry_is_refused_before_sampling() {
    let f = Fixture::new();
    let p = fs::read_dir(f.root.join("cache-seed")).unwrap().next().unwrap().unwrap().path();
    fs::remove_dir_all(p).unwrap();
    f.refusal("empty seed inventory");
}
#[test]
fn wrong_capacity_root_is_refused_before_sampling() {
    let f = Fixture::new();
    save(&f.root.join("capacity-4/workers/capacity.toml"), &toml::toml! {workers=2});
    f.refusal("wrong capacity root");
}
#[test]
fn corrupt_cache_artifact_with_rebound_inventory_still_fails_its_seal() {
    let mut f = Fixture::new();
    let p = fs::read_dir(f.root.join("cache-seed")).unwrap().next().unwrap().unwrap().path().join("artifact");
    fs::write(&p, b"corrupt").unwrap();
    let key = p.strip_prefix(f.root.join("cache-seed")).unwrap().to_str().unwrap();
    f.config["cache_inventory"].as_table_mut().unwrap().insert(key.into(), hash(&p).into());
    f.refusal("cache seed seal differs");
}

#[test]
fn corrupt_store_with_rebound_inventory_still_fails_physical_pack_authentication() {
    let mut f = Fixture::new();
    let p = fs::read_dir(f.root.join("store")).unwrap().next().unwrap().unwrap().path();
    let mut bytes = fs::read(&p).unwrap();
    bytes[8] ^= 1;
    fs::write(&p, bytes).unwrap();
    let name = p.file_name().unwrap().to_str().unwrap();
    f.config["store_inventory"].as_table_mut().unwrap().insert(name.into(), hash(&p).into());
    f.refusal("physical pack seal differs");
}
