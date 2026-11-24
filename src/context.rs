mod codecs;
mod printer;
mod special_filter;
mod total;
mod vr;

use super::command_line::Args;
pub use codecs::{Audio, Video};
use printer::Printer;
pub use special_filter::SpecialFilter;
use total::Total;
pub use vr::{Proj, Res};

#[derive(Default)]
pub struct Context {
	pub audio: Audio,
	pub cutoff: f32,
	pub debug: bool,
	pub default_video: Video,
	pub delete: u8,
	pub filter: Option<String>,
	pub force_overwrite: bool,
	pub mode: String,
	pub preset: Option<&'static str>,
	pub proj: Proj,
	pub pitch_down: i8,
	pub res: Res,
	pub encoding_required: bool,
	pub special: SpecialFilter,
	pub printer: Printer,
	pub timestamps: Option<String>,
	pub total: Total,
	pub user_level: Option<&'static str>,
	pub video: Video,
	pub vr: bool,
	pub zoom: u16,
}

impl Context {
	pub fn new(args: Args) -> std::io::Result<Self> {
		let Args {
			audio,
			bitrate,
			cutoff,
			crf,
			debug,
			default_video,
			delete,
			encoding_required,
			filter,
			force_overwrite,
			height,
			pitch_down,
			preset,
			proj,
			special,
			timestamps,
			width,
			video,
			vr,
			zoom,
			..
		} = args;
		
		Ok(Self {
			audio,
			cutoff: cutoff as f32 / 100.0,
			debug,
			default_video,
			delete,
			encoding_required,
			filter,
			force_overwrite,
			mode: codecs::mode(&video, &audio),
			preset: video.resolve_default_preset(&default_video, preset),
			proj,
			pitch_down,
			res: Res { width, height },
			special,
			printer: Printer::new()?,
			timestamps,
			total: Total::default(),
			user_level: crf.or(bitrate).map(|s| Box::leak(s.into_boxed_str()) as &str),
			video,
			vr,
			zoom,
		})
	}
}
