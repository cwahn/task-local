//! No-std compatible task-local storage implementation.
//!
//! This module provides task-local storage that works in no_std environments,
//! particularly useful for embedded systems with Embassy.

#![no_std]

use core::cell::RefCell;
use core::future::Future;
use core::marker::PhantomPinned;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::{fmt, mem};
use pin_project_lite::pin_project;

// For no_std, we'll use a simpler approach without thread_local
// This is suitable for single-threaded embedded environments
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// A key for task-local data in no_std environments.
///
/// This is a simplified version that works well with single-threaded
/// embedded systems like those using Embassy.
pub struct LocalKey<T: 'static> {
    inner: UnsafeCell<Option<T>>,
    in_use: AtomicBool,
}

// Safety: LocalKey is safe to share between tasks in single-threaded embedded systems
unsafe impl<T: 'static> Sync for LocalKey<T> {}
unsafe impl<T: 'static> Send for LocalKey<T> {}

impl<T: 'static> LocalKey<T> {
    /// Creates a new LocalKey.
    pub const fn new() -> Self {
        LocalKey {
            inner: UnsafeCell::new(None),
            in_use: AtomicBool::new(false),
        }
    }

    /// Sets a value `T` as the task-local value for the future `F`.
    pub fn scope<F>(&'static self, value: T, f: F) -> TaskLocalFuture<T, F>
    where
        F: Future,
    {
        TaskLocalFuture {
            local: self,
            slot: Some(value),
            future: Some(f),
            _pinned: PhantomPinned,
        }
    }

    /// Sets a value `T` as the task-local value for the closure `F`.
    #[track_caller]
    pub fn sync_scope<F, R>(&'static self, value: T, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let mut value = Some(value);
        match self.scope_inner(&mut value, f) {
            Ok(res) => res,
            Err(err) => err.panic(),
        }
    }

    fn scope_inner<F, R>(&'static self, slot: &mut Option<T>, f: F) -> Result<R, ScopeInnerErr>
    where
        F: FnOnce() -> R,
    {
        // Check if already in use (would indicate nested access)
        if self.in_use.swap(true, Ordering::Acquire) {
            return Err(ScopeInnerErr::BorrowError);
        }

        struct Guard<'a, T: 'static> {
            local: &'static LocalKey<T>,
            slot: &'a mut Option<T>,
        }

        impl<T: 'static> Drop for Guard<'_, T> {
            fn drop(&mut self) {
                // Restore the original value and mark as not in use
                unsafe {
                    let inner = &mut *self.local.inner.get();
                    mem::swap(self.slot, inner);
                }
                self.local.in_use.store(false, Ordering::Release);
            }
        }

        // Swap the value into the storage
        unsafe {
            let inner = &mut *self.inner.get();
            mem::swap(slot, inner);
        }

        let guard = Guard { local: self, slot };
        let res = f();
        drop(guard);

        Ok(res)
    }

    /// Accesses the current task-local and runs the provided closure.
    #[track_caller]
    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        match self.try_with(f) {
            Ok(res) => res,
            Err(_) => panic!("cannot access a task-local storage value without setting it first"),
        }
    }

    /// Accesses the current task-local and runs the provided closure.
    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&T) -> R,
    {
        if !self.in_use.load(Ordering::Acquire) {
            return Err(AccessError { _private: () });
        }

        unsafe {
            let inner = &*self.inner.get();
            match inner.as_ref() {
                Some(value) => Ok(f(value)),
                None => Err(AccessError { _private: () }),
            }
        }
    }
}

impl<T: Clone + 'static> LocalKey<T> {
    /// Returns a copy of the task-local value if it implements `Clone`.
    #[track_caller]
    pub fn get(&'static self) -> T {
        self.with(|v| v.clone())
    }
}

impl<T: 'static> fmt::Debug for LocalKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("LocalKey { .. }")
    }
}

pin_project! {
    /// A future that sets a value `T` of a task local for the future `F` during
    /// its execution.
    pub struct TaskLocalFuture<T, F>
    where
        T: 'static,
    {
        local: &'static LocalKey<T>,
        slot: Option<T>,
        #[pin]
        future: Option<F>,
        #[pin]
        _pinned: PhantomPinned,
    }

    impl<T: 'static, F> PinnedDrop for TaskLocalFuture<T, F> {
        fn drop(this: Pin<&mut Self>) {
            let this = this.project();
            if mem::needs_drop::<F>() && this.future.is_some() {
                let mut future = this.future;
                let _ = this.local.scope_inner(this.slot, || {
                    future.set(None);
                });
            }
        }
    }
}

impl<T, F> TaskLocalFuture<T, F>
where
    T: 'static,
{
    /// Returns the value stored in the task local by this `TaskLocalFuture`.
    pub fn take_value(self: Pin<&mut Self>) -> Option<T> {
        let this = self.project();
        this.slot.take()
    }
}

impl<T: 'static, F: Future> Future for TaskLocalFuture<T, F> {
    type Output = F::Output;

    #[track_caller]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut future_opt = this.future;

        let res = this
            .local
            .scope_inner(this.slot, || match future_opt.as_mut().as_pin_mut() {
                Some(fut) => {
                    let res = fut.poll(cx);
                    if res.is_ready() {
                        future_opt.set(None);
                    }
                    Some(res)
                }
                None => None,
            });

        match res {
            Ok(Some(res)) => res,
            Ok(None) => panic!("`TaskLocalFuture` polled after completion"),
            Err(err) => err.panic(),
        }
    }
}

impl<T: 'static, F> fmt::Debug for TaskLocalFuture<T, F>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct TransparentOption<'a, T> {
            value: &'a Option<T>,
        }
        impl<T: fmt::Debug> fmt::Debug for TransparentOption<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.value.as_ref() {
                    Some(value) => value.fmt(f),
                    None => f.pad("<missing>"),
                }
            }
        }

        f.debug_struct("TaskLocalFuture")
            .field("value", &TransparentOption { value: &self.slot })
            .finish()
    }
}

/// An error returned by [`LocalKey::try_with`].
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AccessError {
    _private: (),
}

impl fmt::Debug for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccessError").finish()
    }
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("task-local value not set", f)
    }
}

// Only implement Error trait when std is available
#[cfg(feature = "std")]
impl std::error::Error for AccessError {}

enum ScopeInnerErr {
    BorrowError,
}

impl ScopeInnerErr {
    #[track_caller]
    fn panic(&self) -> ! {
        match self {
            Self::BorrowError => {
                panic!("cannot enter a task-local scope while the task-local storage is borrowed")
            }
        }
    }
}

/// Declares a new task-local key of type [`LocalKey`] for no_std environments.
#[macro_export]
macro_rules! task_local_no_std {
    // empty (base case for the recursion)
    () => {};

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty; $($rest:tt)*) => {
        $crate::__task_local_no_std_inner!($(#[$attr])* $vis $name, $t);
        $crate::task_local_no_std!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty) => {
        $crate::__task_local_no_std_inner!($(#[$attr])* $vis $name, $t);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __task_local_no_std_inner {
    ($(#[$attr:meta])* $vis:vis $name:ident, $t:ty) => {
        $(#[$attr])*
        $vis static $name: $crate::no_std::LocalKey<$t> = $crate::no_std::LocalKey::new();
    };
}
