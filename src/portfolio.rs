use std::sync::mpsc::{channel, Sender};

use crate::prelude::{Amount, AssetCode, Custodian, LotId, SegmentType};
use crate::SegmentType::{Expo, Linear, Liquid, Stable, Unknown};

pub(crate) enum PortfolioMsg {
	Lots(Sender<Vec<Lot>>)
}

pub struct Portfolio {
	pub(crate) tx: Sender<PortfolioMsg>
}

impl Portfolio {
	pub fn lots(&self) -> Vec<Lot> {
		let (tx, rx) = channel();
		self.tx.send(PortfolioMsg::Lots(tx)).expect("Request Lots");
		rx.recv().expect("Recv lots")
	}

	pub fn currency_value(&self) -> Amount {
		self.lots().iter().map(Lot::currency_value).sum()
	}

	pub fn segments(&self) -> Vec<Segment> {
		let mut segments = [
			Segment { segment_type: SegmentType::Liquid, drift_value: 0.0, allocation_value: 0.0, lots: Vec::new() },
			Segment { segment_type: SegmentType::Stable, drift_value: 0.0, allocation_value: 0.0, lots: Vec::new() },
			Segment { segment_type: SegmentType::Linear, drift_value: 0.0, allocation_value: 0.0, lots: Vec::new() },
			Segment { segment_type: SegmentType::Expo, drift_value: 0.0, allocation_value: 0.0, lots: Vec::new() },
			Segment { segment_type: SegmentType::Unknown, drift_value: 0.0, allocation_value: 0.0, lots: Vec::new() },
		];
		self.lots().into_iter().for_each(|it| {
			let segment_type = &it.segment;
			let i = Portfolio::index_of_type(segment_type);
			segments[i].lots.push(it)
		});
		let segment_values = segments.iter().map(Segment::segment_value).collect::<Vec<_>>();
		let full_value = segment_values.iter().sum();
		allocate_amount(full_value).iter()
			.for_each(|(segment, allocated_value)| {
				let i = Self::index_of_type(segment);
				let segment_value = segment_values[i];
				let drift_value = segment_value - allocated_value;
				segments[i].drift_value = drift_value;
				segments[i].allocation_value = *allocated_value;
			});
		segments.to_vec()
	}
	fn index_of_type(segment_type: &SegmentType) -> usize {
		match segment_type {
			SegmentType::Liquid => 0,
			SegmentType::Stable => 1,
			SegmentType::Linear => 2,
			SegmentType::Expo => 3,
			SegmentType::Unknown => 4,
		}
	}
}


#[derive(Clone, PartialEq, Debug)]
pub struct Segment {
	segment_type: SegmentType,
	drift_value: Amount,
	allocation_value: Amount,
	pub lots: Vec<Lot>,
}

impl Segment {
	pub fn segment_type(&self) -> SegmentType { self.segment_type }
	pub fn drift_value(&self) -> Amount { self.drift_value }
	pub fn allocate_value(&self) -> Amount { self.allocation_value }
	pub fn segment_value(&self) -> Amount {
		self.lots.iter().fold(
			0.0f64,
			|sum, next| sum + next.currency_value(),
		)
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct Lot {
	pub lot_id: LotId,
	pub asset_code: AssetCode,
	pub share_count: Amount,
	pub custodian: Custodian,
	pub share_price: Amount,
	pub segment: SegmentType,
}

impl Lot {
	pub fn currency_value(&self) -> Amount { self.share_count * self.share_price }
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
