//! Shared ownership of the one mutable cartridge image.
#![forbid(unsafe_code)]
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct RomImage(Rc<[Cell<u8>]>);

impl RomImage {
    pub fn zeroed(length: usize) -> Self {
        Self((0..length).map(|_| Cell::new(0)).collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn read(&self, offset: usize) -> u8 {
        self.0[offset].get()
    }

    pub fn write(&self, offset: usize, value: u8) {
        self.0[offset].set(value);
    }

    // Legacy cartridge and patch consumers borrow this allocation. Cell keeps
    // its bytes mutable while shared owners exist; access is serialized by Game.
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.0.as_ptr().cast::<u8>().cast_mut()
    }
}
