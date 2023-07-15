//! ## Riji
//!
//! Riji is a cli that is intended to make it easy to fork
//! big projects without needing to copy all files.
//!

/*
features:
- add to path environment
- install packages
- git command

- custom commands ?
*/

use std::env;

use std::path::Path;

use riji::{Script, Result, Error, paint::Red, epaintln};

fn main() {
	if let Err(e) = execute() {
		match e {
			Error::Rhai(e) => {
				epaintln!(Red, "rhai error {:?}", e)
			},
			Error::Parse(e) => {
				epaintln!(Red, "parse error {:?}", e)
			},
			Error::Io(_) => {
				epaintln!(Red, "file \"./riji.rhai\" not found")
				// epaintln!(Red, "io error {:?}", e)
			},
		}
		// return with error
		std::process::exit(1);
	}
}

fn execute() -> Result<()> {
	let mut args = std::env::args()
		.skip(1);
	let cmd = args.next()
		.unwrap_or("help".into());
	let args: Vec<_> = args.collect();

	if cmd == "--version" {
		println!("Riji version {}", env!("CARGO_PKG_VERSION"));
		return Ok(())
	}


	let mut script = if let Ok(file) = env::var("RIJI_SCRIPT") {
		let path = Path::new(&file);
		let parent = path.parent()
			.expect("could not get parent from RIJI_SCRIPT");

		env::set_current_dir(parent)
			.map_err(Error::Io)?;

		let file_name = path.file_name()
			.expect("failed to get file_name form RIJI_SCRIPT");

		let path = format!("./{}", file_name.to_str().unwrap());
		Script::new(path)?
	} else {
		Script::new("./riji.rhai")?
	};

	script.execute(&cmd, args)
}