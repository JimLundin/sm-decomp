//! Complete MiniCrocomire actor and A2:8A75 no-op preinstruction.
const SIZE: usize = 0x20000;
const CURRENT: usize = 0xe54;
const ENEMIES: usize = 0xf78;
const SPEEDS: usize = 0x110187;
const RANDOM: usize = 0x5e5;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    SpawnInstruction,
    LeftInstruction,
    RightInstruction,
    Init,
    Main,
    Animation1,
    Animation2,
    Animation3,
    Animation4,
    Pre5,
    Pre6,
    Move,
    Turn,
    RandomPause,
    Noop,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Done,
    Continue(u16),
    Pause(u8),
}
/// Existing native movement/probe/RNG/projectile boundaries publish complete
/// WRAM synchronously. Capability admission precedes all actor writes/requests.
pub trait Services {
    fn available(&self, entry: Entry) -> bool;
    fn horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String>;
    fn vertical_probe(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String>;
    fn next_random(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn spawn_projectile(&mut self, ram: &mut [u8], argument: u16, k: u16, definition: u16) -> Result<(), String>;
}
fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn current(ram: &[u8]) -> Result<(u16, usize), String> {
    let k = word(ram, CURRENT);
    if k >= 0x800 || k & 63 != 0 {
        return Err("MiniCrocomire current slot outside canonical domain".into());
    }
    Ok((k, ENEMIES + usize::from(k)))
}
fn animation(ram: &mut [u8], list: u16) -> Result<(), String> {
    let (_, b) = current(ram)?;
    put(ram, b + 0x1c, 1);
    put(ram, b + 0x18, 0);
    put(ram, b + 0x1a, list);
    Ok(())
}
fn turn(ram: &mut [u8]) -> Result<(), String> {
    animation(ram, 0x8932)?;
    let (_, b) = current(ram)?;
    if word(ram, b + 0x38) != 1 {
        animation(ram, 0x8958)?;
    }
    Ok(())
}
fn collision(value: u8) -> Result<bool, String> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err("MiniCrocomire native collision outside admitted boolean domain".into()),
    }
}
fn movement(ram: &mut [u8], amount: i32, services: &mut impl Services) -> Result<(), String> {
    let (k, _) = current(ram)?;
    if !collision(services.horizontal(ram, k, amount)?)? {
        let (k, _) = current(ram)?;
        if collision(services.vertical_probe(ram, k, 0x00020000)?)? {
            return Ok(());
        }
    }
    turn(ram)
}
fn pause(ram: &mut [u8], services: &mut impl Services) -> Result<u8, String> {
    services.next_random(ram)?;
    let (_, b) = current(ram)?;
    let sum = (word(ram, b + 0x2a) as u8).wrapping_add(word(ram, RANDOM) as u8);
    if (u16::from(sum).wrapping_sub(2) as i16) < 0 {
        put(ram, b + 0x3a, 0x8a75);
        Ok(1)
    } else {
        Ok(0)
    }
}
fn pre(ram: &mut [u8], right: bool, services: &mut impl Services) -> Result<(), String> {
    let (_, b) = current(ram)?;
    let low = if right { 0x30 } else { 0x34 };
    let amount = ((u32::from(word(ram, b + low + 2)) << 16) | u32::from(word(ram, b + low))) as i32;
    movement(ram, amount, services)?;
    // Even movement's successful Y-probe early return must still request RNG.
    if pause(ram, services)? != 0 {
        animation(ram, if right { 0x896e } else { 0x8948 })?;
    }
    Ok(())
}
/// Ok(None) refuses unchanged. Accepted errors are terminal with ordered partial
/// effects retained. k is intentionally absent: all MiniCrocomire C entries
/// ignore explicit instruction/preinstruction k and use current native index.
pub fn execute(
    entry: Entry,
    jp: u16,
    amount: i32,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<Option<Output>, String> {
    if ram.len() != SIZE {
        return Ok(None);
    }
    let (k, b) = match current(ram) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let mut entry = entry;
    if entry == Entry::Main {
        entry = match word(ram, b + 0x3a) {
            0x8a43 => Entry::Pre5,
            0x8a5c => Entry::Pre6,
            0x8a75 => Entry::Noop,
            _ => return Ok(None),
        };
    }
    if entry == Entry::Init
        && (word(ram, b + 0x3e) > 64
            || rom
                .get(SPEEDS + usize::from(word(ram, b + 0x3e)) * 8..SPEEDS + usize::from(word(ram, b + 0x3e)) * 8 + 8)
                .is_none())
    {
        return Ok(None);
    }
    if entry == Entry::SpawnInstruction
        && (jp < 0x8000 || jp > 0xfffc || rom.get(0x108000 + usize::from(jp)..0x108000 + usize::from(jp) + 2).is_none())
    {
        return Ok(None);
    }
    if matches!(entry, Entry::SpawnInstruction | Entry::Pre5 | Entry::Pre6 | Entry::Move | Entry::RandomPause)
        && !services.available(entry)
    {
        return Ok(None);
    }
    match entry {
        Entry::SpawnInstruction => {
            let argument = word(rom, 0x108000 + usize::from(jp));
            services.spawn_projectile(ram, argument, k, 0xdbf2)?;
            return Ok(Some(Output::Continue(jp + 2)));
        }
        Entry::LeftInstruction => {
            put(ram, b + 0x3a, 0x8a43);
            put(ram, b + 0x38, 0);
            return Ok(Some(Output::Continue(jp)));
        }
        Entry::RightInstruction => {
            put(ram, b + 0x3a, 0x8a5c);
            put(ram, b + 0x38, 1);
            return Ok(Some(Output::Continue(jp)));
        }
        Entry::Init => {
            put(ram, b + 0x16, 0x804d);
            // C promotes parameter2 before multiplying; no16-bit alias allowed.
            let at = SPEEDS + usize::from(word(ram, b + 0x3e)) * 8;
            put(ram, b + 0x32, word(rom, at));
            put(ram, b + 0x30, word(rom, at + 2));
            put(ram, b + 0x36, word(rom, at + 4));
            put(ram, b + 0x34, word(rom, at + 6));
            animation(ram, 0x8932)?;
            put(ram, b + 0x3a, 0x8a43);
            let direction = word(ram, b + 0x3c);
            put(ram, b + 0x38, direction);
            if direction != 0 {
                animation(ram, 0x8958)?;
                put(ram, b + 0x3a, 0x8a5c);
            }
        }
        Entry::Animation1 => animation(ram, 0x8932)?,
        Entry::Animation2 => animation(ram, 0x8948)?,
        Entry::Animation3 => animation(ram, 0x8958)?,
        Entry::Animation4 => animation(ram, 0x896e)?,
        Entry::Pre5 => pre(ram, false, services)?,
        Entry::Pre6 => pre(ram, true, services)?,
        Entry::Move => movement(ram, amount, services)?,
        Entry::Turn => turn(ram)?,
        Entry::RandomPause => return Ok(Some(Output::Pause(pause(ram, services)?))),
        Entry::Noop => (),
        Entry::Main => unreachable!(),
    }
    Ok(Some(Output::Done))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Native {
        events: Vec<&'static str>,
        x: u8,
        y: u8,
        random: u16,
        change_slot: bool,
    }
    impl Services for Native {
        fn available(&self, _: Entry) -> bool {
            true
        }
        fn horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
            self.events.push("x");
            assert_eq!(k, 0);
            assert_eq!(amount, -0x18000);
            if self.change_slot {
                put(ram, CURRENT, 64);
            }
            Ok(self.x)
        }
        fn vertical_probe(&mut self, _: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
            self.events.push("y");
            assert_eq!(k, if self.change_slot { 64 } else { 0 });
            assert_eq!(amount, 0x20000);
            Ok(self.y)
        }
        fn next_random(&mut self, ram: &mut [u8]) -> Result<(), String> {
            self.events.push("random");
            put(ram, RANDOM, self.random);
            Ok(())
        }
        fn spawn_projectile(&mut self, _: &mut [u8], argument: u16, k: u16, definition: u16) -> Result<(), String> {
            self.events.push("spawn");
            assert_eq!((argument, k, definition), (0x1234, 0, 0xdbf2));
            Ok(())
        }
    }
    fn native() -> Native {
        Native { events: vec![], x: 0, y: 1, random: 0, change_slot: false }
    }
    #[test]
    fn a2_table_bank_and_four_word_order_with_nonboolean_direction() {
        let mut rom = vec![0; 0x110187 + 16];
        let mut ram = vec![0; SIZE];
        let mut host = native();
        for (offset, value) in [(0, 0x1111), (2, 0x2222), (4, 0x3333), (6, 0x4444)] {
            put(&mut rom, 0x110187 + 8 + offset, value);
        }
        put(&mut ram, ENEMIES + 0x3e, 1);
        put(&mut ram, ENEMIES + 0x3c, 2);
        execute(Entry::Init, 0, 0, &mut ram, &rom, &mut host).unwrap();
        assert_eq!(
            [
                word(&ram, ENEMIES + 0x32),
                word(&ram, ENEMIES + 0x30),
                word(&ram, ENEMIES + 0x36),
                word(&ram, ENEMIES + 0x34)
            ],
            [0x1111, 0x2222, 0x3333, 0x4444]
        );
        assert_eq!(word(&ram, ENEMIES + 0x38), 2);
        assert_eq!(word(&ram, ENEMIES + 0x1a), 0x8958);
        execute(Entry::Turn, 0, 0, &mut ram, &rom, &mut host).unwrap();
        assert_eq!(word(&ram, ENEMIES + 0x1a), 0x8958);
        put(&mut ram, ENEMIES + 0x38, 1);
        execute(Entry::Turn, 0, 0, &mut ram, &rom, &mut host).unwrap();
        assert_eq!(word(&ram, ENEMIES + 0x1a), 0x8932);
    }
    #[test]
    fn preinstruction_retains_rng_after_y_return_and_reacquires_current() {
        let mut ram = vec![0; SIZE];
        let mut host = native();
        host.change_slot = true;
        put(&mut ram, ENEMIES + 0x36, 0xfffe);
        put(&mut ram, ENEMIES + 0x34, 0x8000);
        execute(Entry::Pre5, 0, 0, &mut ram, &[], &mut host).unwrap();
        assert_eq!(host.events, ["x", "y", "random"]);
        assert_eq!(word(&ram, ENEMIES + 64 + 0x3a), 0x8a75);
        assert_eq!(word(&ram, ENEMIES + 64 + 0x1a), 0x8948);
    }
    #[test]
    fn random_byte_boundaries_and_noop_unknown_selector() {
        for (random, expected) in [(0, 1), (1, 1), (2, 0), (255, 0)] {
            let mut ram = vec![0; SIZE];
            let mut host = native();
            host.random = random;
            assert_eq!(
                execute(Entry::RandomPause, 0, 0, &mut ram, &[], &mut host).unwrap(),
                Some(Output::Pause(expected))
            );
        }
        let mut ram = vec![0; SIZE];
        let mut host = native();
        put(&mut ram, ENEMIES + 0x3a, 0x8a75);
        let before = ram.clone();
        assert_eq!(execute(Entry::Main, 0, 0, &mut ram, &[], &mut host).unwrap(), Some(Output::Done));
        assert_eq!(ram, before);
        put(&mut ram, ENEMIES + 0x3a, 0x8a74);
        let before = ram.clone();
        assert_eq!(execute(Entry::Main, 0, 0, &mut ram, &[], &mut host).unwrap(), None);
        assert_eq!(ram, before);
    }
    #[test]
    fn projectile_reads_literal_a2_operand_and_returns_one_word() {
        let mut rom = vec![0; 0x118980];
        put(&mut rom, 0x11097e, 0x1234);
        put(&mut rom, 0x11897e, 0x5678);
        let mut ram = vec![0; SIZE];
        let mut host = native();
        assert_eq!(
            execute(Entry::SpawnInstruction, 0x897e, 0, &mut ram, &rom, &mut host).unwrap(),
            Some(Output::Continue(0x8980))
        );
        assert_eq!(host.events, ["spawn"]);
        assert_eq!(execute(Entry::SpawnInstruction, 0xffff, 0, &mut ram, &rom, &mut host).unwrap(), None);
    }
    #[test]
    fn x_collision_skips_y_and_invalid_return_is_terminal() {
        let mut ram = vec![0; SIZE];
        let mut host = native();
        host.x = 1;
        put(&mut ram, ENEMIES + 0x38, 1);
        execute(Entry::Move, 0, -0x18000, &mut ram, &[], &mut host).unwrap();
        assert_eq!(host.events, ["x"]);
        assert_eq!(word(&ram, ENEMIES + 0x1a), 0x8932);
        let mut host = native();
        host.x = 2;
        assert_eq!(
            execute(Entry::Move, 0, -0x18000, &mut ram, &[], &mut host),
            Err("MiniCrocomire native collision outside admitted boolean domain".into())
        );
        assert_eq!(host.events, ["x"]);
    }
}
