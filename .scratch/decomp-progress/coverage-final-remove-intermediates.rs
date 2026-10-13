//! Removes only explicitly identified disposable audit/browser outputs.
use std::{fs, path::Path};
fn bytes(p:&Path)->std::io::Result<u64>{let m=p.symlink_metadata()?;if m.is_dir(){fs::read_dir(p)?.try_fold(0,|n,e|Ok(n+bytes(&e?.path())?))}else{Ok(m.len())}}
fn main()->std::io::Result<()> {
    let paths=[
        "/tmp/sm-coverage-compile", "/tmp/sm-coverage-scan",
        "/tmp/sm-progress-context-file-check", "/tmp/sm-progress-context-file-check-v2",
        "/tmp/sm-progress-context-inspection", "/tmp/sm-progress-context-inspection-v2",
        "/tmp/sm-progress-context-inspection-v3", "/tmp/sm-progress-context-served-inspection",
        "/tmp/sm-progress-accuracy-final-offline", "/tmp/sm-progress-accuracy-final-served",
        "/home/jim/sm-decomp/.scratch/decomp-progress/layout-review/before/chromium-profile",
        "/tmp/sm-coverage-reconciliation/source",
    ];
    let mut total=0;
    for name in paths {let p=Path::new(name);if !p.exists(){continue}let n=bytes(p)?;if p.is_dir(){fs::remove_dir_all(p)?}else{fs::remove_file(p)?}println!("{n} bytes removed: {name}");total+=n;}
    println!("total_unsealed_file_bytes={total}");
    Ok(())
}
