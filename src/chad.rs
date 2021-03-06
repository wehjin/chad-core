use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use echo_lib::{Chamber, Echo, ObjectId, Target};

use crate::{chamber, oid, point, target};
use crate::core::{Lot, Squad, SquadMember};

#[derive(Clone, Debug)]
enum Action {
	Snap(Sender<Sender<SearchAction>>),
	AddSquad { id: u64, name: String, owner: u64 },
	AddMember { squad_id: u64, symbol: String, price: f64 },
	AddLot { squad_id: u64, id: u64, symbol: String, shares: f64, account: String },
	DelLot { squad_id: u64, lot_id: u64 },
	SetUnspent { squad_id: u64, unspent: f64 },
}

#[derive(Clone, Debug)]
pub struct Chad { tx: Sender<Action> }

impl Chad {
	pub fn connect_tmp() -> Self {
		let mut path = std::env::temp_dir();
		path.push(format!("{}", rand::random::<u32>()));
		Self::connect(&path)
	}

	pub fn connect(data_dir: &Path) -> Self {
		let echo = Echo::connect("chad", &data_dir);
		let (tx, rx) = channel::<Action>();
		thread::spawn(move || for action in rx {
			handle_action(action, &echo)
		});
		Chad { tx }
	}

	pub fn snap(&self) -> Snap {
		let (tx, rx) = channel();
		self.tx.send(Action::Snap(tx)).expect("Send Snap");
		let sender = rx.recv().expect("Recv SnapSearch");
		Snap { tx: sender }
	}

	pub fn add_squad(&self, id: u64, name: &str, owner: u64) {
		let action = Action::AddSquad { id, name: name.to_string(), owner };
		self.tx.send(action).expect("Send AddSquad");
	}

	pub fn set_unspent(&self, squad_id: u64, unspent: f64) {
		let action = Action::SetUnspent { squad_id, unspent };
		self.tx.send(action).expect("Send SetUnspent");
	}

	pub fn add_member(&self, squad_id: u64, symbol: &str, price: f64) {
		let action = Action::AddMember { squad_id, symbol: symbol.to_string(), price };
		self.tx.send(action).expect("Send AddMember");
	}

	pub fn add_lot(&self, squad_id: u64, id: u64, symbol: &str, account: &str, shares: f64) {
		let account = account.to_string();
		let symbol = symbol.to_string();
		let action = Action::AddLot { squad_id, id, symbol, shares, account };
		self.tx.send(action).expect("Send AddLot");
	}

	pub fn del_lot(&self, squad_id: u64, lot_id: u64) {
		let action = Action::DelLot { squad_id, lot_id };
		self.tx.send(action).expect("Send DelLot");
	}
}

fn handle_action(action: Action, echo: &Echo) {
	match action {
		Action::Snap(reply) => {
			let chamber = echo.chamber().expect("Snap chamber");
			let (tx, rx) = channel();
			thread::spawn(move || for action in rx {
				handle_search(action, &chamber);
			});
			reply.send(tx).expect("Send Snap reply");
		}
		Action::AddSquad { id, name, owner } => add_squad(id, name, owner, echo),
		Action::AddMember { squad_id, symbol, price } => add_member(squad_id, symbol, price, echo),
		Action::AddLot { squad_id, id, symbol, shares, account } => {
			echo.write(|scope| {
				let object_id = ObjectId::String(id.to_string());
				let properties = vec![
					(&point::LOT_ACCOUNT, Target::String(account.to_string())),
					(&point::LOT_SHARES, Target::String(shares.to_string())),
					(&point::LOT_SQUAD, Target::String(squad_id.to_string())),
					(&point::LOT_SYMBOL, Target::String(symbol.to_string()))
				];
				scope.write_object_properties(&object_id, properties);
			}).expect("Write AddLot");
		}
		Action::DelLot { lot_id, .. } => {
			echo.write(|scope| {
				let object_id = ObjectId::String(lot_id.to_string());
				scope.write_object_properties(&object_id, vec![(&point::LOT_SQUAD, Target::String("0".to_string())), ])
			}).expect("Write DelLot");
		}
		Action::SetUnspent { squad_id, unspent } => {
			let chamber = echo.chamber().expect("Chamber in SetUnspent");
			let previous = chamber::unspent(squad_id, &chamber);
			if unspent != previous {
				echo.write(|scope| {
					scope.write_object_properties(
						&oid::from_squad_id(squad_id),
						vec![(&point::SQUAD_UNSPENT, target::from_f64(unspent))],
					)
				}).expect("Write SetUnspent")
			}
		}
	}
}

fn handle_search(search: SearchAction, chamber: &Chamber) {
	match search {
		SearchAction::Squads { owner, reply } => {
			let squads = squads(owner, &chamber);
			reply.send(squads).expect("Send Squads");
		}
	}
}

fn add_squad(id: u64, name: String, owner: u64, echo: &Echo) {
	echo.write(|scope| {
		let object_id = ObjectId::String(id.to_string());
		let properties = vec![
			(&point::SQUAD_NAME, Target::String(name.to_string())),
			(&point::SQUAD_OWNER, Target::String(owner.to_string()))
		];
		scope.write_object_properties(&object_id, properties);
	}).expect("Write AddSquad")
}

