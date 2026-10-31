//! The VRAM tilemap queue at $7E:00D0, checked against the RAM map's description:
//! seven-byte records of size (word), source address (word), source bank (byte) and
//! VRAM destination (word, bit 15 selecting a column), with $7E:0330 the byte offset
//! of the next record and a zero size ending the queue.
use sm_game::display::transfer::{
    Mode7Upload, Source, TilemapQueue, TilemapUpload, TransferHost, VramReadback, drain_tilemaps,
    queue_tilemap,
};

struct Queue {
    ram: Vec<u8>,
    uploads: Vec<TilemapUpload>,
}

impl Queue {
    fn new() -> Self {
        Self {
            ram: vec![0; 0x400],
            uploads: Vec::new(),
        }
    }
}
impl TilemapQueue for Queue {
    fn tilemap_queue_tail(&self) -> u16 {
        u16::from_le_bytes([self.ram[0x330], self.ram[0x331]])
    }
    fn set_tilemap_queue_tail(&mut self, tail: u16) {
        self.ram[0x330..0x332].copy_from_slice(&tail.to_le_bytes());
    }
    fn set_tilemap_queue_byte(&mut self, at: usize, value: u8) {
        self.ram[at] = value;
    }
}
impl TransferHost for Queue {
    fn byte(&self, at: usize) -> u8 {
        self.ram[at]
    }
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.ram[at], self.ram[at + 1]])
    }
    fn upload_mode7(&mut self, _: Mode7Upload) {
        unreachable!("no Mode 7 records")
    }
    fn read_vram(&mut self, _: VramReadback) {
        unreachable!("no readbacks")
    }
    fn upload_tilemap(&mut self, upload: TilemapUpload) {
        self.uploads.push(upload);
    }
}

fn upload(bytes: u16, address: u16, bank: u8, vram_address: u16) -> TilemapUpload {
    TilemapUpload {
        bytes,
        source: Source { address, bank },
        column: vram_address & 0x8000 != 0,
        vram_address,
    }
}

#[test]
fn a_queued_record_is_seven_bytes_at_the_tail() {
    let mut queue = Queue::new();
    queue.set_tilemap_queue_tail(14);
    queue_tilemap(&mut queue, upload(0x400, 0x8034, 0x7e, 0x5123));
    assert_eq!(
        &queue.ram[0xd0 + 14..0xd0 + 21],
        &[0x00, 0x04, 0x34, 0x80, 0x7e, 0x23, 0x51]
    );
    assert_eq!(queue.tilemap_queue_tail(), 21);
}

#[test]
fn queued_records_drain_in_order() {
    let mut queue = Queue::new();
    let records = [
        upload(2, 0x8034, 0x7e, 0x5000),
        upload(0x400, 0xc000, 0x94, 0x8040),
    ];
    for record in records {
        queue_tilemap(&mut queue, record);
    }
    drain_tilemaps(&mut queue);
    assert_eq!(queue.uploads, records);
    assert!(
        queue.uploads[1].column,
        "bit 15 of the destination selects a column"
    );
}
