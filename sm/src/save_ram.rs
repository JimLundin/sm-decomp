//! Shared save bytes used by gameplay, the cartridge bus and snapshots.
#![forbid(unsafe_code)]
use std::{cell::Cell, ops::Range, rc::Rc};

#[derive(Clone, Default)]
pub struct SaveRam(Rc<[Cell<u8>]>);

impl SaveRam {
    pub fn zeroed(length: usize) -> Self {
        Self((0..length).map(|_| Cell::new(0)).collect())
    }

    pub fn read(&self, offset: usize) -> u8 {
        self.0[offset].get()
    }

    pub fn write(&self, offset: usize, value: u8) {
        self.0[offset].set(value);
    }

    pub fn word(&self, offset: usize) -> u16 {
        u16::from_ne_bytes([self.read(offset), self.read(offset + 1)])
    }

    pub fn set_word(&self, offset: usize, value: u16) {
        self.write_bytes(offset, &value.to_ne_bytes());
    }

    pub fn read_bytes(&self, offset: usize, output: &mut [u8]) {
        let cells = &self.0[offset..offset + output.len()];
        for (byte, cell) in output.iter_mut().zip(cells) {
            *byte = cell.get();
        }
    }

    pub fn write_bytes(&self, offset: usize, input: &[u8]) {
        for (cell, &byte) in self.0[offset..offset + input.len()].iter().zip(input) {
            cell.set(byte);
        }
    }

    pub fn bytes(&self, range: Range<usize>) -> Vec<u8> {
        self.0[range].iter().map(Cell::get).collect()
    }

    pub fn fill(&self, range: Range<usize>, value: u8) {
        for cell in &self.0[range] {
            cell.set(value);
        }
    }

    pub fn copy_within(&self, source: Range<usize>, destination: usize) {
        self.write_bytes(destination, &self.bytes(source));
    }
}
