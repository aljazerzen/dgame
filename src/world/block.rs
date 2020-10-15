use super::{ForcePoint};
use crate::math::{polygon::Polygon, vec::*};
use crate::ui::user_controls::Action;
use gamemath::{Mat3, Vec2};

#[typetag::serde(tag = "type")]
pub trait Block: std::fmt::Debug + CloneBlock {
    fn shape(&self) -> &Polygon;
    fn offset(&self) -> Vec2<f32>;
    fn set_offset(&mut self, offset: Vec2<f32>);
    fn angle(&self) -> f32;
    fn set_angle(&mut self, angle: f32);

    fn force(&self) -> ForcePoint {
        ForcePoint::default()
    }

    fn tick(&mut self) {}

    fn mass(&self) -> f32 {
        0.0
    }

    fn apply_action(&mut self, action: &Action);

    fn transform(&self) -> Mat3 {
        translation(self.offset()) * Mat3::rotation(self.angle())
    }
}

pub trait CloneBlock {
    fn clone_block(&self) -> Box<dyn Block>;
}

impl<T> CloneBlock for T
where
    T: Block + Clone + 'static,
{
    fn clone_block(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Block> {
    fn clone(&self) -> Self {
        self.clone_block()
    }
}