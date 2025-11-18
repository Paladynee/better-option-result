//! ^(\/\/! (?:(unsafe ))?([a-zA-Z_][a-zA-Z0-9_]*) *\((.*)\)(?: +-> +([A-Za-z0-9<,> ]+))?(?: |$).*?)$
//! ```ignore
//! BOption<T>
//!
//! is_some() -> bool
//! is_not_some() -> bool
//! is_none() -> bool
//! is_not_none() -> bool
//!
//! is_niche_optimized() -> bool
//!
//! unwrap()             -> T ?panic
//! unwrap_or(T)         -> T
//! unwrap_or_else(|| T) -> T
//! where T: Default
//! unwrap_or_default()  -> T
//!
//! unwrap_none()                        -> () ?panic
//! unwrap_none_or()                     -> () ?Drops T
//! unwrap_none_or_else(|T| () ?Drops T) -> ()
//! unwrap_none_or_default()             -> () ?Drops T
//!
//! where S: AsRef<str>
//! expect(S)      -> T ?panic
//! expect_none(S) -> () ?Drops T + panic
//!
//! unsafe unwrap_unchecked()      -> T ?ub
//! unsafe unwrap_none_unchecked() -> () ?ub
//!
//! as_ref() -> BOption<&T>
//! as_mut() -> BOption<&mut T>
//! 
//! into_option() -> Option<T>
//! into_ffi_option() -> FfiOption<T>
//!
//! into_inspect(|&T|) -> BOption<T>
//!
//! into_result_or(E) -> Result<T, E>
//! into_result_or_else(|| E) -> Result<T, E>
//! where E: Default
//! into_result_or_default() -> Result<T, E>
//!
//! into_bresult_or(E) -> BResult<T, E>
//! into_bresult_or_else(|| E) -> BResult<T, E>
//! where E: Default
//! into_bresult_or_default() -> BResult<T, E>
//!
//! BOption<E>
//! into_result_err_or(T) -> Result<T, E>
//! into_result_err_or_else(|| T) -> Result<T, E>
//! where T: Default
//! into_result_err_or_default() -> Result<T, E>
//!
//! BOption<E>
//! into_bresult_err_or(T) -> BResult<T, E>
//! into_bresult_err_or_else(|| T) -> BResult<T, E>
//! where T: Default
//! into_bresult_err_or_default() -> BResult<T, E>
//!
//! where T: Clone
//! into_cloned() -> BOption<T>
//! where T: Copy
//! into_copied() -> BOption<T>
//!
//! for <U>: mapping T into U
//! into_map(|T| U ?Drops T) -> U
//! into_map_or(U, |T| U ?Drops T) -> U
//! into_map_or_else(|| U, |T| U ? Drops T) -> U
//! where U: Default
//! into_map_or_default(|T| U ? Drops T) -> U
//!
//! for<U>: mapping T into BOption<U>
//! into_map_flatten(BOption<U>) -> BOption<U>
//! into_map_flatten_lazy(|T| BOption<U> ? Drops T) -> BOption<U>
//!
//! into_filter(|&T| bool) -> BOption<T>
//! into_collect(BOption<T>) -> BOption<T>
//! into_collect_lazy(|| BOption<T>) -> BOption<T>
//! into_xor(BOption<T>) -> BOption<T>
//!
//! as_insert(T) -> &mut T ? Drops Arguments::T
//! as_insert_or(T) -> &mut T ? Drops Self::T
//! as_insert_or_else(|| T) &mut T
//! where T: Default
//! as_insert_or_default() -> &mut T
//!
//! as_take() -> BOption<T>
//! as_take_if(|&mut T| bool ? Drops T) BOption<T>
//! as_replace(T) -> BOption<T>
//!
//! for<U>: zipping T with U into a tuple
//! into_zip(U) -> BOption<(T, U)>
//! into_unzip() -> (BOption<T>, BOption<U>)
//! ```
use core::hint::unreachable_unchecked;
use core::mem::{self, ManuallyDrop};
use core::mem::MaybeUninit;
use core::option::Option;

#[repr(C)]
pub enum FfiOptionTag {
    Some,
    None,
}

