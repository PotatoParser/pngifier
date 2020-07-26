use std::io::{self, Read, Write, BufWriter, SeekFrom};
use std::io::prelude::*;
use std::path::PathBuf;
use std::fmt;
use std::fs::{File, OpenOptions};

use crate::util;
use util::{Error, Chunk};

pub static PNG_HEADER: [u8; 8] = [
		0x89,
		0x50, 0x4e, 0x47,
		0x0d, 0x0a,
		0x1a,
		0x0a
	];

pub struct WriteFile {
	path: PathBuf,
	output: BufWriter<Box<dyn Write>>
}

impl Write for WriteFile {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.output.write(&buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.output.flush()
	}
}

impl WriteFile {
	pub fn stdout() -> io::Result<Self> {
		let out: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(std::io::stdout()));
		Ok(Self {
			path: PathBuf::from("stdout"),
			output: out
		})
	}

	pub fn from_string(s: String) -> io::Result<Self> {
		Self::from_pathbuf(PathBuf::from(s))
	}

	pub fn from_pathbuf(p: PathBuf) -> io::Result<Self> {
		let file: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(File::create(p.as_path())?));
		Ok(Self {
			output: file,
			path: p
		})
	}

	pub fn read(&self) -> io::Result<ReadFile> {
		ReadFile::from_pathbuf(self.path.clone())
	}

	pub fn trim(&mut self, buffer_size: usize) -> Result<(), Error> {
		let mut read_file = error_exp!(ReadFail, &self, self.read());
		error_exp!(TrimError, &self, read_file.trim(buffer_size));
		Ok(())
	}

	pub fn write_header(&mut self) -> io::Result<usize> {
		self.write(&PNG_HEADER)
	}

	pub fn write_chunk(&mut self, header_type: &[u8], data: &[u8]) -> io::Result<usize> {
		self.write(&(data.len() as u32).to_be_bytes()[..4])?;
		let merged = [header_type, data].concat();
		self.write(&merged)?;
		self.write(&util::Chunk::get_crc(&merged)[..])
	}
}

impl fmt::Debug for WriteFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", match self.path.to_str() {
			Some(s) => s,
			None => "Unknown"
		})
	}
}

impl fmt::Display for WriteFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

pub struct ReadFile {
	path: PathBuf,
	input: File,
	pub size: u64
}

impl ReadFile {
	// Currently Unused
	pub fn from_string(s: String) -> io::Result<Self> {
		Self::from_pathbuf(PathBuf::from(s))
	}

	pub fn from_pathbuf(p: PathBuf) -> io::Result<Self> {
		let file = File::open(p.as_path())?;
		Ok(Self {
			size: file.metadata()?.len(),
			input: file,
			path: p
		})
	}

	// Currently Unused
	pub fn write(&self) -> io::Result<WriteFile> {
		let path = self.path.clone();
		Ok(WriteFile{
			output: BufWriter::new(Box::new(OpenOptions::new().write(true).open(path.as_path())?)),
			path: path
		})
	}

	pub fn reset(&mut self) -> Result<u64, Error> {
		Ok(error_exp!(ReadFail, &self, self.input.seek(SeekFrom::Start(0))))
	}

	pub fn trim(&mut self, buffer_size: usize) -> io::Result<()> {
		let mut len = self.size;
		let mut buf = vec![0u8; buffer_size];
		loop {
			let mut start: u64 = 0;
			let mut cut = buf.len() as usize;
			if (buffer_size as u64) > len {
				cut = len as usize;
			} else {
				start = len - (buffer_size as u64);
			}
			self.seek(SeekFrom::Start(start))?;
			self.read(&mut buf[..cut])?;
			let mut i = cut - 1;
			while i > 0 {
				if buf[i] != 0 {
					let file = OpenOptions::new().write(true).open(self.path.as_path())?;
					return file.set_len(start + (i as u64) + 1);
				}
				i -= 1;
			}
			len -= cut as u64;
			if len == 0 {
				break;
			}
		}
		Ok(())
	}

	pub fn read_header(&mut self) -> io::Result<()> {
		let mut buffer = [0u8; 8];
		self.read(&mut buffer)?;
		if &buffer[..] == &PNG_HEADER[..] {
			return Ok(());
		}
		Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
	}

	pub fn read_chunk(&mut self) -> io::Result<Option<Chunk>> {
		Chunk::new(self)
	}

	pub fn verify_png(&mut self) -> Result<(), Error> {
		let mut progress_bar = util::ProgressBar::new(self.size, "Verifying PNG");
		error_exp!(InvalidHeader, &self, self.read_header());
		progress!({progress_bar.tick(8);});
		while let Some(k) = error_exp!(ReadChunk, &self, self.read_chunk()) {
			match k.verify_crc() {
				true => (),
				false => error!(InvalidCRC, &self)
			};
			progress!({progress_bar.tick(k.total_data as u64);});
		}
		Ok(())
	}
}

impl Read for ReadFile {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.input.read(buf)
	}
}

impl Seek for ReadFile {
	fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
		self.input.seek(pos)
	}
}

impl fmt::Debug for ReadFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", match self.path.to_str() {
			Some(s) => s,
			None => "Unknown"
		})
	}
}

impl fmt::Display for ReadFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}