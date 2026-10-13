use std::{fs, path::Path};
fn main() {
    let package = Path::new("/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013");
    for name in ["main.rs", "package-readiness.rs"] {
        let path = package.join(name);
        let source = fs::read_to_string(&path).unwrap();
        let source = if name == "package-readiness.rs" {
            source.replace("readiness-recovery-20261013", "READINESS_PREDECESSOR")
                .replace("readiness-retry-20261013", "readiness-final-20261013")
                .replace("READINESS_PREDECESSOR", "readiness-retry-20261013")
                .replace("96c408f3e5422c1cd5db953f669490c2b23026d558667ce11745a3702fc143f1", "76e5c8541c32af2c1cd0f00bf43d9ec8f2f92b05bb6d53692a369228b0b8cd6b")
        } else { source.replace("readiness-retry-20261013", "readiness-final-20261013") };
        fs::write(path, source.replace("01a13500-1ef8-73d3-9a0d-3446b6050eba", "01a13554-bc0d-7662-9378-28eecfa6122f")).unwrap();
    }
}
