use crate::memory::AddressSpace;
use crate::process::block::Process;
use crate::process::error::ProcessError;
use alloc::rc::Rc;
use common::{
    anyhow::{self, anyhow, Error},
    *,
};
use memory::{kilobytes, round_up_to, MapFlags, MapTarget, VirtualAddress, FRAME_SIZE};
use pe::{Address, Pe, PeError};

// temporary
const NOP_EXE: &[u8] = include_bytes!("../../../../userspace/nop.exe");

const STACK_START: u64 = 0x6666_0000;
const INITIAL_STACK_SIZE: u64 = kilobytes(16);

// TODO need to configure via args:
//  * user vs kernel
//  * exe mapped address/pointer/reference

// TODO ensure address space and/or mappings are freed/unmapped on error with e.g. a Bomb guard
pub fn spawn_process() -> anyhow::Result<Rc<Process>> {
    let image = NOP_EXE;

    // parse PE
    let pe = pe::Pe::from_buffer(image).map_err(Error::msg)?;

    // TODO allocate address space
    let mut address_space = AddressSpace::current();

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
    let mapped_slice = {
        // map as RWX and on-demand
        address_space
            .map_range(
                image_base,
                length as u64,
                MapTarget::Any,
                MapFlags::Writeable | MapFlags::Executable,
            )
            .map_err(Error::msg)?;

        // create slice for bounds checking
        // safety: region has been mapped as writeable above
        unsafe { core::slice::from_raw_parts_mut(image_base.as_ptr::<u8>(), length as usize) }
    };

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

        // TODO mark headers as ro
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
        // TODO mmap sections instead of copy
    }

    // allocate a stack
    // TODO guard page to grow stack dynamically
    let stack_bottom = VirtualAddress::with_literal(STACK_START);
    let stack_size = INITIAL_STACK_SIZE;
    address_space
        .map_range(
            stack_bottom,
            stack_size,
            MapTarget::Any,
            MapFlags::Writeable,
        )
        .map_err(Error::msg)?;

    let stack_top = stack_bottom + stack_size;
    let entry_point = image_base + entry_point_rva;

    // TODO actually allocate a process/thread struct
    // TODO flush instruction cache?
    // TODO load address space

    trace!(
        "new stack is at {:?}, calling entrypoint at {:?}",
        stack_top,
        entry_point
    );

    unsafe {
        asm!(
        // switch stack
        "mov rsp, {new_rsp}",

        // stdcall entry point args
        "push 0",
        "push 0",
        "push 0",

        // jmp to entrypoint
        "call {entry}",

        new_rsp = in(reg) stack_top.address(),
        entry = in(reg) entry_point.address(),
        );
    }

    // TODO handle exit

    unimplemented!("process main returned??!!")
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
