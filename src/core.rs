use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Squad {
	pub id: u64,
	pub name: String,
	pub owner: u64,
	pub members: Vec<SquadMember>,
	pub lots: Vec<Lot>,
	pub prices: HashMap<String, f64>,
	pub unspent: f64,
}

impl Squad {
	pub fn shares(&self) -> HashMap<String, f64> {
		let mut hashmap = HashMap::new();
		for lot in &self.lots {
			let previous = *hashmap.get(&lot.symbol).unwrap_or(&0.0);
			let next = previous + lot.shares;
			hashmap.insert(lot.symbol.to_owned(), next);
		}
		hashmap
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SquadMember {
	pub squad_id: u64,
	pub symbol: String,
	pub price: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lot {
	pub squad_id: u64,
	pub id: u64,
	pub symbol: String,
	pub account: String,
	pub shares: f64,
}