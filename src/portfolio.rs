use SegmentType::*;

use crate::prelude::*;

struct MemSegment {
	segment_type: SegmentType,
	name: String,
}

impl Segment for MemSegment {
	fn name(&self) -> &str { &self.name }
	fn asset_type(&self) -> &SegmentType { &self.segment_type }
}

struct MemPortfolio {
	segments: Vec<MemSegment>,
}

impl Portfolio for MemPortfolio {
	fn segment(&self, segment_type: SegmentType) -> &dyn Segment {
		match segment_type {
			Liquid => &self.segments[0],
			Stable => &self.segments[1],
			Linear => &self.segments[2],
			Expo => &self.segments[3]
		}
	}
}

pub fn new() -> impl Portfolio {
	MemPortfolio {
		segments: vec![
			MemSegment { segment_type: Liquid, name: "Liquid".to_string() },
			MemSegment { segment_type: Stable, name: "Stable".to_string() },
			MemSegment { segment_type: Linear, name: "Linear".to_string() },
			MemSegment { segment_type: Expo, name: "Expo".to_string() }
		],
	}
}
