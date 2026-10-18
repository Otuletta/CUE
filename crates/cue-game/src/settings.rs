//! Player-facing settings and the command channel hosts use to drive the game.
//!
//! Settings change presentation only. A theme recolours materials and an
//! assist level changes what the aiming guide draws; neither touches the
//! physics (spec §7.5, §21.4).

use bevy::{gizmos::config::DefaultGizmoConfigGroup, prelude::*};
use cue_core::{
    glam::DVec2,
    rules::eight_ball::{Call, Choice},
};

use crate::{camera::ViewInsets, eight::GameSetup, quality::Quality};

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
                // Walnut, under the grain texture's 62–100 % shading.
                wood: Color::srgb(0.30, 0.17, 0.095),
                leather: Color::srgb(0.035, 0.03, 0.03),
                floor: Color::srgb(0.2, 0.14, 0.1),
                background: Color::srgb(0.035, 0.037, 0.045),
                guide: Color::srgba(1.0, 1.0, 1.0, 0.55),
                guide_accent: Color::srgba(1.0, 0.85, 0.4, 0.75),
                guide_width: 2.0,
            },
            Self::Studio => Palette {
                cloth: Color::srgb(0.05, 0.20, 0.36),
                cushion: Color::srgb(0.04, 0.16, 0.30),
                // Dark espresso wood.
                wood: Color::srgb(0.22, 0.14, 0.095),
                leather: Color::srgb(0.02, 0.02, 0.022),
                floor: Color::srgb(0.14, 0.14, 0.15),
                background: Color::srgb(0.03, 0.032, 0.04),
                guide: Color::srgba(1.0, 1.0, 1.0, 0.6),
                guide_accent: Color::srgba(0.55, 0.85, 1.0, 0.8),
                guide_width: 2.0,
            },
            Self::Contrast => Palette {
                // Renders about mid-grey under the lamp (sRGB 130 in the
                // middle, 80 in the corners): far from both the black eight
                // and the ivory cue ball everywhere.
                cloth: Color::srgb(0.30, 0.32, 0.35),
                cushion: Color::srgb(0.20, 0.22, 0.25),
                wood: Color::srgb(0.02, 0.02, 0.02),
                leather: Color::srgb(0.0, 0.0, 0.0),
                floor: Color::srgb(0.0, 0.0, 0.0),
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
    /// The room's floor, under and around the table.
    pub(crate) floor: Color,
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
    pub(crate) floor: Handle<StandardMaterial>,
}

/// Whether the developer overlay (frame rate, state) is visible.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DevHud(pub bool);

/// Requests from outside the canvas: the host page's controls, keyboard
/// shortcuts handled by the page, tests. Applied at the start of a frame.
#[derive(Message, Debug, Clone, PartialEq)]
pub enum GameCommand {
    /// Set the charged power, `0..=1`, as a slider would.
    SetPower(f64),
    /// Shoot with the current aim, power and tip.
    Shoot,
    /// Set the tip offset in ball radii; clamped to the miscue limit.
    SetTip(DVec2),
    /// Undo the last shot or re-rack (free play only).
    Undo,
    /// Rack the balls again (free play only).
    Rerack,
    /// Change the visual theme.
    SetTheme(Theme),
    /// Change how much the aiming guide shows.
    SetAssists(Assists),
    /// Show or hide the developer overlay.
    ToggleDevHud,
    /// The canvas bands the host's controls cover; the table is framed
    /// clear of them.
    SetViewInsets(ViewInsets),
    /// Reduce motion: the cue strikes without a swing.
    SetReducedMotion(bool),
    /// Change the graphics quality.
    SetQuality(Quality),
    /// Start a game of eight-ball.
    NewGame(GameSetup),
    /// Go back to free play.
    FreePlay,
    /// Eight-ball: call a ball and pocket, or a safety, for the next shot.
    SetCall(Call),
    /// Eight-ball: let the call follow the aim again.
    AutoCall,
    /// Eight-ball: resolve a pending decision.
    Decide(Choice),
    /// Eight-ball: start the next rack of the match.
    NextRack,
    /// Eight-ball: both players agree to a stalemate.
    Stalemate,
    /// Eight-ball: spot the legal ball nearest the head string (WPA 1.6).
    SpotRequest,
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
        (&handles.floor, palette.floor),
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
