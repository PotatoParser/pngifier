extern crate crc32fast;
use crc32fast::Hasher;

use std::io;
use std::io::prelude::*;
use std::convert::TryInto;

use crate::util;
use util::ReadFile;

#[derive(Debug)]
pub struct Chunk {
	header: Vec<u8>,
	data: Vec<u8>,
	crc: Vec<u8>,
	pub total_data: usize
}

impl Chunk {
	// Read entire chunk: length + content + crc
	pub fn new(source: &mut ReadFile) -> io::Result<Option<Self>> {
		let mut buf = vec![0u8; 4];
		let initial_size = source.read(&mut buf)?;
		if initial_size == 0 {
			return Ok(None)
		}

		let size = read_be_u32(&mut &buf[..]) as usize;
		source.read(&mut buf)?;

		let mut data_buf = vec![0u8; size];
		source.read(&mut data_buf)?;

		let mut crc_buf = vec![0u8; 4];
		source.read(&mut crc_buf)?;

		Ok(Some(Chunk {
			header: buf,
			data: data_buf,
			crc: crc_buf,
			total_data: 12 + size
		}))
	}

	pub fn get_header(&self) -> &[u8] {
		&self.header[..]
	}

	pub fn get_data(&self) -> &[u8] {
		&self.data[..]
	}

	pub fn verify_crc(&self) -> bool {
		let merged = [self.get_header(), self.get_data()].concat();
		&Chunk::get_crc(&merged) == &self.crc[..]
	}

	pub fn get_crc(data: &[u8]) -> [u8; 4] {
		let mut crc_hash = Hasher::new();
		crc_hash.update(&data);
		crc_hash.finalize().to_be_bytes()
	}
}

fn read_be_u32(input: &mut &[u8]) -> u32 {
	let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
	*input = rest;
	u32::from_be_bytes(int_bytes.try_into().unwrap())
}