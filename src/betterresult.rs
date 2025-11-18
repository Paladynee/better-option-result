// ^(\/\/! (?:(unsafe ))?([a-zA-Z_][a-zA-Z0-9_]*) *\((.*)\)(?: +-> +([A-Za-z0-9<,> ]+))?(?: |$).*?)$
//! ```ignore
//! BResult<T, E>
//!
//! unwrap()                      -> T ?panic
//! unwrap_or(T)                  -> T ?Drops E
//! unwrap_or_else(|E| T ?Drops E) -> T
//! where T: Default
//! unwrap_or_default()           -> T ?Drops E
//!
//! where E = Infallible | !
//! into_ok_infallible() -> T
//! where T = Infallible | !
//! into_err_infallible() -> E
//!
//! unwrap_err()                      -> E ?panic
//! unwrap_err_or(E)                  -> E ?Drops T
//! unwrap_err_or_else(|T| E ?Drops T) -> E
//! where E: Default
//! unwrap_err_or_default()           -> E ?Drops T
//!
//! where S: AsRef<str>
//! expect(S)     -> T ?Drops E + panic
//! expect_err(S) -> E ?Drops T + panic
//!
//! unsafe unwrap_unchecked()     -> T ?ub
//! unsafe unwrap_err_unchecked() -> E ?ub
//!
//! is_ok()      -> bool
//! is_not_ok()  -> bool
//! is_err()     -> bool
//! is_not_err() -> bool
//!
//! is_niche_optimized() -> bool
//!
//! into_is_ok_and(|T| bool ?Drops T) -> bool ?Drops E
//! into_is_ok_or (|E| bool ?Drops E) -> bool ?Drops T
//!
//! into_is_err_and(|E| bool ?Drops E) -> bool ?Drops T
//! into_is_err_or (|T| bool ?Drops T) -> bool ?Drops E
//!
//! into_boption    () -> BOption<T> Drops E
//! into_boption_err() -> BOption<E> Drops T
//!
//! unsafe into_boption_unchecked    () -> BOption<T> ?ub
//! unsafe into_boption_err_unchecked() -> BOption<E> ?ub
//!
//! into_option    () -> Option<T> Drops E
//! into_option_err() -> Option<E> Drops T
//!
//! unsafe into_option_unchecked    () -> BOption<T> ?ub
//! unsafe into_option_err_unchecked() -> BOption<E> ?ub
//!
//! as_ref() -> BResult<&T, &E>
//! as_mut() -> BResult<&mut T, &mut E>
//!
//! where T: Clone
//! into_cloned() -> BResult<T, E>
//! where T: Copy
//! into_copied() -> BResult<T, E>
//!
//! where E: Clone
//! into_err_cloned() -> BResult<T, E>
//! where E: Copy
//! into_err_copied() -> BResult<T, E>
//!
//! for <U>: mapping T or E into U
//! into_map_ok           (|T| U ?Drops T                ) -> BResult<U, E>
//! into_map_ok_or        (|T| U ?Drops T, U             ) -> U ?Drops E
//! into_map_ok_or_else   (|T| U ?Drops T, |E| U ?Drops E) -> U
//! where U: Default
//! into_map_ok_or_default(|T| U ?Drops T               ) -> U ?Drops E
//!
//! for <F>: mapping T or E into F
//! into_map_err           (                   |E| -> F ?Drops E) -> BResult<T, F>
//! into_map_err_or        (F,                 |E| -> F ?Drops E) -> F ?Drops T
//! into_map_err_or_else   (|T| -> F ?Drops T, |E| -> F ?Drops E) -> F
//! where F: Default
//! into_map_err_or_default(                   |E| -> F ?Drops E) -> F ?Drops T
//!
//! into_self_inspect    (|&T|) -> BResult<T, E>
//! into_self_inspect_err(|&E|) -> BResult<T, E>
//! as_inspect    (|&T|)
//! as_inspect_err(|&E|)
//!
//! for <U>: mapping T into BResult<U, E>
//! into_map_ok_flatten     (    BResult<U, E>         ) -> BResult<U, E> ?Drops T
//! into_map_ok_flatten_lazy(|T| BResult<U, E> ?Drops T) -> BResult<U, E>
//! for <F>: mapping F into BResult<T, F>
//! into_map_err_flatten     (    BResult<T, F>         ) -> BResult<T, F>
//! into_map_err_flatten_lazy(|E| BResult<T, F> ?Drops E) -> BResult<T, F>
//!
//! into_result(Result<T, E>)
//! into_ffi_result(FfiResult<T, E>)
//!
//! FfiResult<T, E>
//!
//! into_result() -> Result<T, E>
//! into_bresult() -> BResult<T, E>
//!
//! where T = BResult<U, E>
//! into_flattened() -> BResult<U, E>
//! ```
use crate::betteroption::BOption;
use crate::betteroption::BOption::{None, Some};
use core::convert::Infallible;
use core::hint::unreachable_unchecked;
use core::mem::ManuallyDrop;
use core::mem::size_of;
use core::result::Result;

