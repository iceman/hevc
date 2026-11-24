use clap::ValueEnum;

/// VR Projection Modes
#[derive(Default, Copy, Clone, ValueEnum)]
pub enum SpecialFilter {
	#[default]
	None,
	Deinterlace,
	Denoise,
	DenoiseLight,
	DenoiseLuma,
}

impl SpecialFilter {
	pub fn filter(&self) -> Option<&str> {
		match self {
			Self::None 		  	=> None,
			Self::Deinterlace	=> Some("bwdif=mode=0"),
			Self::Denoise 		=> Some("hqdn3d=1.5:1.5:3:3"),
			Self::DenoiseLight	=> Some("hqdn3d=1:1:2:2"),
			Self::DenoiseLuma	=> Some("hqdn3d=1.5:0:3:0"),
		}
	}
}
