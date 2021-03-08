use crate::process::ThreadRef;
use alloc::boxed::Box;
use memory::VirtualAddress;

/// Accessed through GS.Base. Dereferencing is done automatically so we can't get an actual
/// reference, access fields through gs:$offset instead
pub struct CpuState {
    isr_stack_top: VirtualAddress,

    current_thread: Option<ThreadRef>,

    /// Kernel stack of current_thread, copied here so asm can access it easily
    current_thread_kernel_stack: VirtualAddress,
}

impl CpuState {
    // VirtualAddress is repr(transparent)
    pub const THREAD_KERNEL_STACK_OFFSET: usize =
        memoffset::offset_of!(CpuState, current_thread_kernel_stack);

    const CURRENT_THREAD_OFFSET: usize = memoffset::offset_of!(CpuState, current_thread);

    /// Returns leaked Box
    pub fn new(isr_stack_top: VirtualAddress) -> *mut Self {
        let state = Self {
            isr_stack_top,
            current_thread: None,
            current_thread_kernel_stack: VirtualAddress::zero(),
        };

        Box::leak(Box::new(state))
    }

    /// # Safety
    /// GS.Base must be pointing to a CpuState
    pub unsafe fn update_current_thread(thread: ThreadRef) {
        // just to make sure before we do some screwy stuff
        assert_eq!(core::mem::size_of::<Option<ThreadRef>>(), 8);

        let stack = thread.kernel_stack();
        let some_thread = core::mem::transmute::<Option<ThreadRef>, u64>(Some(thread));

        asm!(
            "mov gs:{offset_stack}, {stack}",
            "mov gs:{offset_thread}, {thread}",
            offset_stack = const Self::THREAD_KERNEL_STACK_OFFSET,
            stack = in(reg) stack.address(),

            offset_thread = const Self::CURRENT_THREAD_OFFSET,
            thread = in(reg) some_thread,
        )
    }

    /// # Safety
    /// GS.Base must be pointing to a CpuState.
    ///
    /// Panics if current thread not set
    pub unsafe fn current_thread() -> ThreadRef {
        let thread_handle: *const ();
        asm!(
            "mov {}, gs:{offset_thread}",
            out(reg) thread_handle,
            offset_thread = const Self::CURRENT_THREAD_OFFSET,
        );

        // None == null
        if thread_handle.is_null() {
            panic!("no current thread in cpu state")
        }

        // reference is definitely Some now

        // the pointer we have is actually *const ThreadHandle, i.e. an owned ThreadRef
        let borrowed_ref: ThreadRef = core::mem::transmute(thread_handle);

        // get a strong reference to return
        let owned = borrowed_ref.clone();

        // must forget the "owned" thread ref we have, otherwise the strong count will be
        // decremented
        core::mem::forget(borrowed_ref);

        owned
    }
}
