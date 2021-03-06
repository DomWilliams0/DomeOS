use crate::memory::{AddressSpace, AddressSpaceRef};
use crate::process::block::id::{OwnedPid, Pid};
use crate::process::block::thread::ThreadRef;
use crate::spinlock::SpinLock;
use alloc::sync::Arc;
use common::*;
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
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
    /// * None: use default kernel address space
    /// * Some: use own address space, destroyed along with this process
    addr_space: Option<AddressSpace<'static>>,

    pid: OwnedPid,

    pl: ProcessPrivilegeLevel,
}

/// Protected by mutex
pub struct ProcessLockedInner {
    threads: SmallVec<[ThreadRef; 2]>,
}

/// Protected by a refcell
struct ProcessInner {}

#[derive(Copy, Clone)]
pub enum ProcessPrivilegeLevel {
    User,
    Kernel,
}

pub fn new_process(
    addr_space: Option<AddressSpace<'static>>,
    pid: OwnedPid,
    pl: ProcessPrivilegeLevel,
) -> ProcessRef {
    let pid_copy = *pid;
    let process = ProcessRef(Arc::new(ProcessHandle {
        inner_const: ProcessConstantInner {
            addr_space,
            pid,
            pl,
        },
        inner_locked: SpinLock::new(ProcessLockedInner {
            threads: SmallVec::new(),
        }),
        inner_refcell: RefCell::new(ProcessInner {}),
    }));

    trace!("new process {:?}", pid_copy);

    process
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

    // TODO lifetime probably shouldn't be static on the clone
    pub fn address_space(&self) -> Option<AddressSpaceRef<'static, '_>> {
        self.inner_const.addr_space.as_ref().map(|a| a.borrow())
    }
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
    pub fn user(self) -> bool {
        matches!(self, Self::User)
    }
}
