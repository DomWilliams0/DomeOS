use crate::cursor::Reinterpret;
use strum::EnumIter;

#[derive(Debug)]
#[repr(C)]
pub struct DataDirectoryDescriptor {
    pub(crate) rva: u32,
    pub(crate) size: u32,
}

#[derive(Debug, Copy, Clone, EnumIter)]
pub enum DataDirectoryType {
    ExportTable = 0,
    ImportTable,
    ResourceTable,
    ExceptionTable,
    CertificateTable,
    BaseRelocationTable,
    Debug,
    Architecture,
    GlobalPointer,
    TlsTable,
    LoadConfigTable,
    BoundImport,
    Iat,
    DelayImportDescriptor,
    ClrRuntimeHeader,
}

pub trait DataDirectory {
    const ORDINAL: DataDirectoryType;
}

impl DataDirectoryDescriptor {
    pub fn valid(&self) -> bool {
        self.rva != 0 && self.size != 0
    }
}

// safety: raw PE type
unsafe impl Reinterpret for DataDirectoryDescriptor {}
