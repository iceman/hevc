use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use unicode_width::UnicodeWidthStr;

use crate::command_line::Args;

#[derive(Default)]
pub struct File {
	pub path: PathBuf,
	pub size: u64,
	pub deleted: bool,
}

impl File {
	pub(super) fn new_output(path: &Path, args: &Args) -> Self {
		let mut path = path.to_path_buf();
		path.set_extension(&args.filetype);
		
		if let Some(output_dir) = &args.output_dir {
			path = output_dir.join(file_name(&path));
		}
		Self { path, ..Default::default() }
	}
	
	pub(super) fn new_unique(&self, args: &Args) -> Self {
		let stem = self.path.file_stem().expect("Invalid path").to_string_lossy();
		let ext = self.path.extension().and_then(|e| e.to_str()).unwrap_or("");
		for i in 1.. {
			let name = if ext.is_empty() {
				format!("{stem} ({i})")
			} else {
				format!("{stem} ({i}).{ext}")
			};
			let path = self.path.with_file_name(&name);
			if args.force_overwrite || !path.exists() {
				return Self { path, ..Default::default() }
			}
		}
		unreachable!()
	}
	
	pub fn updated_size(&mut self) -> u64 {
		self.size = size(&self.path);
		self.size
	}
	
	pub fn delete(&mut self) -> io::Result<()> {
		let result = fs::remove_file(&self.path);
		self.deleted = result.is_ok();
		result
	}
	
	pub fn name(&self) -> &OsStr {
		file_name(&self.path)
	}
	
	pub fn name_width(&self) -> u16 {
		UnicodeWidthStr::width(self.name().to_string_lossy().as_ref()) as u16
	}
}

pub(super) fn size(path: &Path) -> u64 {
	fs::metadata(path)
		.expect("Called before file exists?")
		.len()
}

fn file_name(path: &Path) -> &OsStr {
	path.file_name().expect("Invalid filename")
}
