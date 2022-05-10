
use super::{RhaiResult, io_err};

use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use rhai::{Array, Engine, Module};


#[derive(Debug, Clone)]
struct Cmd {
	args: Vec<String>,
	envs: Vec<(String, String)>,
	env_clear: bool,
	dir: Option<String>
}

impl Cmd {
	pub fn new(s: &str) -> Self {
		Self {
			args: vec![s.into()],
			envs: vec![],
			env_clear: false,
			dir: None
		}
	}

	pub fn with_args(args_arr: Array) -> RhaiResult<Self> {
		let mut args = vec![];
		for arg in args_arr {
			args.push(arg.into_string()?);
		}

		Ok(Self {
			args,
			envs: vec![],
			env_clear: false,
			dir: None
		})
	}

	pub fn arg(&mut self, arg: &str) {
		self.args.push(arg.into());
	}

	pub fn args(&mut self, args: Array) -> RhaiResult<()> {
		for arg in args {
			self.args.push(arg.into_string()?);
		}
		Ok(())
	}

	pub fn dir(&mut self, s: &str) {
		self.dir = Some(s.into());
	}

	/// if val === 0 the env is removed
	pub fn env(&mut self, key: &str, val: &str) {
		self.envs.push((key.into(), val.into()));
	}

	pub fn env_clear(&mut self) {
		self.env_clear = true;
	}

	fn create_cmd(&mut self) -> RhaiResult<Command> {
		let mut cmd = Command::new(&self.args[0]);
		cmd.args(self.args.iter().skip(1));

		if let Some(dir) = &self.dir {
			let abs_dir = dunce::canonicalize(dir)
				.map_err(io_err)?;
			cmd.current_dir(abs_dir);
		}

		// add env
		if self.env_clear {
			cmd.env_clear();
		}

		for (key, val) in &self.envs {
			if val.is_empty() {
				cmd.env_remove(key);
			} else {
				cmd.env(key, val);
			}
		}

		Ok(cmd)
	}

	pub fn execute(&mut self) -> RhaiResult<()> {
		let cmd_str = self.args.join(" ");
		if let Some(dir) = &self.dir {
			paint_act!("executing {:?} in {:?}", cmd_str, dir);
		} else {
			paint_act!("executing {:?}", cmd_str);
		}

		let mut cmd = self.create_cmd()?;

		let status = cmd.status()
			.expect("failed to execute");

		if status.success() {
			paint_ok!("execution {:?} successful", cmd_str);
			Ok(())
		} else {
			paint_err!("execution {:?} failed", cmd_str);
			Err(err!("execution {:?} failed", self))
		}
	}

	pub fn output(&mut self) -> RhaiResult<String> {
		let cmd_str = self.args.join(" ");
		paint_act!("executing {:?}", cmd_str);

		let mut cmd = self.create_cmd()?;
		cmd.stdin(Stdio::inherit());
		cmd.stderr(Stdio::inherit());

		let output = cmd.output()
			.expect("failed to execute");

		let out_str = String::from_utf8(output.stdout)
			.expect("output is not valid utf8");

		if output.status.success() {
			paint_ok!("execution {:?} successful", cmd_str);
			Ok(out_str)
		} else {
			paint_err!("execution {:?} failed", cmd_str);
			Err(err!("execution {:?} failed", self))
		}
	}

	pub fn execute_parallel(cmds_input: Array) -> RhaiResult<()> {
		let mut cmds = Vec::with_capacity(cmds_input.len());

		for cmd in cmds_input {
			let cmd: Self = cmd.try_cast()
				.ok_or(err!("only command struct allowed"))?;
			cmds.push(cmd);
		}

		let (sender, recv) = mpsc::channel();

		let cmds_count = cmds.len();
		for mut cmd in cmds {
			let sender = sender.clone();
			// spawn a thread watcher
			thread::spawn(move || {
				// spawn command
				let handle = thread::spawn(move || {
					cmd.execute()
						.expect("cmd failed");
				});
				sender.send(handle.join()).expect("sending failed");
			});
		}

		for _ in 0..cmds_count {
			let r = recv.recv().expect("failed to receive message from threads");
			if let Err(e) = r {
				return Err(err!("Some command failed {:?}", e))
			}
		}

		Ok(())
	}
}

pub fn add(engine: &mut Engine) {
	let mut cmd_mod = Module::new();
	cmd_mod.set_native_fn("execute_parallel", Cmd::execute_parallel);

	engine
		.register_fn("cmd", Cmd::new)
		.register_result_fn("cmd", Cmd::with_args)
		.register_fn("arg", Cmd::arg)
		.register_result_fn("args", Cmd::args)
		.register_fn("dir", Cmd::dir)
		.register_fn("env", Cmd::env)
		.register_fn("env_clear", Cmd::env_clear)
		.register_result_fn("execute", Cmd::execute)
		.register_result_fn("output", Cmd::output)
		.register_static_module("cmd", cmd_mod.into());
}