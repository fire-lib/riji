#![allow(unused_macros)]

pub use ansi_term::Colour::{self, Red, Green, White, Blue, Purple, Cyan, Yellow};
pub use ansi_term::Style;

// make windows print colors
#[cfg(windows)]
#[ctor::ctor]
fn init() {
	let _ = output_vt100::try_init();
}

#[macro_export]
macro_rules! paint {
	($color:expr, $fmt:literal $($args:tt)*) => (
		print!(concat!("{}", $fmt, "{}"), $color.prefix() $($args)*, $color.suffix())
	)
}

#[macro_export]
macro_rules! paintln {
	($color:expr, $fmt:literal $($args:tt)*) => (
		println!(concat!("{}", $fmt, "{}"), $color.prefix() $($args)*, $color.suffix())
	)
}

#[macro_export]
macro_rules! epaint {
	($color:expr, $fmt:literal $($args:tt)*) => (
		eprint!(concat!("{}", $fmt, "{}"), $color.prefix() $($args)*, $color.suffix())
	)
}

#[macro_export]
macro_rules! epaintln {
	($color:expr, $fmt:literal $($args:tt)*) => (
		eprintln!(concat!("{}", $fmt, "{}"), $color.prefix() $($args)*, $color.suffix())
	)
}

#[macro_export]
macro_rules! paint_err {
	($($args:tt)*) => (
		$crate::paintln!($crate::paint::Red, $($args)*)
	)
}

#[macro_export]
macro_rules! paint_ok {
	($($args:tt)*) => (
		$crate::paintln!($crate::paint::Green, $($args)*)
	)
}

#[macro_export]
macro_rules! paint_dbg {
	($($args:tt)*) => (
		$crate::paintln!($crate::paint::White.dimmed(), $($args)*)
	)
}

/// paint action
#[macro_export]
macro_rules! paint_act {
	($($args:tt)*) => (
		$crate::paintln!($crate::paint::Yellow, $($args)*)
	)
}