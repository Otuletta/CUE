use glam::DVec2;

use crate::{RACK_GAP, TableSpec};

/// Clearance kept between a spotted ball and the cue ball, which it must not
/// touch (WPA 1.5), metres.
const CUE_SEPARATION: f64 = 0.002;

/// Where to spot a ball, following WPA 1.5.
///
/// On the long string, as close as possible to the foot spot, between the
/// foot spot and the foot rail. If the spot is taken, the ball goes right
/// behind the ball in the way, touching it, except that it never touches the
/// cue ball. If the whole long string below the foot spot is blocked, the
/// ball goes above the foot spot, as close to it as possible.
///
/// `occupied` holds the centres of every object ball on the table; `cue` the
/// cue ball's, if it is on the table.
pub fn spot_position(table: &TableSpec, occupied: &[DVec2], cue: Option<DVec2>) -> DVec2 {
    let r = table.ball.radius;
    let foot = table.foot_spot();
    let limit = table.half_extents().x - r;
    let obstacles: Vec<(DVec2, f64)> = occupied
        .iter()
        .map(|&p| (p, 2.0 * r + RACK_GAP))
        .chain(cue.map(|c| (c, 2.0 * r + CUE_SEPARATION)))
        .collect();
    let blocked_by = |x: f64| {
        obstacles
            .iter()
            .find(|(p, d)| DVec2::new(x, 0.0).distance(*p) < *d - 1e-12)
            .copied()
    };

    // Walk towards the foot rail, hopping just past each ball in the way.
    let mut x = foot.x;
    while x <= limit {
        match blocked_by(x) {
            None => return DVec2::new(x, 0.0),
            Some((p, d)) => x = p.x + (d * d - p.y * p.y).max(0.0).sqrt(),
        }
    }
    // Blocked all the way: walk towards the head instead.
    let mut x = foot.x;
    while x >= -limit {
        match blocked_by(x) {
            None => return DVec2::new(x, 0.0),
            Some((p, d)) => x = p.x - (d * d - p.y * p.y).max(0.0).sqrt(),
        }
    }
    foot
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> TableSpec {
        TableSpec::nine_foot()
    }

    #[test]
    fn a_free_foot_spot_is_used() {
        assert_eq!(spot_position(&table(), &[], None), table().foot_spot());
    }

    #[test]
    fn a_blocked_spot_puts_the_ball_behind_and_touching_the_obstacle() {
        let t = table();
        let r = t.ball.radius;
        let obstacle = t.foot_spot() + DVec2::new(0.01, 0.0);
        let p = spot_position(&t, &[obstacle], None);
        assert!(p.x > obstacle.x, "towards the foot rail");
        assert!(
            (p.distance(obstacle) - (2.0 * r + RACK_GAP)).abs() < 1e-9,
            "touching"
        );
        assert_eq!(p.y, 0.0);
    }

    #[test]
    fn the_spotted_ball_never_touches_the_cue_ball() {
        let t = table();
        let r = t.ball.radius;
        let cue = t.foot_spot();
        let p = spot_position(&t, &[], Some(cue));
        assert!(p.distance(cue) >= 2.0 * r + CUE_SEPARATION - 1e-9);
    }

    #[test]
    fn a_fully_blocked_long_string_spots_above_the_foot_spot() {
        let t = table();
        let r = t.ball.radius;
        let step = 2.0 * r + RACK_GAP;
        let limit = t.half_extents().x - r;
        let mut wall = Vec::new();
        let mut x = t.foot_spot().x;
        while x <= limit + step {
            wall.push(DVec2::new(x, 0.0));
            x += step;
        }
        let p = spot_position(&t, &wall, None);
        assert!(p.x < t.foot_spot().x, "above the foot spot");
        assert!(wall.iter().all(|w| w.distance(p) >= step - 1e-9));
    }
}
