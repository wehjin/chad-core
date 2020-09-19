use std::fmt::Debug;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use echo_lib::{Echo, ObjectId, Point, Target};

use LinkMsg::{AssignAsset, RecentPortfolio, UpdateLot, UpdatePrice};

use crate::{Amount, AssetCode, Custodian, Lot, LotId, Portfolio, SegmentType};
use crate::portfolio::PortfolioMsg;

/// Top level link to a portfolio.
pub trait Link {
	fn assign_asset(&self, asset_code: &AssetCode, segment_type: SegmentType);
	fn update_lot(&self, lot_id: LotId, asset_code: &AssetCode, share_count: Amount, custodian: &Custodian, share_price: Amount);
	fn price_asset(&self, asset_code: &AssetCode, price: Amount);
	fn latest_portfolio(&self) -> Portfolio;
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

const ATTR_ASSET_TYPE: Point = Point::Static { aspect: "Asset", name: "Type" };
const ATTR_ASSET_PRICE: Point = Point::Static { aspect: "Asset", name: "Price" };
const ATTR_LOT_ASSET: Point = Point::Static { aspect: "Lot", name: "Asset" };
const ATTR_LOT_SHARES: Point = Point::Static { aspect: "Lot", name: "Shares" };
const ATTR_LOT_CUSTODIAN: Point = Point::Static { aspect: "Lot", name: "Custodian" };

impl AssetCode {
	fn to_object_id(&self) -> ObjectId {
		match self {
			AssetCode::Common(symbol) => ObjectId::String(format!("asset-code:common:{}", symbol)),
			AssetCode::Custom(symbol) => ObjectId::String(format!("asset-code:custom:{}", symbol))
		}
	}
	fn to_target(&self) -> Target {
		match self {
			AssetCode::Common(sym) => Target::String(format!("common:{}", sym)),
			AssetCode::Custom(sym) => Target::String(format!("custom:{}", sym)),
		}
	}
	fn from_target(target: Option<&Target>) -> Self {
		if let Some(&Target::String(ref s)) = target {
			if s.starts_with("common:") {
				AssetCode::Common(s["common:".len()..].to_string())
			} else if s.starts_with("custom:") {
				AssetCode::Custom(s["custom:".len()..].to_string())
			} else {
				AssetCode::Custom("UNKNOWN".to_string())
			}
		} else {
			AssetCode::Custom("UNKNOWN".to_string())
		}
	}
}

impl SegmentType {
	fn to_target(&self) -> Target { Target::Number(self.default_index() as u64) }
	fn from_target(target: Option<Target>) -> Self {
		let known_types = Self::known_types();
		let i = if let Some(Target::Number(n)) = target { n as usize } else { known_types.len() };
		if i < known_types.len() { known_types[i] } else { SegmentType::Unknown }
	}
}

fn amount_to_target(amount: Amount) -> Target {
	Target::String(format!("{}", amount))
}

fn amount_from_target(target: Option<Target>, fallback: Amount) -> Amount {
	if let Some(Target::String(s)) = target {
		s.parse::<f64>().unwrap_or(fallback)
	} else {
		fallback
	}
}

fn lot_id_to_object_id(lot_id: LotId) -> ObjectId { ObjectId::String(format!("lot-{}", lot_id)) }

fn lot_id_from_object_id(object_id: &ObjectId) -> LotId {
	if let ObjectId::String(s) = object_id {
		s["lot-".len()..].parse::<LotId>().unwrap_or(0)
	} else {
		LotId::from(0 as LotId)
	}
}

pub fn connect_tmp() -> impl Link + Clone + Debug + Send + 'static {
	let mut folder = std::env::temp_dir();
	folder.push(format!("chad-core-{}", rand::random::<u32>()));
	connect(&folder)
}

pub fn connect(data_dir: &Path) -> impl Link + Clone + Debug + Send + 'static {
	let echo = Echo::connect("link-data", data_dir);
	let (tx, rx) = channel();
	thread::spawn(move || {
		for msg in rx {
			match msg {
				AssignAsset(asset_code, segment_type) => {
					echo.write(|scope| {
						let object_id = asset_code.to_object_id();
						scope.write_object_properties(&object_id, vec![(&ATTR_ASSET_TYPE, segment_type.to_target())])
					}).expect("Write AssignAsset");
				}
				UpdateLot { lot_id, asset_code, share_count, custodian, share_price } => {
					echo.write(|scope| {
						{
							let object_id = asset_code.to_object_id();
							scope.write_object_properties(&object_id, vec![(&ATTR_ASSET_PRICE, amount_to_target(share_price))])
						}
						{
							let object_id = lot_id_to_object_id(lot_id);
							scope.write_object_properties(&object_id, vec![
								(&ATTR_LOT_ASSET, asset_code.to_target()),
								(&ATTR_LOT_CUSTODIAN, Target::String(custodian.to_string())),
								(&ATTR_LOT_SHARES, amount_to_target(share_count)),
							]);
						}
					}).expect("Write UpdateLot");
				}
				UpdatePrice(asset_code, price) => {
					echo.write(|scope| {
						let object_id = asset_code.to_object_id();
						scope.write_object_properties(&object_id, vec![(&ATTR_ASSET_PRICE, amount_to_target(price))])
					}).expect("Write UpdatePrice");
				}
				RecentPortfolio(response) => {
					let chamber = echo.chamber().expect("Chamber from echo");
					let (tx, rx) = channel();
					thread::spawn(move || {
						for msg in rx {
							match msg {
								PortfolioMsg::Lots(reply) => {
									let lot_object_ids = chamber.objects_with_point(&ATTR_LOT_SHARES).expect("Read lot object_ids");
									let lots = lot_object_ids.iter().map(|object_id| {
										let targets = chamber.targets_at_object_points(object_id, vec![
											&ATTR_LOT_SHARES,
											&ATTR_LOT_ASSET,
											&ATTR_LOT_CUSTODIAN,
										]);
										let asset_code = AssetCode::from_target(targets.get(&ATTR_LOT_ASSET));
										Lot {
											lot_id: lot_id_from_object_id(object_id),
											asset_code: asset_code.clone(),
											share_count: amount_from_target(targets.get(&ATTR_LOT_SHARES).cloned(), 1.0),
											custodian: if let Some(&Target::String(ref s)) = targets.get(&ATTR_LOT_CUSTODIAN) {
												Custodian::from(s)
											} else {
												Custodian::from("sovereign")
											},
											share_price: {
												let object_id = asset_code.to_object_id();
												let target = chamber.target_at_object_point_or_none(&object_id, &ATTR_ASSET_PRICE);
												amount_from_target(target, 1.0)
											},
											segment: {
												let object_id = asset_code.to_object_id();
												let target = chamber.target_at_object_point_or_none(&object_id, &ATTR_ASSET_TYPE);
												SegmentType::from_target(target)
											},
										}
									}).collect();
									reply.send(lots).expect("Reply to Lots");
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

#[derive(Clone, Debug)]
struct SenderLink {
	tx: Sender<LinkMsg>
}

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
