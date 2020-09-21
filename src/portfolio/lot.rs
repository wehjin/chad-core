use std::collections::HashMap;

use crate::core::{Account, Amount, AssetCode, Custodian, LotId};

/// A Lot is a quantum of asset held in a Portfolio.
#[derive(Clone, PartialEq, Debug)]
pub struct Lot {
	pub lot_id: LotId,
	pub asset_code: AssetCode,
	pub share_count: Amount,
	pub custodian: Custodian,
	pub account: Account,
}

impl Lot {
	/// Value of assets in th Lot.
	pub fn lot_value(&self, prices: &HashMap<AssetCode, Amount>) -> Amount { self.share_count * prices.get(&self.asset_code).cloned().unwrap_or(1.0) }
}

