use crate::cpu::CpuState;
use crate::descriptor_tables::{SEL_USER_CODE, SEL_USER_DATA};
use crate::memory::{AddressSpaceRef, StackGrowth};
use crate::process::block::id::{OwnedPid, Pid};
use crate::process::block::process::{kernel_process, ProcessRef};
use crate::spinlock::SpinLock;
use alloc::sync::Arc;
use common::*;
use core::cell::RefCell;
use core::ops::Deref;
use memory::{MemoryError, VirtualAddress};

#[derive(Clone)]
#[repr(transparent)]
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
    process: ProcessRef,
    kernel_stack: VirtualAddress,

    tid: OwnedPid,
}

/// Protected by mutex
struct ThreadLockedInner {}

/// Protected by a refcell
struct ThreadInner {
    state: ThreadState,
}

#[derive(Default)]
struct ThreadState {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rdi: u64,
    rsi: u64,

    rbp: u64,
    rsp: u64,

    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,

    rflags: u64,
    rip: u64,
    // TODO float regs
    // TODO SSE/MMX if necessary
}

pub enum ThreadProcess {
    Process(ProcessRef),
    KernelThread,
}

impl ThreadRef {
    pub fn new(
        process: ThreadProcess,
        tid: OwnedPid,
        entry_point: VirtualAddress,
    ) -> Result<ThreadRef, MemoryError> {
        let (process, user_stack, kernel_stack) = {
            let proc = match process {
                ThreadProcess::Process(proc) => proc,
                ThreadProcess::KernelThread => kernel_process(),
            };

            let (user, kernel) = proc.allocate_new_thread_stacks()?;
            trace!(
                "thread {:?} has stack at {:?} and kernel stack at {:?}",
                tid,
                user,
                kernel
            );
            (proc, user, kernel)
        };

        let tid_copy = *tid;
        let thread = ThreadRef(Arc::new(ThreadHandle {
            inner_const: ThreadConstantInner {
                process: process.clone(),
                kernel_stack,
                tid,
            },
            inner_locked: SpinLock::new(ThreadLockedInner {}),
            inner_refcell: RefCell::new(ThreadInner {
                state: ThreadState {
                    rsp: user_stack.address(),
                    rip: entry_point.address(),
                    ..ThreadState::default()
                },
            }),
        }));

        // register with process
        let idx = {
            let mut inner = process.inner_locked();
            inner.add_thread(thread.clone()).unwrap() // thread just created so can't already be a member
        };

        trace!(
            "new thread {:?} added to process {:?} as thread #{}",
            tid_copy,
            process.pid(),
            idx
        );

        Ok(thread)
    }

    pub unsafe fn switch_to(&self) -> ! {
        CpuState::update_current_thread(self.clone());

        self.0.run_now()
    }

    pub fn grow_user_stack(&self, growth: StackGrowth) -> Result<(), MemoryError> {
        self.process.grow_user_thread_stack(growth)
    }
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

    pub fn kernel_stack(&self) -> VirtualAddress {
        self.kernel_stack
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        trace!("dropping thread {:?}", self.tid);
        // TODO unmap kernel stack
    }
}

const OFFSET_RAX: usize = memoffset::offset_of!(ThreadState, rax);
const OFFSET_RBX: usize = memoffset::offset_of!(ThreadState, rbx);
const OFFSET_RCX: usize = memoffset::offset_of!(ThreadState, rcx);
const OFFSET_RDX: usize = memoffset::offset_of!(ThreadState, rdx);
const OFFSET_RDI: usize = memoffset::offset_of!(ThreadState, rdi);
const OFFSET_RSI: usize = memoffset::offset_of!(ThreadState, rsi);
const OFFSET_RBP: usize = memoffset::offset_of!(ThreadState, rbp);
const OFFSET_R08: usize = memoffset::offset_of!(ThreadState, r8);
const OFFSET_R09: usize = memoffset::offset_of!(ThreadState, r9);
const OFFSET_R10: usize = memoffset::offset_of!(ThreadState, r10);
const OFFSET_R11: usize = memoffset::offset_of!(ThreadState, r11);
const OFFSET_R12: usize = memoffset::offset_of!(ThreadState, r12);
const OFFSET_R13: usize = memoffset::offset_of!(ThreadState, r13);
const OFFSET_R14: usize = memoffset::offset_of!(ThreadState, r14);
const OFFSET_R15: usize = memoffset::offset_of!(ThreadState, r15);
const OFFSET_RIP: usize = memoffset::offset_of!(ThreadState, rip);
const OFFSET_RSP: usize = memoffset::offset_of!(ThreadState, rsp);
const OFFSET_RFLAGS: usize = memoffset::offset_of!(ThreadState, rflags);

impl ThreadHandle {
    unsafe fn run_now(&self) -> ! {
        self.address_space().load_if_not_current();

        let state = self.thread_state();

        if self.is_user() {
            Self::restore_user_space(state)
        } else {
            Self::restore_kernel_space(state)
        }
    }

