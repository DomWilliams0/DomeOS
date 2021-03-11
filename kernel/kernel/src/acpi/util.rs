use memory::{PhysicalAddress, VirtualAddress};

pub fn validate_checksum<T>(val: &T, len: usize) -> bool {
    let as_bytes = unsafe { core::slice::from_raw_parts(val as *const T as *const u8, len) };

    let sum = as_bytes
        .iter()
        .fold(0_u8, |acc, val| acc.wrapping_add(*val));

    (sum & 0xf) == 0
}

#[inline]
pub fn resolve_phys<T>(ptr: *const T) -> *const T {
    VirtualAddress::from_physical(PhysicalAddress(ptr as u64)).as_const_ptr()
}
