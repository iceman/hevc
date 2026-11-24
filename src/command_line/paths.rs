use std::error::Error;
use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};

use super::Args;

impl Args {
	pub fn paths(&mut self) -> Result<Vec<PathBuf>, Box<dyn Error>> {
		let path_strings = std::mem::take(&mut self.path_strings);
		
		#[cfg(not(target_os = "macos"))]
		self.convert_to_paths(path_strings)?;
		
		#[cfg(target_os = "macos")]
		{
			if std::env::var("TERM_PROGRAM").is_ok_and(|term| term == "ghostty") || cfg!(test) {
				self.convert_to_paths_split_if_needed(path_strings)?;
			} else {
				self.convert_to_paths(path_strings)?;
			}
		}
		
		Ok(std::mem::take(&mut self.paths))
	}
	
	fn convert_to_paths(&mut self, path_strings: Vec<String>) -> Result<(), Box<dyn Error>> {		
		for ps in path_strings {
			self.add_path(PathBuf::from(ps))?;
		}
		Ok(())
	}
	
	#[cfg(target_os = "macos")]
	fn convert_to_paths_split_if_needed(&mut self, path_strings: Vec<String>) -> Result<(), Box<dyn Error>> {		
		for ps in path_strings {
			let paths = ps
				.split("/Volumes")
				.filter(|segment| !segment.is_empty())
				.map(|segment| PathBuf::from(format!("/Volumes{segment}")));
			
			for path in paths {
				self.add_path(path)?;
			}
		}
		Ok(())
	}
	
	fn add_path(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
		if path.is_valid_dir()? {
			if self.output_dir.replace(path).is_some() { // sets output_dir as Some(path) at most once
				return Err("Only one output directory is allowed.".into())
			}
		} else {
			self.paths.push(path);
		}
		Ok(())
	}
}

pub trait PathExt: AsRef<Path> {
	fn is_valid_dir(&self) -> Result<bool, Box<dyn Error>> {
		if cfg!(test) { return Ok(false); }
		match self.as_ref().metadata() {
			Ok(metadata) => Ok(metadata.is_dir()),
			Err(error) if error.kind() == NotFound => {
				Err(std::io::Error::new(NotFound, "Path does not exist").into())
			}
			Err(error) => Err(error.into()),
		}
	}
}

impl<T: AsRef<Path>> PathExt for T {}
