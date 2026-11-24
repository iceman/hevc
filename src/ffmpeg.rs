mod buf_read_ext;
mod command;
mod deref;
mod parse;
mod print;
mod run;
mod timer;

use super::{Audio, Job, Context, Res, Video};
use timer::Timer;

#[derive(Default)]
pub struct Ffmpeg {
	audio: Audio,
	pub context: Context,
	elapsed_secs: u64,
	estimated_final_size: u64,
	job: Job,
	progress: f32,
	ratio: f32,
	remaining_hours: u64,
	remaining_minutes: u64,
	res: Res,
	size_report: String,
	speed: f32,
	src_audio: Option<String>,
	src_video: Option<String>,
	timer: Timer,
	total_duration: Option<u64>,
	video: Video,
}

impl Ffmpeg {
	pub fn new(job: Job, context: Context) -> Self {
		Self {
			job,
			context,
			..Default::default()
		}
		.set_audio_video()
		.set_vr_resolution()
	}
}
