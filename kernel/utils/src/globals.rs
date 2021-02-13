use core::mem::MaybeUninit;

pub struct InitializedGlobal<T> {
    val: MaybeUninit<T>,

    #[cfg(debug_assertions)]
    init: bool,
}

impl<T> InitializedGlobal<T> {
    pub const fn uninit() -> Self {
        Self {
            val: MaybeUninit::uninit(),
            #[cfg(debug_assertions)]
            init: false,
        }
    }

    pub fn init(&mut self, val: T) {
        #[cfg(debug_assertions)]
        {
            assert!(!self.init);
        }

        self.val = MaybeUninit::new(val);

        #[cfg(debug_assertions)]
        {
            self.init = true;
        }
    }

    pub fn get(&mut self) -> &mut T {
        #[cfg(debug_assertions)]
        {
            assert!(self.init);
        }

        // safety: asserted initialized
        unsafe { self.val.assume_init_mut() }
    }
}
