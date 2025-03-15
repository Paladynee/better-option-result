use crate::BetterResult;
use core::cmp;
use core::hint;
use core::mem;
use core::ops::Deref;
use core::ops::DerefMut;
use core::option;
use core::panicking;
use core::pin::Pin;
use core::slice;

use BetterOption::*;
use BetterResult::*;

#[allow(clippy::derived_hash_with_manual_eq)] // PartialEq is manually implemented equivalently
#[derive(Copy, Eq, Debug, Hash)]
pub enum BetterOption<T> {
    Some(T),
    None,
}

// no <[\w&&\s&&,]*:> allowed.
impl<T> BetterOption<T> {
    pub fn unwrap(self) -> T {
        match self {
            Some(val) => val,
            None => unwrap_failed("called `Option::unwrap()` on a `None` value"),
        }
    }

    pub fn unwrap_none(self) {
        match self {
            Some(_) => unwrap_failed("called `Option::unwrap_none()` on a `Some` value"),
            None => (),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Some(x) => x,
            None => default,
        }
    }

    pub fn unwrap_or_lazy<F>(self, default_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Some(x) => x,
            None => default_fn(),
        }
    }

    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Some(x) => x,
            None => Default::default(),
        }
    }

    pub fn expect(self, msg: &str) -> T {
        match self {
            Some(val) => val,
            None => unwrap_failed(msg),
        }
    }

    pub fn expect_none(self, msg: &str) {
        match self {
            Some(_) => unwrap_failed(msg),
            None => (),
        }
    }

    pub unsafe fn unwrap_unchecked(self) -> T {
        match self {
            Some(val) => val,
            // SAFETY: the safety contract must be upheld by the caller.
            None => unsafe { hint::unreachable_unchecked() },
        }
    }

    pub unsafe fn unwrap_none_unchecked(self) {
        match self {
            Some(_) => unsafe { hint::unreachable_unchecked() },
            None => (),
        }
    }

    pub const fn is_some(&self) -> bool {
        matches!(*self, Some(_))
    }

    pub const fn is_not_some(&self) -> bool {
        !self.is_some()
    }

    pub const fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub const fn is_not_none(&self) -> bool {
        self.is_some()
    }

    pub fn into_is_some_and<F>(self, f: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        match self {
            Some(x) => f(x),
            None => false,
        }
    }

    pub fn into_is_some_or<F>(self, f: F) -> bool
    where
        F: FnOnce() -> bool,
    {
        match self {
            Some(_) => true,
            None => f(),
        }
    }

    pub fn into_is_some_nand<F>(self, f: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        match self {
            Some(t) => !f(t),
            None => true,
        }
    }

    pub fn into_is_some_nor<F>(self, f: F) -> bool
    where
        F: FnOnce() -> bool,
    {
        match self {
            Some(_) => true,
            None => !f(),
        }
    }

    pub fn into_is_some_xor<F, G>(self, f: F, g: G) -> bool
    where
        F: FnOnce(T) -> bool,
        G: FnOnce() -> bool,
    {
        match self {
            Some(t) => !f(t),
            None => g(),
        }
    }

    pub fn into_is_some_xnor<F, G>(self, f: F, g: G) -> bool
    where
        F: FnOnce(T) -> bool,
        G: FnOnce() -> bool,
    {
        match self {
            Some(t) => f(t),
            None => !g(),
        }
    }

    pub fn into_is_none_or<F>(self, map: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        match self {
            Some(x) => map(x),
            None => true,
        }
    }

    // todo: into_is_none_* all gates impl

    pub const fn as_ref(&self) -> BetterOption<&T> {
        match *self {
            Some(ref t) => Some(t),
            None => None,
        }
    }

    pub const fn as_mut(&mut self) -> BetterOption<&mut T> {
        match *self {
            Some(ref mut x) => Some(x),
            None => None,
        }
    }

    pub const fn as_pin_ref(self: Pin<&Self>) -> BetterOption<Pin<&T>> {
        // FIXME(const-hack): use `map` once that is possible
        match Pin::get_ref(self).as_ref() {
            // SAFETY: `x` is guaranteed to be pinned because it comes from `self`
            // which is pinned.
            Some(x) => unsafe { Some(Pin::new_unchecked(x)) },
            None => None,
        }
    }

    pub const fn as_pin_mut(self: Pin<&mut Self>) -> BetterOption<Pin<&mut T>> {
        // SAFETY: `get_unchecked_mut` is never used to move the `Option` inside `self`.
        // `x` is guaranteed to be pinned because it comes from `self` which is pinned.
        unsafe {
            // FIXME(const-hack): use `map` once that is possible
            match Pin::get_unchecked_mut(self).as_mut() {
                Some(x) => Some(Pin::new_unchecked(x)),
                None => None,
            }
        }
    }

    const fn len(&self) -> usize {
        // Using the intrinsic avoids emitting a branch to get the 0 or 1.
        let discriminant: isize = core::intrinsics::discriminant_value(self);
        discriminant as usize
    }

    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: When the `Option` is `Some`, we're using the actual pointer
        // to the payload, with a length of 1, so this is equivalent to
        // `slice::from_ref`, and thus is safe.
        // When the `Option` is `None`, the length used is 0, so to be safe it
        // just needs to be aligned, which it is because `&self` is aligned and
        // the offset used is a multiple of alignment.
        //
        // In the new version, the intrinsic always returns a pointer to an
        // in-bounds and correctly aligned position for a `T` (even if in the
        // `None` case it's just padding).
        unsafe { slice::from_raw_parts((self as *const Self).byte_add(mem::offset_of!(Self, Some.0)).cast(), self.len()) }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: When the `Option` is `Some`, we're using the actual pointer
        // to the payload, with a length of 1, so this is equivalent to
        // `slice::from_mut`, and thus is safe.
        // When the `Option` is `None`, the length used is 0, so to be safe it
        // just needs to be aligned, which it is because `&self` is aligned and
        // the offset used is a multiple of alignment.
        //
        // In the new version, the intrinsic creates a `*const T` from a
        // mutable reference  so it is safe to cast back to a mutable pointer
        // here. As with `as_slice`, the intrinsic always returns a pointer to
        // an in-bounds and correctly aligned position for a `T` (even if in
        // the `None` case it's just padding).
        unsafe { slice::from_raw_parts_mut((self as *mut Self).byte_add(core::mem::offset_of!(Self, Some.0)).cast(), self.len()) }
    }

    pub fn into_mapped<U, F>(self, map: F) -> BetterOption<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Some(t) => Some(map(t)),
            None => None,
        }
    }

    pub fn into_mapped_or<U, F>(self, default: U, map: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Some(t) => map(t),
            None => default,
        }
    }

    pub fn into_mapped_or_lazy<U, D, F>(self, default_fn: D, map: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            Some(t) => map(t),
            None => default_fn(),
        }
    }

    pub fn into_mapped_or_default<U, F>(self, map: F) -> U
    where
        F: FnOnce(T) -> U,
        U: Default,
    {
        match self {
            Some(t) => map(t),
            None => Default::default(),
        }
    }

    pub fn into_result<E>(self, e: E) -> BetterResult<T, E> {
        match self {
            Some(t) => Ok(t),
            None => Err(e),
        }
    }

    pub fn into_result_lazy<E, F>(self, f: F) -> BetterResult<T, E>
    where
        F: FnOnce() -> E,
    {
        match self {
            Some(t) => Ok(t),
            None => Err(f()),
        }
    }

    pub fn into_result_default<E>(self) -> BetterResult<T, E>
    where
        E: Default,
    {
        match self {
            Some(t) => Ok(t),
            None => Err(Default::default()),
        }
    }

    pub fn as_deref(&self) -> BetterOption<&T::Target>
    where
        T: Deref,
    {
        self.as_ref().into_mapped(|t| t.deref())
    }

    pub fn as_deref_mut(&mut self) -> BetterOption<&mut T::Target>
    where
        T: DerefMut,
    {
        self.as_mut().into_mapped(|t| t.deref_mut())
    }

    // todo: iter methods

    pub fn into_arg<U>(self, optb: BetterOption<U>) -> BetterOption<U> {
        match self {
            Some(_) => optb,
            None => None,
        }
    }

    pub fn into_arg_lazy<U, F>(self, f: F) -> BetterOption<U>
    where
        F: FnOnce(T) -> BetterOption<U>,
    {
        match self {
            Some(x) => f(x),
            None => None,
        }
    }

    pub fn into_filtered<P>(self, predicate: P) -> Self
    where
        P: FnOnce(&T) -> bool,
    {
        if let Some(x) = self {
            if predicate(&x) {
                return Some(x);
            }
        }
        None
    }

    pub fn into_arg_if_none(self, optb: BetterOption<T>) -> BetterOption<T> {
        match self {
            x @ Some(_) => x,
            None => optb,
        }
    }

    pub fn into_arg_if_none_lazy<F>(self, f: F) -> BetterOption<T>
    where
        F: FnOnce() -> BetterOption<T>,
    {
        match self {
            x @ Some(_) => x,
            None => f(),
        }
    }

    pub fn into_arg_xor(self, optb: BetterOption<T>) -> BetterOption<T> {
        match (self, optb) {
            (a @ Some(_), None) => a,
            (None, b @ Some(_)) => b,
            _ => None,
        }
    }

    pub fn as_mut_insert(&mut self, value: T) -> &mut T {
        *self = Some(value);

        // SAFETY: the code above just filled the option
        unsafe { self.as_mut().unwrap_unchecked() }
    }

    pub fn as_mut_get_or_insert(&mut self, value: T) -> &mut T {
        self.as_mut_get_or_insert_lazy(|| value)
    }

    pub fn as_mut_get_or_insert_default(&mut self) -> &mut T
    where
        T: Default,
    {
        self.as_mut_get_or_insert_lazy(T::default)
    }

    pub fn as_mut_get_or_insert_lazy<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        if let None = self {
            *self = Some(f());
        }

        // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
        // variant in the code above.
        unsafe { self.as_mut().unwrap_unchecked() }
    }

    pub const fn as_mut_take(&mut self) -> BetterOption<T> {
        // FIXME(const-hack) replace `mem::replace` by `mem::take` when the latter is const ready
        mem::replace(self, None)
    }

    pub fn as_mut_take_if<P>(&mut self, predicate: P) -> BetterOption<T>
    where
        P: FnOnce(&mut T) -> bool,
    {
        if self.as_mut().into_mapped_or(false, predicate) {
            self.as_mut_take()
        } else {
            None
        }
    }

    pub const fn as_mut_replace(&mut self, value: T) -> BetterOption<T> {
        mem::replace(self, Some(value))
    }

    pub fn into_zipped<U>(self, other: BetterOption<U>) -> BetterOption<(T, U)> {
        match (self, other) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }

    pub fn into_zipped_lazy<U, F, R>(self, other: BetterOption<U>, f: F) -> BetterOption<R>
    where
        F: FnOnce(T, U) -> R,
    {
        match (self, other) {
            (Some(a), Some(b)) => Some(f(a, b)),
            _ => None,
        }
    }

    pub fn into_core_option(self) -> option::Option<T> {
        match self {
            Some(t) => option::Option::Some(t),
            None => option::Option::None,
        }
    }
}

