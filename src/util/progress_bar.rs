use std::io::{self, Write};

pub struct ProgressBar {
	header: String,
	current: u64,
	total: u64,
}

// Custom progress bar
impl ProgressBar {
	pub fn new(t: u64, h: &str) -> Self {
		Self {
			header: String::from(h),
			current: 0,
			total: t
		}
	}
	
	pub fn tick(&mut self, a: u64) -> Option<()> {
		if self.current == self.total {
			return Some(());
		}
		let size = 40;
		self.current += a;
		let fill = ((self.current * size) / self.total) as usize;
		let t = match fill > 0 {
			true => "=".repeat(fill),
			false => String::from("")
		};
		if fill == size as usize {
			println!("\r{} [\x1b[1;37m{}>\x1b[0m] 100.00%", self.header, t);
			return Some(());
		}
		let k = " ".repeat(size as usize - fill);
		print!("\r{} [\x1b[1;37m{}>{}\x1b[0m] {:.2}%", self.header, t, k, (self.current as f64 / self.total as f64) * 100f64);
		io::stdout().flush().expect("Unable to Flush to Stdout");
		None
	}

	pub fn set_tick(&mut self, a: u64) -> Option<()> {
		self.current = a;
		self.tick(0)	
	}

	pub fn complete(&mut self) -> Option<()> {
		self.current = self.total - 1;
		self.tick(1)
	}

	// Currently Unused
	pub fn _interrupt(&mut self) {
		print!("\r\x1b[K");
		io::stdout().flush().expect("Unable to Flush to Stdout");
	}

	// Currently Unused
	pub fn _resume(&mut self) {
		self.tick(0);
	}
}