extern crate echo_lib;
extern crate rand;

pub mod chad;
pub mod core;

pub(crate) mod target {
	use echo_lib::Target;

	pub fn to_f64(target: Target) -> f64 {
		match target {
			Target::String(s) => s.parse::<f64>().expect("parse price"),
			_ => panic!("Invalid target for price")
		}
	}

	pub fn from_f64(value: f64) -> Target {
		Target::String(format!("{}", value))
	}

	pub fn to_member_ids(target: &Target) -> Vec<u64> {
		target.as_str()
			.split(":")
			.map(|it| it.parse::<u64>().expect("member-id"))
			.collect::<Vec<_>>()
	}
}
