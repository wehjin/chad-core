pub use segment_type::*;

mod segment_type;

/// Identifies an Asset.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum AssetCode {
	Common(String),
	Custom(String),
}

/// A quantity of something.
pub type Amount = f64;

/// Identifies the custodian of a Lot.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Custodian {
	Unknown,
	Custom(String),
}

/// Identifies which account within a custodian holds a Lot.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Account {
	Main,
	Custom(String),
}

/// Identifiers a Lot.
pub type LotId = u64;


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Squad {
	pub id: u64,
	pub name: String,
	pub owner: u64,
	pub members: Vec<SquadMember>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquadMember {
	pub squad_id: u64,
	pub symbol: String,
}
