use decomp::{RomView, WramView, decompress_to_mem};
use std::{env, fs};

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        return Err("usage: decomp ROM WRAM SOURCE DESTINATION OUTPUT".into());
    }
    let rom = fs::read(&args[1]).map_err(|e| e.to_string())?;
    let mut bytes = fs::read(&args[2]).map_err(|e| e.to_string())?;
    let source = args[3].parse::<u32>().map_err(|e| e.to_string())?;
    let destination = args[4].parse::<usize>().map_err(|e| e.to_string())?;
    let mut writes = Vec::new();
    let mut observer = |offset, value| writes.push((offset, value));
    let mut wram = WramView::with_observer(&mut bytes, &mut observer).map_err(|e| format!("{e:?}"))?;
    let result = decompress_to_mem(
        &RomView::new(&rom).map_err(|e| format!("{e:?}"))?,
        &mut wram,
        source,
        destination,
    )
    .map_err(|e| format!("{e:?}"))?;
    drop(wram);
    let trace: String = writes.iter().map(|(offset, value)| format!("{offset} {value}\n")).collect();
    fs::write(format!("{}.writes", args[5]), trace).map_err(|e| e.to_string())?;
    fs::write(&args[5], bytes).map_err(|e| e.to_string())?;
    let cases: Vec<String> = result.cases.iter().map(|s| format!("\"{s}\"")).collect();
    println!(
        "{{\"status\":\"ok\",\"written\":{},\"final_source\":{},\"cases\":[{}]}}",
        result.written,
        result.final_source,
        cases.join(",")
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        // Debug escaping creates a JSON string for the ASCII error messages emitted here.
        println!("{{\"status\":\"error\",\"error\":{error:?}}}");
        std::process::exit(1);
    }
}
