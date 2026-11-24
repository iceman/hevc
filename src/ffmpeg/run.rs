use std::error::Error;
use std::io::{BufReader, Write};

use super::{buf_read_ext::BufReadExt, Ffmpeg};

impl Ffmpeg {
	pub fn run(&mut self) -> Result<(), Box<dyn Error>> {		
		if self.output.path.exists() {
			if !self.context.force_overwrite && !self.confirm_overwrite()? {
				return Err("Skipping file...".into());
			}
			self.output.delete()?;
		}
		
		let mut child = self.command()?;
		let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
		let reader = BufReader::new(stderr);
		let mut last_line = String::new();
		
		self.print_header()?;
		self.set_size_report_cache();
		
		for line in reader.skip(10)?.lines() {
			let line = line?;
			if self.parse(&line)? {
				if self.timer.should_trigger_update() {
					self.update_size_metrics();
					self.set_size_report_cache();
				}
				self.print_progress()?;
			}
			if self.context.debug {
				write!(self.context.printer, "{line}")?;
			}
			self.context.printer.flush()?;
			last_line = line;
		}
		
		if !child.wait()?.success() {
			writeln!(self.context.printer, "\n{last_line}")?;
			return Err("FFmpeg failed to complete successfully".into());
		}
		
		#[cfg(target_os="macos")]
		self.copy_macos_xattr_tags()?;
		
		self.progress = 1.0;
		self.update_size_metrics();
		
		if self.should_delete_input() {
			self.input.delete()?;
			self.normalize_output()?;
		}
		
		self.set_size_report_cache();
		self.print_final()?;
		self.report_to_accumulators();
		Ok(())
	}
	
	fn update_size_metrics(&mut self) {
		self.estimated_final_size = (self.output.updated_size() as f64 / self.progress as f64) as u64;
		self.ratio = self.estimated_final_size as f32 / self.input.size as f32;
	}
	
	fn report_to_accumulators(&mut self) {
		self.context.total.duration += self.timer.total_duration();
		self.context.total.savings += self.input.size.saturating_sub(self.estimated_final_size);
	}
	
	#[cfg(target_os="macos")]
	fn copy_macos_xattr_tags(&self) -> std::io::Result<()> {
		const FINDER_TAGS: &str = "com.apple.metadata:_kMDItemUserTags";
		if let Some(tag_data) = xattr::get(&self.input.path, FINDER_TAGS)? {
			xattr::set(&self.output.path, FINDER_TAGS, &tag_data)?;
			if let Some(output_tag_data) = xattr::get(&self.output.path, FINDER_TAGS)?
				&& tag_data != output_tag_data {
				eprintln!("Copying finder tags failed");
			}
		}
		Ok(())
	}
	
	fn should_delete_input(&self) -> bool {
		match self.context.delete {
			1 => self.ratio_limit(self.context.cutoff),
			2 => true,
			_ => false,
		}
	}
}
