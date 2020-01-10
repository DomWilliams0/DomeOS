use core::ops::Add;

#[derive(Copy, Clone)]
pub struct Port(pub u16);

impl Port {
    pub unsafe fn write_u8(self, val: u8) {
        asm!("outb %al, %dx" :: "{dx}"(self.0), "{al}"(val));
    }

    pub unsafe fn read_u8(self) -> u8 {
        let ret: u8;
        asm!("inb %dx, %al" : "={ax}"(ret) : "{dx}"(self.0) :: "volatile");
        ret
    }
}

impl Add<u16> for Port {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0 + rhs)
    }
}
