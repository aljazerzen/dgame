use super::polygon::Polygon;
use super::vec::*;
use crate::grid::Grid;
use gamemath::{Mat3, Vec2};

#[derive(Default)]
pub struct RectBounds {
    pub top_left: Vec2<f32>,
    pub bottom_right: Vec2<f32>,
}

impl RectBounds {
    pub fn size(&self) -> f32 {
        (self.top_left - self.bottom_right).length()
    }

    pub fn new(vec: Vec2<f32>) -> RectBounds {
        RectBounds {
            top_left: vec,
            bottom_right: vec,
        }
    }

    pub fn polygon(&self) -> Polygon {
        Polygon {
            points: vec![
                self.top_left,
                Vec2::new(self.top_left.x, self.bottom_right.y),
                self.bottom_right,
                Vec2::new(self.bottom_right.x, self.top_left.y),
            ]
            .iter()
            .map(|p| p.into_homogeneous())
            .collect(),
        }
    }

    pub fn expand(mut self, value: f32) -> RectBounds {
        self.top_left -= value.into();
        self.bottom_right += value.into();
        self
    }
}

impl std::ops::AddAssign<Vec2<f32>> for RectBounds {
    fn add_assign(&mut self, right: Vec2<f32>) {
        self.top_left = min(self.top_left, right);
        self.bottom_right = max(self.bottom_right, right);
    }
}

impl std::ops::AddAssign<RectBounds> for RectBounds {
    fn add_assign(&mut self, right: RectBounds) {
        *self += right.bottom_right;
        *self += right.top_left;
    }
}

pub trait BoundingBox {
    fn bounding_box_transformed(&self, position: &Mat3) -> RectBounds;

    fn bounding_box(&self) -> RectBounds {
        self.bounding_box_transformed(&Mat3::default())
    }
}

impl BoundingBox for Polygon {
    fn bounding_box_transformed(&self, position: &Mat3) -> RectBounds {
        let mut bounds = RectBounds::new(self.points[0].into_cartesian());
        for point in &self.points {
            bounds += (*position * *point).into_cartesian();
        }
        bounds
    }
}

impl BoundingBox for Grid {
    fn bounding_box_transformed(&self, position: &Mat3) -> RectBounds {
        let mut bounds = RectBounds::new(0.0.into());
        for entity in &self.entities {
            let entity_position = *position * translation(entity.position.state);
            bounds += entity.shape.bounding_box_transformed(&entity_position);
        }
        bounds
    }
}