#[allow(non_snake_case)]
pub union FfiResultDiscr<T, E> {
    Ok: ManuallyDrop<T>,
    Err: ManuallyDrop<E>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FfiResultTag {
    Ok = 0,
    Err = 1,
}

#[repr(C)]
pub struct FfiResult<T, E> {
    tag: FfiResultTag,
    discriminant: FfiResultDiscr<T, E>,
}

impl<T, E> FfiResult<T, E> {
    pub const fn new_ok(t: T) -> Self {
        FfiResult {
            tag: FfiResultTag::Ok,
            discriminant: FfiResultDiscr { Ok: ManuallyDrop::new(t) },
        }
    }

    pub const fn new_err(e: E) -> Self {
        FfiResult {
            tag: FfiResultTag::Err,
            discriminant: FfiResultDiscr { Err: ManuallyDrop::new(e) },
        }
    }
}

impl<T, E> Drop for FfiResult<T, E> {
    fn drop(&mut self) {
        match self.tag {
            FfiResultTag::Ok => {
                let field = unsafe { &mut self.discriminant.Ok };
                unsafe { ManuallyDrop::drop(field) };
            }
            FfiResultTag::Err => {
                let field = unsafe { &mut self.discriminant.Err };
                unsafe { ManuallyDrop::drop(field) };
            }
        }
    }
}

pub enum BResult<T, E> {
    Ok(T),
    Err(E),
}
use BResult::{Err, Ok};

pub trait IntoBResult<T, E> {
    fn into_bresult(self) -> BResult<T, E>;
}

impl<T, E> IntoBResult<T, E> for Result<T, E> {
    fn into_bresult(self) -> BResult<T, E> {
        match self {
            Result::Ok(t) => Ok(t),
            Result::Err(t) => Err(t),
        }
    }
}

// TODO: replace with deref coercion in FfiResult::into_bresult when deref coercion is not conditionally const, e.g.
// when we get stable const traits.
const fn manually_drop_as_ptr<T>(md: &ManuallyDrop<T>) -> *const T {
    (&raw const *md).cast::<T>()
}

impl<T, E> FfiResult<T, E> {
    pub const fn into_bresult(self) -> BResult<T, E> {
        let this = ManuallyDrop::new(self);
        match unsafe { (&raw const (*manually_drop_as_ptr(&this)).tag).read() } {
            FfiResultTag::Ok => {
                let field_ptr = unsafe { &raw const (*manually_drop_as_ptr(&this)).discriminant.Ok };
                let field = unsafe { field_ptr.cast::<T>().read() };
                Ok(field)
            }
            FfiResultTag::Err => {
                let field_ptr = unsafe { &raw const (*manually_drop_as_ptr(&this)).discriminant.Err };
                let field = unsafe { field_ptr.cast::<E>().read() };
                Err(field)
            }
        }
    }
}

impl<T, E> BResult<T, E> {
    pub fn unwrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(_) => unwrap_ok_failed_default(),
        }
    }

    pub fn unwrap_or(self, default_eager: T) -> T {
        match self {
            Ok(t) => t,
            Err(_) => default_eager,
        }
    }

    pub fn unwrap_or_else(self, default_lazy: impl FnOnce(E) -> T) -> T {
        match self {
            Ok(t) => t,
            Err(e) => default_lazy(e),
        }
    }

    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Ok(t) => t,
            Err(_) => T::default(),
        }
    }

    pub fn unwrap_err(self) -> E {
        match self {
            Ok(_) => unwrap_err_failed_default(),
            Err(e) => e,
        }
    }

    pub fn unwrap_err_or(self, default_eager: E) -> E {
        match self {
            Ok(_) => default_eager,
            Err(e) => e,
        }
    }

    pub fn unwrap_err_or_else(self, default_lazy: impl FnOnce(T) -> E) -> E {
        match self {
            Ok(t) => default_lazy(t),
            Err(e) => e,
        }
    }

    pub fn unwrap_err_or_default(self) -> E
    where
        E: Default,
    {
        match self {
            Ok(_) => E::default(),
            Err(e) => e,
        }
    }

    pub fn expect<S: AsRef<str>>(self, message: S) -> T {
        match self {
            Ok(t) => t,
            Err(_) => unwrap_failed(<S as AsRef<str>>::as_ref(&message)),
        }
    }

    pub fn expect_err<S: AsRef<str>>(self, message: S) -> E {
        match self {
            Ok(_) => unwrap_failed(<S as AsRef<str>>::as_ref(&message)),
            Err(e) => e,
        }
    }

    pub unsafe fn unwrap_unchecked(self) -> T {
        match self {
            Ok(t) => t,
            Err(_) => unsafe { unreachable_unchecked() },
        }
    }

    pub unsafe fn unwrap_err_unchecked(self) -> E {
        match self {
            Ok(_) => unsafe { unreachable_unchecked() },
            Err(e) => e,
        }
    }

    pub const fn is_ok(&self) -> bool {
        match *self {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub const fn is_not_ok(&self) -> bool {
        match *self {
            Ok(_) => false,
            Err(_) => true,
        }
    }

    pub const fn is_err(&self) -> bool {
        match *self {
            Ok(_) => false,
            Err(_) => true,
        }
    }

    pub const fn is_not_err(&self) -> bool {
        match *self {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub const fn niche_optimized() -> bool {
        size_of::<FfiResult<T, E>>() != size_of::<Self>()
    }

    pub const fn is_niche_optimized(&self) -> bool {
        size_of::<FfiResult<T, E>>() != size_of::<Self>()
    }

    pub fn into_is_ok_and(self, cond: impl FnOnce(T) -> bool) -> bool {
        match self {
            Ok(t) => cond(t),
            Err(_) => false,
        }
    }

    pub fn into_is_ok_or(self, cond: impl FnOnce(E) -> bool) -> bool {
        match self {
            Ok(_) => true,
            Err(e) => cond(e),
        }
    }

    pub fn into_is_err_and(self, cond: impl FnOnce(E) -> bool) -> bool {
        match self {
            Ok(_) => false,
            Err(e) => cond(e),
        }
    }

    pub fn into_is_err_or(self, cond: impl FnOnce(T) -> bool) -> bool {
        match self {
            Ok(t) => cond(t),
            Err(_) => true,
        }
    }

    pub fn into_boption(self) -> BOption<T> {
        match self {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }

    pub fn into_boption_err(self) -> BOption<E> {
        match self {
            Ok(_) => None,
            Err(e) => Some(e),
        }
    }

    pub unsafe fn into_boption_unchecked(self) -> BOption<T> {
        match self {
            Ok(t) => Some(t),
            Err(_) => unsafe { unreachable_unchecked() },
        }
    }

    pub unsafe fn into_boption_err_unchecked(self) -> BOption<E> {
        match self {
            Ok(_) => unsafe { unreachable_unchecked() },
            Err(e) => Some(e),
        }
    }

    pub fn into_option(self) -> Option<T> {
        match self {
            Ok(t) => Option::Some(t),
            Err(_) => Option::None,
        }
    }

    pub fn into_option_err(self) -> Option<E> {
        match self {
            Ok(_) => Option::None,
            Err(e) => Option::Some(e),
        }
    }

    pub unsafe fn into_option_unchecked(self) -> Option<T> {
        match self {
            Ok(t) => Option::Some(t),
            Err(_) => unsafe { unreachable_unchecked() },
        }
    }

    pub unsafe fn into_option_err_unchecked(self) -> Option<E> {
        match self {
            Ok(_) => unsafe { unreachable_unchecked() },
            Err(e) => Option::Some(e),
        }
    }

    pub const fn as_ref(&self) -> BResult<&T, &E> {
        match *self {
            Ok(ref t) => Ok(t),
            Err(ref e) => Err(e),
        }
    }

    pub const fn as_mut(&mut self) -> BResult<&mut T, &mut E> {
        match *self {
            Ok(ref mut t) => Ok(t),
            Err(ref mut e) => Err(e),
        }
    }

    pub fn into_map_ok<U>(self, mapper: impl FnOnce(T) -> U) -> BResult<U, E> {
        match self {
            Ok(t) => Ok(mapper(t)),
            Err(e) => Err(e),
        }
    }

    pub fn into_map_ok_or<U>(self, mapper: impl FnOnce(T) -> U, default_if_err: U) -> U {
        match self {
            Ok(t) => mapper(t),
            Err(_) => default_if_err,
        }
    }

    pub fn into_map_ok_or_else<U>(self, mapper_t: impl FnOnce(T) -> U, mapper_e: impl FnOnce(E) -> U) -> U {
        match self {
            Ok(t) => mapper_t(t),
            Err(e) => mapper_e(e),
        }
    }

    pub fn into_map_ok_or_default<U>(self, mapper: impl FnOnce(T) -> U) -> U
    where
        U: Default,
    {
        match self {
            Ok(t) => mapper(t),
            Err(_) => U::default(),
        }
    }

    pub fn into_map_err<F>(self, mapper_err: impl FnOnce(E) -> F) -> BResult<T, F> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(mapper_err(e)),
        }
    }

    pub fn into_map_err_or<F>(self, default_if_ok: F, mapper_err: impl FnOnce(E) -> F) -> F {
        match self {
            Ok(_) => default_if_ok,
            Err(e) => mapper_err(e),
        }
    }

    pub fn into_map_err_or_else<F>(self, mapper_ok: impl FnOnce(T) -> F, mapper_err: impl FnOnce(E) -> F) -> F {
        match self {
            Ok(t) => mapper_ok(t),
            Err(e) => mapper_err(e),
        }
    }

    pub fn into_map_err_or_default<F: Default>(self, mapper_err: impl FnOnce(E) -> F) -> F {
        match self {
            Ok(_) => F::default(),
            Err(e) => mapper_err(e),
        }
    }

    pub fn into_self_inspect_ok(self, inspector: impl FnOnce(&T)) -> BResult<T, E> {
        match self {
            Ok(ref t) => inspector(t),
            Err(_) => {}
        }
        self
    }

    pub fn into_self_inspect_err(self, inspector_err: impl FnOnce(&E)) -> BResult<T, E> {
        match self {
            Ok(_) => {}
            Err(ref e) => inspector_err(e),
        }
        self
    }

    pub fn as_inspect_ok(&self, inspector: impl FnOnce(&T)) {
        match *self {
            Ok(ref t) => inspector(t),
            Err(_) => {}
        }
    }

    pub fn as_inspect_err(&self, inspector_err: impl FnOnce(&E)) {
        match *self {
            Ok(_) => {}
            Err(ref e) => inspector_err(e),
        }
    }

    pub fn into_map_ok_flatten<U>(self, other_if_ok: BResult<U, E>) -> BResult<U, E> {
        match self {
            Ok(_) => other_if_ok,
            Err(e) => Err(e),
        }
    }

    pub fn into_map_ok_flatten_lazy<U>(self, other_if_ok_lazy: impl FnOnce(T) -> BResult<U, E>) -> BResult<U, E> {
        match self {
            Ok(t) => other_if_ok_lazy(t),
            Err(e) => Err(e),
        }
    }

    pub fn into_map_err_flatten<F>(self, other_if_err: BResult<T, F>) -> BResult<T, F> {
        match self {
            Ok(t) => Ok(t),
            Err(_) => other_if_err,
        }
    }

    pub fn into_map_err_flatten_lazy<F>(self, other_if_err_lazy: impl FnOnce(E) -> BResult<T, F>) -> BResult<T, F> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => other_if_err_lazy(e),
        }
    }

    pub const fn into_result(self) -> Result<T, E> {
        let this = ManuallyDrop::new(self);
        match *unsafe { &*manually_drop_as_ptr(&this) } {
            Ok(_) => Result::Ok(unsafe { manually_drop_as_ptr(&this).cast::<T>().read() }),
            Err(_) => Result::Err(unsafe { manually_drop_as_ptr(&this).cast::<E>().read() }),
        }
    }

    pub const fn into_ffi_result(self) -> FfiResult<T, E> {
        let this = ManuallyDrop::new(self);
        let tag = match *unsafe { &*manually_drop_as_ptr(&this) } {
            Ok(_) => FfiResultTag::Ok,
            Err(_) => FfiResultTag::Err,
        };
        let discriminant = match *unsafe { &*manually_drop_as_ptr(&this) } {
            Ok(_) => FfiResultDiscr {
                Ok: ManuallyDrop::new(unsafe { manually_drop_as_ptr(&this).cast::<T>().read() }),
            },
            Err(_) => FfiResultDiscr {
                Err: ManuallyDrop::new(unsafe { manually_drop_as_ptr(&this).cast::<E>().read() }),
            },
        };
        FfiResult { tag, discriminant }
    }
}

