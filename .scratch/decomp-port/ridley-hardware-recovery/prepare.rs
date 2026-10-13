use std::{fs,path::Path};
fn main() {
 let root=Path::new("/home/jim/sm-decomp/.scratch/decomp-port/ridley-hardware-recovery/source/crates/pipeline/src");
 let p=root.join("ridley_subsystem_adapter.rs");
 let mut s=fs::read_to_string(&p).unwrap();
 s=s.replace("vec![0; 20]", "vec![0; PPU_BYTES]").replace("(\"ppu.bin\", 20)","(\"ppu.bin\", PPU_BYTES)").replace("(0..20).collect", "(0..PPU_BYTES as u8).collect");
 // Tests must model the v2 native default discriminator as well.
 s=s.replace("fs::write(dir.join(\"ppu-before.bin\"), vec![0; PPU_BYTES]).unwrap();", "let mut hardware = vec![0; PPU_BYTES]; hardware[0] = 1;\n        fs::write(dir.join(\"ppu-before.bin\"), &hardware).unwrap();");
 s=s.replace("fs::write(dir.join(format!(\"{subject}-{suffix}\")), vec![0; bytes]).unwrap();", "fs::write(dir.join(format!(\"{subject}-{suffix}\")), if suffix == \"ppu.bin\" { hardware.clone() } else { vec![0; bytes] }).unwrap();");
 s=s.replace("(\"ppu.bin\", vec![0; PPU_BYTES])", "(\"ppu.bin\", ppu_before(&fixture, &retained.0).unwrap())");
 s=s.replace("\"before.ppu\" | \"after.ppu\" => vec![0; PPU_BYTES]", "\"before.ppu\" | \"after.ppu\" => ppu_before(&Case { host_words: Some(vec![0;8]), ..Default::default() }, &retained.0).unwrap()");
 fs::write(p,s).unwrap();
 let p=root.join("main.rs");let mut s=fs::read_to_string(&p).unwrap();
 s=s.replace("mod ridley_subsystem_adapter;", "mod ridley_subsystem_adapter;\nmod ridley_subsystem_captured;");
 s=s.replace("Some(\"ridley-subsystem-authenticate\") if args.len() == 2 => {", "Some(\"ridley-subsystem-captured\") if args.len() == 3 => {\n            let manifest = ridley_subsystem_captured::generate(&std::env::current_dir()?, Path::new(&args[1]), Path::new(&args[2]))?;\n            emit(&manifest)\n        }\n        Some(\"ridley-subsystem-authenticate\") if args.len() == 2 => {");
 fs::write(p,s).unwrap();
}
