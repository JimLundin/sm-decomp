//! The audio processor's 64 KiB RAM, shared by the sound driver and the DSP voices;
//! snapshots copy its bytes.
use crate::snapshot::{Fields, State};
use std::{cell::Cell, rc::Rc};

#[derive(Clone)]
pub struct ApuRam(Rc<[Cell<u8>]>);

impl Default for ApuRam {
    fn default() -> Self {
        Self::new()
    }
}

impl ApuRam {
    pub fn new() -> Self {
        Self((0..65536).map(|_| Cell::new(0)).collect())
    }

    pub fn read(&self, address: u16) -> u8 {
        self.read_at(usize::from(address))
    }

    pub fn read_at(&self, address: usize) -> u8 {
        self.0[address].get()
    }

    pub fn write_at(&self, address: usize, value: u8) {
        self.0[address].set(value);
    }

    pub fn fill_range(&self, range: std::ops::Range<usize>, value: u8) {
        for cell in &self.0[range] {
            cell.set(value);
        }
    }

    pub fn copy_from_ram(&self, input: &Self) {
        for (destination, source) in self.0.iter().zip(input.0.iter()) {
            destination.set(source.get());
        }
    }

    pub fn write(&self, address: u16, value: u8) {
        self.write_at(usize::from(address), value);
    }

    pub fn fill(&self, value: u8) {
        self.fill_range(0..65536, value);
    }

    pub fn copy_to(&self, output: &mut [u8; 65536]) {
        for (byte, cell) in output.iter_mut().zip(self.0.iter()) {
            *byte = cell.get();
        }
    }

    pub fn copy_from(&self, input: &[u8; 65536]) {
        for (cell, &byte) in self.0.iter().zip(input) {
            cell.set(byte);
        }
    }

    /// Borrow the stable buffer for the native DSP. Keep an owner alive for the
    /// entire use of the pointer, and serialize all access through the game gate.
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.0.as_ptr().cast::<u8>().cast_mut()
    }
}

impl State for ApuRam {
    const SIZE: usize = 65536;
    fn layout(layout: &mut String) {
        <[u8; Self::SIZE]>::layout(layout);
    }
    fn transfer(&mut self, fields: &mut Fields<'_>) {
        for offset in 0..Self::SIZE {
            let mut byte = self.read(offset as u16);
            byte.transfer(fields);
            self.write(offset as u16, byte);
        }
    }
}
