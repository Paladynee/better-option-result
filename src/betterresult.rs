use crate::BetterOption;
use crate::BetterOption::*;
use BetterResult::*;
use core::fmt;
use core::hint;
use core::ops::{Deref, DerefMut};
use core::result;

#[derive(Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum BetterResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> BetterResult<T, E> {
    pub fn unwrap(self) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Ok(val) => val,
            Err(e) => unwrap_failed("called `Result::unwrap()` on an `Err` value", &e),
        }
    }

    pub fn unwrap_err(self) -> E
    where
        T: fmt::Debug,
    {
        match self {
            Ok(t) => unwrap_failed("called `Result::unwrap()` on an `Err` value", &t),
            Err(val) => val,
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Ok(t) => t,
            Err(_) => default,
        }
    }

    pub fn unwrap_or_lazy<F>(self, default_fn: F) -> T
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Ok(t) => t,
            Err(e) => default_fn(e),
        }
    }

    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Ok(x) => x,
            Err(_) => Default::default(),
        }
    }

    pub fn expect(self, msg: &str) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Ok(t) => t,
            Err(e) => unwrap_failed(msg, &e),
        }
    }

    pub fn expect_err(self, msg: &str) -> E
    where
        T: fmt::Debug,
    {
        match self {
            Ok(e) => unwrap_failed(msg, &e),
            Err(t) => t,
        }
    }

    pub unsafe fn unwrap_unchecked(self) -> T {
        match self {
            Ok(t) => t,
            // SAFETY: the safety contract must be upheld by the caller.
            Err(_) => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub unsafe fn unwrap_err_unchecked(self) -> E {
        match self {
            // SAFETY: the safety contract must be upheld by the caller.
            Ok(_) => unsafe { hint::unreachable_unchecked() },
            Err(e) => e,
        }
    }

    pub const fn is_ok(&self) -> bool {
        matches!(*self, Ok(_))
    }

    pub const fn is_not_ok(&self) -> bool {
        !self.is_ok()
    }

    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub const fn is_not_err(&self) -> bool {
        self.is_ok()
    }

    pub fn into_is_ok_and<F>(self, map: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        match self {
            Ok(x) => map(x),
            Err(_) => false,
        }
    }

    pub fn into_is_ok_or<F>(self, map_err: F) -> bool
    where
        F: FnOnce(E) -> bool,
    {
        match self {
            Ok(_) => true,
            Err(x) => map_err(x),
        }
    }

    pub fn into_is_ok_nand<F>(self, map: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        match self {
            Ok(val) => !map(val),
            Err(_) => true,
        }
    }

    pub fn into_is_ok_nor<F>(self, map_err: F) -> bool
    where
        F: FnOnce(E) -> bool,
    {
        match self {
            Ok(_) => false,
            Err(x) => !map_err(x),
        }
    }

    pub fn into_is_ok_xor<F, G>(self, map: F, map_err: G) -> bool
    where
        F: FnOnce(T) -> bool,
        G: FnOnce(E) -> bool,
    {
        match self {
            Ok(val) => !map(val),
            Err(x) => map_err(x),
        }
    }

    pub fn into_is_ok_xnor<F, G>(self, map: F, map_err: G) -> bool
    where
        F: FnOnce(T) -> bool,
        G: FnOnce(E) -> bool,
    {
        match self {
            Ok(val) => map(val),
            Err(x) => !map_err(x),
        }
    }

    pub fn into_is_err_and<F>(self, map_err: F) -> bool
    where
        F: FnOnce(E) -> bool,
    {
        match self {
            Ok(_) => false,
            Err(x) => map_err(x),
        }
    }

    pub fn into_is_err_or<F>(self, map: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        match self {
            Ok(x) => map(x),
            Err(_) => true,
        }
    }

    pub fn into_is_err_nand<F>(self, map: F) -> bool
    where
        F: FnOnce(E) -> bool,
    {
        match self {
            Err(val) => !map(val),
            Ok(_) => true,
        }
    }

    pub fn into_is_err_nor<F>(self, map: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        match self {
            Ok(x) => !map(x),
            Err(_) => false,
        }
    }

    pub fn into_is_err_xor<G, F>(self, map: F, map_err: G) -> bool
    where
        F: FnOnce(T) -> bool,
        G: FnOnce(E) -> bool,
    {
        match self {
            Ok(x) => map(x),
            Err(val) => !map_err(val),
        }
    }

    pub fn into_is_err_xnor<G, F>(self, map: F, map_err: G) -> bool
    where
        F: FnOnce(T) -> bool,
        G: FnOnce(E) -> bool,
    {
        match self {
            Ok(x) => !map(x),
            Err(val) => map_err(val),
        }
    }

    pub fn into_option(self) -> BetterOption<T> {
        match self {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }

    pub fn into_option_err(self) -> BetterOption<E> {
        match self {
            Ok(_) => None,
            Err(x) => Some(x),
        }
    }

    pub const fn as_ref(&self) -> BetterResult<&T, &E> {
        match *self {
            Ok(ref x) => Ok(x),
            Err(ref x) => Err(x),
        }
    }

    pub const fn as_mut(&mut self) -> BetterResult<&mut T, &mut E> {
        match *self {
            Ok(ref mut x) => Ok(x),
            Err(ref mut x) => Err(x),
        }
    }

    pub fn into_mapped<U, F>(self, map: F) -> BetterResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Ok(t) => Ok(map(t)),
            Err(e) => Err(e),
        }
    }

    pub fn into_mapped_or<U, F>(self, default: U, map: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Ok(t) => map(t),
            Err(_) => default,
        }
    }

    pub fn into_mapped_or_lazy<U, D, F>(self, default_fn: D, map: F) -> U
    where
        D: FnOnce(E) -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            Ok(t) => map(t),
            Err(e) => default_fn(e),
        }
    }

    pub fn into_mapped_or_default<F, U>(self, map: F) -> U
    where
        F: FnOnce(T) -> U,
        U: Default,
    {
        match self {
            Ok(t) => map(t),
            Err(_) => Default::default(),
        }
    }

    pub fn into_mapped_err<F, O>(self, map_err: O) -> BetterResult<T, F>
    where
        O: FnOnce(E) -> F,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(map_err(e)),
        }
    }

    pub fn into_mapped_err_or<F, O>(self, default: BetterResult<T, F>, map_err: O) -> BetterResult<T, F>
    where
        O: FnOnce(E) -> F,
    {
        match self {
            Ok(_) => default,
            Err(e) => Err(map_err(e)),
        }
    }

    pub fn into_self_inspect<F>(self, inspect: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Ok(ref t) = self {
            inspect(t);
        }

        self
    }

    pub fn into_self_inspect_err<F>(self, inspect: F) -> Self
    where
        F: FnOnce(&E),
    {
        if let Err(ref e) = self {
            inspect(e);
        }

        self
    }

    pub fn as_self_inspect<F>(&self, inspect: F)
    where
        F: FnOnce(&T),
    {
        if let Ok(t) = self {
            inspect(t);
        }
    }

    pub fn as_self_inspect_err<F>(&self, inspect: F)
    where
        F: FnOnce(&E),
    {
        if let Err(e) = self {
            inspect(e);
        }
    }

    pub fn as_deref(&self) -> BetterResult<&T::Target, &E>
    where
        T: Deref,
    {
        self.as_ref().into_mapped(|t| t.deref())
    }

    pub fn as_deref_mut(&mut self) -> BetterResult<&mut T::Target, &mut E>
    where
        T: DerefMut,
    {
        self.as_mut().into_mapped(|t| t.deref_mut())
    }

    // todo: iter methods

    pub fn into_ok_of_arg<U>(self, arg: BetterResult<U, E>) -> BetterResult<U, E> {
        match self {
            Ok(_) => arg,
            Err(e) => Err(e),
        }
    }

    pub fn into_ok_of_arg_lazy<U, F>(self, arg_fn: F) -> BetterResult<U, E>
    where
        F: FnOnce(T) -> BetterResult<U, E>,
    {
        match self {
            Ok(t) => arg_fn(t),
            Err(e) => Err(e),
        }
    }
    pub fn into_err_of_arg<F>(self, arg: BetterResult<T, F>) -> BetterResult<T, F> {
        match self {
            Ok(t) => Ok(t),
            Err(_) => arg,
        }
    }

    pub fn into_err_of_arg_lazy<F, G>(self, arg_fn: G) -> BetterResult<T, F>
    where
        G: FnOnce(E) -> BetterResult<T, F>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => arg_fn(e),
        }
    }

    pub fn into_core_result(self) -> result::Result<T, E> {
        match self {
            Ok(t) => result::Result::Ok(t),
            Err(e) => result::Result::Err(e),
        }
    }
}

