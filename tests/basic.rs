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
	let segments = portfolio.segment_reports();
	let values = segments.iter().map(SegmentReport::segment_value).collect::<Vec<_>>();
	assert_eq!(values, vec![0.0, 0.0, 0.0, 3000.0, 0.0]);
	let types = segments.iter().map(SegmentReport::segment_type).collect::<Vec<_>>();
	assert_eq!(types, vec![SegmentType::Liquid, SegmentType::Stable, SegmentType::Linear, SegmentType::Expo, SegmentType::Unknown])
}

#[test]
fn portfolio_computes_segment_drift_values() {
	let link = portfolio_link();
	let expo_asset = AssetCode::Common("EXPO".to_string());
	let linear_asset = AssetCode::Common("LINEAR".to_string());
	let stable_asset = AssetCode::Common("STABLE".to_string());
	let liquid_asset = AssetCode::Common("LIQUID".to_string());
	link.assign_asset(&expo_asset, SegmentType::Expo);
	link.assign_asset(&linear_asset, SegmentType::Linear);
	link.assign_asset(&stable_asset, SegmentType::Stable);
	link.assign_asset(&liquid_asset, SegmentType::Liquid);
	let custodian = "sovereign".to_string();
	link.update_lot(10, &expo_asset, 1.0, &custodian, 0.598);
	link.update_lot(11, &linear_asset, 1.0, &custodian, 0.242);
	link.update_lot(12, &stable_asset, 1.0, &custodian, 0.097);
	link.update_lot(13, &liquid_asset, 1.0, &custodian, 0.063);
	let portfolio = link.latest_portfolio();
	let segments = portfolio.segment_reports();
	let drift_values = segments.iter()
		.map(|it| (it.drift_value() * 1000.0) as i64)
		.collect::<Vec<_>>();
	assert_eq!(drift_values, vec![-1, 1, 2, -2, 0]);
	let allocation_values = segments.iter()
		.map(|it| (it.allocate_value() * 1000.0) as i64)
		.collect::<Vec<_>>();
	assert_eq!(allocation_values, vec![64, 96, 240, 600, 0])
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
