use utils::InitializedGlobal;

pub trait VirtualMemoryManager {}

static mut VIRTUAL_MM: InitializedGlobal<SimpleVirtualMemoryManager> = InitializedGlobal::uninit();

struct SimpleVirtualMemoryManager {}

pub fn init_virtual_allocator() {
    let allocator = SimpleVirtualMemoryManager::new();
    unsafe {
        VIRTUAL_MM.init(allocator);
    }
}

pub fn virtual_allocator() -> &'static mut impl VirtualMemoryManager {
    unsafe { VIRTUAL_MM.get() }
}

impl SimpleVirtualMemoryManager {
    fn new() -> Self {
        todo!()
    }
}

impl VirtualMemoryManager for SimpleVirtualMemoryManager {}