// core aliases
impl<T> BetterOption<T> {
    /// stable alias for `into_is_some_and`
    pub fn is_some_and<F>(self, f: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        self.into_is_some_and(f)
    }

    /// stable alias for `into_is_none_or`
    pub fn is_none_or<F>(self, map: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        self.into_is_none_or(map)
    }

    /// stable alias for `into_mapped`
    pub fn map<U, F>(self, map: F) -> BetterOption<U>
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
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            Some(t) => map(t),
            None => default_fn(),
        }
    }

    /// stable alias for `into_result`
    pub fn ok_or<E>(self, e: E) -> BetterResult<T, E> {
        self.into_result(e)
    }

    /// stable alias for `into_result_lazy`
    pub fn ok_or_else<F, E>(self, f: F) -> BetterResult<T, E>
    where
        F: FnOnce() -> E,
    {
        self.into_result_lazy(f)
    }

    /// stable alias for `into_arg`
    pub fn and<U>(self, optb: BetterOption<U>) -> BetterOption<U> {
        self.into_arg(optb)
    }

    /// stable alias for `into_arg_lazy`
    pub fn and_then<U, F>(self, f: F) -> BetterOption<U>
    where
        F: FnOnce(T) -> BetterOption<U>,
    {
        self.into_arg_lazy(f)
    }

    /// stable alias for `into_filtered`
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: FnOnce(&T) -> bool,
    {
        self.into_filtered(predicate)
    }

    /// stable alias for `into_arg_if_none`
    pub fn or(self, optb: BetterOption<T>) -> BetterOption<T> {
        self.into_arg_if_none(optb)
    }

    /// stable alias for `into_arg_if_none_lazy`
    pub fn or_else<F>(self, f: F) -> BetterOption<T>
    where
        F: FnOnce() -> BetterOption<T>,
    {
        self.into_arg_if_none_lazy(f)
    }

    /// stable alias for `into_arg_xor`
    pub fn xor(self, optb: BetterOption<T>) -> BetterOption<T> {
        self.into_arg_xor(optb)
    }

    /// stable alias for `as_mut_insert`
    pub fn insert(&mut self, value: T) -> &mut T {
        self.as_mut_insert(value)
    }

    /// stable alias for `as_mut_get_or_insert`
    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        self.as_mut_get_or_insert(value)
    }

    /// stable alias for `as_mut_get_or_insert_default`
    pub fn get_or_insert_default(&mut self) -> &mut T
    where
        T: Default,
    {
        self.as_mut_get_or_insert_default()
    }

    /// stable alias for `as_mut_get_or_insert_lazy`
    pub fn get_or_insert_with<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        self.as_mut_get_or_insert_lazy(f)
    }

    /// stable alias for `as_mut_take`
    pub const fn take(&mut self) -> BetterOption<T> {
        // FIXME(const-hack) replace `mem::replace` by `mem::take` when the latter is const ready
        mem::replace(self, None)
    }

    /// stable alias for `as_mut_take_if`
    pub fn take_if<P>(&mut self, predicate: P) -> BetterOption<T>
    where
        P: FnOnce(&mut T) -> bool,
    {
        self.as_mut_take_if(predicate)
    }

    /// stable alias for `as_mut_replace`
    pub const fn replace(&mut self, value: T) -> BetterOption<T> {
        self.as_mut_replace(value)
    }

    /// stable alias for `into_zipped`
    pub fn zip<U>(self, other: BetterOption<U>) -> BetterOption<(T, U)> {
        self.into_zipped(other)
    }

    /// stable alias for `into_zip_with`
    pub fn zip_with<U, F, R>(self, other: BetterOption<U>, f: F) -> BetterOption<R>
    where
        F: FnOnce(T, U) -> R,
    {
        self.into_zipped_lazy(other, f)
    }
}

