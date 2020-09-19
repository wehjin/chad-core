extern crate echo_lib;
extern crate rand;

pub use link::*;
pub use portfolio::{Lot, Portfolio, Segment};

mod portfolio;
mod link;

pub mod prelude {
	pub use crate::{Amount, AssetCode, Custodian, Link, Lot, LotId, Portfolio, Segment, SegmentType};
}

/// A quantity of something.
pub type Amount = f64;

/// Identifier for asset lots.
pub type LotId = u64;

/// Identifier for asset custodians.
pub type Custodian = String;

/// Identifiers for assets.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum AssetCode {
	Common(String),
	Custom(String),
}

/// Asset segments.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SegmentType {
	Liquid,
	Stable,
	Linear,
	Expo,
	Unknown,
}

impl SegmentType {
	pub fn fraction(&self) -> f64 {
		match self {
			SegmentType::Liquid => 0.064f64,
			SegmentType::Stable => 0.096f64,
			SegmentType::Linear => 0.24f64,
			SegmentType::Expo => 0.6f64,
			SegmentType::Unknown => 0.0f64,
		}
	}
	pub fn known_types() -> Vec<SegmentType> {
		vec![SegmentType::Liquid, SegmentType::Stable, SegmentType::Linear, SegmentType::Expo]
	}
	pub fn default_index(&self) -> usize {
		match self {
			SegmentType::Liquid => 0,
			SegmentType::Stable => 1,
			SegmentType::Linear => 2,
			SegmentType::Expo => 3,
			SegmentType::Unknown => 4,
		}
	}
}

