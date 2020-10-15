use crate::grid::Insist;
use crate::math::polygon::Polygon;
use crate::math::vec::*;
use crate::ui::user_controls::Action;
use gamemath::{Mat2, Mat3, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::io::Write;

const ENTITY_SHAPE_DENSITY: f32 = 0.02;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    id: u64,
    pub shape: Polygon,

    #[serde_as(as = "Insist<Vec2Serde<f32>>")]
    pub position: Insist<Vec2<f32>>,
    pub angle: Insist<f32>,

    pub blocks: Vec<Box<dyn Block>>,

    // calculated values
    pub mass: f32,
    pub mass_angular: f32,
}

impl Entity {
    pub fn new(poly: Polygon, blocks: Vec<Box<dyn Block>>) -> Entity {
        use rand::RngCore;
        let mut rng = rand::thread_rng();

        let mut result = Entity {
            id: rng.next_u64(),

            shape: poly,
            position: Insist::default(),
            angle: Insist::default(),

            blocks,

            mass: 0.0,
            mass_angular: 0.0,
        };
        result.redistribute_weight();
        result
    }

    pub fn new_from_block(mut block: Box<dyn Block>) -> Entity {
        block.set_offset(Vec2::default());
        block.set_angle(0.0);
        let shape = block.shape().clone();
        let mut entity = Entity::new(shape, vec![Box::from(block)]);
        entity.position = Insist::default();
        entity
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Accelerate { .. } => {
                // let rotation = Mat2::rotation(-self.angle.state);
                // direction = rotation * direction;
                for block in &mut self.blocks {
                    block.apply_action(&action);
                }
            }
            Action::Rotate { .. } => {
                for block in &mut self.blocks {
                    block.apply_action(&action);
                }
            }
            Action::UpdateShape { new_shape } => {
                let transform =
                    Mat3::rotation(-self.angle.state) * translation(-self.position.state);
                self.expand_shape(transform * *new_shape);
                self.redistribute_weight();
            }
            Action::JoinEntity { mut entity } => {
                let transform = Mat3::rotation(-self.angle.state)
                    * translation(entity.position.state - self.position.state)
                    * Mat3::rotation(entity.angle.state);

                self.expand_shape(transform * entity.shape);

                for mut block in entity.blocks.drain(..) {
                    block.set_offset(
                        (transform * block.offset().into_homogeneous()).into_cartesian(),
                    );
                    block.set_angle(block.angle() + entity.angle.state - self.angle.state);
                    self.add_block(block);
                }
                self.redistribute_weight();
            }
            Action::SaveEntity => {
                self.save_to_file().ok();
            }
            _ => {}
        }

        // if let Some(grid_coordinates) = user_controls.clicked {
        //     let click =
        //         Mat2::rotation(-self.angle.state) * (grid_coordinates - self.position.state);

        //     self.shape.intrude_point(click);

