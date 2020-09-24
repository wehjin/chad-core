use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use echo_lib::{Chamber, Echo, ObjectId, Point, Target};

pub fn connect_tmp() -> Sender<ChadAction> {
	let mut path = std::env::temp_dir();
	path.push(format!("{}", rand::random::<u32>()));
	let (tx, rx) = channel::<ChadAction>();
	thread::spawn(move || {
		let echo = Echo::connect("chad", &path);
		for action in rx {
			handle_action(action, &echo)
		}
	});
	tx
}

#[derive(Clone, Debug)]
pub enum ChadAction {
	AddSquad { id: u64, name: String, owner: u64 },
	AddMember { squad_id: u64, symbol: String },
	AddLot { id: u64, squad_id: u64, symbol: String, shares: f64, account: String },
	Snap(Sender<Sender<SnapSearch>>),
}

#[derive(Clone, Debug)]
pub enum SnapSearch {}

fn handle_action(action: ChadAction, echo: &Echo) {
	match action {
		ChadAction::AddSquad { id, name, owner } => add_squad(id, name, owner, echo),
		ChadAction::AddMember { squad_id, symbol } => add_member(squad_id, symbol, echo),
		ChadAction::AddLot { .. } => {}
		ChadAction::Snap(_) => {}
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
