extern crate chad_core;

use chad_core::{allocate_amount, portfolio};
use chad_core::prelude::*;

#[test]
fn portfolio_contains_segments() {
	let portfolio = portfolio::new();
	let liquid_segment = portfolio.segment(SegmentType::Liquid);
	assert_eq!(liquid_segment.name(), "Liquid");
}

#[test]
fn allocate_computes_sub_amounts() {
	let allocation = allocate_amount(1.0);
	assert_eq!(allocation, vec![
		(SegmentType::Liquid, 1.0f64 - (0.6f64 * 0.4f64 * 0.4f64) - (0.6f64 * 0.4f64) - (0.6f64)),
		(SegmentType::Stable, 0.6f64 * 0.4f64 * 0.4f64),
		(SegmentType::Linear, 0.6f64 * 0.4f64),
		(SegmentType::Expo, 0.6f64),
	]);
}
