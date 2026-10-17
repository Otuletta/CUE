//! Player-facing settings and the command channel hosts use to drive the game.
//!
//! Settings change presentation only. A theme recolours materials and an
//! assist level changes what the aiming guide draws; neither touches the
//! physics (spec §7.5, §21.4).

use bevy::{gizmos::config::DefaultGizmoConfigGroup, prelude::*};
use cue_core::glam::DVec2;

/// How much the aiming guide shows. Every level is an *estimate*: it ignores
/// spin, throw and curve (spec §10.5).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Assists {
    /// No guide: only the cue stick.
    Off,
    /// A line to the first contact and a ghost ball.
    #[default]
    Basic,
    /// Also the object ball's path and the cue ball's tangent line.
    Assisted,
}

impl Assists {
    /// Stable name used by the public API and saved settings.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Basic => "basic",
            Self::Assisted => "assisted",
        }
    }

    /// Parses a name from [`Assists::name`].
    pub fn parse(name: &str) -> Option<Self> {
        [Self::Off, Self::Basic, Self::Assisted]
            .into_iter()
            .find(|a| a.name() == name)
    }
}

/// Visual theme (spec §7.5).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Theme {
    /// A traditional green table with dark wood.
    Classic,
    /// Tournament blue cloth and charcoal rails: the portfolio look.
    #[default]
    Studio,
    /// Maximum legibility: a mid-grey cloth that separates the black and
    /// white balls equally well, near-black rails and a bold guide.
    Contrast,
}

impl Theme {
    /// Stable name used by the public API and saved settings.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Studio => "studio",
            Self::Contrast => "contrast",
        }
    }

    /// Parses a name from [`Theme::name`].
    pub fn parse(name: &str) -> Option<Self> {
        [Self::Classic, Self::Studio, Self::Contrast]
            .into_iter()
            .find(|t| t.name() == name)
    }

    pub(crate) fn palette(self) -> Palette {
        match self {
            Self::Classic => Palette {
                cloth: Color::srgb(0.06, 0.30, 0.24),
                cushion: Color::srgb(0.05, 0.25, 0.20),
                wood: Color::srgb(0.16, 0.09, 0.05),
                leather: Color::srgb(0.035, 0.03, 0.03),
                background: Color::srgb(0.035, 0.037, 0.045),
                guide: Color::srgba(1.0, 1.0, 1.0, 0.55),
                guide_accent: Color::srgba(1.0, 0.85, 0.4, 0.75),
                guide_width: 2.0,
            },
            Self::Studio => Palette {
                cloth: Color::srgb(0.05, 0.20, 0.36),
                cushion: Color::srgb(0.04, 0.16, 0.30),
                wood: Color::srgb(0.055, 0.055, 0.06),
                leather: Color::srgb(0.02, 0.02, 0.022),
                background: Color::srgb(0.03, 0.032, 0.04),
                guide: Color::srgba(1.0, 1.0, 1.0, 0.6),
                guide_accent: Color::srgba(0.55, 0.85, 1.0, 0.8),
                guide_width: 2.0,
            },
            Self::Contrast => Palette {
                // Renders mid-grey under the key light: equally far from the
                // black eight and the ivory cue ball.
                cloth: Color::srgb(0.30, 0.32, 0.35),
                cushion: Color::srgb(0.20, 0.22, 0.25),
                wood: Color::srgb(0.02, 0.02, 0.02),
                leather: Color::srgb(0.0, 0.0, 0.0),
                background: Color::srgb(0.0, 0.0, 0.0),
                guide: Color::srgba(1.0, 0.92, 0.0, 1.0),
                guide_accent: Color::srgba(1.0, 0.35, 0.9, 1.0),
                guide_width: 3.5,
            },
        }
    }
}

/// The colours a theme assigns.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Palette {
    pub(crate) cloth: Color,
    pub(crate) cushion: Color,
    pub(crate) wood: Color,
    pub(crate) leather: Color,
    pub(crate) background: Color,
    pub(crate) guide: Color,
    pub(crate) guide_accent: Color,
    pub(crate) guide_width: f32,
}

/// Material handles the theme recolours.
#[derive(Resource, Debug, Clone)]
pub(crate) struct TableMaterials {
    pub(crate) cloth: Handle<StandardMaterial>,
    pub(crate) cushion: Handle<StandardMaterial>,
    pub(crate) wood: Handle<StandardMaterial>,
    pub(crate) leather: Handle<StandardMaterial>,
}

/// Whether the developer overlay (frame rate, state) is visible.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DevHud(pub bool);

/// Requests from outside the canvas: the host page's controls, keyboard
/// shortcuts handled by the page, tests. Applied at the start of a frame.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub enum GameCommand {
    /// Set the charged power, `0..=1`, as a slider would.
    SetPower(f64),
    /// Shoot with the current aim, power and tip.
    Shoot,
    /// Set the tip offset in ball radii; clamped to the miscue limit.
    SetTip(DVec2),
    /// Undo the last shot or re-rack.
    Undo,
    /// Rack the balls again.
    Rerack,
    /// Change the visual theme.
    SetTheme(Theme),
    /// Change how much the aiming guide shows.
    SetAssists(Assists),
    /// Show or hide the developer overlay.
    ToggleDevHud,
}

/// Recolours the table when the theme changes.
pub(crate) fn apply_theme(
    theme: Res<Theme>,
    handles: Option<Res<TableMaterials>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut clear: ResMut<ClearColor>,
    mut gizmos: ResMut<GizmoConfigStore>,
) {
    let Some(handles) = handles else {
        return;
    };
    let palette = theme.palette();
    for (handle, colour) in [
        (&handles.cloth, palette.cloth),
        (&handles.cushion, palette.cushion),
        (&handles.wood, palette.wood),
        (&handles.leather, palette.leather),
    ] {
        if let Some(mut m) = materials.get_mut(handle) {
            m.base_color = colour;
        }
    }
    clear.0 = palette.background;
    gizmos.config_mut::<DefaultGizmoConfigGroup>().0.line.width = palette.guide_width;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for a in [Assists::Off, Assists::Basic, Assists::Assisted] {
            assert_eq!(Assists::parse(a.name()), Some(a));
        }
        for t in [Theme::Classic, Theme::Studio, Theme::Contrast] {
            assert_eq!(Theme::parse(t.name()), Some(t));
        }
        assert_eq!(Theme::parse("neon"), None);
    }
}
