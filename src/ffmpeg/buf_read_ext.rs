use std::io::{self, BufRead};

pub(super) trait BufReadExt: BufRead + Sized {
	fn lines(mut self) -> impl Iterator<Item = io::Result<String>> {
		let mut buf = Vec::with_capacity(256);
		
		std::iter::from_fn(move || {
			buf.clear();
			match self.read_until(b'\n', &mut buf) {
				Ok(0) => None, // End of file
				Ok(_) => {
					let line = String::from_utf8_lossy(&buf).into_owned();
					Some(Ok(line))
				}
				Err(e) => Some(Err(e)),
			}
		})
	}
	
	fn skip(mut self, count: usize) -> io::Result<Self> {
		for _ in 0..count {
			self.skip_until(b'\n')?;
		}
		Ok(self)
	}
}

impl<R: BufRead> BufReadExt for R {}
