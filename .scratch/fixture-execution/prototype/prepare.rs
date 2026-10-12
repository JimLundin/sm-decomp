//! THROWAWAY: reuse authenticated C object bytes; compile only generated hosts.
use std::{fs,path::PathBuf,process::Command};
fn main(){
    let run=PathBuf::from("/tmp/sm-pipeline-upgrade-integration/out/ticket-03/final-projectile/1791813609706347-2-0");
    let output=PathBuf::from("/tmp/sm-fixture-execution-prototype/native");fs::create_dir_all(&output).unwrap();
    let cache=fs::read_to_string(run.join("c-cache.toml")).unwrap();let mut objects=vec![];
    for line in cache.lines().filter(|line|line.starts_with("key = ")) {
        let key=line.split('"').nth(1).unwrap();
        let entry=PathBuf::from("/tmp/sm-pipeline-upgrade-workers-2/build-cache").join(key);
        let seal=fs::read_to_string(entry.join("seal.toml")).unwrap();
        let expected=seal.lines().next().unwrap().split('"').nth(1).unwrap();
        let hash=Command::new("sha256sum").arg(entry.join("artifact")).output().unwrap();
        assert!(hash.status.success());assert_eq!(String::from_utf8(hash.stdout).unwrap().split_whitespace().next().unwrap(),expected);
        let object=output.join(format!("cached-{}.o",objects.len()));fs::copy(entry.join("artifact"),&object).unwrap();objects.push(object);
    }
    let oracle=fs::read_to_string(run.join("oracle.c")).unwrap();
    let start=oracle.find(" FILE *rf=").unwrap();let end=start+oracle[start..].find("g_rom=rom;").unwrap()+"g_rom=rom;".len();
    let mut prepared=oracle.clone();prepared.replace_range(start..end,"");prepared=prepared.replace("int main(","int PortPrototypeInvoke(");
    assert!(!prepared.contains("fopen(argv[6]"));
    fs::write(output.join("oracle-preloaded.c"),prepared).unwrap();
    for (label,source) in [("shim",run.join("plm-shim.c")),("host",run.join("host-support.c")),("oracle",output.join("oracle-preloaded.c"))]{
        let object=output.join(format!("{label}.o"));
        let result=Command::new("cc").args(["-O2","-fno-strict-aliasing","-ffunction-sections","-fdata-sections","-iquote","/home/jim/sm-decomp/deps/snesrev-sm/src","-c"]).arg(source).arg("-o").arg(&object).output().unwrap();
        assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));objects.push(object);
    }
    let result=Command::new("ar").arg("crs").arg(output.join("libfixture_oracle.a")).args(&objects).status().unwrap();assert!(result.success());
    println!("Prepared {} sealed/generated objects",objects.len());
}
