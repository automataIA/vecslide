use crate::manifest::TrailPoint;

/// Filters a raw trail by removing points too close to the previous kept point.
/// A point is kept only if its Euclidean distance from the last kept point exceeds `threshold`.
/// The first point is always kept.
pub fn filter_trail(points: &[TrailPoint], threshold: f32) -> Vec<TrailPoint> {
    let threshold_sq = threshold * threshold;
    let mut kept: Vec<TrailPoint> = Vec::with_capacity(points.len());

    for point in points {
        let keep = match kept.last() {
            None => true,
            Some(last) => {
                let dx = point.x - last.x;
                let dy = point.y - last.y;
                dx * dx + dy * dy > threshold_sq
            }
        };
        if keep {
            kept.push(point.clone());
        }
    }

    kept
}

/// Returns the opacity [0.0, 1.0] for a trail point at a given playback position.
/// The point is fully opaque at `point_time_ms` and fades to 0.0 after `fade_duration_ms`.
/// Returns 0.0 if the point has not yet been reached or has fully faded.
pub fn trail_opacity(point_time_ms: u64, current_time_ms: u64, fade_duration_ms: u64) -> f32 {
    if current_time_ms < point_time_ms || fade_duration_ms == 0 {
        return 0.0;
    }
    let elapsed = current_time_ms - point_time_ms;
    if elapsed >= fade_duration_ms {
        return 0.0;
    }
    1.0 - (elapsed as f32 / fade_duration_ms as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(time_ms: u64, x: f32, y: f32) -> TrailPoint {
        TrailPoint { time_ms, x, y }
    }

    #[test]
    fn filter_removes_close_points() {
        let points = vec![pt(0, 0.0, 0.0), pt(10, 1.0, 1.0), pt(20, 100.0, 100.0)];
        // threshold=5: (0,0)->(1,1) dist=sqrt(2)≈1.4 < 5, filtered; (0,0)->(100,100) dist>>5, kept
        let result = filter_trail(&points, 5.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[1].x, 100.0);
    }

    #[test]
    fn filter_keeps_all_if_threshold_zero() {
        let points = vec![pt(0, 0.0, 0.0), pt(10, 0.1, 0.1)];
        assert_eq!(filter_trail(&points, 0.0).len(), 2);
    }

    #[test]
    fn opacity_full_at_trigger_time() {
        assert!((trail_opacity(1000, 1000, 500) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn opacity_zero_before_trigger() {
        assert_eq!(trail_opacity(1000, 500, 500), 0.0);
    }

    #[test]
    fn opacity_zero_after_fade() {
        assert_eq!(trail_opacity(1000, 1600, 500), 0.0);
    }

    #[test]
    fn opacity_half_at_midpoint() {
        let op = trail_opacity(1000, 1250, 500);
        assert!((op - 0.5).abs() < 1e-5);
    }
}
