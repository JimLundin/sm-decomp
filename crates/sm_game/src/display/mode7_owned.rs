//! Native lists append into the one packed Mode7 owner, preserving alias effects.
#![forbid(unsafe_code)]
pub trait Queue {
    /// Original diagnostic and selected native allocation, acquired once.
    fn native_list(&mut self, bank: u8, cursor: u16) -> sm_data::cartridge::RomRecord;
    fn tail(&self) -> u16;
    fn write_byte(&mut self, at: usize, value: u8);
    fn publish_tail(&mut self, value: u16);
}
pub trait Append {
    fn source_byte(&mut self, offset: usize) -> u8;
    fn tail(&self) -> u16;
    fn write_byte(&mut self, at: usize, value: u8);
    fn publish_tail(&mut self, value: u16);
}
pub fn enqueue(bank: u8, cursor: u16, host: &mut (impl Queue + ?Sized)) {
    let record = host.native_list(bank, cursor);
    struct Native<'a, H: ?Sized> {
        host: &'a mut H,
        record: sm_data::cartridge::RomRecord,
    }
    impl<H: Queue + ?Sized> Append for Native<'_, H> {
        fn source_byte(&mut self, offset: usize) -> u8 {
            self.record.byte(offset)
        }
        fn tail(&self) -> u16 {
            self.host.tail()
        }
        fn write_byte(&mut self, at: usize, value: u8) {
            self.host.write_byte(at, value);
        }
        fn publish_tail(&mut self, value: u16) {
            self.host.publish_tail(value);
        }
    }
    append(&mut Native { host, record });
}
pub fn append(host: &mut (impl Append + ?Sized)) {
    let mut source = 0usize;
    let mut destination = crate::display::transfer::MODE7_QUEUE + usize::from(host.tail());
    loop {
        let tag = host.source_byte(source);
        let length = if tag & 0x80 != 0 {
            9
        } else if tag & 0x40 != 0 {
            7
        } else {
            break;
        };
        host.write_byte(destination + length, 0);
        for index in 0..length {
            let byte = host.source_byte(source + index);
            host.write_byte(destination + index, byte);
        }
        source += length;
        destination += length;
    }
    let displacement = destination - crate::display::transfer::MODE7_QUEUE;
    let bytes = displacement.to_le_bytes();
    host.publish_tail(u16::from_le_bytes([bytes[0], bytes[1]]));
}

pub trait Drain {
    type Error;
    fn byte(&mut self, at: usize) -> Result<u8, Self::Error>;
    fn word(&mut self, at: usize) -> Result<u16, Self::Error>;
    fn upload(
        &mut self,
        upload: crate::display::transfer_types::Mode7Upload,
    ) -> Result<(), Self::Error>;
}
pub fn drain<H: Drain + ?Sized>(host: &mut H) -> Result<(), H::Error> {
    use crate::display::transfer_types::{Mode7Target, Mode7Upload, Source};
    let mut entry = crate::display::transfer::MODE7_QUEUE;
    loop {
        let tag = host.byte(entry)?;
        if tag & 0xc0 == 0 {
            return Ok(());
        }
        let vram = tag & 0x80 != 0;
        let target = if vram {
            Mode7Target::Vram {
                high_byte: tag & 0x40 != 0,
                address: host.word(entry + 6)?,
                increment: host.byte(entry + 8)?,
            }
        } else {
            Mode7Target::Cgram {
                index: host.byte(entry + 6)?,
            }
        };
        let source = Source {
            address: host.word(entry + 1)?,
            bank: host.byte(entry + 3)?,
        };
        let upload = Mode7Upload {
            mode: tag & 0x1f,
            source,
            bytes: host.word(entry + 4)?,
            target,
        };
        host.upload(upload)?;
        entry += if vram { 9 } else { 7 };
    }
}
