use clap::ValueEnum;

/// VR Projection Modes
#[derive(Default, Copy, Clone, ValueEnum)]
pub enum Proj {
	#[value(hide = true)]
	#[default]
	Hequirect,
	Equirect,
	Fisheye190,
	Dfisheye,
	Cylindrical,
}

impl Proj {
	pub fn ih_fov(&self) -> &str {
		match self {
			Self::Fisheye190 => ":ih_fov=190",
			_ => "",
		}
	}
}

impl std::fmt::Display for Proj {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Self::Hequirect   => "hequirect",
			Self::Equirect    => "equirect",
			Self::Fisheye190  => "fisheye",
			Self::Dfisheye    => "dfisheye",
			Self::Cylindrical => "cylindrical",
		};
		f.write_str(s)
	}
}

#[derive(Default, Clone, Copy)]
pub struct Res {
	pub width: Option<u16>,
	pub height: Option<u16>,
}

impl Res {
	/// Calculate the final pixel dimension after applying a given FOV
	/// to an equirectangular (360°) source. 
	/// 
	/// Returns the approximate zoomed width and height.
	pub fn resolve(orig_width: u16, orig_height: u16, d_fov: f32) -> Self {
		let to_even = |f: f32| {
			let n = f.round() as u16;
			if n.is_multiple_of(2) { n } else { n + 1 }
		};
		
		let width: f32;
		let height: f32;
		let ow = (orig_width / 2) as f32;
		let oh = orig_height as f32;
		
		if ow >= oh {
			width = ow * d_fov / 360.0;
			height = width * (oh / ow);
		} else {
			height = oh * d_fov / 360.0;
			width = height * (ow / oh);
		}
		Self {
			width: Some(to_even(width)),
			height: Some(to_even(height)),
		}
	}
	
	pub fn width(&self) -> u16 {
		self.width.or(self.height).unwrap_or(1000)
	}
	
	pub fn height(&self) -> u16 {
		self.height.or(self.width).unwrap_or(1000)
	}
	
	pub fn is_auto(&self) -> bool {
		self.width.is_none() && self.height.is_none()
	}
}

impl std::fmt::Display for Res {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = format!("{}x{}", self.width(), self.height());
		f.write_str(&s)
	}
}
