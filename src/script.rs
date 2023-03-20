
use std::io;
use std::path::Path;
use std::fs::read_to_string;

use rhai::{
	Engine, Array, Scope, AST, FuncArgs, EvalAltResult, ParseError,
	CallFnOptions
};
use rhai::packages::{StandardPackage, Package};

pub type Result<T> = std::result::Result<T, Error>;

pub type RhaiResult<T> = std::result::Result<T, Box<EvalAltResult>>;

#[derive(Debug)]
pub enum Error {
	Rhai(Box<EvalAltResult>),
	Parse(ParseError),
	Io(io::Error)
}

impl From<Box<EvalAltResult>> for Error {
	fn from(b: Box<EvalAltResult>) -> Self {
		Self::Rhai(b)
	}
}

impl From<ParseError> for Error {
	fn from(e: ParseError) -> Self {
		Self::Parse(e)
	}
}

impl From<io::Error> for Error {
	fn from(e: io::Error) -> Self {
		Self::Io(e)
	}
}

#[derive(Debug)]
pub struct Script {
	engine: Engine,
	scope: Scope<'static>,
	ast: AST
}

impl Script {

	pub fn new(p: impl AsRef<Path>) -> Result<Self> {
		let engine = new_engine();
		let mut scope = Scope::new();
		let ctn = read_to_string(p)?;
		let ast = engine.compile_with_scope(&scope, &ctn)?;
		engine.eval_ast_with_scope(&mut scope, &ast)?;
		//ast.clear_statements();

		Ok(Self { engine, scope, ast })
	}

	fn call_fn(
		&mut self,
		name: &str,
		args: impl FuncArgs
	) -> Result<()> {
		let _ = self.engine.call_fn_with_options(
			CallFnOptions::new()
				.eval_ast(false)
				.rewind_scope(false),
			&mut self.scope,
			&self.ast,
			name,
			args
		)?;

		Ok(())
	}

	pub fn execute(
		&mut self,
		cmd: &str,
		args: Vec<String>
	) -> Result<()> {
		self.call_fn(cmd, args)
	}

}


fn new_engine() -> Engine {
	let mut engine = Engine::new_raw();

	engine.register_global_module(StandardPackage::new().as_shared_module());
	engine.set_fail_on_invalid_map_property(true);

	engine
		.on_debug(|s, src, pos|
			paint_dbg!("{} @ {:?} > {}", src.unwrap_or("unkown"), pos, s)
		)
		.on_print(move |s| {
			println!("{}", s);
		})
		.register_fn("print", print_bool)
		.register_fn("print", print_arr)
		.register_fn("prompt", prompt)
		.register_fn("panic", panic);

	crate::api::cmd::add(&mut engine);
	crate::api::git::add(&mut engine);
	crate::api::fs::add(&mut engine);
	crate::api::regex::add(&mut engine);
	crate::api::other::add(&mut engine);
	crate::api::toml::add(&mut engine);

	engine
}

fn print_arr(arr: Array) -> RhaiResult<()> {
	for el in arr {
		let s = el.into_immutable_string()?;
		println!("{}", s.as_str());
	}
	Ok(())
}

fn print_bool(b: bool) {
	if b {
		println!("true");
	} else {
		println!("false");
	}
}

fn prompt(s: &str) -> bool {
	paint_act!("{}", s);
	println!("y/n");
	let mut input = String::new();
	io::stdin().read_line(&mut input).expect("could not read line");
	let s = input.trim();

	matches!(s, "y" | "Y")
}

fn panic(s: &str) {
	panic!("{}", s);
}