// core aliases
impl<T, E> BetterResult<T, E> {
    /// stable alias for `unwrap_or_lazy`
    pub fn unwrap_or_else<F>(self, default_fn: F) -> T
    where
        F: FnOnce(E) -> T,
    {
        self.unwrap_or_lazy(default_fn)
    }

    /// stable alias for `into_is_ok_and`
    pub fn is_ok_and<F>(self, map: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        self.into_is_ok_and(map)
    }

    /// stable alias for `into_is_err_and`
    pub fn is_err_and<F>(self, map_err: F) -> bool
    where
        F: FnOnce(E) -> bool,
    {
        self.into_is_err_and(map_err)
    }

    /// stable alias for `into_option`
    pub fn ok(self) -> BetterOption<T> {
        self.into_option()
    }

    /// stable alias for `into_option_err`
    pub fn err(self) -> BetterOption<E> {
        self.into_option_err()
    }

    /// stable alias for `into_mapped`
    pub fn map<U, F>(self, map: F) -> BetterResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        self.into_mapped(map)
    }

    /// stable alias for `into_mapped_or`
    pub fn map_or<U, F>(self, default: U, map: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        self.into_mapped_or(default, map)
    }

    /// stable alias for `into_mapped_or_lazy`
    pub fn map_or_else<U, D, F>(self, default_fn: D, map: F) -> U
    where
        D: FnOnce(E) -> U,
        F: FnOnce(T) -> U,
    {
        self.into_mapped_or_lazy(default_fn, map)
    }

    /// stable alias for `into_mapped_err`
    pub fn map_err<F, O>(self, map_err: O) -> BetterResult<T, F>
    where
        O: FnOnce(E) -> F,
    {
        self.into_mapped_err(map_err)
    }

    /// stable alias for `into_self_inspect`
    pub fn inspect<F>(self, inspect: F) -> Self
    where
        F: FnOnce(&T),
    {
        self.into_self_inspect(inspect)
    }

    /// stable alias for `into_self_inspect_err`
    pub fn inspect_err<F>(self, inspect: F) -> Self
    where
        F: FnOnce(&E),
    {
        self.into_self_inspect_err(inspect)
    }

    /// stable alias for `into_arg_if_ok`
    pub fn and<U>(self, arg: BetterResult<U, E>) -> BetterResult<U, E> {
        self.into_ok_of_arg(arg)
    }

    /// stable alias for `into_arg_if_ok_lazy`
    pub fn and_then<U, F>(self, arg_fn: F) -> BetterResult<U, E>
    where
        F: FnOnce(T) -> BetterResult<U, E>,
    {
        self.into_ok_of_arg_lazy(arg_fn)
    }

    /// stable alias for `into_err_of_arg`
    pub fn or<F>(self, arg: BetterResult<T, F>) -> BetterResult<T, F> {
        self.into_err_of_arg(arg)
    }

    /// stable alias for `into_err_of_arg_lazy`
    pub fn or_else<F, O>(self, arg_fn: O) -> BetterResult<T, F>
    where
        O: FnOnce(E) -> BetterResult<T, F>,
    {
        self.into_err_of_arg_lazy(arg_fn)
    }
}

