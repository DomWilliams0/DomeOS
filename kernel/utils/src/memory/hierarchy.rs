use crate::memory::address::{PhysicalAddress, VirtualAddress};
use crate::memory::page_table::PageTable;
use crate::memory::PhysicalFrame;
use crate::{KernelError, KernelResult};
use core::ops::{Deref, DerefMut};

pub trait PageTableHierarchy<'p> {
    type NextLevel: PageTableHierarchy<'p>;
    const NAME: &'static str;

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Err(KernelError::TableNotSupported(Self::NAME))
    }

    fn with_frame(frame: Frame) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Err(KernelError::FrameNotSupported(Self::NAME))
    }

    fn entry_index(addr: VirtualAddress) -> u16;

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>>;

    // fn current(e: &'p mut CommonEntry<'p, P>) -> ResolveResult<'p, Self::NextLevel>;

    // fn traverse(&self, addr: VirtualAddress) -> ResolveResult<'p, Self::NextLevel>;
}

/// PML4T
#[derive(Debug)]
pub struct P4<'p>(&'p mut PageTable<'p, P3<'p>>);

/// PDPT
#[derive(Debug)]
pub struct P3<'p>(&'p mut PageTable<'p, P2<'p>>);

#[derive(Debug)]
pub enum P2<'p> {
    PDT(&'p mut PageTable<'p, P1<'p>>),
    Huge1GPage(Frame),
}

#[derive(Debug)]
pub enum P1<'p> {
    PT(&'p mut PageTable<'p, Frame>),
    Huge2MPage(Frame),
}

#[derive(Debug)]
pub struct Frame(pub PhysicalAddress);

impl<'p> PageTableHierarchy<'p> for P4<'p> {
    type NextLevel = P3<'p>;
    const NAME: &'static str = "PML4";

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self> {
        Ok(Self(table))
    }

    fn entry_index(addr: VirtualAddress) -> u16 {
        addr.pml4t_offset()
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
        Ok(self.0)
    }
}

impl<'p> PageTableHierarchy<'p> for P3<'p> {
    type NextLevel = P2<'p>;
    const NAME: &'static str = "PDP";

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self> {
        Ok(Self(table))
    }

    fn entry_index(addr: VirtualAddress) -> u16 {
        addr.pdp_offset()
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
        Ok(self.0)
    }
}

impl<'p> PageTableHierarchy<'p> for P2<'p> {
    type NextLevel = P1<'p>;
    const NAME: &'static str = "PD";

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self> {
        Ok(Self::PDT(table))
    }

    fn with_frame(frame: Frame) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Ok(Self::Huge1GPage(frame))
    }

    fn entry_index(addr: VirtualAddress) -> u16 {
        addr.pd_offset()
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
        match self {
            P2::PDT(table) => Ok(*table),
            P2::Huge1GPage(frame) => Err(KernelError::NoTableAvailable(Self::NAME, frame.0)),
        }
    }
}

impl<'p> PageTableHierarchy<'p> for P1<'p> {
    type NextLevel = Frame;
    const NAME: &'static str = "PT";

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self> {
        Ok(Self::PT(table))
    }

    fn with_frame(frame: Frame) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Ok(Self::Huge2MPage(frame))
    }

    fn entry_index(addr: VirtualAddress) -> u16 {
        addr.pt_offset()
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
        match self {
            P1::PT(table) => Ok(*table),
            P1::Huge2MPage(frame) => Err(KernelError::NoTableAvailable(Self::NAME, frame.0)),
        }
    }
}

impl<'p> PageTableHierarchy<'p> for Frame {
    type NextLevel = Self;
    const NAME: &'static str = "Page";

    fn with_frame(frame: Frame) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Ok(frame)
    }

    fn entry_index(addr: VirtualAddress) -> u16 {
        todo!()
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
        Err(KernelError::NoTableAvailable(Self::NAME, self.0))
    }
}

impl<'p> Deref for P4<'p> {
    type Target = PageTable<'p, P3<'p>>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'p> DerefMut for P4<'p> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'p> Deref for P3<'p> {
    type Target = PageTable<'p, P2<'p>>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'p> DerefMut for P3<'p> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'p> P4<'p> {
    /// Table is already initialized and should not be cleared
    pub fn with_initialized(table: &'p mut PageTable<'p, P3<'p>>) -> Self {
        P4(table)
    }

    /// # Safety
    /// Frame must be unused and present and writable, this will blat it with zeros
    pub unsafe fn new(frame: PhysicalFrame) -> Self {
        let table: &mut PageTable<_> = frame.as_mut();

        *table = core::mem::zeroed();

        Self(table)
    }
}
