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
	pub fn symbol_shares(&self) -> HashMap<String, f64> {
		let mut hashmap = HashMap::new();
		for lot in &self.lots {
			let previous = *hashmap.get(&lot.symbol).unwrap_or(&0.0);
			let next = previous + lot.shares;
			hashmap.insert(lot.symbol.to_owned(), next);
		}
		hashmap
	}
	pub fn member_portions(&self) -> Vec<f64> {
		let mut sum_weights = 0.0;
		let mut weights = Vec::new();
		for index in 0..self.members.len() {
			let rank = (index + 1) as f64;
			let weight = rank * rank;
			weights.push(weight);
			sum_weights += weight;
		};
		weights.into_iter().map(|it| {
			if sum_weights > 0.0 { it / sum_weights } else { 0.0 }
		}).collect()
	}
	fn member_market_values(&self) -> Vec<f64> {
		let symbol_shares = self.symbol_shares();
		self.members.iter().map(|it| it.market_value(&symbol_shares)).collect()
	}
	pub fn drift_reports(&self) -> Vec<DriftReport> {
		let portions = self.member_portions();
		let market_values = self.member_market_values();
		let sum_targets = market_values.iter().sum::<f64>() + self.unspent;
		let target_values = portions.iter().map(|portion| {
			match sum_targets > 0.0 {
				true => *portion * sum_targets,
				false => 0.0,
			}
		}).collect::<Vec<_>>();
		self.members.clone().into_iter().enumerate().map(|(index, member)| {
			DriftReport {
				member,
				rank: index + 1,
				target_portion: portions[index],
				market_value: market_values[index],
				target_value: target_values[index],
			}
		}).collect()
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct SquadMember {
	pub squad_id: u64,
	pub symbol: String,
	pub price: f64,
}

impl SquadMember {
	pub fn market_value(&self, symbol_shares: &HashMap<String, f64>) -> f64 {
		let shares = *symbol_shares.get(&self.symbol).unwrap_or(&0.0);
		self.price * shares
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lot {
	pub squad_id: u64,
	pub id: u64,
	pub symbol: String,
	pub account: String,
	pub shares: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DriftReport {
	pub member: SquadMember,
	pub rank: usize,
	pub market_value: f64,
	pub target_portion: f64,
	pub target_value: f64,
}

impl DriftReport {
	pub fn symbol(&self) -> &str {
		&self.member.symbol
	}
	pub fn drift_amount(&self) -> f64 {
		self.market_value - self.target_value
	}
	pub fn drift_shares(&self) -> Option<f64> {
		match self.member.price >= 0.0 {
			true => Some(self.drift_amount() / self.member.price),
			false => None
		}
	}
}