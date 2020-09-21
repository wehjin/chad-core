use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};

use crate::core::{Amount, AssetCode, SegmentType};
use crate::core::SegmentType::{Expo, Linear, Liquid, Stable, Unknown};
use crate::portfolio::holding::Holding;
use crate::portfolio::lot::Lot;
use crate::portfolio::Portfolio;
use crate::portfolio::segment::Segment;

pub struct PortfolioLink {
	pub(crate) tx: Sender<PortfolioMsg>
}

pub(crate) enum PortfolioMsg {
	Lots(Sender<Vec<Lot>>),
	AssetAssignments(Sender<HashMap<AssetCode, SegmentType>>),
	Prices(Sender<HashMap<AssetCode, Amount>>),
}

impl Portfolio for PortfolioLink {
	fn prices(&self) -> HashMap<AssetCode, Amount> {
		let (tx, rx) = channel();
		self.tx.send(PortfolioMsg::Prices(tx)).expect("Request Prices");
		rx.recv().expect("Recv prices")
	}

	fn lots(&self) -> Vec<Lot> {
		let (tx, rx) = channel();
		self.tx.send(PortfolioMsg::Lots(tx)).expect("Request Lots");
		rx.recv().expect("Recv lots")
	}

	fn asset_assignments(&self) -> HashMap<AssetCode, SegmentType> {
		let (tx, rx) = channel();
		self.tx.send(PortfolioMsg::AssetAssignments(tx)).expect("Request Assignments");
		rx.recv().expect("Recv assignments")
	}

	fn portfolio_value(&self) -> Amount {
		let prices = self.prices();
		self.lots().iter().map(|it| it.lot_value(&prices)).sum()
	}

	fn holdings(&self) -> Vec<Holding> {
		let mut holdings = HashMap::new();
		self.lots().into_iter().for_each(|lot| {
			let asset_code = lot.asset_code.clone();
			if holdings.get(&asset_code).is_none() {
				holdings.insert(asset_code.clone(), Holding::new(&asset_code));
			}
			let holding = holdings.get_mut(&asset_code).expect("get Holding");
			holding.push_lot(lot)
		});
		holdings.into_iter().map(|(_, holding)| holding).collect()
	}

	fn segments(&self) -> Vec<Segment> {
		let prices = self.prices();
		let holdings_by_type = holdings_by_type(self.holdings(), &self.asset_assignments());
		let segment_values_by_type = {
			let mut map = HashMap::new();
			for segment_type in &SegmentType::all() {
				let holdings = holdings_by_type.get(&segment_type).expect("Holdings with type");
				let value: Amount = holdings.iter().map(|it| it.holding_value(&prices)).sum();
				map.insert(*segment_type, value);
			}
			map
		};
		let portfolio_value: Amount = segment_values_by_type.iter().map(|(_, value)| *value).sum();
		let targets = allocate_amount(portfolio_value);
		let segments = targets.into_iter().map(|(segment_type, target_value)| {
			let segment_value = segment_values_by_type[&segment_type];
			let drift_amount = segment_value - target_value;
			let holdings = holdings_by_type[&segment_type].clone();
			Segment { segment_type, drift_amount, target_value, holdings, segment_value }
		}).collect::<Vec<_>>();
		segments
	}
}

fn holdings_by_type(holdings: Vec<Holding>, assignments: &HashMap<AssetCode, SegmentType>) -> HashMap<SegmentType, Vec<Holding>> {
	let mut holdings_by_type = HashMap::new();
	for segment_type in &SegmentType::all() {
		holdings_by_type.insert(*segment_type, Vec::new());
	}
	for holding in holdings {
		let segment_type = {
			let asset_code = holding.asset_code();
			assignments.get(asset_code).unwrap_or(&SegmentType::Unknown)
		};
		let holdings = holdings_by_type.get_mut(segment_type).expect("get_mut Holdings");
		holdings.push(holding);
	}
	holdings_by_type
}

fn allocate_amount(amount: Amount) -> Vec<(SegmentType, Amount)> {
	let amount = amount.abs();
	let expo = amount * Expo.fraction();
	let linear = amount * Linear.fraction();
	let stable = amount * Stable.fraction();
	let liquid = amount * Liquid.fraction();
	let unknown = amount * Unknown.fraction();
	vec![(Liquid, liquid), (Stable, stable), (Linear, linear), (Expo, expo), (Unknown, unknown)]
}
