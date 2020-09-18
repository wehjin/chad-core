pub use crate::{Lot, Portfolio, Segment, SegmentType};

pub type Amount = f64;

/// Top level link to a portfolio.
pub trait Link {
	fn assign_asset(&self, asset_code: &AssetCode, segment_type: SegmentType);
	fn update_lot(&self, lot_id: LotId, asset_code: &AssetCode, share_count: Amount, custodian: &Custodian, share_price: Amount);
	fn price_asset(&self, asset_code: &AssetCode, price: Amount);
	fn latest_portfolio(&self) -> Portfolio;
}

pub type LotId = u64;
pub type Custodian = String;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum AssetCode {
	Common(String),
	Custom(String),
}