// core aliases
impl<T, U> BetterOption<(T, U)> {
    /// stable alias for `into_unzipped`
    pub fn unzip(self) -> (BetterOption<T>, BetterOption<U>) {
        self.into_unzipped()
    }
}

// core aliases
impl<T> BetterOption<&T> {
    /// stable alias for `into_copied`
    pub const fn copied(self) -> BetterOption<T>
    where
        T: Copy,
    {
        self.into_copied()
    }

    /// stable alias for `into_cloned`
    pub fn cloned(self) -> BetterOption<T>
    where
        T: Clone,
    {
        self.into_cloned()
    }
}

impl<T> From<option::Option<T>> for BetterOption<T> {
    fn from(value: option::Option<T>) -> Self {
        match value {
            option::Option::Some(t) => Some(t),
            option::Option::None => None,
        }
    }
}

impl<T, U> BetterOption<(T, U)> {
    pub fn into_unzipped(self) -> (BetterOption<T>, BetterOption<U>) {
        match self {
            Some((a, b)) => (Some(a), Some(b)),
            None => (None, None),
        }
    }
}

impl<T> BetterOption<&T> {
    pub const fn into_copied(self) -> BetterOption<T>
    where
        T: Copy,
    {
        // FIXME(const-hack): this implementation, which sidesteps using `Option::map` since it's not const
        // ready yet, should be reverted when possible to avoid code repetition
        match self {
            Some(&v) => Some(v),
            None => None,
        }
    }

