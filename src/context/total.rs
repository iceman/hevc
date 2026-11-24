use std::io::{self, Write};
use std::time::Duration;

use crate::{CustomColors, Readable};
use super::Context;

#[derive(Default)]
pub struct Total {
	pub duration: Duration,
	pub savings: u64,
}

impl Context {
	pub fn print_final_totals(mut self) -> io::Result<()> {
		let duration = self.total.duration.readable().soft_green();
		let savings = self.total.savings.readable().soft_green();
		writeln!(self.printer, "\n{duration} {savings}")
	}
}
