use std::error::Error;

mod command_line;
mod ffmpeg;
mod job;
mod context;
mod utils;

use ffmpeg::Ffmpeg;
use job::Job;
use context::{Audio, Context, Proj, Res, SpecialFilter, Video};
use utils::{CustomColors, Readable};

// `Context` distills the command-line arguments and provides a common
// state for accumulator data. Ownership is passed in for each job run
// to avoid borrowing complications/inconveniences.
fn main() -> Result<(), Box<dyn Error>> {
	let (jobs, mut context) = command_line::parse_jobs_and_context()?;
	
	for job in jobs {
		let mut ffmpeg = Ffmpeg::new(job, context);
		
		if let Err(err) = ffmpeg.run() {
			eprintln!("Error: {err}");
		}
		
		context = ffmpeg.context;
	}

	context.print_final_totals()?;
	Ok(())
}
