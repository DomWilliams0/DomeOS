use crate::memory::{AddressSpace, AddressSpaceRef};
use crate::process::block::id::{OwnedPid, Pid};
use crate::process::block::process::ProcessRef;
use crate::spinlock::SpinLock;
use alloc::sync::Arc;
use common::*;
use core::cell::RefCell;
use core::ops::Deref;
use memory::VirtualAddress;

#[derive(Clone)]
pub struct ThreadRef(Arc<ThreadHandle>);

/// Only immutable references are handed out, fields are protected by refcells/mutexes as needed
pub struct ThreadHandle {
    /// Set once on creation and never changed
    inner_const: ThreadConstantInner,
    inner_locked: SpinLock<ThreadLockedInner>,
    inner_refcell: RefCell<ThreadInner>,
}

/// Not protected by mutex/refcell, readonly after creation
pub struct ThreadConstantInner {
    /// Parent process, None for kernel threads
    process: Option<ProcessRef>,

    tid: OwnedPid,
}

/// Protected by mutex
struct ThreadLockedInner {}

/// Protected by a refcell
struct ThreadInner {
    /// Start of usable stack (growing downwards)
    stack_top: VirtualAddress,

    state: ThreadState,
}

struct ThreadState {
    rsp: u64,
    rip: u64,
}

pub fn new_thread(
    tid: OwnedPid,
    stack_top: VirtualAddress,
    process: Option<ProcessRef>,
    entry_point: VirtualAddress,
) -> ThreadRef {
    let tid_copy = *tid;
    let thread = ThreadRef(Arc::new(ThreadHandle {
        inner_const: ThreadConstantInner { process, tid },
        inner_locked: SpinLock::new(ThreadLockedInner {}),
        inner_refcell: RefCell::new(ThreadInner {
            stack_top,
            state: ThreadState {
                rsp: stack_top.address(),
                rip: entry_point.address(),
            },
        }),
    }));

    if let Some(process) = thread.inner_const.process.as_ref() {
        let mut inner = process.inner_locked();
        let idx = inner.add_thread(thread.clone()).unwrap(); // thread just created so can't already be a member
        drop(inner);

        trace!(
            "new thread {:?} added to process {:?} as thread #{}",
            tid_copy,
            process.pid(),
            idx
        );
    } else {
        trace!("new kernel thread {:?}", tid_copy);
    }

    thread
}

impl Deref for ThreadHandle {
    type Target = ThreadConstantInner;

    fn deref(&self) -> &Self::Target {
        &self.inner_const
    }
}

impl Deref for ThreadRef {
    type Target = ThreadHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ThreadConstantInner {
    pub fn tid(&self) -> Pid {
        *self.tid
    }
}

impl ThreadHandle {
    pub unsafe fn run_now(&self) -> ! {
        self.address_space().load_if_not_current();

        if self.is_usermode() {
            // set stack for interrupts
            // TODO necessary every time or just first?
            // {
            //     let rsp: u64;
            //     unsafe {
            //         asm!("mov {0}, rsp", out(reg) rsp);
            //     }
            //     crate::descriptor_tables::tss().set_privilege_stack(0, VirtualAddress(rsp));
            // }
            //
            // TODO restore thread state
            todo!("jmp to usermode")
        }

        let state = self.thread_state();

        asm!(
        // TODO restore general purpose registers except rax

        // setup new stack, using rax
        "mov rsp, rax",

        // push ret address onto new stack
        "push {rip}",

        // restore rax
        "mov rax, {rax}",

        // jmp to rip and restore stack
        "ret",

        rip = in(reg) state.rip,
        rax = const 0xdeadbeef_u32, // TODO actual rax
        in("rax") state.rsp,
        // TODO clobbers
        );

        unreachable!()
    }

    unsafe fn thread_state(&self) -> &ThreadState {
        // ensure we can access state by borrowing
        let _inner = self.inner_refcell.borrow();

        let ptr = self.inner_refcell.as_ptr() as *const ThreadInner;
        &*memoffset::raw_field!(ptr, ThreadInner, state)
    }

    pub fn address_space(&self) -> AddressSpaceRef<'static, '_> {
        self.inner_const
            .process
            .as_ref()
            .and_then(|proc| proc.address_space())
            .unwrap_or_else(AddressSpace::kernel)
    }

    pub fn is_usermode(&self) -> bool {
        match self.inner_const.process.as_ref() {
            None => false,
            Some(proc) => proc.privilege_level().user(),
        }
    }
}
