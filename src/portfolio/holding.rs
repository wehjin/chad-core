use std::collections::HashMap;

use crate::core::{Amount, AssetCode};
use crate::portfolio::lot::Lot;

/// Describes the Lots in a Portfolio that contain an asset.
#[derive(Clone, PartialEq, Debug)]
pub struct Holding {
	asset_code: AssetCode,
	pub lots: Vec<Lot>,
}

impl Holding {
	pub fn new(asset_code: &AssetCode) -> Self {
		Holding { asset_code: asset_code.clone(), lots: Vec::new() }
	}
	pub fn asset_code(&self) -> &AssetCode { &self.asset_code }
	pub fn holding_value(&self, prices: &HashMap<AssetCode, Amount>) -> Amount { self.lots.iter().map(|it| it.lot_value(prices)).sum() }
	pub fn push_lot(&mut self, lot: Lot) {
		self.lots.push(lot)
	}
}
