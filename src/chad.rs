use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use echo_lib::{Chamber, Echo, ObjectId, Point, Target};

use crate::core::{Lot2, Squad, SquadMember};

#[derive(Clone, Debug)]
enum ChadAction {
	Snap(Sender<Sender<SearchAction>>),
	AddSquad { id: u64, name: String, owner: u64 },
	AddMember { squad_id: u64, symbol: String },
	AddLot { squad_id: u64, id: u64, symbol: String, shares: f64, account: String },
}

#[derive(Clone, Debug)]
pub struct Chad { tx: Sender<ChadAction> }

impl Chad {
	pub fn connect_tmp() -> Self {
		let mut path = std::env::temp_dir();
		path.push(format!("{}", rand::random::<u32>()));
		let echo = Echo::connect("chad", &path);
		let (tx, rx) = channel::<ChadAction>();
		thread::spawn(move || for action in rx {
			handle_action(action, &echo)
		});
		Chad { tx }
	}

	pub fn snap(&self) -> Snap {
		let (tx, rx) = channel();
		self.tx.send(ChadAction::Snap(tx)).expect("Send Snap");
		let sender = rx.recv().expect("Recv SnapSearch");
		Snap { tx: sender }
	}

	pub fn add_squad(&self, id: u64, name: &str, owner: u64) {
		let action = ChadAction::AddSquad { id, name: name.to_string(), owner };
		self.tx.send(action).expect("Send AddSquad");
	}

	pub fn add_member(&self, squad_id: u64, symbol: &str) {
		let action = ChadAction::AddMember { squad_id, symbol: symbol.to_string() };
		self.tx.send(action).expect("Send AddMember");
	}

	pub fn add_lot(&self, squad_id: u64, id: u64, symbol: &str, account: &str, shares: f64) {
		let account = account.to_string();
		let symbol = symbol.to_string();
		let action = ChadAction::AddLot { squad_id, id, symbol, shares, account };
		self.tx.send(action).expect("Send AddLot");
	}
}

fn handle_action(action: ChadAction, echo: &Echo) {
	match action {
		ChadAction::Snap(reply) => {
			let chamber = echo.chamber().expect("Snap chamber");
			let (tx, rx) = channel();
			thread::spawn(move || for action in rx {
				handle_search(action, &chamber);
			});
			reply.send(tx).expect("Send Snap reply");
		}
		ChadAction::AddSquad { id, name, owner } => add_squad(id, name, owner, echo),
		ChadAction::AddMember { squad_id, symbol } => add_member(squad_id, symbol, echo),
		ChadAction::AddLot { squad_id, id, symbol, shares, account } => {
			echo.write(|scope| {
				let object_id = ObjectId::String(id.to_string());
				let properties = vec![
					(&LOT_ACCOUNT, Target::String(account.to_string())),
					(&LOT_SHARES, Target::String(shares.to_string())),
					(&LOT_SQUAD, Target::String(squad_id.to_string())),
					(&LOT_SYMBOL, Target::String(symbol.to_string()))
				];
				scope.write_object_properties(&object_id, properties);
			}).expect("Write AddLot");
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
			(&SQUAD_NAME, Target::String(name.to_string())),
			(&SQUAD_OWNER, Target::String(owner.to_string()))
		];
		scope.write_object_properties(&object_id, properties);
	}).expect("Write AddSquad")
}

fn add_member(squad_id: u64, symbol: String, echo: &Echo) {
	let member_id = {
		let mut hasher = DefaultHasher::default();
		squad_id.hash(&mut hasher);
		symbol.hash(&mut hasher);
		hasher.finish()
	};
	let chamber = echo.chamber().expect("Chamber");
	let mut squad_members = squad_members(squad_id, &chamber);
	if !squad_members.contains(&member_id) {
		squad_members.push(member_id);
		echo.write(|scope| {
			scope.write_object_properties(
				&ObjectId::String(member_id.to_string()),
				vec![
					(&MEMBER_SQUAD, Target::String(squad_id.to_string())),
					(&MEMBER_SYMBOL, Target::String(symbol.to_string())),
				],
			);
			scope.write_object_properties(
				&ObjectId::String(squad_id.to_string()),
				vec![(&SQUAD_MEMBERS, Target::String(squad_members.iter().map(u64::to_string).collect::<Vec<_>>().join(":")))],
			);
		}).expect("Write add_member");
	}
}

