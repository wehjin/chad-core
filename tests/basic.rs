extern crate chad_core;

use chad_core::{allocate_amount, Lot, portfolio_link};
use chad_core::prelude::*;

#[test]
fn it_works() {
	let link = portfolio_link();
	let tsla = AssetCode::Common("TSLA".to_string());
	let custodian = "robinhood".to_string();
	link.assign_asset(&tsla, SegmentType::Expo);
	link.price_asset(&tsla, 2000.0);
	link.update_lot(2000, &tsla, 10.0, &custodian, 300.0);
	let portfolio = link.latest_portfolio();
	assert_eq!(portfolio.lots(), vec![
		Lot {
			lot_id: 2000,
			asset_code: tsla.clone(),
			share_count: 10.0,
			custodian: custodian.clone(),
			share_price: 300.0,
			segment: SegmentType::Expo,
		}
	]);
	let segments = portfolio.segments();
	let values = segments.iter().map(Segment::segment_value).collect::<Vec<_>>();
	assert_eq!(values, vec![0.0, 0.0, 0.0, 3000.0, 0.0]);
	let types = segments.iter().map(Segment::segment_type).collect::<Vec<_>>();
	assert_eq!(types, vec![SegmentType::Liquid, SegmentType::Stable, SegmentType::Linear, SegmentType::Expo, SegmentType::Unknown])
}

#[test]
fn allocate_computes_sub_amounts() {
	let allocation = allocate_amount(1.0);
	assert_eq!(allocation, vec![
		(SegmentType::Liquid, 0.064),
		(SegmentType::Stable, 0.096),
		(SegmentType::Linear, 0.240),
		(SegmentType::Expo, 0.600),
		(SegmentType::Unknown, 0.000),
	]);
}
