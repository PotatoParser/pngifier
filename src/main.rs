#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

use std::io::{self, Write};
use std::time::Instant;
use std::path::{Path, PathBuf};

#[macro_use]
mod util;
mod png_convert;
use util::{Error, ReadFile, WriteFile};
use util::color_type::*;

macro_rules! check {
	($t: expr ) => {
		if $t {
			"\x1b[1;37mX\x1b[0m"
		} else {
			"\x1b[1;37m \x1b[0m"
		}
	}
}

static VERSION: &str = "v1.0.0";

fn process() -> Result<(), Error> {
	let matches = clap_app!(pngifier => 
		(version: VERSION)
		(author: "Wilson Nguyen <wilsonqnguyen@gmail.com>")
		(about: "Encodes and decodes files into PNGs and back")
		(version_short: "V")
		(setting: clap::AppSettings::VersionlessSubcommands)
		(setting: clap::AppSettings::ArgRequiredElseHelp)
		(setting: clap::AppSettings::ColorAlways)
		(@subcommand encode =>
			(display_order: 1)
			(@arg WIDTH: -w --width +takes_value !empty_values "Sets the width of the image in pixels")
	        (@arg HEIGHT: -h --height +takes_value !empty_values "Sets the height of the image in pixels")
	        (@arg BYTES: -b --buffer +takes_value !empty_values "Sets the limiting buffer size (ie: 100, 1kb, 10mb, 1gb)")
	        (@arg DEPTH: -d --depth +takes_value !empty_values "Sets color depth. Bit depths of 8-bit and 16-bit are supported")
	        (@arg COLOR_TYPE: -t --type +takes_value !empty_values "Sets the color type (0, 2, 4, 6, g, ga, rgb, rgba)")
        	(@arg INPUT: +required "Sets the input file to use")
       		(@arg ACCEPT: -y --yes "Override all values with yes")
       		(@arg VERIFY: --verify "Verifies the file as a png before attempting to read it")
        	(@arg VERBOSE: -v --verbose "Verbose output")
        	(@arg SILENT: -s --silent "Prevents all outputs")
        	(@arg PROGRESS: -p --progress "Displays the progress")
        	(@arg STREAM: --stream "Streams the output to stdout")
        	(@arg TRIM: --trim "Trims the output (removes trailing null bytes)")
        	(@arg OUTPUT: "Sets the output file")   
		)
		(@subcommand decode =>
			(display_order: 2)
			(@arg BYTES: -b --buffer +takes_value !empty_values "Sets the limiting buffer size (ie: 100, 1kb, 10mb, 1gb)")
        	(@arg INPUT: +required "Sets the input file to use")
       		(@arg ACCEPT: -y --yes "Override all values with yes")
       		(@arg VERIFY: --verify "Verifies the file as a png before attempting to read it")
        	(@arg VERBOSE: -v --verbose "Verbose output")
        	(@arg SILENT: -s --silent "Prevents all outputs")
        	(@arg PROGRESS: -p --progress "Displays the progress")
        	(@arg STREAM: --stream "Streams the output to stdout")
        	(@arg TRIM: --trim "Trims the output (removes trailing null bytes)")
        	(@arg OUTPUT: "Sets the output file")
		)
	).get_matches();

	if let Some(encode) = matches.subcommand_matches("encode") {
		initialize(&encode);

		let (mut read_file, mut write_file) = parse_input_output(&encode, false)?;

		let mut color_type: u8 = 2;
		let mut bit_depth: u8 = 8;
		let mut bytes_per_px = total_bytes(color_type, bit_depth) as u64;

		if let Some(custom_depth) = encode.value_of("DEPTH") {
			match custom_depth.parse::<u8>() {
				Ok(custom_depth) => {
					if !(custom_depth == 8 || custom_depth == 16) {
						error!(ParseBitDepth, custom_depth);
					}
					bit_depth = custom_depth;
					bytes_per_px = total_bytes(color_type, bit_depth) as u64;
				},
				_ => error!(ParseBitDepth, custom_depth)
			};
		}

		if let Some(custom_color_type) = encode.value_of("COLOR_TYPE") {
			match custom_color_type.parse::<u8>() {
				Ok(custom_color_type) => {
					if !(type_exists(custom_color_type)) {
						error!(ParseColorType, custom_color_type);
					}
					color_type = custom_color_type;
					bytes_per_px = total_bytes(color_type, bit_depth) as u64;
				},
				_ => {
					match type_str_translate(custom_color_type) {
						Some(custom_color_type) => {
							let custom_color_type = *custom_color_type;
							if !(type_exists(custom_color_type)) {
								error!(ParseColorType, custom_color_type);
							}
							color_type = custom_color_type;
							bytes_per_px = total_bytes(color_type, bit_depth) as u64;
						},
						_ => error!(ParseColorType, custom_color_type)
					}
				}
			};
		}

		let max_bytes: u64 = read_file.size;
		let mut width: u64 = ((max_bytes / bytes_per_px) as f64).sqrt() as u64;
		let mut chunk_size: u64 = width * bytes_per_px;
		let mut height: u64 = max_bytes / chunk_size;
		let mut buffer_size = chunk_size;
		if max_bytes != chunk_size {
			height += 1;
		}

		if encode.is_present("WIDTH") && encode.is_present("HEIGHT") {
			error!(WidthAndHeightDefined);
		}

		if let Some(custom_width) = parse_arg_u64(encode, "WIDTH", false)? {
			if custom_width != width {
				width = custom_width;
				chunk_size = width * bytes_per_px;
				height = max_bytes / chunk_size;
				if max_bytes != chunk_size {
					height += 1;
				}	
			}
		}

		if let Some(custom_height) = parse_arg_u64(encode, "HEIGHT", true)? {
			if custom_height != height {
				height = custom_height;	
				width = max_bytes / (height * bytes_per_px);
				if max_bytes % (height * bytes_per_px) != 0 {
					width += 1;
				}
				chunk_size = width * bytes_per_px;
			}
		}

		if let Some(custom_buffer) = parse_byte_string(encode)? {
			buffer_size = custom_buffer;
		}

		verbose!({
			println!(
				"\n\x1b[1;36mConfiguration:\x1b[1;33m \n\
				[{}\x1b[1;33m] Verification Mode \n\
				[{}\x1b[1;33m] Trimming \n\
				[{}\x1b[1;33m] Buffer Size: \x1b[1;36m{}\x1b[1;33m \n\
				[{}\x1b[1;33m] Width: \x1b[1;36m{}px\x1b[1;33m \n\
				[{}\x1b[1;33m] Height: \x1b[1;36m{}px\x1b[1;33m \n\
				[{}\x1b[1;33m] Color Type: \x1b[1;36m{}\x1b[1;33m \n\
				[{}\x1b[1;33m] Bit Depth: \x1b[1;36m{}\x1b[0m\n",
			check!(encode.is_present("VERIFY")),
			check!(encode.is_present("TRIM")),
			check!(encode.is_present("BYTES")), buffer_size,
			check!(encode.is_present("WIDTH")), width,
			check!(encode.is_present("HEIGHT")), height,
			check!(encode.is_present("COLOR_TYPE")), color_type,
			check!(encode.is_present("DEPTH")), bit_depth			
			);
		});

		let start = Instant::now();
		error_exp!(
			Encode,
			&read_file,
			png_convert::encode(
				&mut read_file,
				&mut write_file,
				width,
				height,
				chunk_size,
				max_bytes,
				buffer_size as usize,
				bit_depth,
				color_type,
				encode.is_present("TRIM")
			)
		);
		silent!({println!("Encoded \x1b[1;36m'{}'\x1b[0m to \x1b[1;36m'{}'\x1b[0m in \x1b[1;36m{:?}\x1b[0m.", &read_file, &write_file, start.elapsed())});


		if encode.is_present("VERIFY") && !encode.is_present("STREAM") {
			let mut write_file = error_exp!(ReadFail, &write_file, write_file.read());
			verify(&mut write_file)?;			
		}

		return Ok(());
	} else if let Some(decode) = matches.subcommand_matches("decode") {
		initialize(&decode);

		let (mut read_file, mut write_file) = parse_input_output(&decode, true)?;

		let mut buffer_size: usize = 1024 * 1024 * 100;
		if let Some(custom_buffer) = parse_byte_string(decode)? {
			buffer_size = custom_buffer as usize;
		}

		verbose!({
			println!(
				"\n\x1b[1;36mConfiguration:\x1b[1;33m \n\
				[{}\x1b[1;33m] Verification Mode \n\
				[{}\x1b[1;33m] Trimming \n\
				[{}\x1b[1;33m] Buffer Size: \x1b[1;36m{}\x1b[0m\n",
			check!(decode.is_present("VERIFY")),
			check!(decode.is_present("TRIM")),
			check!(decode.is_present("BYTES")), buffer_size,
			);
		});

		if decode.is_present("VERIFY") {
			verify(&mut read_file)?;	
		}

		let start = Instant::now();
		error_exp!(
			Decode,
			&read_file,
			png_convert::decode(
				&mut read_file,
				&mut write_file,
				buffer_size
			)
		);
		silent!({println!("Decoded \x1b[1;36m'{}'\x1b[0m to \x1b[1;36m'{}'\x1b[0m in \x1b[1;36m{:?}\x1b[0m.", &read_file, &write_file, start.elapsed())});

		silent!({
			if decode.is_present("TRIM") && !decode.is_present("STREAM") {
				write_file.trim(buffer_size)?
			}
		});

		return Ok(());
	}
	std::process::exit(1)
}

