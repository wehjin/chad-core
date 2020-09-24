use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use echo_lib::{Chamber, Echo, ObjectId, Point, Target};

#[derive(Clone, Debug)]
enum ChadAction {
	AddSquad { id: u64, name: String, owner: u64 },
	AddMember { squad_id: u64, symbol: String },
	AddLot { id: u64, squad_id: u64, symbol: String, shares: f64, account: String },
	Snap(Sender<Sender<SearchAction>>),
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

	pub fn add_squad(&self, id: u64, name: &str, owner: u64) {
		self.tx.send(ChadAction::AddSquad { id, name: name.to_string(), owner }).expect("Send AddSquad");
	}

	pub fn snap(&self) -> Snap {
		let (tx, rx) = channel();
		self.tx.send(ChadAction::Snap(tx)).expect("Send Snap");
		let sender = rx.recv().expect("Recv SnapSearch");
		Snap { tx: sender }
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Squad {
	pub id: u64,
	pub name: String,
	pub owner: u64,
	pub members: Vec<u64>,
}

fn handle_action(action: ChadAction, echo: &Echo) {
	match action {
		ChadAction::AddSquad { id, name, owner } => add_squad(id, name, owner, echo),
		ChadAction::AddMember { squad_id, symbol } => add_member(squad_id, symbol, echo),
		ChadAction::AddLot { .. } => {}
		ChadAction::Snap(reply) => {
			let chamber = echo.chamber().expect("Snap chamber");
			let (tx, rx) = channel();
			thread::spawn(move || for action in rx {
				handle_search(action, &chamber);
			});
			reply.send(tx).expect("Send Snap reply");
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
	squads_ids.into_iter().map(|object_id| {
		let id = match &object_id {
			ObjectId::Unit => panic!("Unexpected object_id"),
			ObjectId::String(s) => s.parse::<u64>().expect("u64 in object-id"),
		};
		let targets = chamber.targets_at_object_points(
			&object_id,
			vec![&SQUAD_NAME, &SQUAD_MEMBERS],
		);
		let name = targets.get(&SQUAD_NAME).map(|it| it.as_str().to_owned()).unwrap_or_else(|| format!("Squad-{}", id));
		let members = targets.get(&SQUAD_MEMBERS).map(|it| {
			it.as_str()
				.split(":")
				.map(|it| it.parse::<u64>().expect("member-id"))
				.collect::<Vec<_>>()
		}).unwrap_or_else(|| Vec::new());
		Squad { id, name, owner, members }
	}).collect()
}

fn squad_members(squad_id: u64, chamber: &Chamber) -> Vec<u64> {
	chamber.target_at_object_point_or_none(
		&ObjectId::String(squad_id.to_string()),
		&SQUAD_MEMBERS,
	).map(|it| it.as_str()
		.split(":")
		.map(|it| it.parse::<u64>().expect("member-id"))
		.collect::<Vec<_>>()
	).unwrap_or(Vec::new())
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

