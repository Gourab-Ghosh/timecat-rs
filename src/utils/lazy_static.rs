use super::*;
use std::sync::atomic::Ordering;

const LAZY_STATIC_MEMORY_ORDERING: Ordering = Ordering::SeqCst;

pub struct LazyStatic<T> {
    state: RwLock<Option<T>>,
    initializer: fn() -> T,
    is_initialized: AtomicBool,
}

impl<T> LazyStatic<T> {
    pub const fn new(initializer: fn() -> T) -> Self {
        LazyStatic {
            state: RwLock::new(None),
            initializer,
            is_initialized: AtomicBool::new(false),
        }
    }

    fn initialize_once(&self) {
        if !self.is_initialized.load(LAZY_STATIC_MEMORY_ORDERING) {
            let mut state = self.state.write().unwrap();
            if state.is_none() {
                *state = Some((self.initializer)());
            }
            drop(state);
            self.is_initialized.store(true, LAZY_STATIC_MEMORY_ORDERING);
        }
    }
}

impl<T> Deref for LazyStatic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.initialize_once();
        // Create lock in the safe rust and then unwrap the values in unsafe rust
        let state = self
            .state
            .read()
            .expect("LazyStatic instance has previously been poisoned");
        unsafe {
            // Extending the lifetime to 'static, which is safe under controlled use of the RwLock.
            std::mem::transmute::<&T, &T>(state.as_ref().unwrap())
        }
    }
}

impl<T> DerefMut for LazyStatic<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.initialize_once();
        // Create lock in the safe rust and then unwrap the values in unsafe rust
        let mut state = self
            .state
            .write()
            .expect("LazyStatic instance has previously been poisoned");
        unsafe {
            // Extending the lifetime to 'static, which is safe under controlled use of the RwLock.
            std::mem::transmute::<&mut T, &mut T>(state.as_mut().unwrap())
        }
    }
}
