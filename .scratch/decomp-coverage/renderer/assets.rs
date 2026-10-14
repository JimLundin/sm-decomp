use std::fs;
fn main() {
    let r = fs::read("/home/jim/sm-decomp/rom/sm.sfc").unwrap();
    let word = |a: usize| u16::from_le_bytes([r[0x10000 + (a & 0x7fff)], r[0x10000 + (a & 0x7fff) + 1]]);
    for (name, address, count) in [("timers", 0xc0b2, 9), ("frames", 0xc0c4, 9), ("variants", 0xc0d6, 9), ("animations", 0xc0e8, 9), ("baseids", 0xc1e4, 9), ("wire-cmp", 0xb257, 4), ("wire-ptrs", 0xb25f, 4), ("xy-ptrs", 0xc18e, 4)] {
        print!("{name}:");
        for n in 0..count { print!(" {:04x}", word(address + 2*n)); }
        println!();
    }
    for animation in [2,3,6,7,8,9] {
        let p = word(0xc0e8 + 2*(animation-1)) as usize;
        print!("animation {animation}:");
        for n in 0..20 { let x = r[0x10000+(p&0x7fff)+3*n]; print!(" {x}:{}", r[0x10000+(p&0x7fff)+3*n+2]); if x==255 {break} }
        println!();
        let p = word(0xc1e4 + 2*(animation-1)) as usize;
        print!("bases {animation}:"); for n in 0..4 {print!(" {}",word(p+2*n));} println!();
    }
    print!("reserve-data:");for n in 0..32 {print!(" {}",r[0x14165+2*n]);if r[0x14165+2*n]==255 {break}}println!();
    for n in [0,0x1b,0x1f,0x20,0x21,0x22,0x27,0x2b,0x40,0x64,0x68] {
        let p=word(0xc569+2*n) as usize; println!("spritemap {n:#x}: pointer={p:04x} count={}",word(p));
    }
}
