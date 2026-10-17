//! Run the built-in demo or describe the generated tables. The headless
//! recompilation frame runner lives in `diff`.

use game::hw::RamBus;

fn main() {
    let t = game::table();
    if cfg!(sm_generated) {
        println!("generated table: {} function variants", t.len());
        println!("generated instruction entries: {}", game::steps::STEP_TABLE.len());
        println!("run `just iterate` to regenerate and compare compiled/interpreted frames");
        return;
    }
    let mut m = game::machine(Box::new(RamBus::new()), None);
    m.cpu.set_p(0x30);
    m.cpu.s = 0x1ff;
    m.push16(0xfffe); // fake return address for the top-level RTS
    m.call(0x80_8000);
    println!("demo program ran: $0010 = ${:04X}, $0012 = ${:02X}, X = {}", m.read(0x10, true), m.read8(0x12), m.cpu.x);
}
