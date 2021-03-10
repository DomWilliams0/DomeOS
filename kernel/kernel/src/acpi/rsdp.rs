use crate::acpi::sdts::{DescriptionTable, Fadt, FadtRevision1, FadtRevision2};
use crate::acpi::AcpiError;
use common::*;
use core::fmt::{Debug, Formatter};
use memory::{PhysicalAddress, VirtualAddress};

#[repr(C, packed)]
pub struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_addr: u32,
}

// #[repr(C, packed)]
// struct RsdpTwoPointOh {
//     first: Rsdp,
//     length: u32,
//     xsdt_addr: u64,
//     extended_checksum: u8,
//     reserved: [u8; 3],
// }

#[repr(C)]
pub struct AcpiSdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[repr(C)]
pub struct Rsdt {
    header: AcpiSdtHeader,
    others_ptr: u32,
}

impl Rsdp {
    /// # Safety
    /// Physical identity map must be initialized
    pub unsafe fn find_and_validate() -> Result<&'static Self, AcpiError> {
        let rsdp = Self::find().ok_or(AcpiError::RsdpNotFound)?;
        trace!("RSDP: {:?}", rsdp);

        if !validate_checksum(rsdp, core::mem::size_of::<Rsdp>()) {
            return Err(AcpiError::InvalidChecksum("RSDP"));
        }

        if rsdp.revision != 0 {
            return Err(AcpiError::UnsupportedVersion(rsdp.revision));
        }

        Ok(rsdp)
    }

    pub fn rsdt(&self) -> Result<&Rsdt, AcpiError> {
        let rsdt = {
            let ptr = resolve_phys(self.rsdt_addr as *const Rsdt);

            // safety: rsdp checksum validated
            unsafe { &*ptr }
        };
        trace!("RSDT: {:?}", rsdt);

        if !rsdt.validate_checksum() {
            return Err(AcpiError::InvalidChecksum("RSDT"));
        }

        Ok(rsdt)
    }

    /// # Safety
    /// Physical identity map must be initialized
    unsafe fn find() -> Option<&'static Rsdp> {
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
                (sig == b"RSD PTR ").then(|| &*(ptr as *const Rsdp))
            })
    }
}

impl Rsdt {
    fn validate_checksum(&self) -> bool {
        validate_checksum(self, self.header.length as usize)
    }

    fn iter(&self) -> impl Iterator<Item = &AcpiSdtHeader> + '_ {
        let n = (self.header.length as usize - core::mem::size_of::<AcpiSdtHeader>()) / 4;
        unsafe {
            core::slice::from_raw_parts((&self.others_ptr) as *const u32, n)
                .iter()
                .map(|ptr| {
                    let ptr = (*ptr) as *const AcpiSdtHeader;
                    &*resolve_phys(ptr)
                })
        }
    }

    pub fn lookup<T: DescriptionTable>(&self) -> Result<&T, AcpiError> {
        let header = self
            .iter()
            .find(|h| h.signature == *T::SIGNATURE.as_bytes())
            .ok_or(AcpiError::NoSuchTable(T::SIGNATURE))?;

        if !validate_checksum(header, header.length as usize) {
            return Err(AcpiError::InvalidChecksum(T::SIGNATURE));
        }

        trace!("{:?}", header);

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
        if let Ok(fadt) = self.lookup::<FadtRevision1>() {
            Ok(fadt as &dyn Fadt)
        } else {
            self.lookup::<FadtRevision2>().map(|fadt| fadt as &dyn Fadt)
        }
    }
}

fn validate_checksum<T>(val: &T, len: usize) -> bool {
    let as_bytes = unsafe { core::slice::from_raw_parts(val as *const T as *const u8, len) };

    let sum = as_bytes
        .iter()
        .fold(0_u8, |acc, val| acc.wrapping_add(*val));

    (sum & 0xf) == 0
}

fn resolve_phys<T>(ptr: *const T) -> *const T {
    VirtualAddress::from_physical(PhysicalAddress(ptr as u64)).as_const_ptr()
}

impl Debug for Rsdp {
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

impl Debug for Rsdt {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Rsdt({:?}, others={:#x})", self.header, self.others_ptr)
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
