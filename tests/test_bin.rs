use std::process::Command;

#[test]
fn test_help() {
	let output = Command::new("./target/debug/riji")
		.env("RIJI_SCRIPT", "tests/test_help.rhai")
		.arg("help")
		.output()
		.expect("Failed to execute command");

	assert!(output.status.success());

	let stdout = String::from_utf8(output.stdout).unwrap();
	let stderr = String::from_utf8(output.stderr).unwrap();

	assert_eq!(stdout, "test help\n- help\n- test1\n");
	assert_eq!(stderr, "");
}