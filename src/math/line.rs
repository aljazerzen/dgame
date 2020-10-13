use super::segment::Segment;
use super::vec::IntoCartesian;
use gamemath::{Vec2, Vec3};

#[derive(Clone, Copy)]
pub struct Line {
    pub homogeneous: Vec3<f32>,
}

impl Line {
    pub fn intersection(&self, other: &Line) -> Option<Vec2<f32>> {
        let r = self.homogeneous.cross(other.homogeneous);

        if r.z == 0.0 {
            None
        } else {
            Some(r.into_cartesian())
        }
    }

    pub fn horizontal(y: f32) -> Line {
        Line {
            homogeneous: Vec3 {
                x: 0.0,
                y: -1.0,
                z: y,
            },
        }
    }
}

impl From<Segment> for Line {
    fn from(segment: Segment) -> Line {
        let d = segment.direction();

        Line {
            homogeneous: Vec3 {
                x: d.y,
                y: -d.x,
                z: segment.a.y * d.x - segment.a.x * d.y,
            },
        }
    }
}
