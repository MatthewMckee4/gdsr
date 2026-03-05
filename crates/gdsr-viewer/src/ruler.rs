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
    /// Completed measurement: start and end in world coordinates.
    pub measurement: Option<Measurement>,
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
            self.cancel();
        }
    }

    pub fn cancel(&mut self) {
        self.start = None;
        self.measurement = None;
        self.active = false;
    }

    /// Handles a click at the given world coordinates. Returns true if the click was consumed.
    pub fn handle_click(&mut self, wx: f64, wy: f64) -> bool {
        if !self.active {
            return false;
        }

        if let Some(start) = self.start {
            self.measurement = Some(Measurement {
                start,
                end: (wx, wy),
            });
            self.start = None;
            self.active = false;
            true
        } else {
            self.start = Some((wx, wy));
            true
        }
    }

    /// Draws the ruler overlay: in-progress line to cursor, or completed measurement.
    pub fn draw(
        &self,
        painter: &Painter,
        viewport: &Viewport,
        rect: Rect,
        mouse_world: Option<(f64, f64)>,
    ) {
        let stroke = Stroke::new(RULER_WIDTH, RULER_COLOR);

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

        if let Some(m) = &self.measurement {
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
    }
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
    fn ruler_state_click_workflow() {
        let mut ruler = RulerState::default();

        assert!(!ruler.handle_click(0.0, 0.0));

        ruler.active = true;

        assert!(ruler.handle_click(1.0, 2.0));
        assert_eq!(ruler.start, Some((1.0, 2.0)));
        assert!(ruler.measurement.is_none());

        assert!(ruler.handle_click(4.0, 6.0));
        assert!(ruler.start.is_none());
        assert!(ruler.measurement.is_some());
        assert!(!ruler.active);

        let m = ruler
            .measurement
            .as_ref()
            .expect("measurement should exist");
        assert!((m.distance() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn ruler_cancel_clears_state() {
        let mut ruler = RulerState {
            active: true,
            start: Some((1.0, 2.0)),
            measurement: Some(Measurement {
                start: (0.0, 0.0),
                end: (1.0, 1.0),
            }),
        };

        ruler.cancel();
        assert!(!ruler.active);
        assert!(ruler.start.is_none());
        assert!(ruler.measurement.is_none());
    }
}