    unsafe fn restore_kernel_space(state: *const ThreadState) -> ! {
        asm!(
            // move to new stack
            "mov rsp, [rax + {offset_rsp}]",
            "push [rax + {offset_rip}]",

            // restore rflags
            "push [rax + {offset_rflags}]",
            "popfq",

            // push new rax value to pop when we're done with thread ctx
            "push [rax + {offset_rax}]",

            // restore GPRs
            "mov r15, [rax + {offset_r15}]",
            "mov r14, [rax + {offset_r14}]",
            "mov r13, [rax + {offset_r13}]",
            "mov r12, [rax + {offset_r12}]",
            "mov r11, [rax + {offset_r11}]",
            "mov r10, [rax + {offset_r10}]",
            "mov r9,  [rax + {offset_r09}]",
            "mov r8,  [rax + {offset_r08}]",
            "mov rsi, [rax + {offset_rsi}]",
            "mov rdi, [rax + {offset_rdi}]",
            "mov rdx, [rax + {offset_rdx}]",
            "mov rcx, [rax + {offset_rcx}]",
            "mov rbx, [rax + {offset_rbx}]",

            // restore rbp last (needed to access thread state)
            "mov rbp, [rax + {offset_rbp}]",

            // restore rax
            "pop rax",

            // jmp to rip and restore stack
            "ret",

            offset_r15 = const OFFSET_R15, offset_r14 = const OFFSET_R14, offset_r13 = const OFFSET_R13,
            offset_r12 = const OFFSET_R12, offset_r11 = const OFFSET_R11, offset_r10 = const OFFSET_R10,
            offset_r09 = const OFFSET_R09, offset_r08 = const OFFSET_R08, offset_rsi = const OFFSET_RSI,
            offset_rdi = const OFFSET_RDI, offset_rdx = const OFFSET_RDX, offset_rcx = const OFFSET_RCX,
            offset_rbx = const OFFSET_RBX, offset_rax = const OFFSET_RAX, offset_rbp = const OFFSET_RBP,
            offset_rsp = const OFFSET_RSP, offset_rip = const OFFSET_RIP, offset_rflags = const OFFSET_RFLAGS,

            in("rax") state,

            options(noreturn, readonly)
        )
    }

    unsafe fn restore_user_space(state: *const ThreadState) -> ! {
        asm!(
            // disable interrupts
            "cli",

            // setup segment registers
            "mov bx, {ds_user}",
            "mov ds, bx",
            "mov es, bx",
            "mov fs, bx",
            "mov gs, bx", // ss handled by iret

            // user stack in ds
            "push {ds_user}",
            "push [rax + {offset_rsp}]", // user rsp
            "pushf", // eflags

            // user rip in cs
            "push {cs_user}",
            "push [rax + {offset_rip}]", // user rip

            // push new rax value to pop when we're done with thread ctx
            "push [rax + {offset_rax}]",

            // restore GPRs
            "mov r15, [rax + {offset_r15}]",
            "mov r14, [rax + {offset_r14}]",
            "mov r13, [rax + {offset_r13}]",
            "mov r12, [rax + {offset_r12}]",
            "mov r11, [rax + {offset_r11}]",
            "mov r10, [rax + {offset_r10}]",
            "mov r9,  [rax + {offset_r09}]",
            "mov r8,  [rax + {offset_r08}]",
            "mov rsi, [rax + {offset_rsi}]",
            "mov rdi, [rax + {offset_rdi}]",
            "mov rdx, [rax + {offset_rdx}]",
            "mov rcx, [rax + {offset_rcx}]",
            "mov rbx, [rax + {offset_rbx}]",

            // restore rbp last (needed to access thread state)
            "mov rbp, [rax + {offset_rbp}]",

            // restore rax
            "pop rax",

            // jmp to rip
            "iretq",

            ds_user = const SEL_USER_DATA,
            cs_user = const SEL_USER_CODE,

            offset_r15 = const OFFSET_R15, offset_r14 = const OFFSET_R14, offset_r13 = const OFFSET_R13,
            offset_r12 = const OFFSET_R12, offset_r11 = const OFFSET_R11, offset_r10 = const OFFSET_R10,
            offset_r09 = const OFFSET_R09, offset_r08 = const OFFSET_R08, offset_rsi = const OFFSET_RSI,
            offset_rdi = const OFFSET_RDI, offset_rdx = const OFFSET_RDX, offset_rcx = const OFFSET_RCX,
            offset_rbx = const OFFSET_RBX, offset_rax = const OFFSET_RAX, offset_rbp = const OFFSET_RBP,
            offset_rsp = const OFFSET_RSP, offset_rip = const OFFSET_RIP,

            in("rax") state,

            options(noreturn, readonly),
        )
    }

    fn thread_state(&self) -> *const ThreadState {
        // ensure we can access state by borrowing first
        let _inner = self.inner_refcell.borrow();

        let ptr = self.inner_refcell.as_ptr() as *const ThreadInner;
        memoffset::raw_field!(ptr, ThreadInner, state)
    }

    pub fn address_space(&self) -> AddressSpaceRef<'static, '_> {
        self.inner_const.process.address_space()
    }

    pub fn is_user(&self) -> bool {
        self.inner_const.process.privilege_level().is_user()
    }
}
