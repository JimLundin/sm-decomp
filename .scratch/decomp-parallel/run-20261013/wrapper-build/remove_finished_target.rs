use std::{fs, path::Path, process::Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = Path::new("/tmp/sm-decomp-run-20261013-ceres-target");
    let retained = Path::new("/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013");
    assert!(target.is_dir() && !target.is_symlink());
    assert_eq!(fs::canonicalize(target)?, target);
    assert!(retained.join("wrapper-build/source.sha256").is_file());
    assert!(retained.join("wrapper-build/source/original-compiler.d").is_file());
    assert_eq!(fs::read(target.join("release/ceres-assignment-driver"))?,
        fs::read(retained.join("bin/ceres-assignment-driver"))?);
    for name in ["wrapper-cleanup1.toml", "wrapper-cleanup2.toml"] {
        let receipt = fs::read_to_string(retained.join("receipts").join(name))?;
        assert!(receipt.contains("apply = true") && receipt.contains("bytes = 0"));
    }
    let writers = Command::new("lsof").arg("+D").arg(target).output()?;
    assert_eq!(writers.status.code(), Some(1));
    assert!(writers.stdout.is_empty() && writers.stderr.is_empty());
    fs::remove_dir_all(target)?;
    assert!(!target.exists());
    println!("removed finished, retained wrapper target: {}", target.display());
    Ok(())
}
