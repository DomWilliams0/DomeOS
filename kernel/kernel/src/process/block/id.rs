use common::Deref;
use core::fmt::{Debug, Formatter};

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Pid(u64);

/// Allocated on creation and freed globally on destruction
#[derive(Eq, PartialEq, Deref)]
#[repr(transparent)]
pub struct OwnedPid(Pid);

static mut NEXT_PID: Pid = Pid(1);

impl OwnedPid {
    fn allocate() -> Self {
        // TODO recycle used pids
        unsafe {
            let this = NEXT_PID;
            NEXT_PID = Pid(NEXT_PID.0.wrapping_add(1));
            OwnedPid(this)
        }
    }

    fn free(&self) {
        // TODO pool freed pid
    }
}

pub fn new_pid() -> OwnedPid {
    OwnedPid::allocate()
}

impl Drop for OwnedPid {
    fn drop(&mut self) {
        self.free()
    }
}

impl Debug for Pid {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Pid({:#x})", self.0)
    }
}

impl Debug for OwnedPid {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
