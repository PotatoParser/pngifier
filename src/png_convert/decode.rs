extern crate flate2;
use flate2::read::ZlibDecoder;

use std::io::{self, Write};
use std::io::prelude::*;
use std::convert::TryInto;

use crate::util;
use util::color_type::*;
use util::{ReadFile, WriteFile};

#[derive(Debug)]
struct Transformer<R: Read> {
	source: R,
	buffer_size: usize,
	total_read: usize
}

impl<R: Read> Read for Transformer<R> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let mut read_size = self.buffer_size;
		let mut filler = [0u8; 4];
		if read_size > buf.len() {
			read_size = buf.len();
		}
		if read_size > 0 {
			self.total_read += self.source.read(&mut buf[..read_size])?;
			self.buffer_size -= read_size;
			if self.buffer_size == 0 {
				self.total_read += self.source.read(&mut filler)?;
			}
			return Ok(read_size);
		}

		self.total_read += self.source.read(&mut buf[..8])?;
		self.buffer_size = read_be_u32(&mut &buf[..4]) as usize;

		// Found end
		if &buf[4..8] == b"IEND" {
			return Ok(0usize);
		}

		read_size = self.buffer_size;
		if read_size > buf.len() {
			read_size = buf.len();
		}
		self.total_read += self.source.read(&mut buf[..read_size])?;
		self.buffer_size -= read_size;
		if self.buffer_size == 0 {
			// Flush crc
			self.total_read += self.source.read(&mut filler)?;
		}
		return Ok(read_size);
	}
}

// Decodes a PNG created from PNGIFIER back into a file
pub fn decode(
	read_file: &mut ReadFile,
	write_file: &mut WriteFile,
	buffer_size: usize
	) -> io::Result<()> {
	let mut progress_bar = util::ProgressBar::new(read_file.size, "Converting from PNG");
	read_file.read_header()?;

	let hdat = (read_file.read_chunk()?).unwrap();
	let width = read_be_u32(&mut &hdat.get_data()[..4]) as usize;
	let bit_depth = hdat.get_data()[8];
	let color_type = hdat.get_data()[9];
	let multiplier = total_bytes(color_type, bit_depth) as usize;
	let transformer = Transformer {
		source: read_file,
		buffer_size: 0,
		total_read: 0
	};

	let mut inflater = ZlibDecoder::new(transformer);

	let mut remainder: usize = 0;
	let chunk_size: usize = (width * multiplier) + 1;
	let mut buffer = vec![0u8; buffer_size];

	let mut read_size = inflater.read(&mut buffer)?;
	while read_size != 0 {
		let slice = &buffer[..read_size];
		let remaining: usize = (remainder + slice.len()) % chunk_size;
		let complete: usize = (remainder + slice.len()) / chunk_size;
		for i in 0..complete {
			let start = safe_usize((i * chunk_size) + 1, remainder);
			let end = safe_usize((i + 1) * chunk_size, remainder);
			write_file.write(&slice[start..end])?;
		}
		if remaining != 0 {
			let shift: usize = safe_usize((complete * chunk_size) + 1, remainder);
			write_file.write(&slice[shift..])?;
		}
		remainder = remaining;
		progress!({progress_bar.set_tick(inflater.get_ref().total_read as u64);});
		read_size = inflater.read(&mut buffer)?;
	}
	progress!({progress_bar.complete();});
	Ok(())
}

fn safe_usize(base: usize, sub: usize) -> usize {
	if sub > base {
		0usize
	} else {
		base - sub
	}
}

fn read_be_u32(input: &mut &[u8]) -> u32 {
	let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
	*input = rest;
	u32::from_be_bytes(int_bytes.try_into().unwrap())
}