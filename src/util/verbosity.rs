pub static mut VERBOSE: bool = false;
pub static mut PROGRESS: bool = false;
pub static mut SKIP: bool = false;
pub static mut SILENT: bool = false;

#[macro_export]
macro_rules! verbose {
	($body: block) => {
		unsafe {
			if util::verbosity::VERBOSE {
				$body
			}
		}
	};
}

#[macro_export]
macro_rules! progress {
	($body: block) => {
		unsafe {
			if util::verbosity::PROGRESS {
				$body
			}
		}
	};
}

#[macro_export]
macro_rules! skip {
	($body: block) => {
		unsafe {
			if !util::verbosity::SKIP {
				$body
			}
		}
	};
}

#[macro_export]
macro_rules! silent {
	($body: block) => {
		unsafe {
			if !util::verbosity::SILENT {
				$body
			}
		}
	};
}