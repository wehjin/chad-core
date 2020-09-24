use chad_core::chad::Chad;

const OWNER: u64 = 1000;
const SQUAD_ID: u64 = 2000;
const SQUAD_NAME: &str = "Blue";

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