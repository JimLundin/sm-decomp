//! Entry point. Today it runs the self-test program (or reports the generated
//! table size); once `hw::SnesBus` exists this becomes the SDL frontend.

use game::hw::RamBus;

fn main() {
    let t = game::table();
    if cfg!(sm_generated) {
        println!("generated table: {} function variants", t.len());
        println!("next: implement hw::SnesBus on RustySNES, then run `cargo run -p diff`");
        return;
    }
    let mut m = game::machine(Box::new(RamBus::new()), None);
    m.cpu.set_p(0x30);
    m.cpu.s = 0x1ff;
    m.push16(0xfffe); // fake return address for the top-level RTS
    m.call(0x80_8000);
    println!(
        "demo program ran: $0010 = ${:04X}, $0012 = ${:02X}, X = {}",
        m.read(0x10, true),
        m.read8(0x12),
        m.cpu.x
    );
}
