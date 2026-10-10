//! Standalone bounded candidate; does not change the accepted decompressor crate.
const RNG: usize = 0x5e5;
const WRAM: usize = 0x20000;

pub fn next_random(ram: &mut [u8]) -> Result<u16, &'static str> {
    if ram.len() != WRAM { return Err("full canonical WRAM required"); }
    let seed = u16::from_le_bytes([ram[RNG], ram[RNG + 1]]);
    // The original multiplier retains the high-byte overflow as an end-around carry.
    let low_product = u16::from(seed as u8) * 5;
    let carry = (low_product >> 8) + u16::from((seed >> 8) as u8).wrapping_mul(5) as u8 as u16 + 1;
    let word = ((carry as u8 as u16) << 8) | (low_product as u8 as u16);
    let result = word.wrapping_add(carry >> 8).wrapping_add(17);
    let bytes = result.to_le_bytes();
    ram[RNG] = bytes[0];
    ram[RNG + 1] = bytes[1];
    Ok(result)
}

/// Caller supplies one live, exclusive 128 KiB allocation for this synchronous call.
/// No pointer is retained, no callback or panic is permitted. Zero denotes invalid input.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_next_random_v1(ram: *mut u8, len: usize) -> i32 {
    if ram.is_null() || len != WRAM || (ram as usize).checked_add(len).is_none() { return -1; }
    // SAFETY: allocation validity and exclusive access are the caller's ABI contract.
    let bytes = unsafe { std::slice::from_raw_parts_mut(ram, len) };
    i32::from(next_random(bytes).expect("validated extent"))
}

#[cfg(cli)]
fn main() {
    use std::io::Write;
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && args[1] == "exhaustive" {
        let initial: Vec<u8> = (0..WRAM).map(|i| (i as u8).wrapping_mul(37)).collect();
        let mut ram = initial.clone();
        let mut out = std::io::BufWriter::new(std::io::stdout());
        for seed in 0..=u16::MAX {
            ram[RNG..RNG + 2].copy_from_slice(&seed.to_le_bytes());
            let result = next_random(&mut ram).unwrap();
            assert_eq!(&ram[..RNG], &initial[..RNG]);
            assert_eq!(&ram[RNG + 2..], &initial[RNG + 2..]);
            assert_eq!(&ram[RNG..RNG + 2], &result.to_le_bytes());
            out.write_all(&result.to_le_bytes()).unwrap();
        }
    } else if args.len() == 3 {
        let mut ram = std::fs::read(&args[1]).unwrap();
        let result = next_random(&mut ram).unwrap();
        std::fs::write(&args[2], ram).unwrap();
        println!("{}", result);
    } else { std::process::exit(2); }
}
