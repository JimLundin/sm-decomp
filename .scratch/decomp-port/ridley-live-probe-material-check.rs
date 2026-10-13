use std::{fs,path::PathBuf};
fn main(){
 let root=PathBuf::from(std::env::args().nth(1).unwrap());let a=root.join("c-0");let b=root.join("mixed-0");let mut count=0;
 for item in fs::read_dir(a.join("fixtures")).unwrap(){let x=item.unwrap().path();let name=x.file_name().unwrap().to_str().unwrap();if !name.starts_with("ridley-whole-")||!name.ends_with("-before.wram"){continue}
  let end=name.replace("-before.wram","-after.wram");let pre=fs::read(&x).unwrap();let post=fs::read(a.join("fixtures").join(&end)).unwrap();assert_eq!(pre,fs::read(b.join("fixtures").join(name)).unwrap());assert_eq!(post,fs::read(b.join("fixtures").join(&end)).unwrap());let n=pre.iter().zip(post).filter(|(x,y)|**x!=*y).count();println!("{name} changed_bytes={n}");count+=n;
 }assert!(count>0,"no material Rust-owned wholeRidley captures");
}
