use super::line::Line;
use super::vec::*;
use gamemath::Vec2;

#[derive(Clone, Copy)]
pub struct Segment {
    pub a: Vec2<f32>,
    pub b: Vec2<f32>,
}

impl Segment {
    pub fn new(a: Vec2<f32>, b: Vec2<f32>) -> Segment {
        Segment { a, b }
    }

    pub fn direction(&self) -> Vec2<f32> {
        self.b - self.a
    }

    /// Projects point to the segment.
    /// @Result value t which represents position on the line of the segment. 0 means point a and 1 means point b.
    pub fn project_point(&self, point: Vec2<f32>) -> f32 {
        let d = self.direction();
        let a_to_point = point - self.a;
        d.dot(a_to_point) / d.length_squared()
    }

    pub fn intersection_line(self, line: &Line) -> Option<Vec2<f32>> {
        line.intersection(&self.into())
            .map(|p| {
                let alpha = self.project_point(p);

                if 0.0 <= alpha && alpha <= 1.0 {
                    Some(p)
                } else {
                    None
                }
            })
            .flatten()
    }

    pub fn intersection(&self, other: &Segment) -> Option<(f32, f32)> {
        let other_dir_perpendicular = other.direction().perpendicular();
        let self_dir_perpendicular = self.direction().perpendicular();

        let wec_p1 = (self.a - other.a).dot(other_dir_perpendicular);
        let wec_p2 = (self.b - other.a).dot(other_dir_perpendicular);

        if wec_p1 * wec_p2 <= 0.0 {
            let wec_q1 = (other.a - self.a).dot(self_dir_perpendicular);
            let wec_q2 = (other.b - self.a).dot(self_dir_perpendicular);

            if wec_q1 * wec_q2 <= 0.0 {
                return Some((wec_p1 / (wec_p1 - wec_p2), wec_q1 / (wec_q1 - wec_q2)));
            }
        }
        None
    }
}
