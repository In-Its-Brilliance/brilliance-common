pub mod format_grouped_lines;
pub mod runtime_storage;

#[cfg(feature = "full")]
pub mod lock;

pub mod info;

#[cfg(feature = "full")]
#[cfg(debug_assertions)]
pub type SmartRwLock<T> = lock::TimedRwLock<T>;

#[cfg(feature = "full")]
#[cfg(not(debug_assertions))]
pub type SmartRwLock<T> = parking_lot::RwLock<T>;

#[cfg(feature = "full")]
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! timed_lock {
    ($value:expr, $name:literal) => {
        $crate::utils::debug::lock::TimedRwLock::new($value, $name)
    };
}

#[cfg(feature = "full")]
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! timed_lock {
    ($value:expr, $name:literal) => {
        parking_lot::RwLock::new($value)
    };
}