impl<T, E> BResult<&T, E> {
    pub fn into_cloned(self) -> BResult<T, E>
    where
        T: Clone,
    {
        match self {
            Ok(t) => Ok(t.clone()),
            Err(e) => Err(e),
        }
    }

    pub fn into_copied(self) -> BResult<T, E>
    where
        T: Copy,
    {
        match self {
            Ok(t) => Ok(*t),
            Err(e) => Err(e),
        }
    }
}

impl<T, E> BResult<&mut T, E> {
    pub fn into_cloned(self) -> BResult<T, E>
    where
        T: Clone,
    {
        match self {
            Ok(t) => Ok(t.clone()),
            Err(e) => Err(e),
        }
    }

    pub fn into_copied(self) -> BResult<T, E>
    where
        T: Copy,
    {
        match self {
            Ok(t) => Ok(*t),
            Err(e) => Err(e),
        }
    }
}

impl<T, E> BResult<T, &E> {
    pub fn into_err_cloned(self) -> BResult<T, E>
    where
        E: Clone,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.clone()),
        }
    }

    pub fn into_err_copied(self) -> BResult<T, E>
    where
        E: Copy,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(*e),
        }
    }
}

impl<T, E> BResult<T, &mut E> {
    pub fn into_err_cloned(self) -> BResult<T, E>
    where
        E: Clone,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.clone()),
        }
    }

    pub fn into_err_copied(self) -> BResult<T, E>
    where
        E: Copy,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(*e),
        }
    }
}

impl<T, E> BResult<BResult<T, E>, E> {
    pub fn into_flattened(self) -> BResult<T, E> {
        match self {
            Ok(inner) => inner,
            Err(e) => Err(e),
        }
    }
}

impl<T> BResult<T, Infallible> {
    pub fn into_ok_infallible(self) -> T {
        let Ok(t) = self;
        t
    }
}

impl<E> BResult<Infallible, E> {
    pub fn into_err_infallible(self) -> E {
        let Err(e) = self;
        e
    }
}

#[track_caller]
#[inline(always)]
fn unwrap_ok_failed_default() -> ! {
    unwrap_failed("called unwrap on Err value")
}

#[track_caller]
#[inline(always)]
fn unwrap_err_failed_default() -> ! {
    unwrap_failed("called unwrap_err on Ok value")
}

#[track_caller]
#[inline(never)]
fn unwrap_failed(message: &str) -> ! {
    panic!("unwrap failed: {}", message);
}
