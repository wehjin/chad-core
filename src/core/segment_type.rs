/// Enumerates relevant segments of a portfolio.
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
	pub fn known() -> [SegmentType; 4] {
		[SegmentType::Liquid, SegmentType::Stable, SegmentType::Linear, SegmentType::Expo]
	}
	pub fn all() -> [SegmentType; 5] {
		[SegmentType::Liquid, SegmentType::Stable, SegmentType::Linear, SegmentType::Expo, SegmentType::Unknown]
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
