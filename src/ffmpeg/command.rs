use std::error::Error;
use std::process::{Child, Command, Stdio};

use super::{Audio, Ffmpeg, Res};

impl Ffmpeg {
	pub(super) fn command(&mut self) -> Result<Child, Box<dyn Error>> {
		let mut cmd = Command::new("ffmpeg");
		
		if let Some(framerate) = &self.context.timestamps {
			cmd.arg("-fflags").arg("+genpts").arg("-r").arg(framerate);
		}
		
		cmd.arg("-i").arg(&self.input.path);
		
		if self.context.vr {
			let filter = format!(
				"v360=input={}:output=rectilinear:in_stereo=sbs:out_stereo=2d{}:d_fov={}:w={}:h={}:pitch={}",
				self.context.proj, self.context.proj.ih_fov(), self.context.zoom, self.res.width(), self.res.height(), self.context.pitch_down
			);
			cmd.args(["-filter:v", &filter, "-map", "0", "-c", "copy"]);
		} else if let Some(filter) = self.context.special.filter().or(self.context.filter.as_deref()) {
			cmd.arg("-vf").arg(filter);
		}
		
		cmd.arg("-c:v").arg(self.video.codec());
		
		if let Some(preset) = self.context.preset
			&& let Some(flag) = self.video.preset_flag(preset) {
			cmd.args(flag);
		}
		
		if let Some(quality) = self.video.quality(self.context.user_level) {
			cmd.args(quality);
		}
		
		cmd.args(["-c:a", self.audio.codec()])
			.args(["-progress", "pipe:2"])
			.arg(&self.output.path)
			.stderr(Stdio::piped());
			
		let child = cmd.spawn()?;
		Ok(child)
	}
	
	pub(super) fn set_audio_video(mut self) -> Self {
		const ARGS: [&str;6] = ["-v", "error", "-show_entries", "stream=codec_name", "-of", "csv=p=0"];
		if let Some(lines) = self.ffprobe_command(ARGS) {
			let mut iter = lines.into_iter();
			self.src_video = iter.next().map(|mut s| { s.retain(|c| c != ','); s });
			self.src_audio = iter.next().map(|mut s| { s.retain(|c| c != ','); s });
			if let Some(src_video) = &self.src_video && Audio::is_codec(src_video) {
				std::mem::swap(&mut self.src_video, &mut self.src_audio);
			}
		}
		self.video = self.context.video.resolve(&self.context.default_video, &self.src_video);
		self.audio = self.context.audio.resolve(&Audio::default(), &self.src_audio);
		
		// Force re-encoding where required, because context auto vid can have copy set
		if self.context.encoding_required && self.video.is_copy() {
			self.video = self.context.default_video;
		}
		self
	}
	
	pub(super) fn set_vr_resolution(mut self) -> Self {
		const ARGS: [&str;8] = ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0"];
		if self.context.vr {
			if self.context.res.is_auto()
				&& let Some(lines) = self.ffprobe_command(ARGS)
				&& let Some((Ok(w), Ok(h))) = lines.first().and_then(|s| {
					s.trim()
					.split_once(',')
					.map(|(w, h)| (w.trim().parse::<u16>(), h.trim().parse::<u16>()))
				}) {
				self.res = Res::resolve(w, h, self.context.zoom.into())
			} else {
				self.res = self.context.res;
			}
		}
		self
	}
	
	fn ffprobe_command<I, S>(&self, args: I) -> Option<Vec<String>>
	where
		I: IntoIterator<Item = S>,
		S: AsRef<std::ffi::OsStr>,
	{
		let output = Command::new("ffprobe")
			.args(args)
			.arg(&self.input.path)
			.output()
			.ok()?;
		
		if !output.status.success() { return None }
		let stdout = String::from_utf8(output.stdout).ok()?;
		Some(stdout.lines().map(str::to_owned).collect())
	}
}
