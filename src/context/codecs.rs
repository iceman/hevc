use clap::ValueEnum;

/// Common method to resolve local run mode from global mode and src_codec
macro_rules! resolve_constructor {
	() => {
		pub fn resolve(&self, default: &Self, src_codec: &Option<String>) -> Self {
			match self {
				Self::Auto => {
					if let Some(c) = src_codec && Self::CODECS_TO_COPY.contains(&c.as_str()) {
						Self::Copy
					} else {
						*default
					}
				}
				_ => *self
			}
		}
	};
}

/// Global Mode Name
pub(super) fn mode(video: &Video, audio: &Audio) -> String {
	match (video, audio) {
		(Video::Auto, Audio::Auto) => "Automatic".into(),
		(Video::Copy, Audio::Copy) => "Copy".into(),
		_ => format!("{video:?} {audio:?}")
	}
}

/// Video Conversion Modes
/// Two Uses:
///     Global Context mode (includes Auto)
///     Individual file run mode
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Default, Copy, Clone, ValueEnum)]
pub enum Video {
	#[value(hide = true)]
	Auto,
	Copy,
	#[default]
	HEVC,
	#[cfg(target_os="macos")]
	MediaEngine,
	AV1,
	SVTAV1,
	VP9,
	AVC,
}

impl Video {
	const CODECS_TO_COPY: [&str; 2] = ["hevc", "av1"];
	const AUTO_ERROR: &str = "context.video used to call func limited to ffmpeg.video";
	
	resolve_constructor!();
	
	pub fn is_copy(&self) -> bool {
		matches!(self, Self::Copy)
	}
	
	pub fn codec(&self) -> &str {
		match self {
			Self::HEVC 		  => "libx265",
			#[cfg(target_os="macos")]
			Self::MediaEngine => "hevc_videotoolbox",
			Self::AV1		  => "libaom-av1",
			Self::SVTAV1	  => "libsvtav1",
			Self::VP9		  => "libvpx-vp9",
			Self::AVC		  => "libx264",
			Self::Copy 		  => "copy",
			Self::Auto 		  => unreachable!("{}", Self::AUTO_ERROR),
		}
	}
	
	pub fn quality(&self, user_level: Option<&'static str>) -> Option<[&str; 2]> {
		match self {
			Self::HEVC|Self::AVC   			 => Some(["-crf", user_level.unwrap_or("21")]),
			#[cfg(target_os="macos")]
			Self::MediaEngine 	   			 => Some(["-b:v", user_level.unwrap_or("5M")]),
			Self::AV1|Self::SVTAV1|Self::VP9 => Some(["-crf", user_level.unwrap_or("30")]),
			Self::Copy        	   			 => None,
			Self::Auto   	  	   			 => unreachable!("{}", Self::AUTO_ERROR),
		}
	}
	
	pub fn preset_flag(&self, preset: &'static str) -> Option<[&str; 2]> {
		match self {
			Self::HEVC|Self::AVC|Self::SVTAV1 => Some(["-preset", preset]),
			Self::AV1|Self::VP9            	  => Some(["-cpu-used", preset]),
			#[cfg(target_os="macos")]
			Self::MediaEngine    			  => Some(["-quality", preset]),
			Self::Copy           			  => None,
			Self::Auto			 			  => unreachable!("{}", Self::AUTO_ERROR),
		}
	}
	
	pub(super) fn resolve_default_preset(&self, default: &Self, preset: Option<String>) -> Option<&'static str> {
		if let Some(p) = preset {
			self.check_preset(default, &p);
			Some(Box::leak(p.into_boxed_str()))
		} else {
			match self {
				Self::AV1|Self::VP9 => Some("4"),
				#[cfg(target_os="macos")]
				Self::MediaEngine	=> Some("quality"),
				_					=> None,
			}
		}
	}
	
	fn check_preset(&self, default: &Self, preset: &str) {
		let check_codec = match self {
			Self::Auto => default,
			_ => self
		};
		
		let err_and_exit = |error: String| {
			eprintln!("{error}");
			std::process::exit(1);
		};
		
		let common_error = format!("Preset {preset} cannot be used with {self:?}.");
		match check_codec {
			Self::Auto => unreachable!("{}", Self::AUTO_ERROR),
			Self::Copy => {},
			_ if check_codec.preset_opts().contains(&preset) => {},
			_ if check_codec.preset_opts().is_empty() => err_and_exit(common_error),
			_ => err_and_exit(format!("{common_error} Use: {}", check_codec.preset_opts().join(","))),
		}
	}
	
	fn preset_opts(&self) -> &[&str] {
		match self {
			Self::HEVC|Self::AVC => &["placebo","veryslow","slower","slow","medium","fast","faster","veryfast","superfast","ultrafast"],
			Self::SVTAV1         => &["0","1","2","3","4","5","6","7","8","9","10","11","12","13"],
			Self::AV1            => &["0","1","2","3","4","5","6","7","8"],
			#[cfg(target_os="macos")]
			Self::MediaEngine    => &["quality","speed"],
			_					 => &[],
		}
	}
}

impl std::fmt::Display for Video {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Self::HEVC 			   => "hevc",
			#[cfg(target_os="macos")]
			Self::MediaEngine 	   => "hevc",
			Self::AV1|Self::SVTAV1 => "av1",
			Self::AVC 			   => "h264",
			_ => self.codec(),
		};
		f.write_str(s)
	}
}

/// Audio Conversion Modes
/// Two Uses:
///     Global Context mode (includes Auto)
///     Individual file run mode
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Default, Copy, Clone, ValueEnum)]
pub enum Audio {
	#[default]
	AAC,
	#[value(hide = true)]
	Auto,
	Copy,
}

impl Audio {
	const CODECS_TO_COPY: [&str; 1] = ["aac"];
	const COMMON_CODECS: [&str; 7] = ["aac", "mp3", "opus", "flac", "alac", "ac3", "wmav2"];
	const AUTO_ERROR: &str = "context.audio used to call func limited to ffmpeg.audio";
	
	resolve_constructor!();
	
	pub fn codec(&self) -> &str {
		match self {
			Self::AAC  => "aac",
			Self::Copy => "copy",
			Self::Auto => unreachable!("{}", Self::AUTO_ERROR),
		}
	}
	
	pub fn is_codec(codec: &str) -> bool {
		Self::COMMON_CODECS.contains(&codec)
	}
}

impl std::fmt::Display for Audio {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.codec())
	}
}
