
use super::{RhaiResult, io_err, reg_err};

use std::fs;
use std::borrow::Cow;

use rhai::{Engine, ImmutableString, Array, Dynamic};

#[derive(Clone)]
pub struct Regex {
	inner: regex::Regex
}

impl Regex {
	pub fn new(s: &str) -> RhaiResult<Self> {
		regex::Regex::new(s)
			.map_err(reg_err)
			.map(|inner| Self { inner })
	}

	pub fn matches(&mut self, s: &str) -> bool {
		self.inner.is_match(s)
	}

	pub fn matches_file(&mut self, path: &str) -> RhaiResult<bool> {
		let s = fs::read_to_string(path)
			.map_err(io_err)?;

		Ok(self.inner.is_match(&s))
	}

	pub fn find(&mut self, s: &str) -> ImmutableString {
		self.inner.find(s)
			.map(|m| m.as_str())
			.unwrap_or("")
			.into()
	}

	fn capt_to_arr(capt: regex::Captures) -> Array {
		capt.iter()
			.map(|m| m.map(|m| m.as_str()).unwrap_or(""))
			.map(ImmutableString::from)
			.map(Into::into)
			.collect()
	}

	pub fn captures(&mut self, s: &str) -> Array {
		let capt = match self.inner.captures(s) {
			Some(capt) => capt,
			None => return vec![]
		};

		Self::capt_to_arr(capt)
	}

	pub fn captures_all(&mut self, s: &str) -> Array {
		self.inner.captures_iter(s)
			.map(Self::capt_to_arr)
			.map(Dynamic::from_array)
			.collect()
	}

	pub fn replace(&mut self, s: &str, rep: &str) -> ImmutableString {
		self.inner.replace_all(s, rep)
			.into_owned()
			.into()
	}

	pub fn replace_file(&mut self, path: &str, rep: &str) -> RhaiResult<()> {
		// get file content
		let s = fs::read_to_string(path)
			.map_err(io_err)?;

		// let's first replace
		let r = self.inner.replace_all(&s, rep);

		match r {
			// nothing to replace
			Cow::Borrowed(_) => Ok(()),
			Cow::Owned(s) => {
				fs::write(path, s)
					.map_err(io_err)
					.map(|_| ())
			}
		}
	}
}

pub fn add(engine: &mut Engine) {
	engine
		.register_result_fn("regex", Regex::new)
		.register_fn("matches", Regex::matches)
		.register_fn("replace", Regex::replace)
		.register_fn("find", Regex::find)
		.register_fn("captures", Regex::captures)
		.register_fn("captures_all", Regex::captures_all)
		.register_result_fn("replace_file", Regex::replace_file)
		.register_result_fn("matches_file", Regex::matches_file);
}