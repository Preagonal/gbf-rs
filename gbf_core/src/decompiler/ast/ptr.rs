// This file is based on Rust's implementation of `P<T>`.
// Original code from Rust is licensed under MIT and Apache 2.0.
// Copyright (c) The Rust Project Developers
// Source: https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/ptr.rs
// Licensed under either the MIT license (https://opensource.org/licenses/MIT)
// or Apache License 2.0 (http://www.apache.org/licenses/LICENSE-2.0), at your choice.
// See https://www.rust-lang.org/policies/licenses for more information.

// The main modifications we do for GBF is replacing the serialization and deserialization
// with serde's `Serialize` and `Deserialize` traits.

//! The AST pointer.
//!
//! Provides [`P<T>`][struct@P], an owned smart pointer.
//!
//! # Motivations and benefits
//!
//! * **Identity**: sharing AST nodes is problematic for the various analysis
//!   passes (e.g., one may be able to bypass the borrow checker with a shared
//!   `ExprKind::AddrOf` node taking a mutable borrow).
//!
//! * **Efficiency**: folding can reuse allocation space for `P<T>` and `Vec<T>`,
//!   the latter even when the input and output types differ (as it would be the
//!   case with arenas or a GADT AST using type parameters to toggle features).
//!
//! * **Maintainability**: `P<T>` provides an interface, which can remain fully
//!   functional even if the implementation changes (using a special thread-local
//!   heap, for example). Moreover, a switch to, e.g., `P<'a, T>` would be easy
//!   and mostly automated.

use std::fmt::{self, Debug, Display};
use std::ops::{Deref, DerefMut};
use std::{slice, vec};

use serde::{Deserialize, Serialize};

/// An owned smart pointer.
///
/// See the [module level documentation][crate::ptr] for details.
#[derive(Serialize, Deserialize)]
pub struct P<T: ?Sized> {
    ptr: Box<T>,
}

/// Construct a `P<T>` from a `T` value.
#[allow(non_snake_case)]
pub fn P<T: 'static>(value: T) -> P<T> {
    P {
        ptr: Box::new(value),
    }
}

impl<T: 'static> P<T> {
    /// Move out of the pointer.
    /// Intended for chaining transformations not covered by `map`.
    pub fn and_then<U, F>(self, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        f(*self.ptr)
    }

    /// Equivalent to `and_then(|x| x)`.
    pub fn into_inner(self) -> T {
        *self.ptr
    }

    /// Produce a new `P<T>` from `self` without reallocating.
    pub fn map<F>(mut self, f: F) -> P<T>
    where
        F: FnOnce(T) -> T,
    {
        let x = f(*self.ptr);
        *self.ptr = x;

        self
    }

    /// Optionally produce a new `P<T>` from `self` without reallocating.
    pub fn filter_map<F>(mut self, f: F) -> Option<P<T>>
    where
        F: FnOnce(T) -> Option<T>,
    {
        *self.ptr = f(*self.ptr)?;
        Some(self)
    }
}

impl<T: ?Sized> Deref for P<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.ptr
    }
}

impl<T: ?Sized> DerefMut for P<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.ptr
    }
}

impl<T: ?Sized> AsRef<T> for P<T> {
    /// Consumes the `P<T>` and returns the inner `Box<T>`.
    fn as_ref(&self) -> &T {
        &self.ptr
    }
}

impl<T: 'static + Clone> Clone for P<T> {
    fn clone(&self) -> P<T> {
        P((**self).clone())
    }
}

impl<T: ?Sized + Debug> Debug for P<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.ptr, f)
    }
}

impl<T: Display> Display for P<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T> fmt::Pointer for P<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr, f)
    }
}

impl<T> From<T> for P<T> {
    fn from(value: T) -> Self {
        P {
            ptr: Box::new(value),
        }
    }
}

impl<T> P<[T]> {
    /// Creates a new empty `P<[T]>`.
    pub fn new() -> P<[T]> {
        P {
            ptr: Box::default(),
        }
    }

    #[inline(never)]
    /// Creates a new `P<[T]>` from a `Vec<T>`.
    pub fn from_vec(v: Vec<T>) -> P<[T]> {
        P {
            ptr: v.into_boxed_slice(),
        }
    }

    #[inline(never)]
    /// Converts the `P<[T]>` into a `Vec<T>`.
    pub fn into_vec(self) -> Vec<T> {
        self.ptr.into_vec()
    }
}

impl<T> Default for P<[T]> {
    /// Creates an empty `P<[T]>`.
    fn default() -> P<[T]> {
        P::new()
    }
}

impl<T: Clone> Clone for P<[T]> {
    fn clone(&self) -> P<[T]> {
        P::from_vec(self.to_vec())
    }
}

impl<T> From<Vec<T>> for P<[T]> {
    fn from(v: Vec<T>) -> Self {
        P::from_vec(v)
    }
}

impl<T> From<P<[T]>> for Vec<T> {
    fn from(val: P<[T]>) -> Self {
        val.into_vec()
    }
}

impl<T> FromIterator<T> for P<[T]> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> P<[T]> {
        P::from_vec(iter.into_iter().collect())
    }
}

impl<T> IntoIterator for P<[T]> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a P<[T]> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.ptr.iter()
    }
}

impl<T: PartialEq> PartialEq for P<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Eq> Eq for P<T> {}