fn main() {
	// Ignore default "Error: " boilerplate code
	if let Err(e) = process() {
		eprint!("{:?}", e);
		std::process::exit(1);
	}
	std::process::exit(0);
}

// Set verbose levels and enable ANSI escape codes
fn initialize(subcommand: &clap::ArgMatches) {
	set_verbose_levels(&subcommand);
	match util::enable_ansi_support() {
		Ok(_) => (),
		_ => verbose!({println!("\x1b[1;33mWarning: Unable to enable ANSI support for Windows.\x1b[0m")})
	};
	silent!({println!("\x1b[1;35mpngifier {}\x1b[0m", VERSION)});
}

// Set respective verbosity levels
fn set_verbose_levels(subcommand: &clap::ArgMatches) {
	if subcommand.is_present("ACCEPT") {
		unsafe {
			util::verbosity::SKIP = true;
		}
	}	
	if subcommand.is_present("STREAM") {
		unsafe {
			util::verbosity::SKIP = true;
			util::verbosity::SILENT = true;
		}
		return;		
	}
	if subcommand.is_present("SILENT") {
		unsafe {
			util::verbosity::SKIP = true;
			util::verbosity::SILENT = true;
		}
	}
	if subcommand.is_present("VERBOSE") {
		unsafe {
			util::verbosity::VERBOSE = true;
		}
	}
	if subcommand.is_present("PROGRESS") {
		unsafe {
			util::verbosity::PROGRESS = true;
		}
	}
}