impl<T, E> From<result::Result<T, E>> for BetterResult<T, E> {
    fn from(value: result::Result<T, E>) -> Self {
        match value {
            result::Result::Ok(t) => Ok(t),
            result::Result::Err(e) => Err(e),
        }
    }
}

impl<T, E> BetterResult<&T, E> {
    pub fn into_copied(self) -> BetterResult<T, E>
    where
        T: Copy,
    {
        match self {
            Ok(&v) => Ok(v),
            Err(e) => Err(e),
        }
    }

    pub fn into_cloned(self) -> BetterResult<T, E>
    where
        T: Clone,
    {
        self.into_mapped(|t| t.clone())
    }
}

// core aliases
impl<T, E> BetterResult<&T, E> {
    /// stable alias for `into_copied`
    pub fn copied(self) -> BetterResult<T, E>
    where
        T: Copy,
    {
        self.into_copied()
    }

    /// stable alias for `into_cloned`
    pub fn cloned(self) -> BetterResult<T, E>
    where
        T: Clone,
    {
        self.into_cloned()
    }
}

impl<T, E> BetterResult<&mut T, E> {
    pub fn copied(self) -> BetterResult<T, E>
    where
        T: Copy,
    {
        match self {
            Ok(&mut v) => Ok(v),
            Err(e) => Err(e),
        }
    }

    pub fn cloned(self) -> BetterResult<T, E>
    where
        T: Clone,
    {
        self.into_mapped(|t| t.clone())
    }
}

impl<T, E> BetterResult<BetterOption<T>, E> {
    pub fn transpose(self) -> BetterOption<BetterResult<T, E>> {
        match self {
            Ok(Some(x)) => Some(Ok(x)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<T, E> BetterResult<BetterResult<T, E>, E> {
    pub fn flatten(self) -> BetterResult<T, E> {
        // FIXME(const-hack): could be written with `and_then`
        match self {
            Ok(inner) => inner,
            Err(e) => Err(e),
        }
    }
}

// This is a separate function to reduce the code size of the methods
#[inline(never)]
#[cold]
#[track_caller]
fn unwrap_failed(msg: &str, error: &dyn fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}

impl<T, E> Clone for BetterResult<T, E>
where
    T: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Ok(x) => Ok(x.clone()),
            Err(x) => Err(x.clone()),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Ok(to), Ok(from)) => to.clone_from(from),
            (Err(to), Err(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}

// todo: intoiter implementation
