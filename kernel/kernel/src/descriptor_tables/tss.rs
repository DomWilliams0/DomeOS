use common::InitializedGlobal;
use memory::{kilobytes, PhysicalAddress, VirtualAddress};

static mut TSS: InitializedGlobal<TaskStateSegment> = InitializedGlobal::uninit();

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    _reserved: u32,

    privilege_stacks: [VirtualAddress; 3],

    _reserved2: u64,

    interrupt_stacks: [VirtualAddress; 7],
    _reserved3: u64,
    _reserved4: u16,

    iomap_base: u16,
}

pub const IST_IDX_DOUBLE_FAULT: usize = 0;

const IST_STACK_SIZE: usize = kilobytes(8) as usize;

static mut IST_STACK_DOUBLE_FAULT: [u8; IST_STACK_SIZE] = [0; IST_STACK_SIZE];

impl Default for TaskStateSegment {
    fn default() -> Self {
        Self {
            _reserved: 0,
            privilege_stacks: [VirtualAddress::zero(); 3],
            _reserved2: 0,
            interrupt_stacks: [VirtualAddress::zero(); 7],
            _reserved3: 0,
            _reserved4: 0,
            iomap_base: 0,
        }
    }
}

impl TaskStateSegment {
    pub fn init() -> PhysicalAddress {
        let mut tss = Self::default();

        unsafe {
            // TODO allocate stack dynamically instead of static array
            tss.register_ist(IST_IDX_DOUBLE_FAULT, &mut IST_STACK_DOUBLE_FAULT);

            TSS.init(tss);
            let ptr = TSS.get() as *mut Self;
            PhysicalAddress(ptr as u64)
        }
    }

    fn register_ist(&mut self, idx: usize, stack: &'static mut [u8]) {
        let stack_bottom = stack.as_mut_ptr() as u64;
        let stack_top = stack_bottom + stack.len() as u64;

        self.interrupt_stacks[idx] = VirtualAddress::new(stack_top);
    }

    pub fn set_privilege_stack(&mut self, pl: u8, stack: VirtualAddress) {
        common::trace!("tss priv stack[{}] = {:?}", pl, stack);
        self.privilege_stacks[pl as usize] = stack;
    }
}

pub fn tss() -> &'static mut TaskStateSegment {
    unsafe { TSS.get() }
}
