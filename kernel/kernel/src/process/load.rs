use crate::memory::AddressSpace;
use crate::process::block::Process;
use crate::process::error::ProcessError;
use alloc::rc::Rc;
use common::{
    anyhow::{self, anyhow, Error},
    *,
};
use memory::{round_up_to, VirtualAddress, FRAME_SIZE};
use pe::Pe;

// temporary
const NOP_EXE: &[u8] = include_bytes!("../../../../userspace/nop.exe");

// TODO need to configure via args:
//  * user vs kernel
//  * exe mapped address/pointer/reference

pub fn spawn_process() -> anyhow::Result<Rc<Process>> {
    //     spawn_process_impl().map_err(anyhow::Error::msg)
    // }
    //
    // fn spawn_process_impl() -> Result<Rc<Process>, ProcessError> {
    let image = NOP_EXE;

    // parse PE
    let pe = pe::Pe::from_buffer(image).map_err(Error::msg)?;

    // TODO allocate address space
    let mut address_space = AddressSpace::current();

    // check image can be mapped at preferred base without relocating
    let (image_base, pages_needed) = image_bounds(&pe).map_err(Error::msg)?;
    let free_base = address_space
        .find_free_space(image_base, pages_needed)
        .map_err(Error::msg)?;

    if free_base != image_base {
        // TODO support relocations
        return Err(anyhow!(ProcessError::RelocationUnsupported));
    }

    debug!("mapping image at preferred base address {:?}", image_base);

    todo!()
}

/// (image base, size of image in 4k pages)
fn image_bounds(pe: &Pe) -> Result<(VirtualAddress, usize), ProcessError> {
    let opt_header = pe.optional_header()?;
    let base = opt_header.image_base()?;
    let base = VirtualAddress::new_checked(base).ok_or(ProcessError::BadVirtualAddress(base))?;

    let size = opt_header.size_of_image()?;
    let pages_needed = round_up_to(size as u64, FRAME_SIZE) / FRAME_SIZE;
    Ok((base, pages_needed as usize))
}
