#![no_std]
// lint me harder
#![forbid(non_ascii_idents)]
#![deny(
    missing_docs,
    future_incompatible,
    keyword_idents,
    elided_lifetimes_in_paths,
    meta_variable_misuse,
    noop_method_call,
    pointer_structural_match,
    unused_lifetimes,
    unused_qualifications,
    clippy::wildcard_dependencies,
    clippy::debug_assert_with_mut_call,
    clippy::empty_line_after_outer_attr,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::redundant_field_names,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::unneeded_field_pattern,
    clippy::useless_let_if_seq
)]
#![warn(clippy::pedantic)]
// not that hard:
#![allow(
    // we're using `Result<_, _>` unconventionally
    clippy::missing_errors_doc,
    // ideally all the functions must be optimized to nothing, which requires always inlining
    clippy::inline_always
)]

//! Provides safe trivial transmutes in generic context, emulating
//! specialization on stable Rust. These functions are designed for being
//! optimized out by the compiler, so are probably zero-cost in most cases.
//!
//! ```rust
//! fn specialized_function<T: 'static>(x: T) -> String {
//!     // We have an efficient algorithm for `i32` and worse algorithm for any other type.
//!     // With `cismute` we can do:
//!     match cismute::owned::<T, i32>(x) {
//!         Ok(x) => format!("got an i32: {x}"),
//!         Err(x) => format!("got something else"),
//!     }
//! }
//!
//! assert_eq!(specialized_function(42_i32), "got an i32: 42");
//! assert_eq!(specialized_function(":)"), "got something else");
//! ```
//!
//! [`cismute::owned`](owned()) works only for `'static` types. If your type
//! is a reference, you should use [`cismute::reference`](reference()) or
//! [`cismute::mutable`](mutable()).
//!
//! ```rust
//! fn specialized_function<T: 'static>(x: &T) -> String {
//!     // We have an efficient algorithm for `i32` and worse algorithm for any other type.
//!     // With `cismute` we can do:
//!     match cismute::reference::<T, i32>(x) {
//!         Ok(x) => format!("got an i32: {x}"),
//!         Err(x) => format!("got something else"),
//!     }
//! }
//!
//! assert_eq!(specialized_function(&42_i32), "got an i32: 42");
//! assert_eq!(specialized_function(&":)"), "got something else");
//! ```
//!
//! There's also a more generic function [`cismute::value`](value()) which can do
//! all three. Writing all type arguments can be cumbersome, so you can also
//! pass the type pair as an argument via [`cismute::value_with`](value_with()):
//!
//! ```rust
//! use cismute::Pair;
//!
//! fn specialized_function<T: 'static>(x: &T) -> String {
//!     match cismute::value_with(Pair::<(T, i32)>, x) {
//!         Ok(x) => format!("got an i32: {x}"),
//!         Err(x) => format!("got something else"),
//!     }
//! }
//!
//! assert_eq!(specialized_function(&42_i32), "got an i32: 42");
//! assert_eq!(specialized_function(&":)"), "got something else");
//! ```
//!
//! There are also [`switch!()`] macro and [`switch()`] function
//! to match one value with multiple types.

use core::{any::TypeId, marker::PhantomData, mem::ManuallyDrop};

#[cfg(feature = "switch")]
mod branches;

#[cfg(feature = "switch")]
use branches::Branches;

#[repr(C)]
union GenericTransmute<T, U> {
    from: ManuallyDrop<T>,
    to: ManuallyDrop<U>,
}

// Required because transmute doesn't work in generic contexts
#[inline(always)]
unsafe fn generic_transmute<T, U>(from: T) -> U {
    ManuallyDrop::into_inner(
        GenericTransmute {
            from: ManuallyDrop::new(from),
        }
        .to,
    )
}

/// Pair of two types for passing to [`cismute::value_with`](value_with()).
///
/// <!-- I'm sorry, but rustdoc does something weird with reexports -->
/// <div style="display: none">
pub use core::marker::PhantomData as Pair;

