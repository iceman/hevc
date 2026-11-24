//! Provides convenient access to `Job` members through `Ffmpeg` to reduce
//! clutter from repeated chained field and method access.

use std::ops::{Deref, DerefMut};

use super::{Ffmpeg, Job};

impl Deref for Ffmpeg {
	type Target = Job;
	
	fn deref(&self) -> &Self::Target {
		&self.job
	}
}

impl DerefMut for Ffmpeg {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.job
	}
}
