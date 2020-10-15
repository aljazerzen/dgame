use super::{Block, ForcePoint};
use crate::math::{polygon::Polygon, vec::*};
use crate::ui::user_controls::Action;
use gamemath::{Mat2, Mat3, Vec2};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thruster {
    shape: Polygon,
    #[serde_as(as = "Vec2Serde<f32>")]
    offset: Vec2<f32>,
    angle: f32,
    throttle: f32,
    throttle_target: f32,

    #[serde_as(as = "Vec2Serde<f32>")]
    thrust_vector: Vec2<f32>,
}

impl Thruster {
    pub fn new(width: f32, offset: Vec2<f32>, angle: f32) -> Self {
        Thruster {
            shape: Thruster::shape(width),
            offset,
            angle,
            throttle: 0.0,
            throttle_target: 0.0,

            thrust_vector: Vec2::new(0.0, -width * width * 0.05),
        }
    }

    pub fn shape(width: f32) -> Polygon {
        let p = Polygon::from(
            &[
                [0.1, 0.0],
                [0.9, 0.0],
                [1.0, 0.1],
                [1.0, 0.9],
                [0.8, 1.1],
                [0.9, 1.3],
                [1.0, 1.7],
                [0.0, 1.7],
                [0.1, 1.3],
                [0.2, 1.1],
                [0.0, 0.9],
                [0.0, 0.1],
            ][..],
        );

        let (_, center) = p.area_and_centroid();
        let transform = Mat3::identity().scaled(Vec2::new(width, width)) * translation(-center);
        transform * p
    }
}

#[typetag::serde]
impl Block for Thruster {
    fn shape(&self) -> &Polygon {
        &self.shape
    }
    fn offset(&self) -> Vec2<f32> {
        self.offset
    }
    fn set_offset(&mut self, offset: Vec2<f32>) {
        self.offset = offset;
    }

    fn angle(&self) -> f32 {
        self.angle
    }

    fn set_angle(&mut self, angle: f32) {
        self.angle = angle;
    }

    fn force(&self) -> ForcePoint {
        ForcePoint {
            torque: 0.0,
            force: self.thrust_vector * self.throttle,
        }
    }

    fn tick(&mut self) {
        let change = (self.throttle_target - self.throttle).min(0.01);
        self.throttle = (self.throttle + change).min(1.0).max(0.0);
    }

    fn apply_action(&mut self, action: &Action) {
        if let Action::Accelerate {
            direction,
            throttle,
        } = action
        {
            let thrust = Mat2::rotation(self.angle) * self.thrust_vector;
            let directional_factor = direction.dot(thrust) / direction.length() / thrust.length();

            self.throttle_target = throttle * directional_factor.max(0.0);
        }
    }
}
