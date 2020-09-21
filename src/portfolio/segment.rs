use crate::core::{Amount, SegmentType};
use crate::portfolio::holding::Holding;

/// Describes a segment of the Portfolio.
#[derive(Clone, PartialEq, Debug)]
pub struct Segment {
	pub(crate) segment_type: SegmentType,
	pub(crate) drift_amount: Amount,
	pub(crate) target_value: Amount,
	pub(crate) holdings: Vec<Holding>,
	pub(crate) segment_value: Amount,
}

impl Segment {
	pub fn segment_type(&self) -> SegmentType { self.segment_type }
	pub fn segment_value(&self) -> Amount { self.segment_value }
	pub fn target_value(&self) -> Amount { self.target_value }
	pub fn drift_amount(&self) -> Amount { self.drift_amount }
}
