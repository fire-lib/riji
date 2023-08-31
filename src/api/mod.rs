
use crate::script::RhaiResult;

use std::io;

pub type RhaiError = Box<rhai::EvalAltResult>;

macro_rules! err {
	($fmt:literal $($t:tt)*) => (
		Box::new(rhai::EvalAltResult::ErrorRuntime(
			format!($fmt $($t)*).into(),
			rhai::Position::NONE
		))
	)
}

pub mod cmd;
pub mod git;
pub mod fs;
pub mod regex;
pub mod other;
pub mod toml;
pub mod util;

fn git_err(e: git2::Error) -> RhaiError {
	err!("{:?}", e)
}

fn io_err(e: io::Error) -> RhaiError {
	err!("{:?}", e)
}

fn reg_err(e: ::regex::Error) -> RhaiError {
	err!("{:?}", e)
}