extern crate flate2;
use flate2::read::ZlibEncoder;
use flate2::Compression;

use std::io::{self, Read, Write};

use crate::util;
use util::{WriteFile, ReadFile};

#[derive(Debug)]
struct Transformer<R: Read> {
	source: R,
	total_in: u64,
	chunk_size: u64,
	capacity: u64,
	max_bytes: u64,
	trim: bool
}

impl<R: Read> Read for Transformer<R> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {

		let mut read_size = buf.len();
		// When the capacity is completely filled up
		if self.total_in == self.capacity {
			return Ok(0usize);
		}
		// Current data is less than the data of the actual file
		if self.total_in < self.max_bytes {

			// Too much data (overflowing the chunk)
			if read_size as u64 > self.chunk_size {
				read_size = self.chunk_size as usize;
			}

			// Currently at the position to add a filter
			if self.total_in % self.chunk_size == 0 {
				buf[0] = 0u8;
				read_size = self.source.read(&mut buf[1..read_size])?;
				self.total_in += read_size as u64; // DO NOT INCLUDE FILTER
				if read_size != 0 {
					read_size += 1;
				}
				//println!("Apply Filter");
				return Ok(read_size);
			}
			// Normally add data

			// Remaining data to approach chunk size
			let remaining = self.chunk_size - (self.total_in % self.chunk_size);
			read_size = remaining as usize;

			if read_size > buf.len() {
				read_size = buf.len();
			}

			if read_size as u64 + self.total_in > self.max_bytes {
				read_size = (self.max_bytes - self.total_in) as usize;
			}

			read_size = self.source.read(&mut buf[..read_size])?;
			self.total_in += read_size as u64;
			return Ok(read_size);

		}

		if self.trim {
			return Ok(0usize);
		}

		// Fill to capacity
		if read_size as u64 + self.total_in > self.capacity {
			read_size = (self.capacity - self.total_in) as usize;
		}
		for i in buf.iter_mut() {
			*i = 0;
		}
		self.total_in += read_size as u64;
		return Ok(read_size);

	}
}

// Encodes a file into a PNG
pub fn encode(
	read_file: &mut ReadFile,
	write_file: &mut WriteFile,
	width: u64, 
	height: u64, 
	chunk_size: u64, 
	max_bytes: u64, 
	buffer_size: usize,
	bit_depth: u8, 
	color_type: u8,
	trim: bool
	) -> io::Result<()>{

	let transformer = Transformer {
			source: read_file,
			total_in: 0,
			chunk_size: chunk_size,
			capacity: chunk_size * height,
			max_bytes: max_bytes,
			trim: trim
	};
	let mut deflater = ZlibEncoder::new(transformer, Compression::fast());

	let mut buffer = vec![0u8; buffer_size];

	let mut progress_bar = util::ProgressBar::new(chunk_size * height + 3, "Encoding as PNG");

	write_file.write_header()?;

	progress!({progress_bar.tick(1);});
	
	write_file.write_chunk(
		b"IHDR",
		&[
			&(width as u32).to_be_bytes()[..4],
			&(height as u32).to_be_bytes()[..4],
			&[bit_depth, color_type, 0u8, 0u8, 0u8][..]
		].concat()
	)?;
	progress!({progress_bar.tick(1);});


	let mut size = deflater.read(&mut buffer)?;	
	while size != 0 {
		progress!({progress_bar.set_tick(deflater.get_ref().total_in + 2);});
		write_file.write_chunk(
			b"IDAT",
			&buffer[..size]
		)?;

		size = deflater.read(&mut buffer)?;
	}

	write_file.write_chunk(
		b"IEND",
		b""
	)?;
	progress!({progress_bar.complete();});

	write_file.flush()?;

	Ok(())
}