        //     self.redistribute_weight();
        // }
    }

    pub fn add_block(&mut self, block: Box<dyn Block>) {
        let block_shape = block.transform() * Mat3::identity().scaled(Vec2::new(0.999, 0.999)) * block.shape().clone();

        if !self.shape.contains_polygon(&block_shape) {
            return;
        }

        for b in &self.blocks {
            let s = b.transform() * b.shape().clone();
            for p in &s.points {
                if block_shape.contains_point(p.into_cartesian()) {
                    return;
                }
            }
        }

        self.blocks.push(block);
    }

    pub fn tick(&mut self) {
        for block in &mut self.blocks {
            block.tick();
        }
    }

    pub fn expand_shape(&mut self, new_shape: Polygon) {
        let mut polygons = self.shape.clone().intersection(new_shape);

        for poly in polygons.drain(..) {
            if poly.contains_point(Vec2::new(0.0, 0.0)) {
                // let (old_area, _) = self.shape.area_and_centroid();
                self.shape = poly;

                // let (new_area, _) = self.shape.area_and_centroid();
            }
        }
    }

    pub fn force(&self) -> ForcePoint {
        let mut result = ForcePoint::default();

        for block in &self.blocks {
            let mut force_point = block.force();
            force_point.force = Mat2::rotation(block.angle()) * force_point.force;
            force_point.add_force_torque(block.offset());

            result += force_point;
        }

        result.force = Mat2::rotation(self.angle.state) * result.force;
        result
    }

    pub fn redistribute_weight(&mut self) {
        let mass_point = self.mass_point();
        // mass point should be aligned with origin of entity coordinate system
        for block in &mut self.blocks {
            block.set_offset(block.offset() - mass_point.point);
        }
        self.shape = translation(-mass_point.point) * self.shape.clone();

        self.position.state += mass_point.point;

        self.mass = mass_point.mass;
        self.mass_angular = self.mass_angular();
    }

    pub fn mass_angular(&self) -> f32 {
        let mut sum = 0.0;
        for block in &self.blocks {
            sum += block.mass() * block.offset().length_squared();
        }
        let shape_mass = self.shape.area_and_centroid().0.abs() * ENTITY_SHAPE_DENSITY;
        sum += shape_mass * self.shape.radius_of_gyration(Vec2::default());
        sum
    }

    pub fn mass_point(&self) -> MassPoint {
        let (shape_area, centroid) = self.shape.area_and_centroid();
        let mut result = MassPoint {
            point: centroid,
            mass: shape_area.abs() * ENTITY_SHAPE_DENSITY,
        };

        for block in &self.blocks {
            result += MassPoint {
                point: block.offset(),
                mass: block.mass(),
            }
        }

        result
    }

    pub fn projection_to_grid(&self) -> Mat3 {
        translation(self.position.state) * Mat3::rotation(self.angle.state)
    }

    pub fn save_to_file(&self) -> Result<(), std::io::Error> {
        let bytes = rmp_serde::to_vec(self).unwrap();

        let filename = "./data/entities/".to_owned() + &self.id.to_string();

        let mut file = std::fs::File::create(filename)?;
        file.write_all(&bytes)?;

        Ok(())
    }

    pub fn load_from_file(
        filename: std::ffi::OsString,
    ) -> Result<Entity, rmp_serde::decode::Error> {
        let bytes = std::fs::read(filename).unwrap_or_else(|_| Vec::new());

        rmp_serde::from_read_ref(&bytes)
    }

    pub fn list_saved() -> Result<Vec<std::ffi::OsString>, std::io::Error> {
        let res = std::fs::read_dir("./data/entities")?;

        Ok(res
            .filter(|e| {
                e.as_ref()
                    .map(|e| e.file_type().unwrap())
                    .map(|t| t.is_file())
                    .unwrap_or(false)
            })
            .map(|e| e.unwrap().path().into_os_string())
            .collect())
    }
}

impl PartialEq<u64> for Entity {
    fn eq(&self, right: &u64) -> bool {
        self.id == *right
    }
}

#[derive(Clone, Debug, Default)]
pub struct MassPoint {
    point: Vec2<f32>,
    mass: f32,
}

impl std::ops::AddAssign<MassPoint> for MassPoint {
    fn add_assign(&mut self, right: MassPoint) {
        self.point += (right.point - self.point) * (right.mass / self.mass);
        self.mass += right.mass;
    }
}

impl std::ops::Add<MassPoint> for MassPoint {
    type Output = MassPoint;

    fn add(self, right: MassPoint) -> MassPoint {
        let mut result = self;
        result += right;
        result
    }
}

#[derive(Default, Debug, Clone)]
pub struct ForcePoint {
    pub torque: f32,
    pub force: Vec2<f32>,
}

impl ForcePoint {
    fn add_force_torque(&mut self, r: Vec2<f32>) {
        let torque: Vec3<f32> = Vec3::from(r).cross(self.force.into());
        self.torque += torque.z;
    }
}

impl std::ops::AddAssign<ForcePoint> for ForcePoint {
    fn add_assign(&mut self, right: ForcePoint) {
        self.torque += right.torque;
        self.force += right.force;
    }
}

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
