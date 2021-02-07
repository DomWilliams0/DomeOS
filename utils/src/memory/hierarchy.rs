use crate::memory::address::PhysicalAddress;
use crate::memory::page_table::PageTable;

pub trait PageTableHierarchy<'p> {
    type NextLevel: PageTableHierarchy<'p>;

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Option<Self>
    where
        Self: Sized;

    fn with_frame(frame: Frame) -> Option<Self>
    where
        Self: Sized;

    // fn current(e: &'p mut CommonEntry<'p, P>) -> ResolveResult<'p, Self::NextLevel>;

    // fn traverse(&self, addr: VirtualAddress) -> ResolveResult<'p, Self::NextLevel>;
}

/// PML4T
#[derive(Debug)]
pub struct P4<'p>(pub &'p mut PageTable<'p, P3<'p>>);

/// PDPT
#[derive(Debug)]
pub struct P3<'p>(pub &'p mut PageTable<'p, P2<'p>>);

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

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Option<Self> {
        Some(Self(table))
    }

    fn with_frame(_: Frame) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

impl<'p> PageTableHierarchy<'p> for P3<'p> {
    type NextLevel = P2<'p>;

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Option<Self> {
        Some(Self(table))
    }

    fn with_frame(_: Frame) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}

impl<'p> PageTableHierarchy<'p> for P2<'p> {
    type NextLevel = P1<'p>;

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Option<Self> {
        Some(Self::PDT(table))
    }

    fn with_frame(frame: Frame) -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self::Huge1GPage(frame))
    }
}

impl<'p> PageTableHierarchy<'p> for P1<'p> {
    type NextLevel = Frame;

    fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Option<Self> {
        Some(Self::PT(table))
    }

    fn with_frame(frame: Frame) -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self::Huge2MPage(frame))
    }
}

impl<'p> PageTableHierarchy<'p> for Frame {
    type NextLevel = Self;

    fn with_table(_: &'p mut PageTable<'p, Self::NextLevel>) -> Option<Self> {
        None
    }

    fn with_frame(frame: Frame) -> Option<Self>
    where
        Self: Sized,
    {
        Some(frame)
    }
}