/// A reference that can be safely transmuted if underlying type is the same.
///
/// # Safety
/// For any `RefT: Cismutable<'a, T, RefU>` transmutation from `RefT` to
/// `RefU` must be safe if `T` and `U` are the same type.
pub unsafe trait Cismutable<'a, T, U, RefU> {}
unsafe impl<'a, T, U> Cismutable<'a, T, U, &'a U> for &'a T {}
unsafe impl<'a, T, U> Cismutable<'a, T, U, &'a mut U> for &'a mut T {}
unsafe impl<T: 'static, U: 'static> Cismutable<'static, T, U, U> for T {}

mod seal {
    pub trait Phantom<T> {}
}
use seal::Phantom;

impl<T> Phantom<T> for PhantomData<T> {}

/// Transmutes an owned value of type `T` to type `U` if they are the same type.
/// Returns the passed value back if failed.
///
/// See module-level docs for usage example.
#[inline(always)]
pub fn owned<T, U>(val: T) -> Result<U, T>
where
    T: 'static,
    U: 'static,
{
    value(val)
}

/// Transmutes reference to type `T` to `&U` if they are the same type.
/// Returns the passed value back if failed.
///
/// See module-level docs for usage example.
#[inline(always)]
pub fn reference<'a, T, U>(val: &'a T) -> Result<&'a U, &'a T>
where
    T: 'static,
    U: 'static,
{
    value::<'a, T, U, _, _>(val)
}

/// Transmutes a mutable reference to type `T` to `&mut U` if they are the same
/// type. Returns the passed value back if failed.
///
/// See module-level docs for usage example.
#[inline(always)]
pub fn mutable<'a, T, U>(val: &'a mut T) -> Result<&'a mut U, &'a mut T>
where
    T: 'static,
    U: 'static,
{
    value::<'a, T, U, _, _>(val)
}

/// Cismutes `T` or a (possibly mutable) reference to `T` to `U` with the same
/// ownership, i.e. owned value is cismuted like with
/// [`cismute::owned`](owned()), reference is cismuted to reference and mutable
/// reference is cismuted to mutable reference, if they are the same type.
/// Returns the passed value back if failed.
///
/// See module-level docs for usage example.
#[inline(always)]
pub fn value<'a, T, U, RefT, RefU>(val: RefT) -> Result<RefU, RefT>
where
    T: 'static,
    U: 'static,
    RefT: Cismutable<'a, T, U, RefU>,
{
    if TypeId::of::<T>() == TypeId::of::<U>() {
        // SAFETY: T and U are the same type
        Ok(unsafe { generic_transmute::<RefT, RefU>(val) })
    } else {
        Err(val)
    }
}

/// Cismutes `T` or a (possibly mutable) reference to `T` to the type specified
/// by the first argument while preserving ownership, i.e. owned value is
/// cismuted like with [`cismute::owned`](owned()), reference is cismuted to
/// reference and mutable reference is cismuted to mutable reference.
/// Returns the passed value back if failed.
///
/// **Note**: first argument specifies types `T` and `U`, not the reference
/// types.
///
/// See module-level docs for usage example.
#[inline(always)]
pub fn value_with<'a, T, U, P, RefT, RefU>(_: P, val: RefT) -> Result<RefU, RefT>
where
    T: 'static,
    U: 'static,
    RefT: Cismutable<'a, T, U, RefU>,
    P: Phantom<(T, U)>,
{
    value::<T, U, RefT, RefU>(val)
}

