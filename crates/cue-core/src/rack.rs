use std::collections::BTreeMap;

use glam::DVec2;

use crate::{BallGroup, BallId, CoreError, SplitMix64, TableSpec};

/// Clearance left between neighbouring racked balls, in metres.
///
/// A real rack is frozen (balls touching). The solver needs a strictly
/// positive gap so that "touching" is never ambiguous at `t = 0`; five microns
/// is far below anything visible or physically meaningful.
pub const RACK_GAP: f64 = 5.0e-6;

/// Positions of balls on the table plane, keyed by logical id.
///
/// Iteration order is ascending by [`BallId`], which keeps every consumer
/// (physics world construction, snapshots, tests) deterministic.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    positions: BTreeMap<BallId, DVec2>,
}

impl Layout {
    /// Position of a ball, if it is on the table.
    pub fn get(&self, id: BallId) -> Option<DVec2> {
        self.positions.get(&id).copied()
    }

    /// Balls and positions in ascending id order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (BallId, DVec2)> + '_ {
        self.positions.iter().map(|(id, p)| (*id, *p))
    }

    /// Number of balls in the layout.
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Whether the layout is empty.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    fn insert(&mut self, id: BallId, p: DVec2) {
        let previous = self.positions.insert(id, p);
        debug_assert!(previous.is_none(), "{id} placed twice");
    }
}

/// Builds a custom layout (practice drills, tests, future challenges). If an
/// id repeats, the last position wins. Validity is checked by consumers such
/// as the physics world, not here.
impl FromIterator<(BallId, DVec2)> for Layout {
    fn from_iter<I: IntoIterator<Item = (BallId, DVec2)>>(iter: I) -> Self {
        Self {
            positions: iter.into_iter().collect(),
        }
    }
}

/// Slot `(row, index)` of the eight ball: centre of the third row.
const EIGHT_SLOT: (usize, usize) = (2, 1);
/// Back corners of the triangle; they must hold one solid and one stripe.
const CORNER_SLOTS: [(usize, usize); 2] = [(4, 0), (4, 4)];

/// A standard eight-ball rack plus the cue ball on the head spot.
///
/// The apex sits on the foot spot and the triangle opens towards the foot
/// rail. The eight ball is centred in the third row and the two back corners
/// hold one ball from each group. All remaining placements are drawn from
/// `seed`, so the same seed always yields the same rack.
pub fn eight_ball_rack(table: &TableSpec, seed: u64) -> Result<Layout, CoreError> {
    table.validate()?;

    let r = table.ball.radius;
    let pitch = 2.0 * r + RACK_GAP;
    let row_pitch = pitch * 3f64.sqrt() / 2.0;
    let apex = table.foot_spot();
    let slot_position = |row: usize, i: usize| {
        apex + DVec2::new(
            row as f64 * row_pitch,
            (i as f64 - row as f64 / 2.0) * pitch,
        )
    };

    if !table.contains_ball(slot_position(4, 0)) || !table.contains_ball(slot_position(4, 4)) {
        return Err(CoreError::InvalidTableSpec(
            "the rack does not fit on the table",
        ));
    }

    let mut rng = SplitMix64::new(seed);
    let of_group = |g| {
        BallId::objects()
            .filter(move |b| b.group() == g)
            .collect::<Vec<_>>()
    };
    let mut solids = of_group(BallGroup::Solids);
    let mut stripes = of_group(BallGroup::Stripes);
    rng.shuffle(&mut solids);
    rng.shuffle(&mut stripes);

    let mut corners = [
        solids.pop().expect("there are seven solids"),
        stripes.pop().expect("there are seven stripes"),
    ];
    if rng.below(2) == 1 {
        corners.swap(0, 1);
    }

    let mut rest: Vec<BallId> = solids.into_iter().chain(stripes).collect();
    rng.shuffle(&mut rest);
    let mut rest = rest.into_iter();

    let mut layout = Layout::default();
    layout.insert(BallId::CUE, table.head_spot());
    for row in 0..5 {
        for i in 0..=row {
            let slot = (row, i);
            let ball = if slot == EIGHT_SLOT {
                BallId::EIGHT
            } else if let Some(c) = CORNER_SLOTS.iter().position(|s| *s == slot) {
                corners[c]
            } else {
                rest.next().expect("twelve free slots for twelve balls")
            };
            layout.insert(ball, slot_position(row, i));
        }
    }
    debug_assert!(rest.next().is_none());
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use proptest::prelude::*;

    use super::*;

    fn rack(seed: u64) -> (TableSpec, Layout) {
        let table = TableSpec::nine_foot();
        let layout = eight_ball_rack(&table, seed).expect("standard table must rack");
        (table, layout)
    }

    fn position(layout: &Layout, n: u8) -> DVec2 {
        layout.get(BallId::new(n).unwrap()).unwrap()
    }

    #[test]
    fn rack_contains_every_ball_once() {
        let (_, layout) = rack(1);
        assert_eq!(layout.len(), 16);
        assert!(BallId::all().all(|b| layout.get(b).is_some()));
    }

    #[test]
    fn apex_is_on_the_foot_spot_and_cue_on_the_head_spot() {
        let (table, layout) = rack(1);
        let apex = layout
            .iter()
            .filter(|(b, _)| !b.is_cue())
            .min_by(|a, b| a.1.x.total_cmp(&b.1.x))
            .unwrap();
        assert_relative_eq!(apex.1.x, table.foot_spot().x);
        assert_relative_eq!(apex.1.y, 0.0);
        assert_eq!(layout.get(BallId::CUE), Some(table.head_spot()));
    }

    #[test]
    fn eight_ball_is_centred_in_the_third_row() {
        let (table, layout) = rack(3);
        let eight = position(&layout, 8);
        let row_pitch = (2.0 * table.ball.radius + RACK_GAP) * 3f64.sqrt() / 2.0;
        assert_relative_eq!(eight.x, table.foot_spot().x + 2.0 * row_pitch);
        assert_relative_eq!(eight.y, 0.0);
    }

    #[test]
    fn same_seed_same_rack_different_seed_different_rack() {
        assert_eq!(rack(99).1, rack(99).1);
        assert_ne!(rack(99).1, rack(100).1);
    }

    proptest! {
        #[test]
        fn back_corners_hold_one_ball_of_each_group(seed: u64) {
            let (_, layout) = rack(seed);
            let back_x = layout.iter().map(|(_, p)| p.x).fold(f64::MIN, f64::max);
            let mut corners: Vec<_> = layout
                .iter()
                .filter(|(_, p)| (p.x - back_x).abs() < 1e-9)
                .collect();
            corners.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
            let left = corners.first().unwrap().0.group();
            let right = corners.last().unwrap().0.group();
            prop_assert!(matches!(
                (left, right),
                (BallGroup::Solids, BallGroup::Stripes) | (BallGroup::Stripes, BallGroup::Solids)
            ));
        }

        #[test]
        fn no_two_balls_overlap_and_all_are_on_the_table(seed: u64) {
            let (table, layout) = rack(seed);
            let balls: Vec<_> = layout.iter().collect();
            for (i, (a, pa)) in balls.iter().enumerate() {
                prop_assert!(table.contains_ball(*pa), "{a} off the table");
                for (b, pb) in &balls[i + 1..] {
                    let gap = pa.distance(*pb) - table.ball.diameter();
                    prop_assert!(gap >= RACK_GAP * 0.999, "{a} and {b} overlap: {gap}");
                }
            }
        }
    }
}
