//! The original HDMA object definition: DMA control, target register and the
//! bank-local instruction list.

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SpawnHdmaObject_Args {
    pub dma_control: u8,
    pub target_register: u8,
    pub instruction_list: u16,
}

impl SpawnHdmaObject_Args {
    pub const fn new(dma_control: u8, target_register: u8, instruction_list: u16) -> Self {
        Self {
            dma_control,
            target_register,
            instruction_list,
        }
    }
}
