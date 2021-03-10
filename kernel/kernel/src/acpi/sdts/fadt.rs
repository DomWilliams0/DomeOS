use crate::acpi::rsdp::AcpiSdtHeader;
use crate::acpi::sdts::DescriptionTable;
use bit_field::BitField;

pub trait Fadt {
    fn has_8042_ps2_controller(&self) -> bool;
}

#[repr(C, packed)]
pub struct FadtRevision1 {
    header: AcpiSdtHeader,
    firmware_ctrl: u32,
    dsdt: u32,
    model: u8,
    reserved1: u8,
    sci_int: u16,
    smi_cmd: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    reserved2: u8,
    pm1a_evt_blk: u32,
    pm1b_evt_blk: u32,
    pm1a_cnt_blk: u32,
    pm1b_cnt_blk: u32,
    pm2_cnt_blk: u32,
    pm_tmr_blk: u32,
    gpe0_blk: u32,
    gpe1_blk: u32,
    pm1_evt_len: u8,
    pm1_cnt_len: u8,
    pm2_cnt_len: u8,
    pm_tmr_len: u8,
    gpe0_blk_len: u8,
    gpe1_blk_len: u8,
    gpe1_base: u8,
    reserved3: u8,
    plvl2_lat: u16,
    plvl3_lat: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alrm: u8,
    mon_alrm: u8,
    century: u8,
    reserved4: u8,
    reserved4a: u8,
    reserved4b: u8,
    flags: u32,
}

#[repr(C, packed)]
pub struct FadtRevision2 {
    header: AcpiSdtHeader,
    firmware_ctrl: u32,
    dsdt: u32,
    reserved: u8,
    preferred_power_management_profile: u8,
    sci_interrupt: u16,
    smi_command_port: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    pstate_control: u8,
    pm1a_event_block: u32,
    pm1b_event_block: u32,
    pm1a_control_block: u32,
    pm1b_control_block: u32,
    pm2control_block: u32,
    pmtimer_block: u32,
    gpe0block: u32,
    gpe1block: u32,
    pm1event_length: u8,
    pm1control_length: u8,
    pm2control_length: u8,
    pmtimer_length: u8,
    gpe0length: u8,
    gpe1length: u8,
    gpe1base: u8,
    cstate_control: u8,
    worst_c2latency: u16,
    worst_c3latency: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    century: u8,
    boot_architecture_flags: u16,
    reserved2: u8,
    flags: u32,
    reset_reg: GenericAddressStructure,
    reset_value: u8,
    reserved3: [u8; 3],

    x_firmware_control: u64,
    x_dsdt: u64,
    x_pm1a_event_block: GenericAddressStructure,
    x_pm1b_event_block: GenericAddressStructure,
    x_pm1a_control_block: GenericAddressStructure,
    x_pm1b_control_block: GenericAddressStructure,
    x_pm2control_block: GenericAddressStructure,
    x_pmtimer_block: GenericAddressStructure,
    x_gpe0block: GenericAddressStructure,
    x_gpe1block: GenericAddressStructure,
}

#[repr(C, packed)]
struct GenericAddressStructure {
    address_space: u8,
    bit_width: u8,
    bit_offset: u8,
    access_size: u8,
    address: u64,
}

impl<T: Fadt> DescriptionTable for T {
    const SIGNATURE: &'static str = "FACP";
}

impl Fadt for FadtRevision1 {
    fn has_8042_ps2_controller(&self) -> bool {
        // too old to specify, assume yes
        true
    }
}

impl Fadt for FadtRevision2 {
    fn has_8042_ps2_controller(&self) -> bool {
        // might be unaligned
        unsafe { self.boot_architecture_flags.get_bit(1) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_check() {
        assert_eq!(core::mem::size_of::<GenericAddressStructure>(), 12);
        assert_eq!(memoffset::offset_of!(FadtRevision1, flags), 112);
        assert_eq!(memoffset::offset_of!(FadtRevision2, flags), 112);
    }
}
