use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use echo_lib::{Chamber, Echo, ObjectId, Point, Target};

use Msg::{AssignAsset, RecentPortfolio, UpdateLot, UpdatePrice};

use crate::core::{Account, Amount, AssetCode, Custodian, LotId, SegmentType};
use crate::portfolio::link::{PortfolioLink, PortfolioMsg};
use crate::portfolio::lot::Lot;

/// Top level link to a portfolio.
#[derive(Clone, Debug)]
pub struct StorageLink { tx: Sender<Msg> }

impl StorageLink {
	pub fn assign_asset(&self, asset_code: &AssetCode, segment_type: SegmentType) {
		self.tx.send(AssignAsset(asset_code.clone(), segment_type)).expect("AssignAsset");
	}

	pub fn update_lot(&self, lot_id: LotId, asset_code: &AssetCode, share_count: Amount, custodian: &Custodian, share_price: Amount) {
		let asset_code = asset_code.to_owned();
		let custodian = custodian.to_owned();
		self.tx.send(UpdateLot { lot_id, asset_code, share_count, custodian, share_price }).expect("UpdateLot");
	}

	pub fn price_asset(&self, asset_code: &AssetCode, price: Amount) {
		self.tx.send(UpdatePrice(asset_code.to_owned(), price)).expect("UpdatePrice");
	}

	pub fn latest_portfolio(&self) -> PortfolioLink {
		let (tx, rx) = channel();
		self.tx.send(RecentPortfolio(tx)).expect("RecentPortfolio");
		let tx = rx.recv().expect("Recv RecentPortfolio");
		PortfolioLink { tx }
	}
}

enum Msg {
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

const CUSTODIAN_UNKNOWN_STRING: &str = "--unknown--";

impl Custodian {
	fn to_target(&self) -> Target {
		match self {
			Custodian::Unknown => Target::String(CUSTODIAN_UNKNOWN_STRING.to_string()),
			Custodian::Custom(s) => Target::String(s.to_string())
		}
	}
	fn from_target(target: Option<Target>) -> Self {
		if let Some(target) = target {
			match target {
				Target::String(s) => if s == CUSTODIAN_UNKNOWN_STRING {
					Custodian::Unknown
				} else {
					Custodian::Custom(s.to_string())
				},
				_ => Custodian::Unknown
			}
		} else {
			Custodian::Unknown
		}
	}
}

const COMMON_PREFIX: &str = "asset-code:common:";
const CUSTOM_PREFIX: &str = "asset-code:custom:";

impl AssetCode {
	fn to_object_id(&self) -> ObjectId {
		match self {
			AssetCode::Common(symbol) => ObjectId::String(format!("{}{}", COMMON_PREFIX, symbol)),
			AssetCode::Custom(symbol) => ObjectId::String(format!("{}{}", CUSTOM_PREFIX, symbol))
		}
	}
	fn from_object_id(oid: &ObjectId) -> Self {
		if let ObjectId::String(s) = oid {
			if s.starts_with(COMMON_PREFIX) {
				AssetCode::Common(s[COMMON_PREFIX.len()..].to_string())
			} else if s.starts_with(CUSTOM_PREFIX) {
				AssetCode::Custom(s[CUSTOM_PREFIX.len()..].to_string())
			} else {
				AssetCode::Custom("UNKNOWN".to_string())
			}
		} else {
			AssetCode::Custom("UNKNOWN".to_string())
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
		let known_types = Self::known();
		let i = if let Some(Target::Number(n)) = target {
			n as usize
		} else {
			known_types.len()
		};
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

pub fn connect_storage_tmp() -> StorageLink {
	let mut folder = std::env::temp_dir();
	folder.push(format!("chad-core-{}", rand::random::<u32>()));
	connect_storage(&folder)
}

pub fn connect_storage(data_dir: &Path) -> StorageLink {
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
								(&ATTR_LOT_CUSTODIAN, custodian.to_target()),
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
									let lots = read_lots(&chamber);
									reply.send(lots).expect("Reply to Lots");
								}
								PortfolioMsg::AssetAssignments(reply) => {
									let assignments = read_assignments(&chamber);
									reply.send(assignments).expect("Reply to AssetAssignments");
								}
								PortfolioMsg::Prices(reply) => {
									let prices = read_prices(&chamber);
									reply.send(prices).expect("Reply to Prices");
								}
							}
						}
					});
					response.send(tx).expect("Reply to RecentPortfolio")
				}
			}
		}
	});
	StorageLink { tx }
}

fn read_prices(chamber: &Chamber) -> HashMap<AssetCode, Amount> {
	let mut hash_map = HashMap::new();
	let asset_oids = chamber.objects_with_point(&ATTR_ASSET_PRICE).expect("Read asset-prices");
	for oid in &asset_oids {
		let asset_code = AssetCode::from_object_id(oid);
		let option = chamber.target_at_object_point_or_none(oid, &ATTR_ASSET_PRICE);
		let price = amount_from_target(option, 1.0);
		hash_map.insert(asset_code, price);
	};
	hash_map
}

fn read_assignments(chamber: &Chamber) -> HashMap<AssetCode, SegmentType> {
	let mut hash_map = HashMap::new();
	let asset_oids = chamber.objects_with_point(&ATTR_ASSET_TYPE).expect("Read asset-types");
	for oid in &asset_oids {
		let asset_code = AssetCode::from_object_id(oid);
		let segment_type = SegmentType::from_target(chamber.target_at_object_point_or_none(oid, &ATTR_ASSET_TYPE));
		hash_map.insert(asset_code, segment_type);
	}
	hash_map
}

fn read_lots(chamber: &Chamber) -> Vec<Lot> {
	let lot_object_ids = chamber.objects_with_point(&ATTR_LOT_SHARES).expect("Read lot object_ids");
	lot_object_ids.iter().map(|object_id| {
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
			custodian: Custodian::from_target(targets.get(&ATTR_LOT_CUSTODIAN).cloned()),
			account: Account::Main,
		}
	}).collect()
}

