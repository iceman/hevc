mod paths;
mod validate;

use clap::{Parser, value_parser};
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;

use super::{Audio, Job, Context, Proj, SpecialFilter, Video};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[arg(required = true, num_args = 1.., value_name = "PATHS including optional output directory")]
	path_strings: Vec<String>,
	
	#[arg(short, long, value_enum, default_value_t = Audio::Auto)]
	pub audio: Audio,
	
	#[arg(short, long, value_enum, default_value_t = Video::Auto)]
	pub video: Video,
	
	#[arg(short, long = "default", value_enum, default_value_t = Video::HEVC)]
	pub default_video: Video,
		
	/// Force Overwrite
	#[arg(short, long = "force")]
	pub force_overwrite: bool,
	
	/// Delete Input (x = auto by output file size cutoff, xx = force delete)
	#[arg(short = 'x', long = "delete", action = clap::ArgAction::Count)]
	pub delete: u8,
	
	/// Cutoff for color indication and delete
	#[arg(long, default_value = "90")]
	pub cutoff: u8,
	
	/// -preset [medium], av1: -cpu-used (1-13 [4]), media-engine: -quality (quality/speed [quality])
	#[arg(short, long)]
	pub preset: Option<String>,
	
	#[arg(short = 'r', long)]
	pub filter: Option<String>,
	
	#[arg(short, long, conflicts_with = "filter", value_enum, default_value_t = SpecialFilter::None, value_name = "SPECIAL_FILTER")]
	pub special: SpecialFilter,
	
	/// [default: HEVC-21, AV1-30, SVTAV1-32)] Constant Rate Factor (0–51, better-smaller)
	#[arg(short, long, conflicts_with = "bitrate", value_parser = validate::crf)]
	pub crf: Option<String>,
	
	/// [default: 5M] Target bitrate for media-engine (examples: 500k, 5M)
	#[arg(short, long, conflicts_with = "crf", value_parser = validate::bitrate)]
	#[cfg_attr(not(target_os = "macos"), arg(hide = true))]
	pub bitrate: Option<String>,
	
	#[arg(short = 't', long, default_value = "mkv")]
	pub filetype: OsString,
	
	/// Regenerate Timestamps
	#[arg(short = 'm', long, value_name = "FRAMERATE", value_parser = validate::framerate)]
	pub timestamps: Option<String>,

	/// Convert to 2D view (options below)
	#[arg(long, conflicts_with_all = ["filter", "special"])]
	pub vr: bool,
	
	/// VR Width (auto if not provided, squared if only width provided)
	#[arg(short = 'w', long, requires = "vr", value_parser = validate::even_u16)]
	pub width: Option<u16>,
	
	/// VR Height
	#[arg(long, requires = "vr", value_parser = validate::even_u16)]
	pub height: Option<u16>,
	
	#[arg(short = 'j', long, value_enum, default_value_t = Proj::Hequirect, requires = "vr", value_name = "PROJECTION")]
	pub proj: Proj,
	
	/// d_fov field of view out of 360 (smaller is more zoomed in)
	#[arg(short, long, default_value = "125", value_parser = value_parser!(u16).range(1..=360), requires = "vr", value_name = "FIELD_OF_VIEW")]
	pub zoom: u16,
	
	#[arg(short = 'i', long = "pitch", default_value = "30", requires = "vr")]
	pub pitch_down: i8,
	
	/// Print raw ffmpeg output
	#[arg(long)]
	pub debug: bool,
	
	#[arg(skip)]
	pub output_dir: Option<PathBuf>,
	
	#[arg(skip)]
	pub paths: Vec<PathBuf>,
}

pub fn parse_jobs_and_context() -> Result<(Vec<Job>, Context), Box<dyn Error>> {
	Args::parse().jobs_and_context()
}

impl Args {
	fn jobs_and_context(mut self) -> Result<(Vec<Job>, Context), Box<dyn Error>> {
		self.pitch_down = -self.pitch_down;
		
		if self.vr && self.video.is_copy() {
			return Err("VR flag cannot be used with video copy.".into())
		}
				
		let jobs = self.paths()?
			.into_iter()
			.map(|path| Job::new(path, &self))
			.collect::<Vec<Job>>();
		
		if jobs.is_empty() {
			return Err("No files specified.".into())
		}
		
		Ok((jobs, Context::new(self)?))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn args_with_paths(paths: Vec<&str>) -> Args {
		Args {
			audio: Audio::Auto,
			bitrate: None,
			crf: None,
			cutoff: 90,
			debug: false,
			default_video: Video::HEVC,
			delete: 0,
			filetype: "mkv".into(),
			filter: None,
			force_overwrite: false,
			height: None,
			output_dir: None,
			paths: Vec::new(),
			path_strings: paths.into_iter().map(String::from).collect(),
			pitch_down: -30,
			preset: None,
			proj: Proj::Hequirect,
			special: SpecialFilter::None,
			timestamps: None,
			video: Video::Auto,
			vr: false,
			width: None,
			zoom: 130,
		}
	}
	
	#[test]
	fn test_invalid_crf() {
		assert!(validate::crf("99").is_err());
	}
	
	#[test]
	fn test_invalid_bitrate() {
		assert!(validate::bitrate("500").is_err());
	}
	
	#[test]
	fn test_no_input_files() {
		let args = args_with_paths(vec![]);
		let result = args.jobs_and_context();
		assert!(result.is_err());
	}
	
	#[test]
	#[cfg(target_os = "macos")]
	fn test_split_volumes() -> Result<(), Box<dyn Error>> {
		let mut args = args_with_paths(vec!["/Volumes/VideoA.mp4/Volumes/VideoB.mp4"]);
		let paths = args.paths()?;
		assert_eq!(
			paths,
			vec![
				PathBuf::from("/Volumes/VideoA.mp4"),
				PathBuf::from("/Volumes/VideoB.mp4"),
			]
		);
		Ok(())
	}
}
