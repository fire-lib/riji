
use super::{RhaiResult, io_err, git_err};
use crate::paint::{Style, Green, Red, Cyan};
use crate::ref_cell::{RefCell, Ref};

use std::{ops, fs};
use std::rc::Rc;
// use std::cell::{RefCell, Ref};
use std::str::from_utf8;
use std::path::{Path, PathBuf};

use git2::{
	Repository, DiffFormat, DiffLine, ApplyLocation,
	IndexAddOption, build::CheckoutBuilder,
	Oid, ResetType
};
use rhai::{Engine};

struct Inner {
	repo: RefCell<Repository>,
	path: PathBuf
}

#[derive(Clone)]
pub struct Git {
	inner: Rc<Inner>
}

impl Git {

	pub fn new(path: &str) -> RhaiResult<Self> {
		Ok(Self::from_repo(
			path,
			Repository::discover(path)
				.map_err(git_err)?
		))
	}

	fn from_repo(path: &str, repo: Repository) -> Self {
		Self {
			inner: Rc::new(Inner {
				repo: repo.into(),
				path: path.into()
			})
		}
	}

	#[allow(dead_code)]
	fn root(&self) -> PathBuf {
		self.inner.path.clone()
	}

	/// Opens a repository cloning it
	/// if needed.
	pub fn clone(url: &str, path: &str) -> RhaiResult<Self> {
		paint_act!("git clone: {:?} into: {:?}", url, path);
		if Path::is_dir(path.as_ref()) {
			return Self::new(path)
		}

		Ok(Self::from_repo(
			path,
			Repository::clone(url, path)
				.map_err(git_err)?
		))
	}

	// pub fn print_diff(&mut self) -> RhaiResult<()> {
	// 	let repo = self.inner.borrow();
	// 	let diff = repo.diff_index_to_workdir(
	// 		None,
	// 		None
	// 	).map_err(git_err)?;
	// 	diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
	// 		let col = line_color(&line);
	// 		print!("{}", col.prefix());
	// 		match line.origin() {
	// 			'+' | '-' | ' ' => print!("{}", line.origin()),
	// 			_ => {}
	// 		}
	// 		print!("{}", from_utf8(line.content()).unwrap());
	// 		print!("{}", col.suffix());
	// 		true
	// 	}).map_err(git_err)?;

	// 	// for delta in diff.deltas() {
	// 	// 	println!("diff delta {:?}", delta);
	// 	// }
	// 	Ok(())
	// }

	pub fn diff(&mut self) -> Diff {
		Diff { inner: self.clone() }
	}

	fn apply_diff<D>(&mut self, diff: D) -> RhaiResult<()>
	where D: RawDiff {
		let diff = diff.raw_diff()?;
		self.inner.repo.borrow()
			.apply(&*diff, ApplyLocation::WorkDir, None)
			.map_err(git_err)?;
		Ok(())
	}

	// pub fn apply_files(&mut self, files: DiffFiles) -> RhaiResult<()> {
	// 	let root = self.root();
	// 	// let files = diff.files(root)?;
	// 	for (orig, new) in files.inner {
	// 		let new = root.join(new);
	// 		if let Some(orig) = orig {
	// 			// copy orig
	// 			fs::copy(orig, new)
	// 				.map_err(io_err)?;
	// 		} else {
	// 			// remove new
	// 			fs::remove(new)
	// 				.map_err(io_err)?;
	// 		}
	// 	}

	// 	Ok(())
	// }

	// pub fn stash(&mut self) -> RhaiResult<()> {
	// 	let sign = Signature::now("riji", "riji@riji.ch")
	// 		.map_err(git_err)?;
	// 	let mut repo = self.inner.repo.borrow_mut();
	// 	repo.stash_save(&sign, "riji tmp stash", None)
	// 		.map_err(git_err)?;
	// 	Ok(())
	// }

	// pub fn unstash(&mut self) -> RhaiResult<()> {
	// 	let mut latest = 0;
	// 	let mut repo = self.inner.repo.borrow_mut();
	// 	repo.stash_foreach(|id, _, _| {
	// 		latest = id;
	// 		true
	// 	}).map_err(git_err)?;
	// 	repo.stash_pop(latest, None)
	// 		.map_err(git_err)
	// }

