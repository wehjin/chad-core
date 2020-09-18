use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;

pub use portfolio::{Lot, Portfolio, SegmentReport};
use prelude::*;

use crate::LinkMsg::{AssignAsset, RecentPortfolio, UpdateLot, UpdatePrice};
use crate::portfolio::PortfolioMsg;
use crate::prelude::SegmentType::{Expo, Linear, Liquid, Stable};
use crate::SegmentType::Unknown;

mod portfolio;
pub mod prelude;


pub fn allocate_amount(amount: Amount) -> Vec<(SegmentType, Amount)> {
	let amount = amount.abs();
	let expo = amount * Expo.fraction();
	let linear = amount * Linear.fraction();
	let stable = amount * Stable.fraction();
	let liquid = amount * Liquid.fraction();
	let unknown = amount * Unknown.fraction();
	vec![(Liquid, liquid), (Stable, stable), (Linear, linear), (Expo, expo), (Unknown, unknown)]
}

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
}

enum LinkMsg {
	AssignAsset(AssetCode, SegmentType),
	UpdateLot {
		lot_id: LotId,
		asset_code: AssetCode,
		share_count: Amount,
		custodian: Custodian,
		share_price: Amount,
	},
	UpdatePrice(AssetCode, Amount),
	RecentPortfolio(Sender<Sender<PortfolioMsg>>),
}

pub fn portfolio_link() -> impl Link {
	let (tx, rx) = channel();
	thread::spawn(move || {
		let mut assignments = HashMap::new();
		let mut lots = HashMap::new();
		let mut prices = HashMap::new();
		for msg in rx {
			match msg {
				AssignAsset(asset_code, segment_type) => {
					assignments.insert(asset_code, segment_type);
				}
				UpdateLot { lot_id, asset_code, share_count, custodian, share_price } => {
					prices.insert(asset_code.clone(), share_price);
					let record = LotRecord { asset_code, share_count, custodian };
					lots.insert(lot_id, record);
				}
				UpdatePrice(asset_code, price) => {
					prices.insert(asset_code, price);
				}
				RecentPortfolio(response) => {
					let (tx, rx) = channel();
					thread::spawn({
						let assignments = assignments.clone();
						let lots = lots.clone();
						let prices = prices.clone();
						move || {
							for msg in rx {
								match msg {
									PortfolioMsg::Lots(reply) => {
										let lots = lots.iter().map(|(lot_id, record)| Lot {
											lot_id: *lot_id,
											asset_code: record.asset_code.to_owned(),
											share_count: record.share_count.to_owned(),
											custodian: record.custodian.to_owned(),
											share_price: *prices.get(&record.asset_code).unwrap_or(&1.0),
											segment: *assignments.get(&record.asset_code).unwrap_or(&SegmentType::Unknown),
										}).collect();
										reply.send(lots).expect("Reply to Lots");
									}
								}
							}
						}
					});
					response.send(tx).expect("Reply to RecentPortfolio")
				}
			}
		}
	});
	SenderLink { tx }
}

#[derive(Clone, PartialEq, Debug)]
struct LotRecord {
	pub asset_code: AssetCode,
	pub share_count: Amount,
	pub custodian: Custodian,
}

struct SenderLink { tx: Sender<LinkMsg> }

impl Link for SenderLink {
	fn assign_asset(&self, asset_code: &AssetCode, segment_type: SegmentType) {
		self.tx.send(AssignAsset(asset_code.clone(), segment_type)).expect("AssignAsset");
	}

	fn update_lot(&self, lot_id: LotId, asset_code: &AssetCode, share_count: Amount, custodian: &Custodian, share_price: Amount) {
		self.tx.send(UpdateLot {
			lot_id,
			asset_code: asset_code.to_owned(),
			share_count,
			custodian: custodian.to_owned(),
			share_price,
		}).expect("UpdateLot");
	}

	fn price_asset(&self, asset_code: &AssetCode, price: Amount) {
		self.tx.send(UpdatePrice(asset_code.to_owned(), price)).expect("UpdatePrice");
	}

	fn latest_portfolio(&self) -> Portfolio {
		let (tx, rx) = channel();
		self.tx.send(RecentPortfolio(tx)).expect("RecentPortfolio");
		let tx = rx.recv().expect("Recv RecentPortfolio");
		Portfolio { tx }
	}
}
