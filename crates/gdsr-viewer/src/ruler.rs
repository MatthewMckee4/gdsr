use egui::{Color32, FontId, Painter, Pos2, Rect, Shape, Stroke};

use crate::viewport::Viewport;

const RULER_COLOR: Color32 = Color32::from_rgb(255, 200, 50);
const RULER_WIDTH: f32 = 1.5;
const ENDPOINT_RADIUS: f32 = 4.0;

/// Tracks the state of the ruler/measurement tool.
#[derive(Default)]
pub struct RulerState {
    /// Whether the ruler tool is active (listening for clicks).
    pub active: bool,
    /// First point in world coordinates, set on first click.
    pub start: Option<(f64, f64)>,
    /// All completed measurements.
    pub measurements: Vec<Measurement>,
}

/// A completed measurement between two world-space points.
#[derive(Clone, Copy)]
pub struct Measurement {
    pub start: (f64, f64),
    pub end: (f64, f64),
}

impl Measurement {
    /// Euclidean distance in world units (meters).
    pub fn distance(&self) -> f64 {
        let dx = self.end.0 - self.start.0;
        let dy = self.end.1 - self.start.1;
        (dx * dx + dy * dy).sqrt()
    }
}

impl RulerState {
    pub fn toggle(&mut self) {
        self.active = !self.active;
        if !self.active {
            self.start = None;
        }
    }

    /// Cancels any in-progress measurement. Does not remove completed measurements.
    pub fn cancel(&mut self) {
        self.start = None;
        self.active = false;
    }

    /// Removes all completed measurements and cancels any in-progress one.
    pub fn clear_all(&mut self) {
        self.start = None;
        self.measurements.clear();
        self.active = false;
    }

    /// Handles a click at the given world coordinates. Returns true if the click was consumed.
    pub fn handle_click(&mut self, wx: f64, wy: f64) -> bool {
        if !self.active {
            return false;
        }

        if let Some(start) = self.start {
            self.measurements.push(Measurement {
                start,
                end: (wx, wy),
            });
            self.start = None;
            true
        } else {
            self.start = Some((wx, wy));
            true
        }
    }

    /// Draws all ruler overlays: completed measurements and in-progress line to cursor.
    pub fn draw(
        &self,
        painter: &Painter,
        viewport: &Viewport,
        rect: Rect,
        mouse_world: Option<(f64, f64)>,
    ) {
        let stroke = Stroke::new(RULER_WIDTH, RULER_COLOR);

        for m in &self.measurements {
            draw_measurement(painter, viewport, rect, m, stroke);
        }

        if let Some(start) = self.start {
            let s_start = viewport.world_to_screen(start.0, start.1, rect);
            draw_endpoint(painter, s_start);

            if let Some((mx, my)) = mouse_world {
                let s_end = viewport.world_to_screen(mx, my, rect);
                painter.add(Shape::LineSegment {
                    points: [s_start, s_end],
                    stroke,
                });
                draw_endpoint(painter, s_end);

                let distance = ((mx - start.0).powi(2) + (my - start.1).powi(2)).sqrt();
                let label = format_distance(distance);
                let midpoint = Pos2::new(
                    f32::midpoint(s_start.x, s_end.x),
                    f32::midpoint(s_start.y, s_end.y),
                );
                draw_label(painter, midpoint, &label);
            }
        }
    }
}

fn draw_measurement(
    painter: &Painter,
    viewport: &Viewport,
    rect: Rect,
    m: &Measurement,
    stroke: Stroke,
) {
    let s_start = viewport.world_to_screen(m.start.0, m.start.1, rect);
    let s_end = viewport.world_to_screen(m.end.0, m.end.1, rect);

    painter.add(Shape::LineSegment {
        points: [s_start, s_end],
        stroke,
    });
    draw_endpoint(painter, s_start);
    draw_endpoint(painter, s_end);

    let label = format_distance(m.distance());
    let midpoint = Pos2::new(
        f32::midpoint(s_start.x, s_end.x),
        f32::midpoint(s_start.y, s_end.y),
    );
    draw_label(painter, midpoint, &label);
}

fn draw_endpoint(painter: &Painter, center: Pos2) {
    painter.circle_filled(center, ENDPOINT_RADIUS, RULER_COLOR);
}

fn draw_label(painter: &Painter, pos: Pos2, text: &str) {
    let font = FontId::proportional(14.0);
    let galley = painter.layout_no_wrap(text.to_owned(), font, Color32::WHITE);
    let padding = 4.0;
    let text_rect = egui::Rect::from_min_size(
        Pos2::new(
            pos.x - galley.size().x / 2.0 - padding,
            pos.y - galley.size().y - padding * 2.0 - 4.0,
        ),
        egui::Vec2::new(
            galley.size().x + padding * 2.0,
            galley.size().y + padding * 2.0,
        ),
    );
    painter.rect_filled(
        text_rect,
        4.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 200),
    );
    let text_pos = Pos2::new(pos.x - galley.size().x / 2.0, text_rect.min.y + padding);
    painter.galley(text_pos, galley, Color32::WHITE);
}

