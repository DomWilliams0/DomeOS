#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: u64,
}
