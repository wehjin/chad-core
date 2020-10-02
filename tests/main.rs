use chad_core::chad::Chad;
use chad_core::core::{DriftReport, Lot, SquadMember};

const OWNER: u64 = 1000;
const SQUAD_ID: u64 = 2000;
const SQUAD_NAME: &str = "Blue";
const SYMBOL1: &str = "KO";
const SYMBOL2: &str = "PEP";
const LOT_ID1: u64 = 3000;
const LOT_ID2: u64 = 3001;
const ACCOUNT1: &str = "main";
const SHARES1: f64 = 10.0;
const PRICE1: f64 = 5.0;
const UNSPENT1: f64 = 100.0;

#[test]
fn set_unspent_updates_squad() {
	let chad = Chad::connect_tmp();
	chad.add_squad(SQUAD_ID, SQUAD_NAME, OWNER);
	chad.set_unspent(SQUAD_ID, UNSPENT1);
	let squad = chad.snap().squads(OWNER).first().cloned().expect("First squad");
	assert_eq!(UNSPENT1, squad.unspent);
}

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
fn add_member_adds_member_to_squad_and_sets_price_at_symbol() {
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
fn add_member_places_new_member_at_index_0() {
	let chad = Chad::connect_tmp();
	chad.add_squad(SQUAD_ID, SQUAD_NAME, OWNER);
	chad.add_member(SQUAD_ID, SYMBOL1, PRICE1);
	chad.add_member(SQUAD_ID, SYMBOL2, PRICE1);
	let squads = chad.snap().squads(OWNER);
	let squad = squads.first().expect("First squad");
	let symbols = squad.members.iter().map(|it| it.symbol.to_owned()).collect::<Vec<_>>();
	assert_eq!(vec![SYMBOL2.to_string(), SYMBOL1.to_string()], symbols);
}

#[test]
fn add_lot_registers_a_lot() {
	let chad = Chad::connect_tmp();
	chad.add_squad(SQUAD_ID, SQUAD_NAME, OWNER);
	chad.add_member(SQUAD_ID, SYMBOL1, PRICE1);
	chad.add_lot(SQUAD_ID, LOT_ID1, SYMBOL1, ACCOUNT1, SHARES1);
	let squads = chad.snap().squads(OWNER);
	let squad = squads.first().expect("First squad");
	let lots = squad.lots.first().expect("First lot");
	assert_eq!(&Lot {
		squad_id: SQUAD_ID,
		id: LOT_ID1,
		symbol: SYMBOL1.to_string(),
		account: ACCOUNT1.to_string(),
		shares: SHARES1,
	}, lots);
}

#[test]
fn squad_shares_match_inputs() {
	let chad = Chad::connect_tmp();
	chad.add_squad(SQUAD_ID, SQUAD_NAME, OWNER);
	chad.add_member(SQUAD_ID, SYMBOL1, PRICE1);
	chad.add_lot(SQUAD_ID, LOT_ID1, SYMBOL1, ACCOUNT1, SHARES1);
	let squads = chad.snap().squads(OWNER);
	let squad = squads.first().expect("First squad");
	let shares = squad.symbol_shares();
	assert_eq!(SHARES1, shares[SYMBOL1])
}

#[test]
fn squad_produces_drift_reports() {
	let chad = Chad::connect_tmp();
	chad.add_squad(SQUAD_ID, SQUAD_NAME, OWNER);
	chad.add_member(SQUAD_ID, SYMBOL1, 1.0);
	chad.add_lot(SQUAD_ID, LOT_ID1, SYMBOL1, ACCOUNT1, 3.0);
	chad.add_member(SQUAD_ID, SYMBOL2, 1.0);
	chad.add_lot(SQUAD_ID, LOT_ID2, SYMBOL2, ACCOUNT1, 2.0);
	let squads = chad.snap().squads(OWNER);
	let squad = squads.first().expect("First squad");
	let reports = squad.drift_reports();
	let drifts_shares = reports.iter().map(|it| (it.drift_amount(), it.drift_shares())).collect::<Vec<_>>();
	assert_eq!(vec![(1.0, Some(1.0)), (-1.0, Some(-1.0))], drifts_shares);
	assert_eq!(vec![
		DriftReport {
			member: SquadMember {
				squad_id: SQUAD_ID,
				symbol: SYMBOL2.to_string(),
				price: 1.0,
			},
			rank: 1,
			market_value: 2.0,
			target_portion: 0.2,
			target_value: 1.0,
		},
		DriftReport {
			member: SquadMember {
				squad_id: SQUAD_ID,
				symbol: SYMBOL1.to_string(),
				price: 1.0,
			},
			rank: 2,
			market_value: 3.0,
			target_portion: 0.8,
			target_value: 4.0,
		}
	], reports);
}
