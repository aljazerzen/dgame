use gamemath::{Mat3, Vec2, Vec3};

pub trait Perpendicular {
  fn perpendicular(self: &Self) -> Self;
}

impl<T: std::ops::Neg<Output = T> + Copy> Perpendicular for Vec3<T> {
  fn perpendicular(&self) -> Self {
    Vec3 {
      x: -self.y,
      y: self.x,
      z: self.z,
    }
  }
}

impl<T: std::ops::Neg<Output = T> + Copy> Perpendicular for Vec2<T> {
  fn perpendicular(&self) -> Self {
    Vec2 {
      x: -self.y,
      y: self.x,
    }
  }
}

pub trait IntoHomogeneous<T> {
  fn into_homogeneous(self: &Self) -> Vec3<T>;
}

impl IntoHomogeneous<f32> for Vec2<f32> {
  fn into_homogeneous(self: &Vec2<f32>) -> Vec3<f32> {
    Vec3 {
      x: self.x,
      y: self.y,
      z: 1.0,
    }
  }
}

pub trait IntoCartesian<T> {
  fn into_cartesian(self: &Self) -> Vec2<T>;
}

impl<T: std::ops::Div<Output = T> + Copy> IntoCartesian<T> for Vec3<T> {
  fn into_cartesian(self: &Vec3<T>) -> Vec2<T> {
    Vec2 {
      x: self.x / self.z,
      y: self.y / self.z,
    }
  }
}

pub fn min(a: Vec2<f32>, b: Vec2<f32>) -> Vec2<f32> {
  Vec2 {
    x: a.x.min(b.x),
    y: a.y.min(b.y),
  }
}

pub fn max(a: Vec2<f32>, b: Vec2<f32>) -> Vec2<f32> {
  Vec2 {
    x: a.x.max(b.x),
    y: a.y.max(b.y),
  }
}

pub fn from_int(a: Vec2<i32>) -> Vec2<f32> {
  Vec2::new(a.x as f32, a.y as f32)
}

#[allow(dead_code)]
pub fn from_float(a: Vec2<f32>) -> Vec2<i32> {
  Vec2::new(a.x as i32, a.y as i32)
}

pub fn modulo<T: std::ops::Rem<Output = T> + std::ops::Add<Output = T> + Copy>(
  a: &Vec2<T>,
  b: &Vec2<T>,
) -> Vec2<T> {
  Vec2 {
    x: ((a.x % b.x) + b.x) % b.x,
    y: ((a.y % b.y) + b.y) % b.y,
  }
}

pub fn translation(vector: Vec2<f32>) -> Mat3 {
  ((1.0, 0.0, vector.x), (0.0, 1.0, vector.y), (0.0, 0.0, 1.0)).into()
}

pub fn phase_out(val: f32) -> f32 {
  if val > 0.0 {
    return (val - (0.05 * (val + 1.0))).max(0.0);
  }
  if val < 0.0 {
    return (val - (0.05 * (val - 1.0))).min(0.0);
  }
  0.0
}
