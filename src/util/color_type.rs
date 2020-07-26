use std::collections::HashMap;

lazy_static! {
	static ref COLOR_TYPE_MAP: HashMap<u8, u8> = {
		let mut m = HashMap::new();
		m.insert(0, 1);
		m.insert(2, 3);
		m.insert(4, 2);
		m.insert(6, 4);
		m
	};
	static ref COLOR_TYPE_STR: HashMap<String, u8> = {
		let mut m = HashMap::new();
		m.insert(String::from("g"), 0);
		m.insert(String::from("ga"), 4);
		m.insert(String::from("rgb"), 2);
		m.insert(String::from("rgba"), 6);
		m
	};
}

pub fn type_to_size(color_type: u8) -> usize {
	*COLOR_TYPE_MAP.get(&color_type).unwrap() as usize
}

pub fn total_bytes(color_type: u8, bit_depth: u8) -> usize {
	type_to_size(color_type) * (bit_depth as usize) / 8usize
}

pub fn type_exists(color_type: u8) -> bool {
	COLOR_TYPE_MAP.contains_key(&color_type)
}

pub fn type_str_translate(input: &str) -> Option<&u8> {
	COLOR_TYPE_STR.get(input)
}