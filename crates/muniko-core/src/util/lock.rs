use std::sync::{Mutex, MutexGuard};

/// Locks even after a panic elsewhere; the guarded data stays valid for our simple updates.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
