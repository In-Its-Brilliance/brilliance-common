use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(debug_assertions)]
use std::time::{Duration, Instant};

pub struct TimedRwLock<T> {
    inner: RwLock<T>,
    #[cfg(debug_assertions)]
    name: &'static str,
}

impl<T: Default> Default for TimedRwLock<T> {
    fn default() -> Self {
        Self {
            inner: RwLock::new(T::default()),
            #[cfg(debug_assertions)]
            name: "default",
        }
    }
}

impl<T> TimedRwLock<T> {
    #[cfg(debug_assertions)]
    pub fn new(value: T, name: &'static str) -> Self {
        Self { inner: RwLock::new(value), name }
    }

    #[cfg(not(debug_assertions))]
    pub fn new(value: T, _name: &'static str) -> Self {
        Self { inner: RwLock::new(value) }
    }

    #[cfg(debug_assertions)]
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        let start = Instant::now();
        let guard = self.inner.read();
        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(1) {
            log::warn!(target: "debug", "[{}] read() waited for {:.1?}", self.name, elapsed);
        }
        guard
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.inner.read()
    }

    #[cfg(debug_assertions)]
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        let start = Instant::now();
        let guard = self.inner.write();
        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(1) {
            log::warn!(target: "debug", "[{}] write() waited for {:.1?}", self.name, elapsed);
        }
        guard
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.inner.write()
    }
}
