
#![allow(dead_code)]

// this is mostly a copy of std
// but transpose is added
// if possible transpose should solved in a different way


use std::cell::{Cell, UnsafeCell};
use std::ops::{Deref, DerefMut};

// Mostly a copy from std
pub struct RefCell<T: ?Sized> {
	borrow: Cell<BorrowFlag>,
	value: UnsafeCell<T>
}

// impl fmt::Debug for Borr

// Positive values represent the number of `Ref` active. Negative values
// represent the number of `RefMut` active. Multiple `RefMut`s can only be
// active at a time if they refer to distinct, nonoverlapping components of a
// `RefCell` (e.g., different ranges of a slice).
#[derive(Debug, Clone, Copy)]
struct BorrowFlag(isize);

impl BorrowFlag {

	const fn unused() -> Self {
		Self(0)
	}

	const fn is_writing(&self) -> bool {
		self.0 < 0
	}

	const fn is_reading(&self) -> bool {
		self.0 > 0
	}

	/// Returns true if the flag could be updated
	#[must_use]
	fn add_ref(&mut self) -> bool {
		if self.is_writing() {
			false
		} else {
			match self.0.checked_add(1) {
				Some(n) => {
					self.0 = n;
					true
				},
				None => false
			}
		}
	}

	fn rm_ref(&mut self) {
		self.0 -= 1;
		assert!(self.0 >= 0);
	}

	/// at the moment this cannot be added
	/// but only set
	#[must_use]
	fn add_ref_mut(&mut self) -> bool {
		if self.0 != 0 {
			false
		} else {
			self.0 -= 1;
			true
		}
	}

	fn rm_ref_mut(&mut self) {
		self.0 += 1;
		assert_eq!(self.0, 0);
	}

}

impl<T> RefCell<T> {
	pub const fn new(v: T) -> Self {
		Self {
			value: UnsafeCell::new(v),
			borrow: Cell::new(BorrowFlag::unused())
		}
	}
}

impl<T: ?Sized> RefCell<T> {

	/// ## Panics
	/// If the value is already mutably borrowed.
	pub fn increment_ref(&self) {
		let mut flag = self.borrow.get();
		if !flag.add_ref() {
			panic!("value borrowed mutably");
		}
		self.borrow.set(flag);
	}

	/// ## Panics
	/// If the values is already mutably or immutably
	/// borrowed.
	pub fn increment_ref_mut(&self) {
		let mut flag = self.borrow.get();
		if !flag.add_ref_mut() {
			panic!("value borrowed mutably");
		}
		self.borrow.set(flag);
	}

	// /// ## Safety
	// /// You are only allowed to call this function
	// /// once after you called increment_ref.
	// /// Any reference you received for example from
	// /// leak_ref must be dropped before this call.
	// pub unsafe fn decrement_ref(&self) {
	// 	let mut flag = self.borrow.get();
	// 	flag.rm_ref();
	// 	self.borrow.set(flag);
	// }

	/// ## Panics
	/// If the value is already mutably borrowed.
	pub fn leak_ref<'a>(&'a self) -> &'a T {
		self.increment_ref();

		// safe because increment_ref checked
		// that there aren't any mutable reference
		// active
		unsafe { &*self.value.get() }
	}

	pub fn borrow(&self) -> Ref<'_, &T> {
		self.increment_ref();

		Ref {
			// the borrow flag ensures that no
			// mutable access exists and will exist
			// while the reference is alive
			value: unsafe { &*self.value.get() },
			borrow: BorrowRef { inner: &self.borrow, mutably: false }
		}
	}

	pub fn borrow_mut(&self) -> Ref<'_, &mut T> {
		self.increment_ref_mut();

		Ref {
			value: unsafe { &mut *self.value.get() },
			borrow: BorrowRef { inner: &self.borrow, mutably: true }
		}
	}

}

unsafe impl<T: ?Sized> Send for RefCell<T> where T: Send {}

impl<T> From<T> for RefCell<T> {
	fn from(value: T) -> Self {
		Self::new(value)
	}
}

struct BorrowRef<'a> {
	inner: &'a Cell<BorrowFlag>,
	mutably: bool
}

impl Drop for BorrowRef<'_> {
	fn drop(&mut self) {
		let mut flag = self.inner.get();
		if !self.mutably {
			flag.rm_ref();
		} else {
			flag.rm_ref_mut();
		}
		self.inner.set(flag);
	}
}

pub struct Ref<'a, T: 'a> {
	value: T,
	borrow: BorrowRef<'a>
}

impl<'a, T> Ref<'a, T> {
	pub unsafe fn map<F, U>(me: Self, f: F) -> Ref<'a, U>
	where F: FnOnce(T) -> U {
		Ref {
			value: f(me.value),
			borrow: me.borrow
		}
	}
}

impl<'a, R, E> Ref<'a, Result<R, E>> {
	pub fn transpose(me: Self) -> Result<Ref<'a, R>, E> {
		match me.value {
			Ok(value) => Ok(Ref {
				value,
				borrow: me.borrow
			}),
			Err(e) => Err(e)
		}
	}
}

impl<T> Deref for Ref<'_, T> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T> DerefMut for Ref<'_, T> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}