//! This module is full of hackery and dark magic.
//! Either spend a day fixing it and quietly submit a PR or don't mention it to anybody.
use core::{mem, ptr};

pub const fn ptr_size_bits() -> usize {
    mem::size_of::<usize>() * 8
}

pub fn map_in_place_2<T, U, F: FnOnce(U, T) -> T>((k, v): (U, &mut T), f: F) {
    unsafe {
        // # Safety
        //
        // If the closure panics, we must abort otherwise we could double drop `T`
        let _promote_panic_to_abort = AbortOnPanic;

        ptr::write(v, f(k, ptr::read(v)));
    }
}

/// # Safety
///
/// Requires that you ensure the reference does not become invalid.
/// The object has to outlive the reference.
pub unsafe fn change_lifetime_const<'a, 'b, T>(x: &'a T) -> &'b T {
    &*(x as *const T)
}

/// # Safety
///
/// Requires that you ensure the reference does not become invalid.
/// The object has to outlive the reference.
pub unsafe fn change_lifetime_mut<'a, 'b, T>(x: &'a mut T) -> &'b mut T {
    &mut *(x as *mut T)
}

/// A simple wrapper around `T`
///
/// This is to prevent UB when using `HashMap::get_key_value`, because
/// `HashMap` doesn't expose an api to get the key and value, where
/// the value is a `&mut T`.
///
/// See [#10](https://github.com/xacrimon/dashmap/issues/10) for details
///
/// This type is meant to be an implementation detail, but must be exposed due to the `Dashmap::shards`
#[repr(transparent)]
pub struct SharedValue<T> {
    value: T,
}

impl<T: Clone> Clone for SharedValue<T> {
    fn clone(&self) -> Self {
        let inner = self.get().clone();

        Self { value: inner }
    }
}

unsafe impl<T: Send> Send for SharedValue<T> {}

unsafe impl<T: Sync> Sync for SharedValue<T> {}

impl<T> SharedValue<T> {
    /// Create a new `SharedValue<T>`
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    /// Get a shared reference to `T`
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Get an unique reference to `T`
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Unwraps the value
    pub fn into_inner(self) -> T {
        self.value
    }
}

struct AbortOnPanic;

impl Drop for AbortOnPanic {
    fn drop(&mut self) {
        if std::thread::panicking() {
            std::process::abort()
        }
    }
}
