use crossterm::{cursor, ExecutableCommand};
use std::io::{self, stdout, Stdout, Write};

#[derive(Debug)]
pub struct Printer {
	stream: Option<Stdout>,
}

#[allow(clippy::derivable_impls)]
impl Default for Printer {
	fn default() -> Self {
		Self {
			#[cfg(not(test))] stream: None,
			#[cfg(test)] 	  stream: Some(stdout()),
		}
	}
}

impl Printer {
	// Should only be called once because it installs the ctrl-C handler.
	// Outer struct `..Default::default()` calls create multiple temp structs
	// during the runtime prelude. A non-functional default() is provided
	// for these throwaway structs and a minimal one for tests.
	// `new()` is the functional constructor.
	pub(super) fn new() -> io::Result<Self> {
		ensure_cursor_restored_on_interrupt();
		let mut printer = Self { stream: Some(stdout()) };
		printer.hide_cursor()?;
		Ok(printer)
	}
	
	pub fn show_cursor(&mut self) -> io::Result<()> {
		if let Some(stream) = self.stream.as_mut() {
			stream.execute(cursor::Show)?;
		}
		Ok(())
	}
	
	pub fn hide_cursor(&mut self) -> io::Result<&mut Stdout> {
		self.stream.as_mut().unwrap().execute(cursor::Hide)
	}
}

impl Write for Printer {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.stream.as_mut().unwrap().write(buf)
	}
	
	fn flush(&mut self) -> io::Result<()> {
		self.stream.as_mut().unwrap().flush()
	}
}

impl Drop for Printer {
	fn drop(&mut self) {
		let _ = self.show_cursor();
	}
}

fn ensure_cursor_restored_on_interrupt() {
	ctrlc::set_handler(|| {
		let _ = stdout().execute(cursor::Show);
		std::process::exit(0);
	})
	.expect("Error setting Ctrl-C handler");
}
