use crate::interrupts::apic::ApicError;

pub fn init() -> Result<(), ApicError> {
    let apic = super::apic::init()?;

    // allow all interrupts for now
    apic.set_threshold(0, 0);

    unsafe {
        super::i8259::disable();
        super::i8259::remap();
    }

    Ok(())
}
