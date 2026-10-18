//! Collision geometry of the table, derived from a [`TableSpec`].
//!
//! The cushions are straight segments: the four rail noses, interrupted by
//! the pockets, and two angled facings per pocket. Where segments meet there
//! is a sharp vertex (the jaw point at the mouth, and the end of each facing
//! deep in the throat), which balls can strike on its own. A pocket is a
//! circular drop region behind its mouth: a ball whose centre enters it
//! falls.
//!
//! ```text
//!              facing end ●
//!                        ╱  ← facing (cushion rubber)
//!   rail nose ─────────● jaw point
//!                       ╲
//!                         (mouth)       ◯ drop region
//! ```
//!
//! Construction checks that each throat is closed by its drop region, so no
//! ball can slip between the facings and leave the table.

use std::f64::consts::FRAC_1_SQRT_2;

pub use cue_core::Rail;
use cue_core::{PocketId, PocketSpec, TableSpec, glam::DVec2};

use crate::SimError;

/// Which part of the cushioning a contact was with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cushion {
    /// The nose of a rail.
    Rail(Rail),
    /// A facing or jaw point of a pocket.
    Jaw(PocketId),
}

/// A straight piece of cushion nose. Balls touch it when their centre is one
/// radius away on the `normal` side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    /// Start point.
    pub a: DVec2,
    /// End point.
    pub b: DVec2,
    /// Unit normal pointing away from the cushion, towards where balls are.
    pub normal: DVec2,
    /// What this segment belongs to.
    pub cushion: Cushion,
    /// The rail it is part of; a facing belongs to the rail it is cut into.
    pub rail: Rail,
}

impl Segment {
    /// Unit vector from `a` to `b`.
    pub fn direction(&self) -> DVec2 {
        (self.b - self.a).normalize()
    }

    /// Length in metres.
    pub fn length(&self) -> f64 {
        self.a.distance(self.b)
    }
}

/// A sharp corner of the cushioning that balls can strike directly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    /// Position on the table plane.
    pub at: DVec2,
    /// What this vertex belongs to.
    pub cushion: Cushion,
    /// The rail it is part of.
    pub rail: Rail,
}

/// A pocket's drop region and mouth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pocket {
    /// Which pocket.
    pub id: PocketId,
    /// Centre of the drop region.
    pub centre: DVec2,
    /// Radius of the drop region: a ball whose centre enters it falls.
    pub radius: f64,
    /// The two jaw points that bound the mouth.
    pub jaws: [DVec2; 2],
}

/// The table's complete collision geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct TableGeometry {
    segments: Vec<Segment>,
    vertices: Vec<Vertex>,
    pockets: Vec<Pocket>,
    outlines: Vec<[DVec2; 4]>,
    outer: DVec2,
}

/// The two facings of one pocket, each from its jaw point into the rail.
struct Facings {
    pocket: Pocket,
    /// `(jaw point, facing end)` pairs.
    facings: [(DVec2, DVec2); 2],
}

