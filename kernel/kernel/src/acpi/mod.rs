use crate::acpi::rsdp::Rsdp;
use common::*;

mod rsdp;
mod rsdt;
mod sdts;
mod util;

#[derive(Display)]
pub enum AcpiError {
    /// RSDP not found
    RsdpNotFound,

    /// Checksum for {0:?} is invalid
    InvalidChecksum(&'static str),

    /// Only ACPI 1.0 is supported (revision 0, but found revision {0})
    UnsupportedVersion(u8),

    /// No description table found with signature {0:?}
    NoSuchTable(&'static str),

    /// Description table with signature {signature:?} expected to be {expected} bytes but is actually {actual}
    LengthMismatch {
        signature: &'static str,
        expected: usize,
        actual: usize,
    },

    /// Description table with signature {signature:?} expected to be revision {expected} is actually {actual}
    RevisionMismatch {
        signature: &'static str,
        expected: u8,
        actual: u8,
    },

    /// No support for 8042 PS/2 controller
    NoPs2Controller,
}

/// # Safety
/// Physical identity map must be initialized
pub unsafe fn init() -> Result<(), AcpiError> {
    let rsdp = Rsdp::find_and_validate()?;
    let rsdt = rsdp.get_rsdt()?;

    for (i, sdt) in rsdt.iter().enumerate() {
        trace!("{}: {:?}", i, sdt);
    }

    // check for ps/2
    let fadt = rsdt.lookup_fadt()?;
    if !fadt.has_8042_ps2_controller() {
        return Err(AcpiError::NoPs2Controller);
    }

    Ok(())
}
