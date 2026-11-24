mod file;

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::command_line::Args;
use file::File;

#[derive(Default)]
pub struct Job {
	pub input: File,
	pub output: File,
	pub orig_output_path: Option<PathBuf>,
}

impl Job {
	pub fn new(path: PathBuf, args: &Args) -> Self {
		let mut orig_output_path = None;
		let mut output = File::new_output(&path, args);
		if path == output.path {
			// input & output paths are the same
			// store the original path for normalize_output()
			(output, orig_output_path) = (output.new_unique(args), Some(output.path));
		}
		let size = file::size(&path);
		let input = File { path, size, ..Default::default() };
		Self { input, output, orig_output_path }
	}
	
	/// Rename output from tmp name to input path, replacing the input file
	pub fn normalize_output(&mut self) -> io::Result<()> {
		if let Some(orig_output_path) = self.orig_output_path.take()
			&& !self.input.path.exists()
		{
			fs::rename(&self.output.path, &orig_output_path)?;
			self.output.path = orig_output_path;
		}
		Ok(())
	}
}