impl TableGeometry {
    /// Builds the geometry. Fails if the dimensions would leave a gap
    /// through which a ball could escape the table.
    pub fn new(spec: &TableSpec) -> Result<Self, SimError> {
        spec.validate()
            .map_err(|e| SimError::InvalidLayout(e.to_string()))?;
        let half = spec.half_extents();
        let pockets: Vec<Facings> = PocketId::ALL
            .iter()
            .map(|&id| {
                if id.is_corner() {
                    corner(spec, id)
                } else {
                    side(spec, id)
                }
            })
            .collect();

        let mut geometry = Self {
            segments: Vec::new(),
            vertices: Vec::new(),
            pockets: Vec::new(),
            outlines: Vec::new(),
            outer: half + DVec2::splat(spec.cushion_width),
        };

        for f in &pockets {
            for (index, &(jaw, end)) in f.facings.iter().enumerate() {
                let along = (end - jaw).normalize();
                let mut normal = along.perp();
                if normal.dot(f.pocket.centre - jaw) < 0.0 {
                    normal = -normal;
                }
                let cushion = Cushion::Jaw(f.pocket.id);
                let rail = facing_rail(f.pocket.id, index);
                geometry.segments.push(Segment {
                    a: jaw,
                    b: end,
                    normal,
                    cushion,
                    rail,
                });
                geometry.vertices.push(Vertex {
                    at: jaw,
                    cushion,
                    rail,
                });
                geometry.vertices.push(Vertex {
                    at: end,
                    cushion,
                    rail,
                });
                if end.distance(f.pocket.centre) > f.pocket.radius {
                    return Err(SimError::InvalidLayout(format!(
                        "the {} pocket's throat is wider than its drop region",
                        f.pocket.id
                    )));
                }
            }
            geometry.pockets.push(f.pocket);
        }

        // Rail noses between the jaws, each paired with the facing ends that
        // close it off for rendering.
        let end_at = |jaw: DVec2| {
            pockets
                .iter()
                .flat_map(|f| f.facings)
                .find(|(j, _)| j.distance(jaw) < 1e-12)
                .map(|(_, e)| e)
                .expect("every rail section ends at a jaw point")
        };
        let jaw = |id: PocketId, index: usize| {
            pockets
                .iter()
                .find(|f| f.pocket.id == id)
                .map(|f| f.facings[index].0)
                .expect("all six pockets exist")
        };
        // Facing index 0 is on the long rail, 1 on the short rail (corners);
        // for sides, 0 is towards −X and 1 towards +X.
        let sections = [
            (
                Rail::Head,
                jaw(PocketId::HeadFar, 1),
                jaw(PocketId::HeadNear, 1),
                DVec2::X,
            ),
            (
                Rail::Foot,
                jaw(PocketId::FootFar, 1),
                jaw(PocketId::FootNear, 1),
                DVec2::NEG_X,
            ),
            (
                Rail::Far,
                jaw(PocketId::HeadFar, 0),
                jaw(PocketId::SideFar, 0),
                DVec2::Y,
            ),
            (
                Rail::Far,
                jaw(PocketId::SideFar, 1),
                jaw(PocketId::FootFar, 0),
                DVec2::Y,
            ),
            (
                Rail::Near,
                jaw(PocketId::HeadNear, 0),
                jaw(PocketId::SideNear, 0),
                DVec2::NEG_Y,
            ),
            (
                Rail::Near,
                jaw(PocketId::SideNear, 1),
                jaw(PocketId::FootNear, 0),
                DVec2::NEG_Y,
            ),
        ];
        for (rail, a, b, normal) in sections {
            geometry.segments.push(Segment {
                a,
                b,
                normal,
                cushion: Cushion::Rail(rail),
                rail,
            });
            geometry.outlines.push([a, b, end_at(b), end_at(a)]);
        }
        Ok(geometry)
    }

    /// All cushion segments: rail noses and pocket facings.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// All sharp cushion corners.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// All six pockets, in [`PocketId::ALL`] order.
    pub fn pockets(&self) -> &[Pocket] {
        &self.pockets
    }

    /// One pocket.
    pub fn pocket(&self, id: PocketId) -> &Pocket {
        self.pockets
            .iter()
            .find(|p| p.id == id)
            .expect("all six pockets exist")
    }

    /// Outline of each rail's cushion section as a quadrilateral on the table
    /// plane (two nose points, then the two facing ends behind them). For
    /// rendering; collision uses the segments.
    pub fn cushion_outlines(&self) -> &[[DVec2; 4]] {
        &self.outlines
    }

    /// Half extents of the area enclosed by the back of the cushions. A ball
    /// centre beyond it has escaped the table.
    pub fn outer_half_extents(&self) -> DVec2 {
        self.outer
    }

    /// The rails a ball of radius `r` centred at `p` is touching, to within
    /// `tolerance` metres: those it is *frozen* to (WPA 2.7). As a bit set of
    /// [`Rail::bit`].
    pub fn touching(&self, p: DVec2, r: f64, tolerance: f64) -> u8 {
        let segments = self.segments.iter().filter_map(|s| {
            let along = (p - s.a).dot(s.direction());
            let gap = (p - s.a).dot(s.normal) - r;
            ((0.0..=s.length()).contains(&along) && gap.abs() <= tolerance).then_some(s.rail)
        });
        let vertices = self
            .vertices
            .iter()
            .filter_map(|v| (p.distance(v.at) - r <= tolerance).then_some(v.rail));
        segments
            .chain(vertices)
            .fold(0, |set, rail| set | rail.bit())
    }

