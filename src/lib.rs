//! i am sick and tired of the Result, Option API naming
//! conventions in Rust. `is`, `as`, `to`, `into` all the way through.
#![no_std]
#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(offset_of_enum)]
#![feature(panic_internals)]
// todo: feature flags, for into_ok, into_err
// todo: remove core_intrinsics, panic_internals, offset_of_enum feature dependencies

mod betteroption;
mod betterresult;

pub use betteroption::BetterOption::{self, *};
pub use betterresult::BetterResult::{self, *};
