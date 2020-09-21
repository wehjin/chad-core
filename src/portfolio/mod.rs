use std::collections::HashMap;

use crate::core::{Amount, AssetCode, SegmentType};
use crate::portfolio::holding::Holding;
use crate::portfolio::lot::Lot;
use crate::portfolio::segment::Segment;

pub mod holding;
pub mod link;
pub mod lot;
pub mod segment;

/// A Portfolio contains lots and assigns assets into segments.
pub trait Portfolio {
	/// Prices of assets.
	fn prices(&self) -> HashMap<AssetCode, Amount>;
	/// All Lots in the Portfolio.
	fn lots(&self) -> Vec<Lot>;
	/// Assignments between assets and segment.
	fn asset_assignments(&self) -> HashMap<AssetCode, SegmentType>;
	/// Combined value of lots in the Portfolio.
	fn portfolio_value(&self) -> Amount;
	/// Holdings in the Portfolio.
	fn holdings(&self) -> Vec<Holding>;
	/// Segments of the Portfolio.
	fn segments(&self) -> Vec<Segment>;
}