#![allow(clippy::upper_case_acronyms)]

use crate::memory::address::{PhysicalAddress, VirtualAddress};
use crate::memory::page_table::PageTable;
use crate::memory::PhysicalFrame;
use crate::{KernelError, KernelResult};
use derive_more::{Deref, DerefMut};

pub trait PageTableHierarchy<'p>: core::fmt::Debug {
    type NextLevel: PageTableHierarchy<'p>;
    const NAME: &'static str;

    fn with_table(_table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Err(KernelError::TableNotSupported(Self::NAME))
    }

    fn with_frame(_frame: Frame) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Err(KernelError::FrameNotSupported(Self::NAME))
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>>;
}

pub trait HasTable<'p>: PageTableHierarchy<'p> {
    fn entry_index(addr: VirtualAddress) -> u16;
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

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self> {
        Ok(Self(table))
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
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

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self> {
        Ok(Self(table))
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
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

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self> {
        Ok(Self::PDT(table))
    }

    fn with_frame(frame: Frame) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Ok(Self::Huge1GPage(frame))
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
        match self {
            P2::PDT(table) => Ok(*table),
            P2::Huge1GPage(frame) => Err(KernelError::NoTableAvailable(Self::NAME, frame.0)),
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

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> KernelResult<Self> {
        Ok(Self::PT(table))
    }

    fn with_frame(frame: Frame) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Ok(Self::Huge2MPage(frame))
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
        match self {
            P1::PT(table) => Ok(*table),
            P1::Huge2MPage(frame) => Err(KernelError::NoTableAvailable(Self::NAME, frame.0)),
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

    fn with_frame(frame: Frame) -> KernelResult<Self>
    where
        Self: Sized,
    {
        Ok(frame)
    }

    fn table_mut(&mut self) -> KernelResult<&mut PageTable<'p, Self::NextLevel>> {
        Err(KernelError::NoTableAvailable(Self::NAME, self.0))
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
