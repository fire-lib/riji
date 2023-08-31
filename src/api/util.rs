use super::RhaiResult;

use std::fmt::Write;

use rhai::{Engine, Module};

use rand::RngCore;


// returns a random value that can be used in /etc/machine-id
fn random_machine_id() -> RhaiResult<String> {
	let mut bytes = [0u8; 16];
	rand::thread_rng().fill_bytes(&mut bytes);

	let mut s = String::with_capacity(32);
	for b in bytes {
		write!(s, "{b:02x}").unwrap();
	}

	Ok(s)
}


pub fn add(engine: &mut Engine) {
	let mut util_mod = Module::new();
	util_mod.set_native_fn("random_machine_id", random_machine_id);
	engine
		.register_static_module("util", util_mod.into());
}