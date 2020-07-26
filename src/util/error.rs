use std::fmt;

#[macro_export]
macro_rules! error {
	($t:ident) => {
		return Err(Error::$t);
	};
	
	($t:ident, $msg:expr) => {
		return Err(Error::$t(format!("{}", $msg)));
	};
}

#[macro_export]
macro_rules! error_exp {
	($t:ident, $exp:expr) => {
		match $exp {
			Ok(a) => a,
			_ => error!($t)
		}
	};

	($t:ident, $msg:expr, $exp:expr) => {
		match $exp {
			Ok(a) => a,
			_ => error!($t, $msg)
		}
	};
}

pub enum Error {
	ParseWidth(String),
	ParseHeight(String),
	ParseBuffer(String),
	ParseColorType(String),
	ParseBitDepth(String),
	WidthAndHeightDefined,
	ReadFail(String),
	WriteFail(String),
	InputDoesNotExist(String),
	InputNotAFile(String),
	ReadChunk(String),
	InvalidCRC(String),
	InvalidHeader(String),
	Encode(String),
	Decode(String),
	TrimError(String)
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "\n\x1b[1;31mError: {}\x1b[0m", match self {
			Error::ParseWidth(s) => format!("Invalid width of '{}'.", s),
			Error::ParseHeight(s) => format!("Invalid height of '{}'.", s),
			Error::ParseBuffer(s) => format!("Invalid buffer size of '{}'.", s),
			Error::ParseColorType(s) => format!("Invalid color type of '{}'. Color types of 0, 2, 4 & 6 are supported.", s),
			Error::ParseBitDepth(s) => format!("Invalid bit depth of '{}'. Bit depths of 8-bit and 16-bit are supported.", s),
			Error::WidthAndHeightDefined => String::from("Cannot define both a width and height of the image."),
			Error::ReadFail(s) => format!("Unable to open '{}' for reading.", s),
			Error::WriteFail(s) => format!("Unable to open '{}' for writing.", s),
			Error::InputDoesNotExist(s) => format!("Input file of '{}' does not exist.", s),
			Error::InputNotAFile(s) => format!("Input of '{}' is not a file.", s),
			Error::ReadChunk(s) => format!("Unable to read chunk from '{}'.", s),
			Error::InvalidCRC(s) => format!("Unable to verify crc from '{}'.", s),
			Error::InvalidHeader(s) => format!("Image of '{}' has invalid header.", s),
			Error::Encode(s) => format!("Unable to encode '{}' as PNG.", s),
			Error::Decode(s) => format!("Unable to decode '{}' from PNG.", s),
			Error::TrimError(s) => format!("Unable to trim '{}'.", s)
		})
	}
}