    /// Deepest overlap of a ball of radius `r` at `p` with any cushion, as
    /// `(cushion, depth)`. `None` when it touches nothing.
    pub fn penetration(&self, p: DVec2, r: f64) -> Option<(Cushion, f64)> {
        let segments = self.segments.iter().filter_map(|s| {
            let along = (p - s.a).dot(s.direction());
            let gap = (p - s.a).dot(s.normal);
            ((0.0..=s.length()).contains(&along) && gap > -r && gap < r)
                .then_some((s.cushion, r - gap))
        });
        let vertices = self
            .vertices
            .iter()
            .filter_map(|v| (p.distance(v.at) < r).then_some((v.cushion, r - p.distance(v.at))));
        segments.chain(vertices).max_by(|a, b| a.1.total_cmp(&b.1))
    }
}

fn corner(spec: &TableSpec, id: PocketId) -> Facings {
    let PocketSpec {
        mouth,
        facing_angle,
        shelf,
        drop_radius,
    } = spec.corner_pocket;
    let half = spec.half_extents();
    let (sx, sz) = signs(id);
    // The mouth runs at 45° across the corner.
    let a = mouth * FRAC_1_SQRT_2;
    let long_jaw = DVec2::new(sx * (half.x - a), sz * half.y);
    let short_jaw = DVec2::new(sx * half.x, sz * (half.y - a));
    // Facing direction relative to the nose: `along` continues towards the
    // corner, `into` goes into the rail.
    let (along, into) = (-facing_angle.cos(), facing_angle.sin());
    let length = spec.cushion_width / into;
    let long_end = long_jaw + length * DVec2::new(sx * along, sz * into);
    let short_end = short_jaw + length * DVec2::new(sx * into, sz * along);

    let mouth_middle = (long_jaw + short_jaw) / 2.0;
    let axis = DVec2::new(sx, sz).normalize();
    Facings {
        pocket: Pocket {
            id,
            centre: mouth_middle + (shelf + drop_radius) * axis,
            radius: drop_radius,
            jaws: [long_jaw, short_jaw],
        },
        facings: [(long_jaw, long_end), (short_jaw, short_end)],
    }
}

fn side(spec: &TableSpec, id: PocketId) -> Facings {
    let PocketSpec {
        mouth,
        facing_angle,
        shelf,
        drop_radius,
    } = spec.side_pocket;
    let half = spec.half_extents();
    let (_, sz) = signs(id);
    let minus_jaw = DVec2::new(-mouth / 2.0, sz * half.y);
    let plus_jaw = DVec2::new(mouth / 2.0, sz * half.y);
    // `cos` is negative above 90°, so both facings lean towards the middle.
    let (along, into) = (facing_angle.cos(), facing_angle.sin());
    let length = spec.cushion_width / into;
    let plus_end = plus_jaw + length * DVec2::new(along, sz * into);
    let minus_end = minus_jaw + length * DVec2::new(-along, sz * into);
    Facings {
        pocket: Pocket {
            id,
            centre: DVec2::new(0.0, sz * (half.y + shelf + drop_radius)),
            radius: drop_radius,
            jaws: [minus_jaw, plus_jaw],
        },
        facings: [(minus_jaw, minus_end), (plus_jaw, plus_end)],
    }
}

fn signs(id: PocketId) -> (f64, f64) {
    let (x, z) = id.signs();
    (f64::from(x), f64::from(z))
}