    pub fn into_cloned(self) -> BetterOption<T>
    where
        T: Clone,
    {
        match self {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }
}

impl<T> BetterOption<&mut T> {
    pub const fn copied(self) -> BetterOption<T>
    where
        T: Copy,
    {
        match self {
            Some(&mut t) => Some(t),
            None => None,
        }
    }

    pub fn cloned(self) -> BetterOption<T>
    where
        T: Clone,
    {
        match self {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }
}

impl<T, E> BetterOption<BetterResult<T, E>> {
    pub fn into_result_transposed(self) -> BetterResult<BetterOption<T>, E> {
        match self {
            Some(Ok(x)) => Ok(Some(x)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

// core aliases
impl<T, E> BetterOption<BetterResult<T, E>> {
    /// stable alias for `into_result_transposed`
    pub fn transpose(self) -> BetterResult<BetterOption<T>, E> {
        self.into_result_transposed()
    }
}

#[cold]
#[track_caller]
const fn unwrap_failed(msg: &str) -> ! {
    panicking::panic_display(&msg)
}

// todo: intoiter implementation

impl<T> Clone for BetterOption<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Some(to), Some(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}

impl<T> Default for BetterOption<T> {
    fn default() -> BetterOption<T> {
        None
    }
}

impl<T> From<T> for BetterOption<T> {
    fn from(val: T) -> BetterOption<T> {
        Some(val)
    }
}

impl<'a, T> From<&'a BetterOption<T>> for BetterOption<&'a T> {
    fn from(o: &'a BetterOption<T>) -> BetterOption<&'a T> {
        o.as_ref()
    }
}

impl<'a, T> From<&'a mut BetterOption<T>> for BetterOption<&'a mut T> {
    fn from(o: &'a mut BetterOption<T>) -> BetterOption<&'a mut T> {
        o.as_mut()
    }
}

impl<T: PartialEq> PartialEq for BetterOption<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // Spelling out the cases explicitly optimizes better than
        // `_ => false`
        match (self, other) {
            (Some(l), Some(r)) => *l == *r,
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}

impl<T: PartialOrd> PartialOrd for BetterOption<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Some(l), Some(r)) => l.partial_cmp(r),
            (Some(_), None) => option::Option::Some(cmp::Ordering::Greater),
            (None, Some(_)) => option::Option::Some(cmp::Ordering::Less),
            (None, None) => option::Option::Some(cmp::Ordering::Equal),
        }
    }
}

impl<T: Ord> Ord for BetterOption<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (Some(l), Some(r)) => l.cmp(r),
            (Some(_), None) => cmp::Ordering::Greater,
            (None, Some(_)) => cmp::Ordering::Less,
            (None, None) => cmp::Ordering::Equal,
        }
    }
}

impl<T> BetterOption<BetterOption<T>> {
    pub fn into_flatten(self) -> BetterOption<T> {
        // FIXME(const-hack): could be written with `and_then`
        match self {
            Some(inner) => inner,
            None => None,
        }
    }
}