	// pub fn index(&mut self) -> RhaiResult<GitIndex> {
	// 	let idx = self.inner.repo.borrow().index()
	// 		.map_err(git_err)?;

	// 	Ok(GitIndex {
	// 		inner: Rc::new(idx)
	// 	})
	// }

	// fn add_all(&self) -> RhaiResult<GitIndex> {
	// 	let &mut idx = self.inner.repo.borrow().index()
	// 		.map_err(git_err)?;
	// 	idx.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
	// 		.map_err(git_err)?;

	// 	Ok(GitIndex {
	// 		inner: Rc::new(idx)
	// 	})
	// }

	pub fn force_head(&mut self) -> RhaiResult<()> {
		let repo = self.inner.repo.borrow();
		let mut ops = CheckoutBuilder::new();
		ops.force()
			.remove_untracked(true);

		repo.checkout_head(Some(&mut ops))
			.map_err(git_err)
	}

	fn find_tag(&mut self, tag: &str) -> RhaiResult<Oid> {
		let tag = format!("refs/tags/{}", tag);
		let mut id = None;
		self.inner.repo.borrow()
			.tag_foreach(|tag_id, name| {
				let name = from_utf8(name).ok();
				if matches!(name, Some(n) if n == tag) {
					id = Some(tag_id);
					false
				} else {
					true
				}
			}).map_err(git_err)?;
		id.ok_or(err!("tag not found {}", tag))
	}

	pub fn checkout_tag(&mut self, tag: &str) -> RhaiResult<()> {
		paint_act!("checkout tag {:?}", tag);
		let tag_id = self.find_tag(tag)?;

		let repo = self.inner.repo.borrow();

		let obj = repo.find_object(tag_id, None)
			.map_err(git_err)?;

		let mut opts = CheckoutBuilder::new();
		opts.force()
			.remove_untracked(true);

		repo.reset(&obj, ResetType::Hard, Some(&mut opts))
			.map_err(git_err)
	}

}

#[derive(Clone)]
pub struct Diff {
	// we cannot store a diff
	// since that want's to hold a reference
	// to its repository
	// but rhai does not allow references
	inner: Git
}

impl Diff {

	fn compute<'a>(&'a self) -> RhaiResult<Ref<'a, git2::Diff<'a>>> {
		// Todo this is probably not the right way
		// but it works. (help me!)

		let repo = self.inner.inner.repo.borrow();
		Ref::transpose(unsafe {Ref::map(repo, |repo| {

			let mut index = repo.index()
				.map_err(git_err)?;

			// first we need to add all files to a new index
			index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
				.map_err(git_err)?;

			// get the head tree
			let head = repo.head().map_err(git_err)?;
			// let head = head.target().expect("head has not target");

			// let tree = repo.find_tree(head).map_err(git_err)?;

			let tree = head.peel_to_tree().map_err(git_err)?;

			// then compare 
			repo.diff_tree_to_index(
				// &old_index,
				Some(&tree),
				Some(&index),
				None
			).map_err(git_err)
		})})
	}

	// fn statuses<'a>(&'a self) -> RhaiResult<Ref<'a, git2::Statuses<'a>>> {
	// 	let mut opts = StatusOptions::new();
	// 	opts.show(StatusShow::IndexAndWorkdir);

	// 	let repo = self.inner.inner.repo.borrow();

	// 	Ref::transpose(
	// 		unsafe {
	// 			Ref::map(repo, |repo| {
	// 				repo.statuses(Some(&mut opts))
	// 					.map_err(git_err)
	// 			})
	// 		}
	// 	)
	// }

	fn print(&mut self) -> RhaiResult<()> {
		let diff = self.compute()?;
		diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
			let col = line_color(&line);
			print!("{}", col.prefix());
			match line.origin() {
				'+' | '-' | ' ' => print!("{}", line.origin()),
				_ => {}
			}
			print!("{}", from_utf8(line.content()).unwrap());
			print!("{}", col.suffix());
			true
		}).map_err(git_err)?;
		Ok(())
	}

	fn to_string(&mut self) -> RhaiResult<String> {
		let diff = self.compute()?;
		let mut s = String::new();
		diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
			match line.origin() {
				'+' | '-' | ' ' => s.push(line.origin()),
				_ => {}
			}
			s.push_str(from_utf8(line.content()).unwrap());
			true
		}).map_err(git_err)?;
		Ok(s)
	}

	fn to_file(&mut self, file: &str) -> RhaiResult<()> {
		let s = self.to_string()?;
		fs::write(file, s)
			.map_err(io_err)
	}

	// /// the first is the origin file
	// /// if it is empty you should remove the destination path
	// fn files(&mut self) -> RhaiResult<DiffFiles> {

	// 	let mut list = vec![];

	// 	let root = self.inner.root();
	// 	// let diff = self.compute()?;
	// 	let statuses = self.statuses()?;

	// 	for status in statuses.iter() {
	// 		let delta = status.index_to_workdir()
	// 			.expect("failed to call index to workdir");
	// 		let old = delta.old_file().path();
	// 		let new = delta.new_file().path();

	// 		match (old, new) {
	// 			(None, None) => continue,
	// 			// delete
	// 			(Some(path), None) => {
	// 				list.push((None, path.into()));
	// 			},
	// 			// create
	// 			(None, Some(path)) => {
	// 				list.push((Some(root.join(path)), path.into()));
	// 			},
	// 			// update
	// 			(Some(a), Some(b)) if a == b => {
	// 				list.push((Some(root.join(a)), a.into()));
	// 			},
	// 			// remove & create
	// 			(Some(a), Some(b)) => {
	// 				// remove
	// 				list.push((None, a.into()));
	// 				// create new
	// 				list.push((Some(root.join(b)), b.into()));
	// 			}
	// 		}
	// 	}

	// 	Ok(DiffFiles { inner: list })
	// }
}

