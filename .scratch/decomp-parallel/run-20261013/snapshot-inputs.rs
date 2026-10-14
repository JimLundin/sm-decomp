//! Freeze listed regular-file inputs; evidence crate owns hashing and storage.
use std::{env, fs, io, path::Path};
fn main() -> io::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    assert_eq!(args.len(), 2, "usage: INPUTS.paths NEW_CAPTURE_DIRECTORY");
    let list = fs::read_to_string(&args[0])?;
    let target = Path::new(&args[1]);
    assert!(!target.exists(), "preserve existing captured inputs");
    fs::create_dir_all(target)?;
    let mut map = String::from("capture_member\tabsolute_source\tbytes\n");
    let mut count = 0;
    let mut total = 0;
    for name in list.lines().filter(|name| !name.is_empty()) {
        let source = Path::new(name);
        assert!(source.is_absolute() && !name.contains('\t'), "input path must be absolute and TSV-safe");
        let metadata = source.symlink_metadata()?;
        assert!(metadata.is_file() && !metadata.file_type().is_symlink(), "listed input must be a regular file");
        let data = fs::read(source)?;
        let member = format!("members/{}", name.trim_start_matches('/'));
        let dest = target.join(&member);
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::write(&dest, &data)?;
        assert_eq!(data, fs::read(source)?, "input changed while snapshotting");
        map.push_str(&format!("{member}\t{name}\t{}\n", data.len()));
        count += 1;
        total += data.len();
    }
    fs::write(target.join("path-map.tsv"), map)?;
    fs::write(target.join("inputs.paths"), list)?;
    println!("Captured {count} regular-file inputs, {total} bytes; no hashing performed outside evidence crate");
    Ok(())
}
