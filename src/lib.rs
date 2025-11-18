//! i am sick and tired of the Result, Option API naming
//! conventions in Rust. `is`, `as`, `to`, `into` all the way through.
#![no_std]

pub mod betteroption;
pub mod betterresult;

pub mod prelude {
    pub use super::betteroption::{BOption, IntoBOption};
    pub use super::betterresult::{BResult, IntoBResult};
}
