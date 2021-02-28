use core::ops::Add;

#[derive(Copy, Clone)]
pub struct Port(pub u16);

impl Port {
    pub unsafe fn write_u8(self, val: u8) {
        asm!("out dx, al", in("dx") self.0, in("al") val, options(nomem, nostack));
    }

    pub unsafe fn read_u8(self) -> u8 {
        let ret: u8;
        asm!("in al, dx", out("al") ret, in("dx") self.0, options(nomem, nostack));
        ret
    }
}

impl Add<u16> for Port {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0 + rhs)
    }
}
