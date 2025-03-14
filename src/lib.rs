//! i am sick and tired of the Result, Option API naming
//! conventions in Rust. `is`, `as`, `to`, `into` all the way through.
#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(offset_of_enum)]
#![feature(panic_internals)]
// todo: feature flags, for into_ok, into_err, unwrap_failed

mod betteroption;
mod betterresult;

pub use betteroption::BetterOption;
pub use betterresult::BetterResult;
