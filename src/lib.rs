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

pub(crate) mod oid {
	use echo_lib::ObjectId;

	pub fn from_squad_id(id: u64) -> ObjectId {
		ObjectId::String(id.to_string())
	}
}

pub(crate) mod chamber {
	use echo_lib::Chamber;

	use crate::{oid, point, target};

	pub fn unspent(squad_id: u64, chamber: &Chamber) -> f64 {
		let oid = oid::from_squad_id(squad_id);
		let target = chamber.target_at_object_point_or_none(&oid, &point::SQUAD_UNSPENT);
		target.map(target::to_f64).unwrap_or(0.0)
	}
}


pub(crate) mod point {
	use echo_lib::Point;

	pub const PRICE_F64: Point = Point::Static { aspect: "Price", name: "f64" };
	pub const SQUAD_NAME: Point = Point::Static { aspect: "Squad", name: "name" };
	pub const SQUAD_OWNER: Point = Point::Static { aspect: "Squad", name: "owner" };
	pub const SQUAD_MEMBERS: Point = Point::Static { aspect: "Squad", name: "members" };
	pub const SQUAD_UNSPENT: Point = Point::Static { aspect: "Squad", name: "unspent" };
	pub const MEMBER_SQUAD: Point = Point::Static { aspect: "Member", name: "squad" };
	pub const MEMBER_SYMBOL: Point = Point::Static { aspect: "Member", name: "symbol" };
	pub const LOT_SQUAD: Point = Point::Static { aspect: "Lot", name: "squad" };
	pub const LOT_SYMBOL: Point = Point::Static { aspect: "Lot", name: "symbol" };
	pub const LOT_SHARES: Point = Point::Static { aspect: "Lot", name: "shares" };
	pub const LOT_ACCOUNT: Point = Point::Static { aspect: "Lot", name: "account" };
}