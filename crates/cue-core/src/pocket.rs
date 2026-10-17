use std::fmt;

/// One of the six pockets.
///
/// Named by table end and long rail, using the same convention as the
/// cushions in `cue-physics`: the **head** end is `−X` (behind the head
/// string), the **foot** end is `+X` (where the rack sits), the **far** rail is
/// `−Z` and the **near** rail is `+Z`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PocketId {
    /// Corner at `(−X, −Z)`.
    HeadFar,
    /// Corner at `(−X, +Z)`.
    HeadNear,
    /// Middle of the far rail, `(0, −Z)`.
    SideFar,
    /// Middle of the near rail, `(0, +Z)`.
    SideNear,
    /// Corner at `(+X, −Z)`.
    FootFar,
    /// Corner at `(+X, +Z)`.
    FootNear,
}

impl PocketId {
    /// All six pockets in a stable order.
    pub const ALL: [Self; 6] = [
        Self::HeadFar,
        Self::HeadNear,
        Self::SideFar,
        Self::SideNear,
        Self::FootFar,
        Self::FootNear,
    ];

    /// Whether this is a corner pocket.
    pub const fn is_corner(self) -> bool {
        !matches!(self, Self::SideFar | Self::SideNear)
    }

    /// Signs of the pocket's position: `x ∈ {−1, 0, 1}`, `z ∈ {−1, 1}`.
    pub const fn signs(self) -> (i8, i8) {
        match self {
            Self::HeadFar => (-1, -1),
            Self::HeadNear => (-1, 1),
            Self::SideFar => (0, -1),
            Self::SideNear => (0, 1),
            Self::FootFar => (1, -1),
            Self::FootNear => (1, 1),
        }
    }

    /// Stable kebab-case name, used in the public JS events.
    pub const fn name(self) -> &'static str {
        match self {
            Self::HeadFar => "head-far",
            Self::HeadNear => "head-near",
            Self::SideFar => "side-far",
            Self::SideNear => "side-near",
            Self::FootFar => "foot-far",
            Self::FootNear => "foot-near",
        }
    }
}

impl fmt::Display for PocketId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Dimensions of one kind of pocket (corner or side).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PocketSpec {
    /// Mouth width between the two cushion noses, metres.
    pub mouth: f64,
    /// Angle between the cushion nose and the pocket facing, radians. Above
    /// 90° the facings converge, so the throat is narrower than the mouth.
    pub facing_angle: f64,
    /// Distance from the middle of the mouth line to the edge of the hole in
    /// the slate, metres.
    pub shelf: f64,
    /// Radius of the drop region: a ball whose centre enters it falls.
    /// A modelling parameter, not a WPA dimension.
    pub drop_radius: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_corners_two_sides() {
        assert_eq!(PocketId::ALL.iter().filter(|p| p.is_corner()).count(), 4);
        assert!(PocketId::ALL.iter().all(|p| p.signs().1 != 0));
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<_> = PocketId::ALL.iter().map(|p| p.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 6);
    }
}
