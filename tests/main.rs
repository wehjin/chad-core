use chad_core::chad::Chad;
use chad_core::core::{Lot, SquadMember};

const OWNER: u64 = 1000;
const SQUAD_ID: u64 = 2000;
const SQUAD_NAME: &str = "Blue";
const SYMBOL1: &str = "KO";
const LOT_ID: u64 = 3000;
const ACCOUNT1: &str = "main";
const SHARES1: f64 = 10.0;
const PRICE1: f64 = 5.0;

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
fn add_member_appends_member_to_squad_and_sets_price_at_symbol() {
	let chad = Chad::connect_tmp();
	chad.add_squad(SQUAD_ID, SQUAD_NAME, OWNER);
	chad.add_member(SQUAD_ID, SYMBOL1, PRICE1);
	let squads = chad.snap().squads(OWNER);
	let squad = squads.first().expect("First squad");
	let member = squad.members.first().expect("First member");
	assert_eq!(&SquadMember { squad_id: SQUAD_ID, symbol: SYMBOL1.to_string(), price: PRICE1 }, member);
	let prices = &squad.prices;
	assert_eq!(PRICE1, prices[SYMBOL1]);
}

#[test]
fn add_lot_registers_a_lot() {
	let chad = Chad::connect_tmp();
	chad.add_squad(SQUAD_ID, SQUAD_NAME, OWNER);
	chad.add_member(SQUAD_ID, SYMBOL1, PRICE1);
	chad.add_lot(SQUAD_ID, LOT_ID, SYMBOL1, ACCOUNT1, SHARES1);
	let squads = chad.snap().squads(OWNER);
	let squad = squads.first().expect("First squad");
	let lots = squad.lots.first().expect("First lot");
	assert_eq!(&Lot {
		squad_id: SQUAD_ID,
		id: LOT_ID,
		symbol: SYMBOL1.to_string(),
		account: ACCOUNT1.to_string(),
		shares: SHARES1,
	}, lots);
}