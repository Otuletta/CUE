/// One of the four rails.
///
/// Named with the same convention as [`crate::PocketId`]: the **head** rail
/// is at `−X`, the **foot** rail at `+X`, the **far** rail at `−Z` and the
/// **near** rail at `+Z`. Pocket facings and jaws belong to the rail they are
/// cut into (WPA 2.1: "the cushions, tops of the rails, pockets and pocket
/// liners are parts of the rails").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rail {
    /// Short rail behind the head string (`−X`).
    Head,
    /// Short rail behind the rack (`+X`).
    Foot,
    /// Long rail at `−Z`.
    Far,
    /// Long rail at `+Z`.
    Near,
}

impl Rail {
    /// All four rails in a stable order.
    pub const ALL: [Self; 4] = [Self::Head, Self::Foot, Self::Far, Self::Near];

    /// Stable name used in public events.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Head => "head",
            Self::Foot => "foot",
            Self::Far => "far",
            Self::Near => "near",
        }
    }

    /// A distinct bit per rail, for compact sets.
    pub const fn bit(self) -> u8 {
        match self {
            Self::Head => 1,
            Self::Foot => 2,
            Self::Far => 4,
            Self::Near => 8,
        }
    }
}
