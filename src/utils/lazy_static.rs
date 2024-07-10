use super::*;
use std::sync::OnceLock;

pub struct LazyStatic<T> {
    state: OnceLock<T>,
    initializer: fn() -> T,
}

impl<T> LazyStatic<T> {
    pub const fn new(initializer: fn() -> T) -> Self {
        LazyStatic {
            state: OnceLock::new(),
            initializer,
        }
    }
}

impl<T> Deref for LazyStatic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.state.get_or_init(self.initializer)
    }
}

impl<T> DerefMut for LazyStatic<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.state.get().is_none() {
            let _ = self.state.set((self.initializer)());
        }
        self.state.get_mut().unwrap()
    }
}
