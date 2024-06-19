use super::*;

pub struct LazyStatic<T> {
    state: RwLock<LazyStaticState<T>>,
    initializer: fn() -> T,
}

enum LazyStaticState<T> {
    Uninitialized,
    Initialized(T),
}

impl<T> LazyStatic<T> {
    pub const fn new(initializer: fn() -> T) -> Self {
        LazyStatic {
            state: RwLock::new(LazyStaticState::Uninitialized),
            initializer,
        }
    }
}

impl<T> Deref for LazyStatic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let mut state = self
            .state
            .write()
            .expect("LazyStatic instance has previously been poisoned");
        if let LazyStaticState::Uninitialized = *state {
            *state = LazyStaticState::Initialized((self.initializer)());
        }
        drop(state);
        if let LazyStaticState::Initialized(ref value) = *self
            .state
            .read()
            .expect("LazyStatic instance has previously been poisoned")
        {
            unsafe {
                // Extending the lifetime to 'static, which is safe under controlled use of the RwLock.
                std::mem::transmute::<&T, &T>(value)
            }
        } else {
            unreachable!()
        }
    }
}

impl<T> DerefMut for LazyStatic<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let mut state = self
            .state
            .write()
            .expect("LazyStatic instance has previously been poisoned");
        if let LazyStaticState::Uninitialized = *state {
            *state = LazyStaticState::Initialized((self.initializer)());
        }
        if let LazyStaticState::Initialized(ref mut value) = *state {
            unsafe {
                // Extending the lifetime to 'static, which is safe under controlled use of the RwLock.
                std::mem::transmute::<&mut T, &mut T>(value)
            }
        } else {
            unreachable!()
        }
    }
}
