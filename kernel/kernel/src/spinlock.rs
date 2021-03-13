pub struct SpinLock<T>(spin::Mutex<T>);

impl<T> SpinLock<T> {
    pub fn new(val: T) -> Self {
        Self(spin::Mutex::new(val))
    }

    /// Asserts we're not in an interrupt handler on debug builds
    pub fn lock(&self) -> spin::MutexGuard<T> {
        #[cfg(debug_assertions)]
        {
            // avoid nested panics, we're already screwed
            if !crate::panic::is_panicking() {
                assert!(
                    !crate::interrupts::is_in_interrupt(),
                    "shouldn't take a lock in an interrupt handler"
                );
            }
        }

        self.0.lock()
    }

    pub fn try_lock(&self) -> Option<spin::MutexGuard<T>> {
        self.0.try_lock()
    }

    pub fn is_locked(&self) -> bool {
        self.0.is_locked()
    }

    pub unsafe fn force_unlock(&self) {
        self.0.force_unlock()
    }
}