fn add_member(squad_id: u64, symbol: String, price: f64, echo: &Echo) {
	let member_id = {
		let mut hasher = DefaultHasher::default();
		squad_id.hash(&mut hasher);
		symbol.hash(&mut hasher);
		hasher.finish()
	};
	let chamber = echo.chamber().expect("Chamber");
	let previous_price = price_for_symbol(&symbol, &chamber);
	let mut squad_members = squad_members(squad_id, &chamber);
	if !squad_members.contains(&member_id) {
		squad_members.insert(0, member_id);
		echo.write(|scope| {
			if Some(price) != previous_price {
				scope.write_object_properties(
					&symbol_oid(&symbol),
					vec![
						(&point::PRICE_F64, target::from_f64(price))
					],
				);
			}
			scope.write_object_properties(
				&ObjectId::String(member_id.to_string()),
				vec![
					(&point::MEMBER_SQUAD, Target::String(squad_id.to_string())),
					(&point::MEMBER_SYMBOL, Target::String(symbol.to_string())),
				],
			);
			scope.write_object_properties(
				&oid::from_squad_id(squad_id),
				vec![(&point::SQUAD_MEMBERS, Target::String(squad_members.iter().map(u64::to_string).collect::<Vec<_>>().join(":")))],
			);
		}).expect("Write add_member");
	}
}

fn squads(owner: u64, chamber: &Chamber) -> Vec<Squad> {
	let squads_ids = chamber.objects_with_property(&point::SQUAD_OWNER, &Target::String(owner.to_string())).expect("Squad-ids");
	squads_ids.into_iter().map(|oid| {
		let squad_id = id(&oid);
		let targets = chamber.targets_at_object_points(&oid, vec![
			&point::SQUAD_NAME,
			&point::SQUAD_MEMBERS,
			&point::SQUAD_UNSPENT,
		]);
		let name = targets.get(&point::SQUAD_NAME).map(string).unwrap_or_else(|| format!("Squad-{}", squad_id));
		let member_ids = targets.get(&point::SQUAD_MEMBERS).map(target::to_member_ids).unwrap_or_else(Vec::new);
		let members: Vec<SquadMember> = member_ids.into_iter().map(|member_id| squad_member(member_id, chamber)).collect();
		let unspent = targets.get(&point::SQUAD_UNSPENT).cloned().map(target::to_f64).unwrap_or(0.0);
		let lots = lots(squad_id, chamber);
		let prices = member_prices(&members, chamber);
		Squad { id: squad_id, name, owner, members, lots, prices, unspent }
	}).collect()
}

fn member_prices(members: &Vec<SquadMember>, chamber: &Chamber) -> HashMap<String, f64> {
	let mut prices = HashMap::new();
	for member in members {
		let string = &member.symbol;
		let price = price_for_symbol(string, chamber).expect("Price exists");
		prices.insert(string.to_owned(), price);
	}
	prices
}

fn price_for_symbol(symbol: &String, chamber: &Chamber) -> Option<f64> {
	let oid = symbol_oid(symbol);
	let price_target = chamber.target_at_object_point_or_none(&oid, &point::PRICE_F64);
	price_target.map(target::to_f64)
}


fn symbol_oid(symbol: &String) -> ObjectId {
	ObjectId::String(symbol.to_string())
}

fn lots(squad_id: u64, chamber: &Chamber) -> Vec<Lot> {
	let object_ids = chamber.objects_with_property(&point::LOT_SQUAD, &Target::String(squad_id.to_string())).expect("lot objects");
	object_ids.into_iter().map(|oid| {
		let id = id(&oid);
		let targets = chamber.targets_at_object_points(&oid, vec![&point::LOT_SQUAD, &point::LOT_SYMBOL, &point::LOT_SHARES, &point::LOT_ACCOUNT]);
		let symbol = targets.get(&point::LOT_SYMBOL).map(string).unwrap_or_else(String::new);
		let account = targets.get(&point::LOT_ACCOUNT).map(string).unwrap_or_else(String::new);
		let shares = targets.get(&point::LOT_SHARES).map(shares).unwrap_or(0.0);
		Lot { squad_id, id, symbol, account, shares }
	}).collect()
}

fn shares(target: &Target) -> f64 {
	target.as_str().parse().unwrap_or(0.0)
}

fn string(target: &Target) -> String {
	target.as_str().to_string()
}

fn id(oid: &ObjectId) -> u64 {
	match oid {
		ObjectId::Unit => panic!("Unexpected ObjectId::Unit"),
		ObjectId::String(ref s) => s.parse::<u64>().expect("s is u64"),
	}
}

fn squad_member(member_id: u64, chamber: &Chamber) -> SquadMember {
	let member_oid = ObjectId::String(member_id.to_string());
	let targets = chamber.targets_at_object_points(
		&member_oid,
		vec![&point::MEMBER_SYMBOL, &point::MEMBER_SQUAD],
	);
	let squad_id = targets.get(&point::MEMBER_SQUAD).expect("Squad target").as_str().parse::<u64>().expect("u64");
	let symbol = targets.get(&point::MEMBER_SYMBOL).expect("Symbol target").as_str().to_string();
	let price = price_for_symbol(&symbol, chamber).expect("price exists for member");
	SquadMember { squad_id, symbol, price }
}

fn squad_members(squad_id: u64, chamber: &Chamber) -> Vec<u64> {
	let target = chamber.target_at_object_point_or_none(
		&ObjectId::String(squad_id.to_string()),
		&point::SQUAD_MEMBERS,
	);
	target.map(|ref it| target::to_member_ids(it)).unwrap_or(Vec::new())
}

#[derive(Clone, Debug)]
enum SearchAction {
	Squads { owner: u64, reply: Sender<Vec<Squad>> }
}

#[derive(Clone, Debug)]
pub struct Snap { tx: Sender<SearchAction> }

impl Snap {
	pub fn squads(&self, owner: u64) -> Vec<Squad> {
		let (tx, rx) = channel();
		self.tx.send(SearchAction::Squads { owner, reply: tx }).expect("Send Squads search");
		rx.recv().expect("Recv Squads reply")
	}
}
