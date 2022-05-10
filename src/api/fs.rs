
use super::{RhaiResult, io_err};

use std::{fs, env};
use std::io::Write;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
#[cfg(target_family = "unix")]
use std::process::Command;

use rhai::{Array, Engine, ImmutableString, Module};

fn create_dir(s: &str) -> RhaiResult<()> {
	if is_dir(s)? {
		return Ok(())
	}

	paint_act!("create directory {:?}", s);
	fs::create_dir_all(s)
		.map_err(io_err)
}

fn is_dir(s: &str) -> RhaiResult<bool> {
	Ok(Path::is_dir(s.as_ref()))
}

fn is_file(s: &str) -> RhaiResult<bool> {
	Ok(Path::is_file(s.as_ref()))
}

fn is_prog(s: &str) -> RhaiResult<bool> {
	which(s).map(|s| !s.is_empty())
}

fn which(s: &str) -> RhaiResult<String> {
	if is_file(&format!("./{}", s))? {
		return full_path(&format!("./{}", s));
	}

	// else check env
	if let Ok(path) = env::var("PATH") {
		for p in path.split(":") {
			let raw_path = format!("{}/{}", p, s);
			if is_file(&raw_path)? {
				return full_path(&raw_path)
			}
		}
	}

	Ok(String::new())
}

fn path_to_string(path: PathBuf) -> RhaiResult<String> {
	path.into_os_string()
		.into_string()
		.map_err(|_| err!("invalid utf8"))
}

fn full_path(s: &str) -> RhaiResult<String> {
	dunce::canonicalize(s)
		.map_err(io_err)
		.and_then(path_to_string)
}

pub(crate) fn write_file_str(path: &str, s: &str) -> RhaiResult<()> {
	fs::write(path, s)
		.map_err(|e| err!("could not write to {} error {:?}", path, e))
}

pub(crate) fn read_file(path: &str) -> RhaiResult<String> {
	fs::read_to_string(path)
		.map_err(|e| err!("could not read file {} error {:?}", path, e))
}

fn read_dir(path: &str) -> RhaiResult<Array> {
	fs::read_dir(path)
		.map_err(io_err)?
		.map(|r| {
			r.map_err(io_err)
				.and_then(|e| {
					e.file_name().into_string()
						.map_err(|_| err!("invalid utf8"))
				})
				.map(ImmutableString::from)
				.map(Into::into)
		})
		.collect()
}

// converts an array of strings to a string with lines
fn lines(arr: Array) -> RhaiResult<String> {
	let mut s = String::new();
	for el in arr {
		s.push_str(
			el.into_immutable_string()?
				.as_str()
		);
		s.push('\n');
	}

	Ok(s)
}

fn write_file_arr(path: &str, arr: Array) -> RhaiResult<()> {
	let s = lines(arr)?;
	write_file_str(path, &s)
}

fn append_str(path: &str, s: &str) -> RhaiResult<()> {
	OpenOptions::new()
		.write(true)
		.create(true)
		.append(true)
		.open(path)
		.map_err(io_err)?
		.write_all(s.as_bytes())
		.map_err(io_err)
}

fn append_arr(path: &str, arr: Array) -> RhaiResult<()> {
	let s = lines(arr)?;
	append_str(path, &s)
}

// deletes a file or a folder
fn delete(path: &str) -> RhaiResult<()> {
	let p = Path::new(path);
	if p.is_dir() {
		paint_act!("delete directory {:?}", path);
		fs::remove_dir_all(p)
			.map_err(io_err)
	} else if p.is_file() {
		paint_act!("delete file {:?}", path);
		fs::remove_file(p)
			.map_err(io_err)
	} else {
		Ok(())
	}
}

// deletes a file or a folder
fn rename(from: &str, to: &str) -> RhaiResult<()> {
	paint_act!("move {:?} to {:?}", from, to);
	fs::rename(from, to)
		.map_err(io_err)
}

// for the moment let's use cp command
#[cfg(target_family = "unix")]
fn copy(from: &str, to: &str) -> RhaiResult<()> {
	paint_act!("copy {:?} to {:?}", from, to);

	let status = Command::new("cp")
		.args(&["-r", from, to])
		.status()
		.expect("failed to execute");

	if status.success() {
		Ok(())
	} else {
		Err(err!("execution {:?} failed", status))
	}
}

// https://stackoverflow.com/a/65192210
// probably should be replaced with
#[cfg(not(target_family = "unix"))]
fn copy(from: &str, to: &str) -> RhaiResult<()> {
	fs::copy(from, to)
		.map(|_| ())
		.map_err(io_err)
}

fn contains(path: &str, patt: &str) -> RhaiResult<bool> {
	let s = fs::read_to_string(path)
		.map_err(io_err)?;
	Ok(s.contains(patt))
}

pub fn add(engine: &mut Engine) {
	let mut fs_mod = Module::new();
	fs_mod.set_native_fn("is_dir", is_dir);
	fs_mod.set_native_fn("is_file", is_file);
	fs_mod.set_native_fn("is_prog", is_prog);
	fs_mod.set_native_fn("create_dir", create_dir);
	fs_mod.set_native_fn("write", write_file_str);
	fs_mod.set_native_fn("write", write_file_arr);
	fs_mod.set_native_fn("read", read_file);
	fs_mod.set_native_fn("delete", delete);
	fs_mod.set_native_fn("move", rename);
	fs_mod.set_native_fn("copy", copy);
	fs_mod.set_native_fn("full_path", full_path);
	fs_mod.set_native_fn("append", append_str);
	fs_mod.set_native_fn("append", append_arr);
	fs_mod.set_native_fn("contains", contains);
	fs_mod.set_native_fn("read_dir", read_dir);
	fs_mod.set_native_fn("which", which);
	engine
		.register_static_module("fs", fs_mod.into());
}