/// The rail a pocket facing is cut into. Facing 0 of a corner is on the long
/// rail and 1 on the short rail; both side facings are on the long rail.
fn facing_rail(id: PocketId, index: usize) -> Rail {
    let (x, z) = id.signs();
    let long = if z < 0 { Rail::Far } else { Rail::Near };
    match (id.is_corner(), index, x < 0) {
        (true, 1, true) => Rail::Head,
        (true, 1, false) => Rail::Foot,
        _ => long,
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    fn geometry() -> (TableSpec, TableGeometry) {
        let spec = TableSpec::nine_foot();
        let g = TableGeometry::new(&spec).expect("standard table builds");
        (spec, g)
    }

    #[test]
    fn has_six_rail_sections_twelve_facings_and_six_pockets() {
        let (_, g) = geometry();
        let rails = g
            .segments()
            .iter()
            .filter(|s| matches!(s.cushion, Cushion::Rail(_)))
            .count();
        assert_eq!(rails, 6);
        assert_eq!(g.segments().len(), 18);
        assert_eq!(g.vertices().len(), 24);
        assert_eq!(g.pockets().len(), 6);
    }

    #[test]
    fn mouths_match_the_spec() {
        let (spec, g) = geometry();
        for p in g.pockets() {
            let expected = if p.id.is_corner() {
                spec.corner_pocket.mouth
            } else {
                spec.side_pocket.mouth
            };
            assert_relative_eq!(p.jaws[0].distance(p.jaws[1]), expected, epsilon = 1e-12);
        }
    }

    #[test]
    fn throats_are_narrower_than_mouths() {
        // Facing angles above 90° make the facings converge.
        let (_, g) = geometry();
        for p in g.pockets() {
            let ends: Vec<_> = g
                .segments()
                .iter()
                .filter(|s| s.cushion == Cushion::Jaw(p.id))
                .map(|s| s.b)
                .collect();
            let throat = ends[0].distance(ends[1]);
            let mouth = p.jaws[0].distance(p.jaws[1]);
            assert!(throat < mouth, "{}: throat {throat} vs mouth {mouth}", p.id);
            assert!(
                throat > 2.0 * TableSpec::nine_foot().ball.radius,
                "a ball must fit"
            );
        }
    }

    #[test]
    fn segment_normals_face_the_playing_side() {
        let (_, g) = geometry();
        for s in g.segments() {
            assert_relative_eq!(s.normal.length(), 1.0, epsilon = 1e-12);
            assert!(s.normal.dot(s.direction()).abs() < 1e-12);
            if let Cushion::Rail(_) = s.cushion {
                // Rail normals point at the table centre.
                assert!(s.normal.dot(-s.a) > 0.0, "{s:?}");
            }
        }
    }

    #[test]
    fn drop_regions_do_not_reach_the_playing_surface() {
        // A ball placed anywhere on the playing surface must not already be
        // in a pocket.
        let (spec, g) = geometry();
        let limit = spec.half_extents() - DVec2::splat(spec.ball.radius);
        for p in g.pockets() {
            let nearest = p.centre.clamp(-limit, limit);
            assert!(nearest.distance(p.centre) > p.radius, "{}", p.id);
        }
    }

    #[test]
    fn penetration_detects_overlap_with_noses_and_jaws() {
        let (spec, g) = geometry();
        let r = spec.ball.radius;
        let half = spec.half_extents();
        assert!(g.penetration(DVec2::ZERO, r).is_none());
        // Touching the foot rail exactly: no overlap.
        let touching = DVec2::new(half.x - r, 0.0);
        assert!(g.penetration(touching, r).is_none_or(|(_, d)| d < 1e-12));
        let (cushion, depth) = g.penetration(touching + DVec2::new(0.001, 0.0), r).unwrap();
        assert_eq!(cushion, Cushion::Rail(Rail::Foot));
        assert_relative_eq!(depth, 0.001, epsilon = 1e-12);
        // Sitting on a jaw point.
        let jaw = g.pocket(PocketId::SideNear).jaws[1];
        let (cushion, _) = g.penetration(jaw, r).unwrap();
        assert_eq!(cushion, Cushion::Jaw(PocketId::SideNear));
    }

    #[test]
    fn every_cushion_part_belongs_to_the_rail_it_is_cut_into() {
        let (spec, g) = geometry();
        let r = spec.ball.radius;
        let half = spec.half_extents();
        for s in g.segments() {
            if let Cushion::Rail(rail) = s.cushion {
                assert_eq!(s.rail, rail);
            }
        }
        // A corner's two facings: one on each rail it joins.
        let corner: Vec<Rail> = g
            .segments()
            .iter()
            .filter(|s| s.cushion == Cushion::Jaw(PocketId::HeadFar))
            .map(|s| s.rail)
            .collect();
        assert_eq!(corner, vec![Rail::Far, Rail::Head]);
        assert!(
            g.segments()
                .iter()
                .filter(|s| s.cushion == Cushion::Jaw(PocketId::SideNear))
                .all(|s| s.rail == Rail::Near)
        );
        // Frozen to the foot rail, to two rails in a corner, to none.
        let foot = DVec2::new(half.x - r, 0.1);
        assert_eq!(g.touching(foot, r, 1e-5), Rail::Foot.bit());
        assert_eq!(g.touching(foot - DVec2::new(1e-4, 0.0), r, 1e-5), 0);
        assert_eq!(g.touching(DVec2::ZERO, r, 1e-5), 0);
    }

    #[test]
    fn rejects_drop_regions_that_leave_the_throat_open() {
        let mut spec = TableSpec::nine_foot();
        spec.corner_pocket.drop_radius = spec.ball.radius * 1.01;
        assert!(TableGeometry::new(&spec).is_err());
    }
}
