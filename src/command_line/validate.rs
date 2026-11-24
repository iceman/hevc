use regex::Regex;

pub fn crf(s: &str) -> Result<String, String> {
	match s.parse::<u8>() {
		Ok(n) if n <= 51 => Ok(s.to_owned()),
		_ => Err("CRF must be between 0 and 51.".into()),
	}
}

pub fn bitrate(s: &str) -> Result<String, String> {
	if Regex::new(r"^\d+[kMG]$").unwrap().is_match(s) {
		Ok(s.to_owned())
	} else {
		Err("Bitrate must be a valid size (500k, 5M, etc).".into())
	}
}

pub fn framerate(s: &str) -> Result<u16, String> {
	s.parse::<u16>()
		.map_err(|_| "Timestamp must be a valid frame rate.".into())
}

pub fn even_u16(s: &str) -> Result<u16, String> {
	let value = s
		.parse::<u16>()
		.map_err(|_| "Value must be a valid number.".to_owned())?;

	if value.is_multiple_of(2) {
		Ok(value)
	} else {
		Err("Value must be an even number.".into())
	}
}
