use regex::Regex;
use std::io;
use std::sync::LazyLock;

use super::Ffmpeg;

impl Ffmpeg {
	pub(super) fn parse(&mut self, line: &str) -> io::Result<bool> {
		if self.total_duration.is_none()
			&& let Some(total_duration) = total_duration(line)
		{
			self.set_total_duration(total_duration)?;
			self.timer.set_delay(total_duration);
		}
		
		if let Some(total_duration) = self.total_duration
			&& let Some(elapsed_secs) = elapsed_seconds(line)
			&& let Some(speed) = speed(line)
		{
			self.progress = elapsed_secs as f32 / total_duration as f32;
			let remaining_secs = (total_duration - elapsed_secs) as f32 / speed;
			let (hours, minutes) = hours_minutes_from_seconds(remaining_secs as u64);
			self.remaining_hours = hours;
			self.remaining_minutes = minutes;
			self.elapsed_secs = elapsed_secs;
			self.speed = speed;
			Ok(true)
		} else {
			Ok(false)
		}
	}
	
	#[cold]
	fn set_total_duration(&mut self, total_duration: u64) -> io::Result<()> {
		let (hours, minutes) = hours_minutes_from_seconds(total_duration);
		self.print_duration(hours, minutes)?;
		self.total_duration = Some(total_duration);
		Ok(())
	}
}

fn total_duration(line: &str) -> Option<u64> {
	static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap());
	parse_seconds(line, &RE)
}

fn elapsed_seconds(line: &str) -> Option<u64>  {
	static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"time=(\d{2}):(\d{2}):(\d{2})\.(\d{2,6})").unwrap());
	parse_seconds(line, &RE)
}

fn speed(line: &str) -> Option<f32> {
	static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"speed=([\d.]+)").unwrap());
	RE.captures(line)
		.and_then(|caps| caps.get(1))
		.map(|m| m.as_str().parse::<f32>().unwrap_or_default())
}

fn parse_seconds(line: &str, re: &Regex) -> Option<u64> {
	let caps = re.captures(line)?;
	let mut nums = [0; 3];
	for (i, n) in nums.iter_mut().enumerate() {
		*n = caps.get(i + 1)?.as_str().parse().ok()?;
	}
	Some(nums[0] * 3600 + nums[1] * 60 + nums[2]) // h,m,s,cs
}

fn hours_minutes_from_seconds(seconds: u64) -> (u64, u64) {
	let (hours, seconds) = (seconds / 3600, seconds % 3600);
	(hours, seconds / 60)
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn test_parse_line_sets_progress() {
		let mut ffmpeg = Ffmpeg::default();
	
		// Simulate the first line with duration info
		let duration_line = "Duration: 00:01:40.00, start: 0.000000, bitrate: 128 kb/s";
		assert!(!ffmpeg.total_duration.is_some());
		let set = ffmpeg.parse(duration_line).unwrap();
		assert!(!set); // This line only sets duration
		assert_eq!(ffmpeg.total_duration, Some(100)); // 1*60 + 40
	
		// Simulate a later line with progress info
		let progress_line = "frame=123 time=00:00:20.00 bitrate=300kbits/s speed=2.0x";
		let updated = ffmpeg.parse(progress_line).unwrap();
		assert!(updated);
	
		assert_eq!(ffmpeg.elapsed_secs, 20);
		assert_eq!(ffmpeg.speed, 2.0);
		assert_eq!(ffmpeg.progress, 0.2);
		assert_eq!(ffmpeg.remaining_hours, 0);
		assert_eq!(ffmpeg.remaining_minutes, 0); // 40 seconds left / 2.0 speed = 20s left
	}

	#[test]
	fn test_total_duration_parsing() {
		let line = "Duration: 00:03:45.20, start: 0.000000, bitrate: 192 kb/s";
		assert_eq!(total_duration(line), Some(225)); // 3*60 + 45
	}

	#[test]
	fn test_elapsed_seconds_parsing() {
		let line = "frame=1234 time=00:01:15.32 bitrate=1500.3kbits/s speed=1.25x";
		assert_eq!(elapsed_seconds(line), Some(75)); // 1*60 + 15
	}

	#[test]
	fn test_speed_parsing() {
		let line = "speed=2.50x";
		assert_eq!(speed(line), Some(2.5));
	}

	#[test]
	fn test_parse_seconds_duration() {
		let re = Regex::new(r"Duration: (\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap();
		let line = "Duration: 01:02:03.45";
		assert_eq!(parse_seconds(line, &re), Some(3723)); // 1*3600 + 2*60 + 3
	}

	#[test]
	fn test_hours_minutes_from_seconds() {
		assert_eq!(hours_minutes_from_seconds(3739), (1, 2)); // 1 hr, 2 min
	}
}
