use crate::entity_controller::EntityController;
use crate::grid::Insist;
use crate::math::polygon::Polygon;
use crate::math::vec::*;
use crate::ui::user_controls::UserControls;
use gamemath::{Mat2, Mat3, Vec2, Vec3};

const ENTITY_SHAPE_DENSITY: f32 = 0.02;

#[derive(Clone, Debug)]
pub struct Entity {
  id: u64,
  pub shape: Polygon,
  pub position: Insist<Vec2<f32>>,
  pub angle: Insist<f32>,

  pub controller: Option<EntityController>,
  pub blocks: Vec<Block>,

  // calculated values
  pub mass: f32,
  pub mass_angular: f32,
}

impl Entity {
  pub fn new(poly: Polygon, position: Insist<Vec2<f32>>, angle: Insist<f32>) -> Entity {
    use rand::RngCore;
    let mut rng = rand::thread_rng();

    let mut result = Entity {
      id: rng.next_u64(),

      shape: poly,
      position,
      angle,

      controller: None,
      blocks: Vec::default(),

      mass: 0.0,
      mass_angular: 0.0,
    };
    result.redistribute_weight();
    result
  }

  pub fn get_id(&self) -> u64 {
    self.id
  }

  pub fn use_controls(&mut self, user_controls: &UserControls) {
    if let Some(controller) = &mut self.controller {
      controller.update(user_controls);
    }

    if let Some(grid_coordinates) = user_controls.clicked {
      let click = Mat2::rotation(-self.angle.state) * (grid_coordinates - self.position.state);

      self.shape.intrude_point(click);

      self.redistribute_weight();
    }
  }

  pub fn tick(&mut self) {
    for block in &mut self.blocks {
      block.tick();
    }
  }

  pub fn force(&self) -> ForcePoint {
    let controller_force = self
      .controller
      .as_ref()
      .map(|c| c.force.clone())
      .unwrap_or_default();

    let mut result = controller_force;

    for block in &self.blocks {
      let mut force_point = block.force();
      force_point.force = block.offset + Mat2::rotation(block.angle) * force_point.force;
      force_point.add_force_torque(block.offset);

      result += force_point;
    }

    result.force = Mat2::rotation(self.angle.state) * result.force;
    result
  }

  pub fn redistribute_weight(&mut self) {
    let mass_point = self.mass_point();
    // mass point should be aligned with origin of entity coordinate system
    for block in &mut self.blocks {
      block.offset -= mass_point.point;
    }
    self.shape.mul_left(translation(-mass_point.point));

    self.position.state += mass_point.point;

    self.mass = mass_point.mass;
    self.mass_angular = self.mass_angular();
  }

  pub fn mass_angular(&self) -> f32 {
    let mut sum = 0.0;
    for block in &self.blocks {
      sum += block.mass() * block.offset.length_squared();
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
        point: block.offset,
        mass: block.mass(),
      }
    }

    result
  }

  pub fn projection_to_grid(&self) -> Mat3 {
    translation(self.position.state) * Mat3::rotation(self.angle.state)
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

pub trait Machine {
  fn force(&self) -> ForcePoint {
    ForcePoint::default()
  }

  fn tick(&mut self) {}

  fn mass(&self) -> f32 {
    0.0
  }
}

#[derive(Clone, Debug)]
pub struct Block {
  pub shape: Polygon,
  pub offset: Vec2<f32>,
  pub angle: f32,

  variant: BlockVariant,
}

#[derive(Clone, Debug)]
pub enum BlockVariant {
  Thruster { thruster: Box<Thruster> },
  Gyroscope,
}

impl Block {
  fn get_machine(&self) -> Option<&impl Machine> {
    match &self.variant {
      BlockVariant::Thruster { thruster } => Some(thruster.as_ref()),
      _ => None,
    }
  }

  fn get_machine_mut(&mut self) -> Option<&mut impl Machine> {
    match &mut self.variant {
      BlockVariant::Thruster { thruster } => Some(thruster.as_mut()),
      _ => None,
    }
  }
}

impl Machine for Block {
  fn force(&self) -> ForcePoint {
    self.get_machine().map(|m| m.force()).unwrap_or_default()
  }

  fn tick(&mut self) {
    if let Some(m) = self.get_machine_mut() {
      m.tick();
    }
  }

  fn mass(&self) -> f32 {
    self
      .get_machine()
      .map(|m| m.mass())
      .or_else(|| Some(self.shape.area_and_centroid().0))
      .unwrap()
  }
}

#[derive(Clone, Debug)]
pub struct Thruster {
  throttle: f32,
  throttle_target: f32,
  thrust_vector: Vec2<f32>,
}

impl Thruster {
  pub fn new_block(width: f32, offset: Vec2<f32>, angle: f32) -> Block {
    Block {
      shape: Thruster::shape(width),
      offset,
      angle,
      variant: BlockVariant::Thruster {
        thruster: Box::from(Thruster::new(width)),
      },
    }
  }

  fn new(width: f32) -> Self {
    Thruster {
      throttle: 0.0,
      throttle_target: 5.0,

      thrust_vector: Vec2::new(0.0, -width * width * 0.005),
    }
  }

  pub fn shape(width: f32) -> Polygon {
    let p = Polygon::from(
      &[
        [0.0, 0.0],
        [width, 0.0],
        [width, width],
        [width * 0.8, width],
        [width, width * 1.6],
        [0.0, width * 1.6],
        [width * 0.2, width],
        [0.0, width],
      ][..],
    );

    let (_, center) = p.area_and_centroid();
    translation(-center) * p
  }
}

impl Machine for Thruster {
  fn force(&self) -> ForcePoint {
    ForcePoint {
      torque: 0.0,
      force: self.thrust_vector * self.throttle,
    }
  }

  fn tick(&mut self) {
    self.throttle = (self.throttle + (self.throttle_target - self.throttle).min(0.01))
      .min(1.0)
      .max(0.0);
  }
}
