use crate::prelude::{Amount, SegmentType};
use crate::prelude::SegmentType::{Expo, Linear, Liquid, Stable};

pub mod prelude {
	#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
	pub enum SegmentType {
		Liquid,
		Stable,
		Linear,
		Expo,
	}

	impl SegmentType {
		pub fn fraction(&self) -> f64 {
			match self {
				SegmentType::Liquid => 0.4f64 * 0.4f64 * 0.4f64,
				SegmentType::Stable => 0.6f64 * 0.4f64 * 0.4f64,
				SegmentType::Linear => 0.6f64 * 0.4f64,
				SegmentType::Expo => 0.6f64,
			}
		}
	}

	pub trait Segment {
		fn name(&self) -> &str;
		fn asset_type(&self) -> &SegmentType;
	}

	pub type Amount = f64;

	pub trait Portfolio {
		fn segment(&self, segment_type: SegmentType) -> &dyn Segment;
	}
}

pub mod portfolio;

pub fn allocate_amount(amount: Amount) -> Vec<(SegmentType, Amount)> {
	let amount = amount.abs();
	let expo = amount * Expo.fraction();
	let linear = amount * Linear.fraction();
	let stable = amount * Stable.fraction();
	let liquid = amount - stable - linear - expo;
	vec![(Liquid, liquid), (Stable, stable), (Linear, linear), (Expo, expo)]
}
