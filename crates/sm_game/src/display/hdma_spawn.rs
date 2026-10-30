//! The original HDMA object definition: DMA control, target register and the
//! bank-local instruction list.

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct HdmaObjectDefinition {
    pub dma_control: u8,
    pub target_register: u8,
    pub instruction_list: u16,
}

impl HdmaObjectDefinition {
    pub const fn new(dma_control: u8, target_register: u8, instruction_list: u16) -> Self {
        Self {
            dma_control,
            target_register,
            instruction_list,
        }
    }
}
