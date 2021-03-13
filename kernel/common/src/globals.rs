use core::marker::PhantomData;
use core::mem::MaybeUninit;

pub type InitializedGlobal<T> = RawInitializedGlobal<T, DebugOnly<T>>;
pub type InitializedGlobalChecked<T> = RawInitializedGlobal<T, Always<T>>;

pub struct RawInitializedGlobal<T, IF: InitFlag> {
    val: MaybeUninit<T>,
    init: IF,
}

pub trait InitFlag {
    fn assert_init(&self);
    fn init(&mut self);
    fn assert_uninit(&self);
}

#[derive(Default)]
pub struct Always<T>(bool, PhantomData<T>);

#[derive(Default)]
pub struct DebugOnly<T>(#[cfg(debug_assertions)] Always<T>);

impl<T, IF: InitFlag> RawInitializedGlobal<T, IF> {
    pub fn init(&mut self, val: T) {
        self.init.assert_uninit();
        self.val = MaybeUninit::new(val);
        self.init.init();
    }

    pub fn get(&mut self) -> &mut T {
        self.init.assert_init();

        // safety: asserted initialized
        unsafe { self.val.assume_init_mut() }
    }
}
impl<T> RawInitializedGlobal<T, DebugOnly<T>> {
    pub const fn uninit() -> Self {
        Self {
            val: MaybeUninit::uninit(),
            init: DebugOnly(
                #[cfg(debug_assertions)]
                Always(false, PhantomData),
            ),
        }
    }
}

impl<T> RawInitializedGlobal<T, Always<T>> {
    pub const fn uninit() -> Self {
        Self {
            val: MaybeUninit::uninit(),
            init: Always(false, PhantomData),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.init.0
    }
}

impl<T> InitFlag for Always<T> {
    fn assert_init(&self) {
        assert!(
            self.0,
            "tried to access {} before it was initialized",
            core::any::type_name::<T>()
        );
    }

    fn init(&mut self) {
        self.0 = true;
    }

    fn assert_uninit(&self) {
        assert!(
            !self.0,
            "tried to initialize {} multiple times",
            core::any::type_name::<T>()
        );
    }
}

impl<T> InitFlag for DebugOnly<T> {
    fn assert_init(&self) {
        #[cfg(debug_assertions)]
        self.0.assert_init();
    }

    fn init(&mut self) {
        #[cfg(debug_assertions)]
        self.0.init();
    }

    fn assert_uninit(&self) {
        #[cfg(debug_assertions)]
        self.0.assert_uninit();
    }
}
