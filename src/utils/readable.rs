use std::time::Duration;

pub trait Readable {
	fn readable(self) -> String;
}

impl Readable for u64 {
	fn readable(self) -> String {
		let size_mb = self / 1_000_000;
		if size_mb < 1000 {
			format!("{}MB", size_mb)
		} else {
			format!("{:.2}GB", self as f64 / 1_000_000_000.0)
		}
	}
}

impl Readable for Duration {
	fn readable(self) -> String {
		let secs = self.as_secs();
		
		let hours = secs / 3600;
		let minutes = (secs % 3600) / 60;
		
		match (hours, minutes) {
			(h, m) if h > 0 => format!("{h}h{m}m"),
			(_, m)          => format!("{m}m"),
		}
	}
}
