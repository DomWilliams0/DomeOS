use common::*;
use core::fmt::{Debug, Formatter};

use crate::acpi::rsdp::AcpiSdtHeader;
use crate::acpi::sdts::{DescriptionTable, Fadt, FadtRevision1, FadtRevision3};
use crate::acpi::util::{resolve_phys, validate_checksum};
use crate::acpi::AcpiError;

#[repr(C)]
pub struct Rsdt {
    pub header: AcpiSdtHeader,
    pub others_ptr: (),
}

#[derive(Deref)]
pub struct RsdtWrapper<'a> {
    #[deref]
    rsdt: &'a Rsdt,
    ty: RsdtType,
}

#[derive(Copy, Clone)]
pub enum RsdtType {
    Rsdt,
    Xsdt,
}

impl<'a> RsdtWrapper<'a> {
    /// # Safety
    /// Ptr must be physical and pointing to a RSDT/XSDT
    pub unsafe fn new(ptr: *const Rsdt, ty: RsdtType) -> Self {
        Self {
            rsdt: &*resolve_phys(ptr),
            ty,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &AcpiSdtHeader> + '_ {
        let ptr_size = match self.ty {
            RsdtType::Rsdt => 4,
            RsdtType::Xsdt => 8,
        };

        let n = (self.header.length as usize - core::mem::size_of::<AcpiSdtHeader>()) / ptr_size;

        let start = (&self.rsdt.others_ptr) as *const _ as *const u8;
        (0..n).map(move |i| unsafe {
            let ptr_ptr = start.add(i * ptr_size);
            let ptr = match self.ty {
                RsdtType::Rsdt => *(ptr_ptr as *const u32) as *const AcpiSdtHeader,
                RsdtType::Xsdt => *(ptr_ptr as *const u64) as *const AcpiSdtHeader,
            };
            &*resolve_phys(ptr)
        })
    }

    pub fn lookup<T: DescriptionTable>(&self, expected_revision: u8) -> Result<&T, AcpiError> {
        let header = self
            .iter()
            .find(|h| h.signature == *T::SIGNATURE.as_bytes())
            .ok_or(AcpiError::NoSuchTable(T::SIGNATURE))?;

        if !validate_checksum(header, header.length as usize) {
            return Err(AcpiError::InvalidChecksum(T::SIGNATURE));
        }

        if header.revision != expected_revision {
            return Err(AcpiError::RevisionMismatch {
                signature: T::SIGNATURE,
                expected: expected_revision,
                actual: header.revision,
            });
        }

        let expected_size = core::mem::size_of::<T>();
        let actual_size = header.length as usize;
        if expected_size != actual_size {
            Err(AcpiError::LengthMismatch {
                signature: T::SIGNATURE,
                expected: expected_size,
                actual: actual_size,
            })
        } else {
            Ok(unsafe { &*(header as *const _ as *const T) })
        }
    }

    pub fn lookup_fadt(&self) -> Result<&dyn Fadt, AcpiError> {
        if let Ok(fadt) = self.lookup::<FadtRevision1>(1) {
            Ok(fadt as &dyn Fadt)
        } else {
            self.lookup::<FadtRevision3>(3)
                .map(|fadt| fadt as &dyn Fadt)
        }
    }
}

impl Debug for Rsdt {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Rsdt({:?})", self.header)
    }
}