/// Formats a distance in meters to human-readable units.
pub fn format_distance(meters: f64) -> String {
    let abs = meters.abs();
    if abs < 1e-6 {
        format!("{:.2} nm", meters * 1e9)
    } else if abs < 1e-3 {
        format!("{:.3} \u{00B5}m", meters * 1e6)
    } else {
        format!("{:.4} mm", meters * 1e3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_distance_nanometers() {
        insta::assert_snapshot!(format_distance(5e-8), @"50.00 nm");
    }

    #[test]
    fn format_distance_micrometers() {
        insta::assert_snapshot!(format_distance(1.5e-6), @"1.500 µm");
    }

    #[test]
    fn format_distance_millimeters() {
        insta::assert_snapshot!(format_distance(2.5e-3), @"2.5000 mm");
    }

    #[test]
    fn format_distance_zero() {
        insta::assert_snapshot!(format_distance(0.0), @"0.00 nm");
    }

    #[test]
    fn format_distance_boundary_nm_to_um() {
        insta::assert_snapshot!(format_distance(1e-6), @"1.000 µm");
    }

    #[test]
    fn format_distance_boundary_um_to_mm() {
        insta::assert_snapshot!(format_distance(1e-3), @"1.0000 mm");
    }

    #[test]
    fn measurement_distance_horizontal() {
        let m = Measurement {
            start: (0.0, 0.0),
            end: (3e-6, 0.0),
        };
        assert!((m.distance() - 3e-6).abs() < 1e-15);
    }

    #[test]
    fn measurement_distance_diagonal() {
        let m = Measurement {
            start: (0.0, 0.0),
            end: (3e-6, 4e-6),
        };
        assert!((m.distance() - 5e-6).abs() < 1e-15);
    }

    #[test]
    fn ruler_state_toggle() {
        let mut ruler = RulerState::default();
        assert!(!ruler.active);

        ruler.toggle();
        assert!(ruler.active);

        ruler.toggle();
        assert!(!ruler.active);
    }

    #[test]
    fn multiple_measurements() {
        let mut ruler = RulerState {
            active: true,
            ..Default::default()
        };

        ruler.handle_click(0.0, 0.0);
        ruler.handle_click(3.0, 4.0);
        assert_eq!(ruler.measurements.len(), 1);
        assert!(
            ruler.active,
            "should stay active after completing a measurement"
        );

        ruler.handle_click(10.0, 10.0);
        ruler.handle_click(13.0, 14.0);
        assert_eq!(ruler.measurements.len(), 2);
        assert!(ruler.active);
    }

    #[test]
    fn cancel_preserves_completed_measurements() {
        let mut ruler = RulerState {
            active: true,
            ..Default::default()
        };

        ruler.handle_click(0.0, 0.0);
        ruler.handle_click(1.0, 0.0);
        assert_eq!(ruler.measurements.len(), 1);

        ruler.handle_click(5.0, 5.0);
        assert!(ruler.start.is_some());

        ruler.cancel();
        assert!(!ruler.active);
        assert!(ruler.start.is_none());
        assert_eq!(
            ruler.measurements.len(),
            1,
            "completed measurements should be preserved"
        );
    }

    #[test]
    fn clear_all_removes_everything() {
        let mut ruler = RulerState {
            active: true,
            ..Default::default()
        };

        ruler.handle_click(0.0, 0.0);
        ruler.handle_click(1.0, 0.0);
        ruler.handle_click(2.0, 2.0);

        ruler.clear_all();
        assert!(!ruler.active);
        assert!(ruler.start.is_none());
        assert!(ruler.measurements.is_empty());
    }

    #[test]
    fn format_distance_very_large() {
        let s = format_distance(1.0);
        assert!(s.contains("mm"));
    }

    #[test]
    fn format_distance_negative() {
        let s = format_distance(-5e-8);
        assert!(s.contains("nm"));
    }

    #[test]
    fn measurement_distance_zero() {
        let m = Measurement {
            start: (0.0, 0.0),
            end: (0.0, 0.0),
        };
        assert!((m.distance() - 0.0).abs() < 1e-20);
    }

    #[test]
    fn measurement_distance_extreme_coords() {
        let m = Measurement {
            start: (-1e6, -1e6),
            end: (1e6, 1e6),
        };
        let d = m.distance();
        assert!(d.is_finite());
        assert!(d > 0.0);
    }

    #[test]
    fn ruler_many_measurements() {
        let mut ruler = RulerState {
            active: true,
            ..Default::default()
        };
        for i in 0..100 {
            ruler.handle_click(f64::from(i), 0.0);
            ruler.handle_click(f64::from(i) + 0.5, 0.0);
        }
        assert_eq!(ruler.measurements.len(), 100);
        ruler.clear_all();
        assert!(ruler.measurements.is_empty());
    }

    #[test]
    fn toggle_twice_returns_to_original_state() {
        let mut ruler = RulerState::default();
        ruler.toggle();
        ruler.toggle();
        assert!(!ruler.active);
        assert!(ruler.start.is_none());
    }

    #[test]
    fn click_ignored_when_inactive() {
        let mut ruler = RulerState::default();
        assert!(!ruler.handle_click(0.0, 0.0));
        assert!(ruler.measurements.is_empty());
    }
}
