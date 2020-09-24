use chad_core::chad::Chad;
use chad_core::core::SquadMember;

const OWNER: u64 = 1000;
const SQUAD_ID: u64 = 2000;
const SQUAD_NAME: &str = "Blue";
const SYMBOL1: &str = "KO";

#[test]
fn add_squad_produces_a_squad_in_next_snap() {
	let chad = Chad::connect_tmp();
	chad.add_squad(SQUAD_ID, SQUAD_NAME, OWNER);
	let squads = chad.snap().squads(OWNER);
	let squad = squads.first().expect("First squad");
	assert_eq!(SQUAD_ID, squad.id);
	assert_eq!(SQUAD_NAME, squad.name);
	assert_eq!(OWNER, squad.owner);
	assert!(squad.members.is_empty());
}

#[test]
fn add_member_appends_member_to_squad() {
	let chad = Chad::connect_tmp();
	chad.add_squad(SQUAD_ID, SQUAD_NAME, OWNER);
	chad.add_member(SQUAD_ID, SYMBOL1);
	let squads = chad.snap().squads(OWNER);
	let squad = squads.first().expect("First squad");
	let member = squad.members.first().expect("First member");
	assert_eq!(&SquadMember { squad_id: SQUAD_ID, symbol: SYMBOL1.to_string() }, member);
}