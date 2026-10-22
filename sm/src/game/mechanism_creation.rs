//! Ordered allocation and initialization of room mechanisms from every source.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Collision,
    Hardcoded,
    Room,
}

pub enum Source<H, B, P> {
    Collision {
        header: H,
    },
    Hardcoded {
        x: u8,
        y: u8,
        header: H,
    },
    Room {
        address: u16,
    },
    /// Authored native content; encoded sources must retain their lazy reads.
    Prepared {
        header: H,
        block: B,
        before: P,
        argument: u16,
    },
}

pub enum Field<C: Context + ?Sized> {
    Block(C::Block),
    Header(C::Header),
    Argument(u16),
    Auxiliary(u16),
    Before(C::Pre),
    Program(C::Program),
    Remaining(u16),
    DefaultDraw,
    IdleTimer,
    Selected,
}

/// State and content access only. Writes become visible before the next read
/// or callback, including reads that fail or replace the current content image.
pub trait Context {
    type Header: Copy;
    type Block: Copy;
    type Pre: Copy;
    type Program;
    type Entry;
    type Definition;
    type Setup;
    fn vacant(&self, slot: usize) -> bool;
    fn write(&mut self, slot: usize, field: Field<Self>);
    fn current_block(&mut self) -> Self::Block;
    fn width(&mut self) -> u8;
    fn block(index: u16) -> Self::Block;
    fn header(value: u16) -> Self::Header;
    fn before(kind: SourceKind) -> Self::Pre;
    fn entry(&mut self, address: u16) -> Self::Entry;
    fn byte(&mut self, entry: &Self::Entry, offset: usize) -> u8;
    fn definition(&mut self, header: Self::Header, kind: SourceKind) -> Self::Definition;
    fn program(&mut self, definition: &Self::Definition) -> Self::Program;
    fn setup(&mut self, definition: &Self::Definition) -> Self::Setup;
    fn execute(&mut self, slot: usize, setup: Self::Setup, kind: SourceKind) -> u8;
}

fn word<C: Context>(c: &mut C, entry: &C::Entry, offset: usize) -> u16 {
    u16::from_le_bytes([c.byte(entry, offset), c.byte(entry, offset + 1)])
}

pub fn create<C: Context>(c: &mut C, source: Source<C::Header, C::Block, C::Pre>) -> u8 {
    let Some(slot) = (0..40).rev().find(|&slot| c.vacant(slot)) else {
        return 0;
    };
    let (header, kind) = match source {
        Source::Collision { header } => {
            let block = c.current_block();
            c.write(slot, Field::Block(block));
            c.write(slot, Field::Header(header));
            c.write(slot, Field::Before(C::before(SourceKind::Collision)));
            (header, SourceKind::Collision)
        }
        Source::Hardcoded { x, y, header } => {
            let row = u16::from(y) * u16::from(c.width());
            let block = C::block(row.wrapping_add(u16::from(x)).wrapping_mul(2));
            c.write(slot, Field::Block(block));
            c.write(slot, Field::Header(header));
            c.write(slot, Field::Argument(0));
            c.write(slot, Field::Auxiliary(0));
            c.write(slot, Field::Before(C::before(SourceKind::Hardcoded)));
            (header, SourceKind::Hardcoded)
        }
        Source::Room { address } => {
            let entry = c.entry(address);
            let y = u16::from(c.byte(&entry, 3));
            let row = y * u16::from(c.width());
            let x = u16::from(c.byte(&entry, 2));
            c.write(
                slot,
                Field::Block(C::block(row.wrapping_add(x).wrapping_mul(2))),
            );
            let argument = word(c, &entry, 4);
            c.write(slot, Field::Argument(argument));
            let header = C::header(word(c, &entry, 0));
            let stored_header = C::header(word(c, &entry, 0));
            c.write(slot, Field::Header(stored_header));
            c.write(slot, Field::Auxiliary(0));
            c.write(slot, Field::Before(C::before(SourceKind::Room)));
            (header, SourceKind::Room)
        }
        Source::Prepared {
            header,
            block,
            before,
            argument,
        } => {
            c.write(slot, Field::Block(block));
            c.write(slot, Field::Header(header));
            c.write(slot, Field::Argument(argument));
            c.write(slot, Field::Auxiliary(0));
            c.write(slot, Field::Before(before));
            (header, SourceKind::Hardcoded)
        }
    };
    let definition = c.definition(header, kind);
    let program = c.program(&definition);
    c.write(slot, Field::Program(program));
    c.write(slot, Field::Remaining(1));
    c.write(slot, Field::DefaultDraw);
    c.write(slot, Field::IdleTimer);
    if kind == SourceKind::Collision {
        c.write(slot, Field::Argument(0));
        c.write(slot, Field::Auxiliary(0));
    }
    c.write(slot, Field::Selected);
    let setup = c.setup(&definition);
    c.execute(slot, setup, kind)
}