fn squads(owner: u64, chamber: &Chamber) -> Vec<Squad> {
	let squads_ids = chamber.objects_with_property(&SQUAD_OWNER, &Target::String(owner.to_string())).expect("Squad-ids");
	squads_ids.into_iter().map(|oid| {
		let id = id(&oid);
		let targets = chamber.targets_at_object_points(&oid, vec![&SQUAD_NAME, &SQUAD_MEMBERS]);
		let name = targets.get(&SQUAD_NAME).map(string).unwrap_or_else(|| format!("Squad-{}", id));
		let member_ids = targets.get(&SQUAD_MEMBERS).map(target_to_member_ids).unwrap_or_else(Vec::new);
		let members = member_ids.into_iter().map(|member_id| squad_member(member_id, chamber)).collect();
		let lots = lots(id, chamber);
		Squad { id, name, owner, members, lots }
	}).collect()
}

fn lots(squad_id: u64, chamber: &Chamber) -> Vec<Lot2> {
	let object_ids = chamber.objects_with_property(&LOT_SQUAD, &Target::String(squad_id.to_string())).expect("lot objects");
	object_ids.into_iter().map(|oid| {
		let id = id(&oid);
		let targets = chamber.targets_at_object_points(&oid, vec![&LOT_SQUAD, &LOT_SYMBOL, &LOT_SHARES, &LOT_ACCOUNT]);
		let symbol = targets.get(&LOT_SYMBOL).map(string).unwrap_or_else(String::new);
		let account = targets.get(&LOT_ACCOUNT).map(string).unwrap_or_else(String::new);
		let shares = targets.get(&LOT_SHARES).map(shares).unwrap_or(0.0);
		Lot2 { squad_id, id, symbol, account, shares }
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
	let object_id = ObjectId::String(member_id.to_string());
	let targets = chamber.targets_at_object_points(
		&object_id,
		vec![&MEMBER_SYMBOL, &MEMBER_SQUAD],
	);
	SquadMember {
		squad_id: targets.get(&MEMBER_SQUAD).expect("Squad target").as_str().parse::<u64>().expect("u64"),
		symbol: targets.get(&MEMBER_SYMBOL).expect("Symbol target").as_str().to_string(),
	}
}

fn squad_members(squad_id: u64, chamber: &Chamber) -> Vec<u64> {
	let target = chamber.target_at_object_point_or_none(
		&ObjectId::String(squad_id.to_string()),
		&SQUAD_MEMBERS,
	);
	target.map(|ref it| target_to_member_ids(it)).unwrap_or(Vec::new())
}

fn target_to_member_ids(target: &Target) -> Vec<u64> {
	target.as_str()
		.split(":")
		.map(|it| it.parse::<u64>().expect("member-id"))
		.collect::<Vec<_>>()
}

const SQUAD_NAME: Point = Point::Static { aspect: "Squad", name: "name" };
const SQUAD_OWNER: Point = Point::Static { aspect: "Squad", name: "owner" };
const SQUAD_MEMBERS: Point = Point::Static { aspect: "Squad", name: "members" };
const MEMBER_SQUAD: Point = Point::Static { aspect: "Member", name: "squad" };
const MEMBER_SYMBOL: Point = Point::Static { aspect: "Member", name: "symbol" };
const LOT_SQUAD: Point = Point::Static { aspect: "Lot", name: "squad" };
const LOT_SYMBOL: Point = Point::Static { aspect: "Lot", name: "symbol" };
const LOT_SHARES: Point = Point::Static { aspect: "Lot", name: "shares" };
const LOT_ACCOUNT: Point = Point::Static { aspect: "Lot", name: "account" };


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