#[repr(C)]
pub struct FfiOption<T> {
    tag: FfiOptionTag,
    discr: MaybeUninit<T>,
}

impl<T> FfiOption<T> {
    pub const fn new_ok(t: T) -> Self {
        FfiOption {
            tag: FfiOptionTag::Some,
            discr: MaybeUninit::new(t),
        }
    }

    pub const fn new_none() -> Self {
        FfiOption {
            tag: FfiOptionTag::None,
            discr: MaybeUninit::uninit(),
        }
    }
}

impl<T> Drop for FfiOption<T> {
    fn drop(&mut self) {
        match self.tag {
            FfiOptionTag::Some => {
                unsafe { self.discr.assume_init_drop() }
            }
            FfiOptionTag::None => {},
        }
    }
}

impl<T> FfiOption<T> {
    pub fn into_boption(self) -> BOption<T> {
        let this = ManuallyDrop::new(self);
        match this.tag {
            FfiOptionTag::Some => {
                let t = unsafe { this.discr.assume_init_read() };
                BOption::Some(t)
            },
            FfiOptionTag::None => BOption::None,
        }
    }
}

pub trait IntoBOption<T> {
    fn into_boption(self) -> BOption<T>;
}

impl<T> IntoBOption<T> for Option<T> {
    fn into_boption(self) -> BOption<T> {
        match self {
            Option::Some(t) => Some(t),
            Option::None => None,
        }
    }
}

pub enum BOption<T> {
    Some(T),
    None,
}
use BOption::{None, Some};

use crate::betterresult::BResult;
use crate::betterresult::BResult::{Err, Ok};

