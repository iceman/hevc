use crossterm::cursor::{self, MoveUp, MoveDown, MoveToColumn};
use crossterm::terminal::{Clear, ClearType::CurrentLine};
use crossterm::QueueableCommand;
use std::io::{self, stdin, Write};

use crate::utils::{ColoredString, Colorize, CustomColors, Readable};
use super::Ffmpeg;

macro_rules! write {
	($this:expr, $($arg:tt)*) => {
		std::write!($this.context.printer, $($arg)*)
	};
}

macro_rules! writeln {
	($this:expr, $($arg:tt)*) => {
		std::writeln!($this.context.printer, $($arg)*)
	};
}

impl Ffmpeg {
	pub(super) fn confirm_overwrite(&mut self) -> io::Result<bool> {		
		writeln!(self, "\nOutput file already exists: {}", self.job.output.name().display())?;
		write!(self, "Do you want to delete it? [y/N]: ")?;
		
		self.context.printer.show_cursor()?;
		self.context.printer.flush()?;
		
		let mut answer = String::new();
		stdin().read_line(&mut answer)?;
		
		self.context.printer.hide_cursor()?;
		
		Ok(answer.trim().to_lowercase().starts_with('y'))
	}
	
	pub(super) fn print_header(&mut self) -> io::Result<()> {
		let ts_mark = self
			.context
			.timestamps
			.as_ref()
			.map(|f| format!(" : TS({f}fps)"))
			.unwrap_or_default();
		writeln!(self, "\n{}{ts_mark}", self.job.input.name().display())?;
		
		if self.output.deleted {
			writeln!(self, "Existing file has been overwritten")?;
		}
		
		if self.context.vr {
			writeln!(self,
				"🥽 VR Mode - 📐: {} - 🔍: {} - ↕️: {} - 📺: {}",
				self.context.proj,
				self.context.zoom,
				self.context.pitch_down,
				self.res,
			)?;
		}
		self.context.printer.flush()
	}
	
	pub(super) fn print_duration(&mut self, hours: u64, minutes: u64) -> io::Result<()> {
		let settings = |[k, v]: [&str; 2]| format!(" {} {}", k, v);
		writeln!(self,
			"{} : {hours:02}:{minutes:02} : {}[{}{}{}] : {}[{}]",
			self.context.mode,
			self.src_video.as_deref().unwrap_or("?"),
			self.video,
			self.context.user_level.map(|_| {
					self.video.quality(self.context.user_level).map(settings).unwrap_or_default()
				})
				.unwrap_or_default(),
			self.context.preset.map(|p| {
					self.video.preset_flag(p).map(settings).unwrap_or_default()
				})
				.unwrap_or_default(),
			self.src_audio.as_deref().unwrap_or("?"),
			self.audio,
		)
	}
	
	pub(super) fn print_progress(&mut self) -> io::Result<()> {
		self.context.printer.queue(MoveToColumn(0))?
			.queue(Clear(CurrentLine))?;
		
		write!(self,
			"{} – ⚡: {:.2}x – 🕒: {:02}:{:02} – 💾: {}",
			self.progress_bar(),
			self.speed,
			self.remaining_hours,
			self.remaining_minutes,
			self.size_report,
		)
	}
	
	pub(super) fn print_final(&mut self) -> io::Result<()> {
		if self.context.delete != 0 {
			if Self::get_position()?.1 != 0 { // cursor not at top
				let jump = if self.output.deleted {3} else {2};
				let width = self.input.name_width();
				self.context.printer.queue(MoveUp(jump))?
				   .queue(MoveToColumn(width))?;
				write!(self, "{}", self.input_deleted_mark())?;
				self.context.printer.queue(MoveDown(jump))?;
			} else { // cursor at top row (terminal cleared by user)
				self.context.printer.queue(Clear(CurrentLine))?;
				writeln!(self, "{}", self.input_deleted_mark())?;
			}
		}
		self.context.printer.queue(Clear(CurrentLine))?
 		   .queue(MoveToColumn(0))?;
		writeln!(self, "Final Size: {}", self.size_report)?;
		let path = self.output.path.display().to_string();
		writeln!(self, "{} –> {}", self.timer.total_duration().readable(), path)
	}
	
	fn get_position() -> io::Result<(u16, u16)> {
		for _ in 0..3 {
			let result = cursor::position();
			if result.is_ok() {
				return result
			}
			std::thread::sleep(std::time::Duration::from_millis(50));
		}
		cursor::position()
	}
	
	fn input_deleted_mark(&self) -> ColoredString {
		if self.input.deleted {
			" [deleted]".soft_green()
		} else {
			" [kept]".gruvbox_red()
		}
	}
	
	/// Format Ratio and Savings
	/// Savings only displays if:
	/// 1. Final size not 0
	/// 2. Size diff not close (bit shift to bucket them by 2²⁴ or within ~16MB)
	/// 3. Final size less than input size (subtraction does not overflow)
	pub(super) fn set_size_report_cache(&mut self) {
		let savings = if self.estimated_final_size != 0
			&& (self.estimated_final_size >> 24) != (self.input.size >> 24)
			&& let Some(s) = self.input.size.checked_sub(self.estimated_final_size)
		{
			format!("-{} ", s.readable())
		} else {
			String::new()
		};
		
		self.size_report = format!("{}/{} {}",
			self.estimated_final_size.readable(),
			self.input.size.readable(),
			self.colorize(format!("{}({:.0}%)", savings, self.ratio * 100.0)),
		);
	}
	
	fn progress_bar(&self) -> ColoredString {
		const WIDTH: usize = 15;
		const ON: &str = "█";
		const OFF: &str = "░";
		
		if self.progress > 1.0 {
			return format!("{} ERR", ON.repeat(WIDTH)).bright_red()
		}
		
		let filled_width = (self.progress * WIDTH as f32).round() as usize;
		let empty_width = WIDTH - filled_width;
		
		let filled = ON.repeat(filled_width);
		let empty = OFF.repeat(empty_width);
		let progress_percent = self.progress.min(0.99) * 100.0;
		let bar = format!("{filled}{empty} {progress_percent:2.0}%");
		self.colorize(bar)
	}
	
	pub(super) fn ratio_limit(&self, limit: f32) -> bool {
		self.ratio <= if self.video.is_copy() { 1.05 } else { limit }
	}
	
	fn colorize(&self, output: String) -> ColoredString {
		match self.ratio {
			0.0 => output.normal(), // force initial normal
			_ if self.ratio_limit(0.80) => output.soft_green(),
			r if r > self.context.cutoff => output.gruvbox_red(),
			_ => output.normal(),
		}
	}
}
