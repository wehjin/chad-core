use std::sync::mpsc::{channel, Sender};

use crate::prelude::{Amount, AssetCode, Custodian, LotId, SegmentType};

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

	pub fn segments(&self) -> Vec<Segment> {
		let mut segments = [
			Segment { segment_type: SegmentType::Liquid, lots: Vec::new() },
			Segment { segment_type: SegmentType::Stable, lots: Vec::new() },
			Segment { segment_type: SegmentType::Linear, lots: Vec::new() },
			Segment { segment_type: SegmentType::Expo, lots: Vec::new() },
			Segment { segment_type: SegmentType::Unknown, lots: Vec::new() },
		];
		self.lots().into_iter().for_each(|it| {
			let i = match it.segment {
				SegmentType::Liquid => 0,
				SegmentType::Stable => 1,
				SegmentType::Linear => 2,
				SegmentType::Expo => 3,
				SegmentType::Unknown => 4,
			};
			segments[i].lots.push(it)
		});
		segments.to_vec()
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct Segment {
	segment_type: SegmentType,
	pub lots: Vec<Lot>,
}

impl Segment {
	pub fn segment_type(&self) -> SegmentType { self.segment_type }
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