//Convert string to u64
fn parse_arg_u64(subcommand: &clap::ArgMatches, option_name: &str, height: bool) -> Result<Option<u64>, Error> {
	match subcommand.value_of(option_name) {
		Some(c) => {
			match c.parse::<u64>() {
				Ok(c) => {
					return Ok(Some(c));
				},
				_ => {
					if height {
						error!(ParseHeight, c);
					}
					error!(ParseWidth, c);
				}
			};
		}
		_ => {
			return Ok(None);
		}
	};
}

// Convert byte to u64
fn parse_byte_string(subcommand: &clap::ArgMatches) -> Result<Option<u64>, Error> {
	match subcommand.value_of("BYTES") {
		Some(input_str) => {
		    let t = input_str.clone();
		    let t = t.to_lowercase();
		    let mut byte_match = "b";
		    let mut byte_match_index = input_str.len();
		    for i in &["gb", "mb", "kb", "b"] {
		        match t.rfind(i) {
		            Some(s) => {
		                byte_match = i;
		                byte_match_index = s;
		                break;
		            }
		            None => ()
		        }
		    }
		    let num = match t[..byte_match_index].trim().parse::<u64>() {
		        Ok(n) => n,
		        Err(_) => error!(ParseBuffer, input_str)
		    };
		    match byte_match {
		        "gb" => Ok(Some(num * 1024 * 1024 * 1024)),
		        "mb" => Ok(Some(num * 1024 * 1024)),
		        "kb" => Ok(Some(num * 1024)),
		        "b" => Ok(Some(num)),
		        _ => error!(ParseBuffer, input_str)
		    }
		},
		None => Ok(None)
	}
}

// Convert input & output choices to respective ReadFile and WriteFile
fn parse_input_output<'a>(subcommand: &'a clap::ArgMatches, decode: bool) -> Result<(ReadFile, WriteFile), Error>{
	let input = subcommand.value_of("INPUT").unwrap();
	let input: &str = &format!("{}", input);

	let mut output: &str = &format!("{}.png", input)[..];
	if decode {
		match input.rfind(".png") {
			Some(index) => output = &input[..index],
			None => ()
		};
	}

	if let Some(s) = subcommand.value_of("OUTPUT") {
		output = s;
	}

	let read_path = PathBuf::from(input);
	let read_path_ref = read_path.as_path();

	if !read_path_ref.exists() {
		error!(InputDoesNotExist, input);			
	}

	if !read_path_ref.is_file() {
		error!(InputNotAFile, input);
	}

	let read_file = match ReadFile::from_pathbuf(read_path) {
		Ok(file) => file,
		_ => error!(ReadFail, input)
	};

	let write_file = match subcommand.is_present("STREAM") {
		true => WriteFile::stdout(),
		false => {
			skip!({
				let write_path = Path::new(output);
				if write_path.exists() {
					let mut user_input = String::new();
					print!("\x1b[1;33mWarning: The output file of '\x1b[1;36m{}\x1b[0m\x1b[1;33m' currently exists. Would you like to override it? (\x1b[1;32my\x1b[1;33m/\x1b[1;31mN\x1b[1;33m): \x1b[0m", output);
					io::stdout().flush().expect("Unable to Flush to stdout.");
					io::stdin().read_line(&mut user_input).expect("Unable to read input");
					user_input = user_input.to_lowercase();
					match user_input.trim() {
						"yes" | "y" => (),
						_ => std::process::exit(1)
					};
				}
			});
			WriteFile::from_string(output.to_string())
		}
	};

	let write_file = match write_file {
		Ok(file) => file,
		_ => error!(WriteFail, output)
	};

	Ok((read_file, write_file))
}

// Verifies output/input file as a PNG
fn verify(read_file: &mut ReadFile) -> Result<(), Error> {
	read_file.reset()?;
	let start = Instant::now();
	read_file.verify_png()?;
	silent!({println!("Verified '\x1b[1;36m{}\x1b[0m' in \x1b[1;36m{:?}\x1b[0m.", &read_file, start.elapsed())});
	read_file.reset()?;
	Ok(())
}