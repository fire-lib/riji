
use super::{RhaiResult, fs};


use toml::{to_string_pretty, from_str};

use rhai::{Engine, Dynamic, Module};

fn read(path: &str) -> RhaiResult<Dynamic> {
	let ctn = fs::read_file(path)?;
	from_str(&ctn)
		.map_err(|e| err!("toml: failed to deserialize {} error {:?}", path, e))
}

fn parse(ctn: &str) -> RhaiResult<Dynamic> {
	from_str(&ctn)
		.map_err(|e| err!("toml: failed to deserialize error {:?}", e))
}

fn write(path: &str, data: Dynamic) -> RhaiResult<()> {
	let s = stringify(data)?;
	fs::write_file_str(path, &s)
}

fn stringify(data: Dynamic) -> RhaiResult<String> {
	// first convert the data to a toml::Value which prevents from receiving
	// the values after table error
	let value = toml::Value::try_from(data)
		.map_err(|e| err!(
			"toml: failed to serialize error {:?}",
			e
		))?;

	to_string_pretty(&value)
		.map_err(|e| err!(
			"toml: failed to serialize error {:?}",
			e
		))
}

pub fn add(engine: &mut Engine) {
	let mut toml_mod = Module::new();
	toml_mod.set_native_fn("read", read);
	toml_mod.set_native_fn("parse", parse);
	toml_mod.set_native_fn("write", write);
	toml_mod.set_native_fn("stringify", stringify);
	engine
		.register_static_module("toml", toml_mod.into());
}