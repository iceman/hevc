use std::time::{Duration, Instant};

const INITIAL_DELAY: u64 = 15;
const SECOND_DELAY: u64 = 30;
const SHORT_DELAY: u64 = 60;
const LONG_DELAY: u64 = 4 * 60;
const CUTOFF: u64 = 600;

#[derive(Debug)]
pub(super) struct Timer {
	total: Instant,
	last: Instant,
	interval: Duration,
	delay: u64,
}

impl Default for Timer {
	fn default() -> Self {
		Self {
			total: Instant::now(),
			last: Instant::now(),
			interval: Duration::from_secs(INITIAL_DELAY),
			delay: LONG_DELAY,
		}
	}	
}

impl Timer {
	pub(super) fn set_delay(&mut self, total_duration: u64) {
		if total_duration < CUTOFF {
			self.delay = SHORT_DELAY;
		}
	}
	
	pub(super) fn total_duration(&self) -> Duration {
		self.total.elapsed()
	}
	
	pub(super) fn should_trigger_update(&mut self) -> bool {
		if self.last.elapsed() >= self.interval {
			self.last = Instant::now();
			self.set_interval();
			true
		} else {
			false
		}
	}
	
	fn set_interval(&mut self) {
		let new_secs = match self.interval.as_secs() {
			INITIAL_DELAY => SECOND_DELAY,
			SECOND_DELAY => SHORT_DELAY,
			_ => self.delay,
		};
		self.interval = Duration::from_secs(new_secs);
	}
}
