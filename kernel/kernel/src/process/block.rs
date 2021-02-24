use crate::memory::AddressSpace;
use alloc::rc::Rc;
use memory::VirtualAddress;
use smallvec::SmallVec;

pub struct Process {
    /// None: use default kernel address space
    /// Some: use own address space, destroyed along with this process
    addr_space: Option<AddressSpace<'static>>,

    threads: SmallVec<[Thread; 2]>,
}

pub struct Thread {
    /// Start of usable stack (growing downwards)
    stack_top: VirtualAddress,

    /// Parent process, None for kernel threads
    process: Option<Rc<Process>>,
}
