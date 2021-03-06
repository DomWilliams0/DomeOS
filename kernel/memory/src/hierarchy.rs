#![allow(clippy::upper_case_acronyms)]

use crate::address::{PhysicalAddress, VirtualAddress};
use crate::error::MemoryResult;
use crate::page_table::PageTable;
use crate::{MemoryError, PhysicalFrame};
use common::*;

pub trait PageTableHierarchy<'p>: core::fmt::Debug {
    type NextLevel: PageTableHierarchy<'p>;
    const NAME: &'static str;

    fn with_table(_table: &'p mut PageTable<'p, Self::NextLevel>) -> MemoryResult<Self>
    where
        Self: Sized;

    fn with_frame(_frame: Frame) -> MemoryResult<Self>
    where
        Self: Sized,
    {
        Err(MemoryError::FrameNotSupported(Self::NAME))
    }

    fn table(&self) -> MemoryResult<&PageTable<'p, Self::NextLevel>>;

    fn table_mut(&mut self) -> MemoryResult<&mut PageTable<'p, Self::NextLevel>>;
}

pub trait HasTable<'p>: PageTableHierarchy<'p> {
    fn entry_index(addr: VirtualAddress) -> u16;
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AnyLevel {
    P4,
    P3,
    P2,
    P1,
    Frame,
}

/// PML4T
#[derive(Debug, Deref, DerefMut)]
pub struct P4<'p>(&'p mut PageTable<'p, P3<'p>>);

/// PDPT
#[derive(Debug, Deref, DerefMut)]
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

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> MemoryResult<Self> {
        Ok(Self(table))
    }

    fn table(&self) -> MemoryResult<&PageTable<'p, Self::NextLevel>> {
        Ok(self.0)
    }

    fn table_mut(&mut self) -> MemoryResult<&mut PageTable<'p, Self::NextLevel>> {
        Ok(self.0)
    }
}

impl<'p> HasTable<'p> for P4<'p> {
    fn entry_index(addr: VirtualAddress) -> u16 {
        addr.pml4t_offset()
    }
}

impl<'p> PageTableHierarchy<'p> for P3<'p> {
    type NextLevel = P2<'p>;
    const NAME: &'static str = "PDP";

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> MemoryResult<Self> {
        Ok(Self(table))
    }

    fn table(&self) -> MemoryResult<&PageTable<'p, Self::NextLevel>> {
        Ok(self.0)
    }

    fn table_mut(&mut self) -> MemoryResult<&mut PageTable<'p, Self::NextLevel>> {
        Ok(self.0)
    }
}

impl<'p> HasTable<'p> for P3<'p> {
    fn entry_index(addr: VirtualAddress) -> u16 {
        addr.pdp_offset()
    }
}

impl<'p> PageTableHierarchy<'p> for P2<'p> {
    type NextLevel = P1<'p>;
    const NAME: &'static str = "PD";

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> MemoryResult<Self> {
        Ok(Self::PDT(table))
    }

    fn with_frame(frame: Frame) -> MemoryResult<Self>
    where
        Self: Sized,
    {
        Ok(Self::Huge1GPage(frame))
    }

    //noinspection DuplicatedCode
    fn table(&self) -> MemoryResult<&PageTable<'p, Self::NextLevel>> {
        match self {
            P2::PDT(table) => Ok(*table),
            P2::Huge1GPage(frame) => {
                Err(MemoryError::NoTableAvailable(Self::NAME, frame.0.address()))
            }
        }
    }

    //noinspection DuplicatedCode
    fn table_mut(&mut self) -> MemoryResult<&mut PageTable<'p, Self::NextLevel>> {
        match self {
            P2::PDT(table) => Ok(*table),
            P2::Huge1GPage(frame) => {
                Err(MemoryError::NoTableAvailable(Self::NAME, frame.0.address()))
            }
        }
    }
}

impl<'p> HasTable<'p> for P2<'p> {
    fn entry_index(addr: VirtualAddress) -> u16 {
        addr.pd_offset()
    }
}

impl<'p> PageTableHierarchy<'p> for P1<'p> {
    type NextLevel = Frame;
    const NAME: &'static str = "PT";

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> MemoryResult<Self> {
        Ok(Self::PT(table))
    }

    fn with_frame(frame: Frame) -> MemoryResult<Self>
    where
        Self: Sized,
    {
        Ok(Self::Huge2MPage(frame))
    }

    //noinspection DuplicatedCode
    fn table(&self) -> MemoryResult<&PageTable<'p, Self::NextLevel>> {
        match self {
            P1::PT(table) => Ok(*table),
            P1::Huge2MPage(frame) => {
                Err(MemoryError::NoTableAvailable(Self::NAME, frame.0.address()))
            }
        }
    }

    //noinspection DuplicatedCode
    fn table_mut(&mut self) -> MemoryResult<&mut PageTable<'p, Self::NextLevel>> {
        match self {
            P1::PT(table) => Ok(*table),
            P1::Huge2MPage(frame) => {
                Err(MemoryError::NoTableAvailable(Self::NAME, frame.0.address()))
            }
        }
    }
}

impl<'p> HasTable<'p> for P1<'p> {
    fn entry_index(addr: VirtualAddress) -> u16 {
        addr.pt_offset()
    }
}

impl<'p> PageTableHierarchy<'p> for Frame {
    type NextLevel = Self;
    const NAME: &'static str = "Page";

    fn with_table(_table: &'p mut PageTable<'p, Self::NextLevel>) -> MemoryResult<Self>
    where
        Self: Sized,
    {
        Err(MemoryError::TableNotSupported(Self::NAME))
    }

    fn with_frame(frame: Frame) -> MemoryResult<Self>
    where
        Self: Sized,
    {
        Ok(frame)
    }

    fn table(&self) -> MemoryResult<&PageTable<'p, Self::NextLevel>> {
        Err(MemoryError::NoTableAvailable(Self::NAME, self.0.address()))
    }

    fn table_mut(&mut self) -> MemoryResult<&mut PageTable<'p, Self::NextLevel>> {
        Err(MemoryError::NoTableAvailable(Self::NAME, self.0.address()))
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
        frame.zero();

        Self(frame.as_mut())
    }

    /// # Safety
    /// Ensure caller is aware the returned value still points to the same physical address
    pub unsafe fn clone(&self) -> Self {
        #[allow(clippy::cast_ref_to_mut)]
        P4(&mut *(self.0 as *const _ as *mut _))
    }

    pub fn table_mut(&mut self) -> &mut PageTable<'p, P3<'p>> {
        self.0
    }

    pub fn ensure_accessible(&mut self) {
        if !VirtualAddress::is_identity_mapped_physical(self.0) {
            // is not currently accessible, needs to be offset into identity mapped region
            let virt = VirtualAddress::from_physical(PhysicalAddress(self.0 as *mut _ as u64));
            self.0 = unsafe { &mut *virt.as_ptr() }
        }
    }
}

impl PartialEq for P4<'_> {
    fn eq(&self, other: &Self) -> bool {
        let this = self.0 as *const PageTable<P3>;
        let other = other.0 as *const PageTable<P3>;
        core::ptr::eq(this, other)
    }
}

impl Eq for P4<'_> {}
