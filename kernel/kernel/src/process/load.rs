use crate::memory::AddressSpace;
use crate::process::block::{new_pid, new_process, new_thread, ProcessPrivilegeLevel, ProcessRef};
use crate::process::error::ProcessError;

use common::{
    anyhow::{self, anyhow, Error},
    *,
};
use memory::{kilobytes, round_up_to, MapFlags, MapTarget, VirtualAddress, FRAME_SIZE};
use pe::{Address, Pe, PeError};

// temporary
const NOP_EXE: &[u8] = include_bytes!("../../../../userspace/nop.exe");

const STACK_START: u64 = 0x6666_0000;
const INITIAL_STACK_SIZE: u64 = kilobytes(128);

// TODO need to configure via args:
//  * user vs kernel
//  * exe mapped address/pointer/reference

// TODO ensure address space and/or mappings are freed/unmapped on error with e.g. a Bomb guard
/// For testing only
/// Switches to new address space
pub fn experiment_new_process() -> anyhow::Result<ProcessRef> {
    let image = NOP_EXE;

    // allocate and load new addr space for PE
    // TODO not for kernel threads
    let mut address_space = AddressSpace::new().map_err(Error::msg)?;
    unsafe {
        address_space.load_unconditionally();
    }

    // parse PE
    // TODO parse in new userspace process in its own address space
    //  * PE loader needs to be available as a userspace dll
    //      * not necessarily on filesystem
    //  * PE loader needs to replace itself with the new process
    let pe = pe::Pe::from_buffer(image).map_err(Error::msg)?;

    // check image can be mapped at preferred base without relocating
    let (image_base, pages_needed, entry_point_rva) =
        extract_optional_header(&pe).map_err(Error::msg)?;
    let free_base = address_space
        .find_free_space(image_base, pages_needed)
        .map_err(Error::msg)?;

    if free_base != image_base {
        // TODO support relocations
        return Err(anyhow!(ProcessError::RelocationUnsupported));
    }

    debug!("mapping image at preferred base address {:?}", image_base);

    let entry_point_rva = entry_point_rva
        .ok_or(ProcessError::NoEntrypoint)
        .map_err(Error::msg)?;

    let length = pages_needed * FRAME_SIZE as usize;
    let mut mapped = {
        // map as RWX and userspace (TODO depends on options)
        address_space
            .map_range(
                image_base,
                length as u64,
                MapTarget::Any,
                MapFlags::Writeable | MapFlags::Executable | MapFlags::User,
            )
            .map_err(Error::msg)?
    };
    let mapped_slice = mapped.write().unwrap(); // mapped as writeable

    trace!("mapped {:#x} bytes for image", length);

    // copy headers
    // TODO mmap to exe file instead of copying manually
    {
        let headers = pe.headers().map_err(Error::msg)?;
        let dst = mapped_slice
            .get_mut(..headers.len())
            .ok_or(ProcessError::LengthMismatch {
                src: headers.len(),
                dst: length,
            })
            .map_err(Error::msg)?;

        dst.copy_from_slice(headers);

        // TODO update header permissions as ro
    }

    // copy sections
    for section in pe.sections().map_err(Error::msg)? {
        let section = match section.and_then(|s| s.as_mappable()).map_err(Error::msg)? {
            Some(mappable) => mappable,
            None => continue,
        };

        debug!("mapping section {:?}", section);

        let dst = {
            let start = section.virtual_address.into_usize();

            mapped_slice
                .get_mut(start..start + section.virtual_size)
                .ok_or(PeError::VirtualSliceOutOfBounds {
                    what: "section",
                    addr: section.virtual_address,
                    length,
                })
                .map_err(Error::msg)?
        };

        // copy raw data if any
        let zero_from = if let Some((size, offset)) = section.raw_data {
            let src = pe.slice(offset, size).map_err(Error::msg)?;
            let src = &src[..dst.len()]; // shorten to virtual size
            dst.copy_from_slice(src); // TODO panics if virtual size > raw size
            dst.len()
        } else {
            0
        };

        // zero the rest
        let zeros = section.virtual_size - zero_from;
        if zeros > 0 {
            trace!("zeroing {:#x} bytes in section {}", zeros, section.name);
            let slice = &mut dst[zero_from..section.virtual_size];
            slice.fill(0);
        }

        // TODO protect sections properly
    }

    // allocate a stack
    // TODO guard page to grow stack dynamically
    let stack_bottom = VirtualAddress::with_literal(STACK_START);
    let stack_size = INITIAL_STACK_SIZE;
    let mapped = address_space
        .map_range(
            stack_bottom,
            stack_size,
            MapTarget::Any,
            MapFlags::Writeable | MapFlags::User | MapFlags::Commit,
        )
        .map_err(Error::msg)?;

    let stack_top = mapped.end_address() - 64;
    let entry_point = image_base + entry_point_rva;

    // TODO allocate heap
    // TODO respect PE requested heap+stack commit/reserve
    // TODO flush instruction cache?

    let proc = new_process(
        Some(address_space),
        new_pid(),
        ProcessPrivilegeLevel::Kernel,
    );
    let _thread = new_thread(new_pid(), stack_top, Some(proc.clone()), entry_point);
    Ok(proc)
}

/// (image base, size of image in 4k pages, entrypoint RVA)
fn extract_optional_header(pe: &Pe) -> Result<(VirtualAddress, usize, Option<u64>), ProcessError> {
    let opt_header = pe.optional_header()?;
    let base = opt_header.image_base()?;
    let base = VirtualAddress::new_checked(base).ok_or(ProcessError::BadVirtualAddress(base))?;

    let size_of_image = opt_header.size_of_image()?;
    let pages_needed = round_up_to(size_of_image as u64, FRAME_SIZE) / FRAME_SIZE;

    Ok((base, pages_needed as usize, opt_header.entry_point()))
}
