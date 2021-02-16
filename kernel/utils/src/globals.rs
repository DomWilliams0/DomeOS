use core::mem::MaybeUninit;

pub struct InitializedGlobal<T> {
    val: MaybeUninit<T>,
    init: bool,
}

impl<T> InitializedGlobal<T> {
    pub const fn uninit() -> Self {
        Self {
            val: MaybeUninit::uninit(),
            init: false,
        }
    }

    pub fn init(&mut self, val: T) {
        debug_assert!(!self.init);

        self.val = MaybeUninit::new(val);
        self.init = true;
    }

    pub fn get(&mut self) -> &mut T {
        debug_assert!(self.init);

        // safety: asserted initialized
        unsafe { self.val.assume_init_mut() }
    }

    pub fn is_initialized(&self) -> bool {
        self.init
    }
}
