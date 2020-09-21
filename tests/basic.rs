extern crate chad_core;

use chad_core::core::{Account, AssetCode, Custodian, SegmentType};
use chad_core::portfolio::lot::Lot;
use chad_core::portfolio::Portfolio;
use chad_core::portfolio::segment::Segment;

#[test]
fn portfolio_computes_drift_values() {
	let link = chad_core::connect_tmp();
	let expo_asset = AssetCode::Common("EXPO".to_string());
	let linear_asset = AssetCode::Common("LINEAR".to_string());
	let stable_asset = AssetCode::Common("STABLE".to_string());
	let liquid_asset = AssetCode::Common("LIQUID".to_string());
	link.assign_asset(&expo_asset, SegmentType::Expo);
	link.assign_asset(&linear_asset, SegmentType::Linear);
	link.assign_asset(&stable_asset, SegmentType::Stable);
	link.assign_asset(&liquid_asset, SegmentType::Liquid);
	let custodian = Custodian::Custom("sovereign".to_string());
	link.update_lot(10, &expo_asset, 1.0, &custodian, 0.598);
	link.update_lot(11, &linear_asset, 1.0, &custodian, 0.242);
	link.update_lot(12, &stable_asset, 1.0, &custodian, 0.097);
	link.update_lot(13, &liquid_asset, 1.0, &custodian, 0.063);
	let portfolio = link.latest_portfolio();
	let segments = portfolio.segments();
	let drift_values = segments.iter()
		.map(|it| (it.drift_amount() * 1000.0) as i64)
		.collect::<Vec<_>>();
	assert_eq!(drift_values, vec![-1, 1, 2, -2, 0]);
	let allocation_values = segments.iter()
		.map(|it| (it.target_value() * 1000.0) as i64)
		.collect::<Vec<_>>();
	assert_eq!(allocation_values, vec![64, 96, 240, 600, 0])
}

#[test]
fn links_set_asset_prices() {
	let link = chad_core::connect_tmp();
	let tsla = AssetCode::Common("TSLA".to_string());
	let custodian = Custodian::Custom("robinhood".to_string());
	link.update_lot(2000, &tsla, 10.0, &custodian, 1.0);
	link.price_asset(&tsla, 2.0);
	let portfolio = link.latest_portfolio();
	assert_eq!(20.0, portfolio.portfolio_value());
}

#[test]
fn link_assigns_assets() {
	let link = chad_core::connect_tmp();
	let tsla = AssetCode::Common("TSLA".to_string());
	let custodian = Custodian::Custom("robinhood".to_string());
	link.update_lot(2000, &tsla, 10.0, &custodian, 300.0);
	link.assign_asset(&tsla, SegmentType::Expo);
	let portfolio = link.latest_portfolio();
	let expo = &portfolio.segments()[SegmentType::Expo.default_index()];
	let expo_value = expo.segment_value();
	assert_eq!(3000.0, expo_value);
}

#[test]
fn link_updates_lots() {
	let link = chad_core::connect_tmp();
	let tsla = AssetCode::Common("TSLA".to_string());
	let custodian = Custodian::Custom("robinhood".to_string());
	link.update_lot(2000, &tsla, 10.0, &custodian, 300.0);
	let portfolio = link.latest_portfolio();
	assert_eq!(portfolio.lots(), vec![
		Lot {
			lot_id: 2000,
			asset_code: tsla.clone(),
			share_count: 10.0,
			custodian: custodian.clone(),
			account: Account::Main,
		}
	]);
}

#[test]
fn portfolio_produces_segments() {
	let link = chad_core::connect_tmp();
	let tsla = AssetCode::Common("TSLA".to_string());
	let custodian = Custodian::Custom("robinhood".to_string());
	link.assign_asset(&tsla, SegmentType::Expo);
	link.update_lot(2000, &tsla, 10.0, &custodian, 300.0);
	let portfolio = link.latest_portfolio();
	let segments = portfolio.segments();
	let values = segments.iter().map(Segment::segment_value).collect::<Vec<_>>();
	assert_eq!(values, vec![0.0, 0.0, 0.0, 3000.0, 0.0]);
	let types = segments.iter().map(Segment::segment_type).collect::<Vec<_>>();
	assert_eq!(types, vec![SegmentType::Liquid, SegmentType::Stable, SegmentType::Linear, SegmentType::Expo, SegmentType::Unknown])
}
