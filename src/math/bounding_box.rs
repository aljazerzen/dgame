use super::vec::*;
use gamemath::Vec2;

#[derive(Default)]
pub struct BoundingBox {
    pub top_left: Vec2<f32>,
    pub bottom_right: Vec2<f32>,
}

impl BoundingBox {
    pub fn size(&self) -> f32 {
        (self.top_left - self.bottom_right).length()
    }
}

impl std::ops::AddAssign<Vec2<f32>> for BoundingBox {
    fn add_assign(&mut self, right: Vec2<f32>) {
        self.top_left = min(self.top_left, right);
        self.bottom_right = max(self.bottom_right, right);
    }
}