impl<T> BOption<T> {
    pub const fn is_some(&self) -> bool {
        match *self {
            Some(_) => true,
            None => false,
        }
    }
    pub const fn is_not_some(&self) -> bool {
        match *self {
            Some(_) => false,
            None => true,
        }
    }
    pub const fn is_none(&self) -> bool {
        match *self {
            Some(_) => false,
            None => true,
        }
    }
    pub const fn is_not_none(&self) -> bool {
        match *self {
            Some(_) => true,
            None => false,
        }
    }
    pub const fn niche_optimized() -> bool {
        size_of::<Self>() != size_of::<FfiOption<T>>()
    }
    pub const fn is_niche_optimized(&self) -> bool {
        size_of::<Self>() != size_of::<FfiOption<T>>()
    }
    pub fn unwrap(self) -> T {
        match self {
            Some(t) => t,
            None => unwrap_failed_default(),
        }
    }
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Some(t) => t,
            None => default,
        }
    }
    pub fn unwrap_or_else(self, default_fn: impl FnOnce() -> T) -> T {
        match self {
            Some(t) => t,
            None => default_fn(),
        }
    }
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Some(t) => t,
            None => T::default(),
        }
    }
    pub fn unwrap_none(self) {
        match self {
            Some(_) => unwrap_none_failed_default(),
            None => (),
        }
    }
    pub fn unwrap_none_or(self) {
        match self {
            Some(t) => drop(t),
            None => (),
        }
    }
    pub fn unwrap_none_or_else(self, default_none: impl FnOnce(T)) {
        match self {
            Some(t) => default_none(t),
            None => (),
        }
    }
    pub fn unwrap_none_or_default(self) {
        match self {
            Some(t) => drop(t),
            None => <()>::default(),
        }
    }
    pub fn expect<S>(self, message: S) -> T
    where
        S: AsRef<str>,
    {
        match self {
            Some(t) => t,
            None => unwrap_failed(message.as_ref()),
        }
    }
    pub fn expect_none<S>(self, message: S)
    where
        S: AsRef<str>,
    {
        match self {
            Some(t) => {
                drop(t);
                unwrap_failed(message.as_ref())
            }
            None => (),
        }
    }
    pub unsafe fn unwrap_unchecked(self) -> T {
        match self {
            Some(t) => t,
            None => unsafe { unreachable_unchecked() },
        }
    }
    pub unsafe fn unwrap_none_unchecked(self) {
        match self {
            Some(_) => unsafe { unreachable_unchecked() },
            None => (),
        }
    }
    pub fn as_ref(&self) -> BOption<&T> {
        match *self {
            Some(ref t) => BOption::Some(t),
            None => BOption::None,
        }
    }
    pub fn as_mut(&mut self) -> BOption<&mut T> {
        match *self {
            Some(ref mut t) => BOption::Some(t),
            None => BOption::None,
        }
    }
    pub fn into_option(self) -> Option<T> {
        match self {
            Some(t) => Option::Some(t),
            None => Option::None,
        }
    }
    pub fn into_ffi_option(self) -> FfiOption<T> {
        match self {
            Some(t) => FfiOption {
                tag: FfiOptionTag::Some,
                discr: MaybeUninit::new(t),
            },
            None => FfiOption {
                tag: FfiOptionTag::None,
                discr: MaybeUninit::uninit(),
            },
        }
    }
    pub fn into_self_inspect(self, inspector: impl FnOnce(&T)) -> BOption<T> {
        if let Some(ref t) = self {
            inspector(t);
        }
        self
    }
    pub fn as_inspect(&self, inspector: impl FnOnce(&T)) {
        if let Some(ref t) = *self {
            inspector(t);
        }
    }

    pub fn into_result_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Some(t) => Result::Ok(t),
            None => Result::Err(err),
        }
    }
    pub fn into_result_or_else<E>(self, default_err_lazy: impl FnOnce() -> E) -> Result<T, E> {
        match self {
            Some(t) => Result::Ok(t),
            None => Result::Err(default_err_lazy()),
        }
    }
    pub fn into_result_or_default<E>(self) -> Result<T, E>
    where
        E: Default,
    {
        match self {
            Some(t) => Result::Ok(t),
            None => Result::Err(E::default()),
        }
    }
    pub fn into_bresult_or<E>(self, default_err: E) -> BResult<T, E> {
        match self {
            Some(t) => Ok(t),
            None => Err(default_err),
        }
    }
    pub fn into_bresult_or_else<E>(self, default_err_lazy: impl FnOnce() -> E) -> BResult<T, E> {
        match self {
            Some(t) => Ok(t),
            None => Err(default_err_lazy()),
        }
    }
    pub fn into_bresult_or_default<E>(self) -> BResult<T, E>
    where
        E: Default,
    {
        match self {
            Some(t) => Ok(t),
            None => Err(E::default()),
        }
    }
    pub fn into_map<U>(self, mapper: impl FnOnce(T) -> U) -> BOption<U> {
        match self {
            Some(t) => Some(mapper(t)),
            None => None,
        }
    }
    pub fn into_map_or<U>(self, default: U, mapper: impl FnOnce(T) -> U) -> U {
        match self {
            Some(t) => mapper(t),
            None => default,
        }
    }
    pub fn into_map_or_else<U>(self, default_lazy: impl FnOnce() -> U, mapper: impl FnOnce(T) -> U) -> U {
        match self {
            Some(t) => mapper(t),
            None => default_lazy(),
        }
    }
    pub fn into_map_or_default<U>(self, mapper: impl FnOnce(T) -> U) -> U
    where
        U: Default,
    {
        match self {
            Some(t) => mapper(t),
            None => U::default(),
        }
    }
    pub fn into_map_flatten<U>(self, other: BOption<U>) -> BOption<U> {
        match self {
            Some(_) => other,
            None => None,
        }
    }
    pub fn into_map_flatten_lazy<U>(self, other_lazy: impl FnOnce(T) -> BOption<U>) -> BOption<U> {
        match self {
            Some(t) => other_lazy(t),
            None => None,
        }
    }
    pub fn into_filter(self, filter: impl FnOnce(&T) -> bool) -> BOption<T> {
        match self {
            Some(t) if filter(&t) => Some(t),
            _ => None,
        }
    }
    pub fn into_collect(self, other: BOption<T>) -> BOption<T> {
        match self {
            Some(t) => Some(t),
            None => other,
        }
    }
    pub fn into_collect_lazy(self, other_lazy: impl FnOnce() -> BOption<T>) -> BOption<T> {
        match self {
            Some(t) => Some(t),
            None => other_lazy(),
        }
    }
    pub fn into_xor(self, other: BOption<T>) -> BOption<T> {
        match (self, other) {
            (Some(t), None) => Some(t),
            (None, Some(t)) => Some(t),
            _ => None,
        }
    }
    pub fn as_insert(&mut self, default: T) -> &mut T {
        match *self {
            Some(ref mut t) => t,
            None => {
                *self = Some(default);
                unsafe { self.as_mut().unwrap_unchecked() }
            }
        }
    }
    pub fn as_insert_or(&mut self, other: T) -> &mut T {
        match *self {
            Some(ref mut t) => drop(mem::replace(t, other)),
            None => mem::forget(mem::replace(self, Some(other))),
        }
        unsafe { self.as_mut().unwrap_unchecked() }
    }
    pub fn as_insert_or_else(&mut self, other_lazy: impl FnOnce() -> T) -> &mut T {
        match *self {
            Some(ref mut t) => t,
            None => {
                mem::forget(mem::replace(self, Some(other_lazy())));
                unsafe { self.as_mut().unwrap_unchecked() }
            }
        }
    }
    pub fn as_insert_or_default(&mut self) -> &mut T
    where
        T: Default,
    {
        match *self {
            Some(ref mut t) => t,
            None => {
                mem::forget(mem::replace(self, Some(T::default())));
                unsafe { self.as_mut().unwrap_unchecked() }
            }
        }
    }
    pub fn as_take(&mut self) -> BOption<T> {
        mem::replace(self, None)
    }
    pub fn as_take_if(&mut self, condition: impl FnOnce(&mut T) -> bool) -> BOption<T> {
        match *self {
            Some(ref mut t) => {
                if condition(t) {
                    mem::replace(self, None)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn as_replace(&mut self, replacer: T) -> BOption<T> {
        mem::replace(self, Some(replacer))
    }
    pub fn into_zip<U>(self, other: U) -> BOption<(T, U)> {
        match self {
            Some(t) => Some((t, other)),
            None => None,
        }
    }
}

impl<E> BOption<E> {
    pub fn into_result_err_or<T>(self, ok: T) -> Result<T, E> {
        match self {
            Some(e) => Result::Err(e),
            None => Result::Ok(ok),
        }
    }
    pub fn into_result_err_or_else<T>(self, default_ok_lazy: impl FnOnce() -> T) -> Result<T, E> {
        match self {
            Some(e) => Result::Err(e),
            None => Result::Ok(default_ok_lazy()),
        }
    }
    pub fn into_result_err_or_default<T>(self) -> Result<T, E>
    where
        T: Default,
    {
        match self {
            Some(e) => Result::Err(e),
            None => Result::Ok(T::default()),
        }
    }
    pub fn into_bresult_err_or<T>(self, default_ok: T) -> BResult<T, E> {
        match self {
            Some(e) => Err(e),
            None => Ok(default_ok),
        }
    }
    pub fn into_bresult_err_or_else<T>(self, default_ok_lazy: impl FnOnce() -> T) -> BResult<T, E> {
        match self {
            Some(e) => Err(e),
            None => Ok(default_ok_lazy()),
        }
    }
    pub fn into_bresult_err_or_default<T>(self) -> BResult<T, E>
    where
        T: Default,
    {
        match self {
            Some(e) => Err(e),
            None => Ok(T::default()),
        }
    }
}

impl<T, U> BOption<(T, U)> {
    pub fn into_unzip(self) -> (BOption<T>, BOption<U>) {
        match self {
            Some((t, u)) => (Some(t), Some(u)),
            None => (None, None),
        }
    }
}
impl<T: Clone> BOption<&T> {
    pub fn into_cloned(self) -> BOption<T> {
        match self {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }
}

impl<T: Copy> BOption<&T> {
    pub fn into_copied(self) -> BOption<T> {
        match self {
            Some(t) => Some(*t),
            None => None,
        }
    }
}

#[track_caller]
#[inline(always)]
fn unwrap_failed_default() -> ! {
    unwrap_failed("called unwrap on none value")
}

#[track_caller]
#[inline(always)]
fn unwrap_none_failed_default() -> ! {
    unwrap_failed("called unwrap_none on some value")
}

#[track_caller]
#[inline(never)]
fn unwrap_failed(message: &str) -> ! {
    panic!("unwrap failed: {}", message);
}
