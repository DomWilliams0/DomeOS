use crate::memory::{
    AddressSpace, AddressSpaceRef, ProcessKernelStacks, ProcessUserStacks, StackGrowth, Stacks,
};
use crate::process::block::id::{OwnedPid, Pid};
use crate::process::block::new_pid;
use crate::process::block::thread::ThreadRef;
use crate::spinlock::SpinLock;
use alloc::sync::Arc;
use common::*;
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use memory::{MemoryError, VirtualAddress};
use smallvec::SmallVec;

#[derive(Clone)]
pub struct ProcessRef(Arc<ProcessHandle>);

/// Only immutable references are handed out, fields are protected by refcells/mutexes as needed
pub struct ProcessHandle {
    /// Set once on creation and never changed
    inner_const: ProcessConstantInner,
    inner_locked: SpinLock<ProcessLockedInner>,
    inner_refcell: RefCell<ProcessInner>,
}

/// Not protected by mutex/refcell, readonly after creation
pub struct ProcessConstantInner {
    addr_space: AddressSpace<'static>,

    /// If true, destroy the address space on process exit
    owns_addr_space: bool,

    pid: OwnedPid,

    pl: ProcessPrivilegeLevel,
}

/// Protected by mutex
pub struct ProcessLockedInner {
    threads: SmallVec<[ThreadRef; 2]>,
    kernel_stacks: Stacks<ProcessKernelStacks>,
}

/// Protected by a refcell
struct ProcessInner {
    /// Not protected by mutex because:
    /// * Page fault handler needs to grow a stack (immutable ref needed)
    /// * New threads (after the main thread) are done by other user threads only, so will be
    ///   running on a single core only
    user_stacks: Stacks<ProcessUserStacks>,
}

#[derive(Copy, Clone)]
pub enum ProcessPrivilegeLevel {
    User,
    Kernel,
}

pub enum ProcessAddressSpace {
    Owned(AddressSpace<'static>),

    /// Use default shared kernel address space
    Kernel,
}

/// Shared kernel "process" that owns kernel threads and allocates their stacks
static mut KERNEL_PROCESS: InitializedGlobal<ProcessRef> = InitializedGlobal::uninit();

/// Must be called once only before other process/thread creation
pub fn init_kernel_process() {
    let process = ProcessRef::new(
        ProcessAddressSpace::Kernel,
        new_pid(),
        ProcessPrivilegeLevel::Kernel,
    );
    unsafe {
        KERNEL_PROCESS.init(process);
    }
}

impl ProcessRef {
    pub fn new(addr_space: ProcessAddressSpace, pid: OwnedPid, pl: ProcessPrivilegeLevel) -> Self {
        let pid_copy = *pid;
        let (addr_space, owns_addr_space) = match addr_space {
            ProcessAddressSpace::Owned(space) => (space, true),
            ProcessAddressSpace::Kernel => (AddressSpace::kernel(), false),
        };

        let process = ProcessRef(Arc::new(ProcessHandle {
            inner_const: ProcessConstantInner {
                addr_space,
                owns_addr_space,
                pid,
                pl,
            },
            inner_locked: SpinLock::new(ProcessLockedInner {
                threads: SmallVec::new(),
                kernel_stacks: Stacks::default(),
            }),
            inner_refcell: RefCell::new(ProcessInner {
                user_stacks: Stacks::default(),
            }),
        }));

        trace!("new process {:?}", pid_copy);

        process
    }
}

impl Deref for ProcessHandle {
    type Target = ProcessConstantInner;

    fn deref(&self) -> &Self::Target {
        &self.inner_const
    }
}

impl Deref for ProcessRef {
    type Target = ProcessHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ProcessHandle {
    #[inline]
    pub fn inner_lock(&self) -> &SpinLock<ProcessLockedInner> {
        &self.inner_locked
    }

    #[inline]
    pub fn inner_locked(&self) -> impl DerefMut<Target = ProcessLockedInner> + '_ {
        self.inner_locked.lock()
    }

    pub fn address_space(&self) -> AddressSpaceRef<'static, '_> {
        self.inner_const.addr_space.borrow()
    }

    /// (user thread stack, kernel thread stack)
    pub fn allocate_new_thread_stacks(
        &self,
    ) -> Result<(VirtualAddress, VirtualAddress), MemoryError> {
        let user = {
            let mut inner = self.inner_refcell.borrow_mut();
            inner.user_stacks.new_stack()
        };

        let kernel = {
            let mut inner = self.inner_locked.lock();
            inner.kernel_stacks.new_stack()
        };

        // mutex and refcell dropped asap

        let (user, _) = user?;
        let (kernel, _) = kernel?;
        Ok((user, kernel))
    }

    pub fn grow_user_thread_stack(&self, growth: StackGrowth) -> Result<(), MemoryError> {
        let inner = self.inner_refcell.borrow();
        inner.user_stacks.grow_stack(growth)
    }
}

pub fn kernel_process() -> ProcessRef {
    unsafe { KERNEL_PROCESS.get().clone() }
}

impl ProcessLockedInner {
    /// Err if already present
    pub fn add_thread(&mut self, thread: ThreadRef) -> Result<usize, ()> {
        // TODO add unsafe variant that doesn't do member check
        if self.threads.iter().any(|t| t.tid() == thread.tid()) {
            return Err(());
        }

        let idx = self.threads.len();
        self.threads.push(thread);
        Ok(idx)
    }

    pub fn threads(&self) -> impl Iterator<Item = &ThreadRef> + '_ {
        self.threads.iter()
    }
}

impl ProcessConstantInner {
    pub fn pid(&self) -> Pid {
        *self.pid
    }

    pub fn privilege_level(&self) -> ProcessPrivilegeLevel {
        self.pl
    }
}

impl ProcessPrivilegeLevel {
    pub fn is_user(self) -> bool {
        matches!(self, Self::User)
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        trace!("dropping process {:?}", self.pid);

        // TODO kill all threads
        // TODO destroy address space if owned
    }
}