impl RawDiff for Diff {
	fn raw_diff(&self) -> RhaiResult<MaybeRef<'_, git2::Diff<'_>>> {
		let diff = self.compute()?;
		Ok(MaybeRef::Has(diff))
	}
}

#[derive(Clone)]
pub struct DiffInFile {
	inner: Rc<git2::Diff<'static>>
}

impl DiffInFile {
	fn from_file(s: &str) -> RhaiResult<Self> {
		let b = fs::read_to_string(s)
			.map_err(io_err)?;
		let diff = git2::Diff::from_buffer(b.as_bytes())
			.map_err(git_err)?;

		Ok(Self {
			inner: Rc::new(diff)
		})
	}
}

impl RawDiff for DiffInFile {
	fn raw_diff(&self) -> RhaiResult<MaybeRef<'_, git2::Diff<'_>>> {
		Ok(MaybeRef::Borrowed(&*self.inner))
	}
}

trait RawDiff {
	fn raw_diff(&self) -> RhaiResult<MaybeRef<'_, git2::Diff<'_>>>;
}

enum MaybeRef<'a, T: 'a> {
	Has(Ref<'a, T>),
	// No(T),
	Borrowed(&'a T)
}

impl<'a, T> ops::Deref for MaybeRef<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		match self {
			Self::Has(r) => &*r,
			// Self::No(t) => &t,
			Self::Borrowed(t) => t
		}
	}
}

// #[derive(Debug, Clone)]
// pub struct DiffFiles {
// 	// the first path is absolute to the root repo
// 	// the second path is relative to the destination repo
// 	inner: Vec<(Option<PathBuf>, PathBuf)>
// }


// #[derive(Clone)]
// pub struct GitIndex {
// 	inner: Rc<Index>
// }

fn line_color(line: &DiffLine) -> Style {
	match line.origin() {
		'+' => Green.normal(),
		'-' => Red.normal(),
		// '>' => Green.normal(),
		// '<' => Red.normal(),
		'F' => Style::new().bold(),
		'H' => Cyan.normal(),
		_ => Style::new()
	}
}

pub fn add(engine: &mut Engine) {
	engine
		.register_result_fn("git", Git::new)
		.register_result_fn("git_clone", Git::clone)
		.register_fn("diff", Git::diff)
		.register_result_fn("apply_diff", Git::apply_diff::<DiffInFile>)
		.register_result_fn("apply_diff", Git::apply_diff::<Diff>)
		.register_result_fn("force_head", Git::force_head)
		.register_result_fn("checkout_tag", Git::checkout_tag)
		.register_result_fn("print", Diff::print)
		.register_result_fn("to_file", Diff::to_file)
		.register_result_fn("to_string", Diff::to_string)
		.register_result_fn("diff_from_file", DiffInFile::from_file);
}