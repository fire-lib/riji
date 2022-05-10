#![doc = include_str!("../README.md")]

#[macro_use]
pub mod paint;

mod script;
pub use script::{Script, Result, Error};

mod api;

mod ref_cell;