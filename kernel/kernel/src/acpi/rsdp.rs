use core::fmt::{Debug, Formatter};

use common::*;

use crate::acpi::rsdt::{Rsdt, RsdtType, RsdtWrapper};

use crate::acpi::util::{resolve_phys, validate_checksum};
use crate::acpi::AcpiError;

pub trait Rsdp: Debug {
    fn rsdp(&self) -> &RsdpOnePointOh;
    fn validate_checksum(&self) -> bool;
    fn rsdt(&self) -> RsdtWrapper;
}

#[repr(C, packed)]
pub struct RsdpOnePointOh {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_addr: u32,
}

#[repr(C, packed)]
struct RsdpTwoPointOh {
    first: RsdpOnePointOh,
    length: u32,
    xsdt_addr: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[repr(C)]
pub struct AcpiSdtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

impl dyn Rsdp {
    /// # Safety
    /// Physical identity map must be initialized
    pub unsafe fn find_and_validate() -> Result<&'static Self, AcpiError> {
        let rsdp = Self::find().ok_or(AcpiError::RsdpNotFound)?;
        trace!("RSDP: {:?}", rsdp);

        let rsdp = match rsdp.revision {
            0 => rsdp as &dyn Rsdp,
            2 => &*(rsdp as *const _ as *const RsdpTwoPointOh),
            _ => return Err(AcpiError::UnsupportedVersion(rsdp.revision)),
        };

        if !rsdp.validate_checksum() {
            return Err(AcpiError::InvalidChecksum("RSDP"));
        }

        Ok(rsdp)
    }

    pub fn get_rsdt(&self) -> Result<RsdtWrapper, AcpiError> {
        let rsdt = Rsdp::rsdt(self);
        trace!("RSDT: {:?}", rsdt.header);

        if !validate_checksum(*rsdt, rsdt.header.length as usize) {
            return Err(AcpiError::InvalidChecksum("RSDT"));
        }

        Ok(rsdt)
    }

    /// # Safety
    /// Physical identity map must be initialized
    unsafe fn find() -> Option<&'static RsdpOnePointOh> {
        // TODO look in multiboot
        let physical_search_regions = [0x40e..0x80e, 0xe0000..0xfffff];

        physical_search_regions
            .iter()
            .flat_map(|range| {
                range.clone().step_by(16).map(|addr| {
                    core::ptr::slice_from_raw_parts(resolve_phys(addr as *const u8), 8)
                        as *const [u8; 8]
                })
            })
            .find_map(|ptr| {
                let sig = &*ptr;
                (sig == b"RSD PTR ").then(|| &*(ptr as *const RsdpOnePointOh))
            })
    }
}

impl Rsdp for RsdpOnePointOh {
    fn rsdp(&self) -> &RsdpOnePointOh {
        self
    }

    fn validate_checksum(&self) -> bool {
        validate_checksum(self, core::mem::size_of::<RsdpOnePointOh>())
    }

    fn rsdt(&self) -> RsdtWrapper {
        // safety: rsdp checksum validated
        unsafe { RsdtWrapper::new(self.rsdt_addr as *const Rsdt, RsdtType::Rsdt) }
    }
}
impl Rsdp for RsdpTwoPointOh {
    fn rsdp(&self) -> &RsdpOnePointOh {
        &self.first
    }

    fn validate_checksum(&self) -> bool {
        validate_checksum(self, core::mem::size_of::<RsdpOnePointOh>())
            && validate_checksum(self, core::mem::size_of::<RsdpTwoPointOh>())
    }

    fn rsdt(&self) -> RsdtWrapper {
        let ptr = if self.xsdt_addr == 0 {
            // fallback to 32 bit addr if zero
            self.first.rsdt_addr as u64
        } else {
            self.xsdt_addr
        };

        // safety: rsdp checksum validated
        unsafe { RsdtWrapper::new(ptr as *const Rsdt, RsdtType::Xsdt) }
    }
}

impl Debug for RsdpOnePointOh {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Rsdp(checksum={:#x}, oem_id=", self.checksum)?;

        match core::str::from_utf8(&self.oem_id) {
            Ok(s) => write!(f, "'{}'", s),
            Err(_) => write!(f, "{:?}", self.oem_id),
        }?;

        let rsdt_addr = self.rsdt_addr;
        write!(
            f,
            ", revision={}, rsdt_addr={:#x})",
            self.revision, rsdt_addr
        )
    }
}

impl Debug for RsdpTwoPointOh {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let len = self.length;
        let xsdt_addr = self.xsdt_addr;

        write!(
            f,
            "Rsdp2({:?}, length={:?}, xsdt_addr={:#x})",
            self.first, len, xsdt_addr
        )
    }
}

impl Debug for AcpiSdtHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "AcpiStdHeader(")?;

        match core::str::from_utf8(&self.signature) {
            Ok(s) => write!(f, "'{}'", s),
            Err(_) => write!(f, "{:?}", self.oem_id),
        }?;

        write!(
            f,
            ", length={}, revision={}, checksum={:#x}, oem_id=",
            self.length, self.revision, self.checksum
        )?;

        match core::str::from_utf8(&self.oem_id) {
            Ok(s) => write!(f, "'{}'", s),
            Err(_) => write!(f, "{:?}", self.oem_id),
        }?;

        write!(f, ", oem_table_id=")?;

        match core::str::from_utf8(&self.oem_table_id) {
            Ok(s) => write!(f, "'{}'", s),
            Err(_) => write!(f, "{:?}", self.oem_table_id),
        }?;

        write!(
            f,
            ", creator_id={:#x}, creator_revision={})",
            self.creator_id, self.creator_revision
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_check() {
        assert_eq!(core::mem::size_of::<AcpiSdtHeader>(), 36);
    }
}
