
use super::RhaiResult;

use uuid::Uuid;

use rhai::{Engine, Array};

// Uuid::new_v4()

fn uuid_v4() -> String {
	Uuid::new_v4().to_string()
}

/// empty if not found
fn env_var(key: &str) -> String {
	std::env::var(key).unwrap_or_else(|_| String::new())
}

fn env_os() -> &'static str {
	if cfg!(target_os = "windows") {
		return "windows";
	} else if cfg!(target_os = "linux") {
		return "linux";
	} else {
		return "unknown";
	}
}

fn trim(s: &str) -> String {
	s.trim().into()
}

fn lowercase(s: &str) -> String {
	s.to_lowercase()
}

fn uppercase(s: &str) -> String {
	s.to_uppercase()
}

fn starts_with(a: &str, b: &str) -> bool {
	a.starts_with(b)
}

fn ends_with(a: &str, b: &str) -> bool {
	a.ends_with(b)
}

fn sort_strs(a: Array) -> RhaiResult<Array> {
	let mut ar: Vec<_> = a.into_iter()
		.map(|a| a.into_immutable_string()
			.map_err(|e| err!("{:?}", e)))
		.collect::<RhaiResult<_>>()?;
	ar.sort();

	let ar = ar.into_iter()
		.map(Into::into)
		.collect();

	Ok(ar)
}

pub fn add(engine: &mut Engine) {
	engine
		.register_fn("uuid_v4", uuid_v4)
		.register_fn("env_var", env_var)
		.register_fn("trim", trim)
		.register_fn("lowercase", lowercase)
		.register_fn("uppercase", uppercase)
		.register_fn("starts_with", starts_with)
		.register_fn("ends_with", ends_with)
		.register_fn("sort_strs", sort_strs)
		.register_fn("env_os", env_os);
}