/// Try to match `T` with several (up to 32) other types. This function requires
/// the `switch` feature, as it increases build time considerably.
///
/// ```rust
/// # use std::fmt::Debug;
/// fn specialized_function<T: Debug + 'static>(val: T) -> String {
///     cismute::switch(
///         val,
///         (
///             |x: i32| format!("got an i32: {x}"),
///             |x: char| format!("got a char: {x}"),
///         ),
///     )
///     .unwrap_or_else(|x| format!("got something else: {x:?}"))
/// }
///
/// assert_eq!(specialized_function(42_i32), "got an i32: 42");
/// assert_eq!(specialized_function('!'), "got a char: !");
/// assert_eq!(specialized_function([1, 2]), "got something else: [1, 2]");
/// ```
///
/// Switching on references requires specifying the source type. This is a
/// limitation of Rust type system.
///
/// ```rust
/// # use std::fmt::Debug;
/// use std::marker::PhantomData as X;
///
/// fn specialized_function<T: Debug + 'static>(val: &mut T) -> String {
///     cismute::switch(
///         val,
///         cismute::from(
///             X::<T>,
///             (
///                 |x: &mut i32| format!("got an i32: {x}"),
///                 |x: &mut char| format!("got a char: {x}"),
///             ),
///         ),
///     )
///     .unwrap_or_else(|x| format!("got something else: {x:?}"))
/// }
///
/// assert_eq!(specialized_function(&mut 42_i32), "got an i32: 42");
/// assert_eq!(specialized_function(&mut '!'), "got a char: !");
/// assert_eq!(
///     specialized_function(&mut [1, 2]),
///     "got something else: [1, 2]"
/// );
/// ```
#[inline(always)]
#[cfg(feature = "switch")]
pub fn switch<R, T, RefT, Args, Tuple>(val: RefT, branches: Tuple) -> Result<R, RefT>
where
    Tuple: Branches<R, T, RefT, Args>,
{
    branches.dispatch(val)
}

/// Helper function for [`switch()`].
#[inline(always)]
#[cfg(feature = "switch")]
pub fn from<R, T, P, RefT, Args, Tuple>(_: P, branches: Tuple) -> impl Branches<R, T, RefT, Args>
where
    P: Phantom<T>,
    Tuple: Branches<R, T, RefT, Args>,
{
    branches
}

/// Try to match a value with any number of types. This macro _does not_ require
/// the `switch` feature.
///
/// ```rust
/// # use std::fmt::Debug;
/// fn specialized_function<T: Debug + 'static>(val: T) -> String {
///     cismute::switch!(val; T => {
///         x: i32 => format!("got an i32: {x}"),
///         x: char => format!("got a char: {x}"),
///     }).unwrap_or_else(|x| format!("got something else: {x:?}"))
/// }
///
/// assert_eq!(specialized_function(42_i32), "got an i32: 42");
/// assert_eq!(specialized_function('!'), "got a char: !");
/// assert_eq!(specialized_function([1, 2]), "got something else: [1, 2]");
/// ```
///
/// It can also be used with (possibly mutable) references:
///
/// ```rust
/// # use std::fmt::Debug;
/// fn specialized_function<T: Debug + 'static>(val: &mut T) -> String {
///     cismute::switch!(val; T => {
///         x: i32 => format!("got an i32: {x}"),
///         x: char => format!("got a char: {x}"),
///     }).unwrap_or_else(|x| format!("got something else: {x:?}"))
/// }
///
/// assert_eq!(specialized_function(&mut 42_i32), "got an i32: 42");
/// assert_eq!(specialized_function(&mut '!'), "got a char: !");
/// assert_eq!(specialized_function(&mut [1, 2]), "got something else: [1, 2]");
/// ````
#[macro_export]
macro_rules! switch {
    ($val:expr; $source:ty => { $($name:ident: $type:ty => $expr:expr),+ $(,)? }) => {
        #[allow(clippy::never_loop)]
        match $val {
            val => loop {
                $(
                    let val = match $crate::value_with($crate::Pair::<($source, $type)>, val) {
                        Ok($name) => break Ok($expr),
                        Err(val) => val,
                    };
                )+

                break Err(val);
            },
        }
    };
}
