use gamemath::{Vec2, Vec3};
use rand::{thread_rng, Rng};

pub struct Stars {
  pub points: Vec<Vec3<f32>>,
  pub field_size: Vec2<f32>,
}

impl Stars {
  pub fn new(view_size: Vec2<f32>) -> Stars {
    let mut rng = thread_rng();

    let depth = 10.0;
    let field_size = view_size * (depth + 1.0);
    let count = (field_size.x * field_size.y / 15_000.0) as usize;

    println!("generating {:?} stars", count);

    let mut points: Vec<Vec3<f32>> = Vec::with_capacity(count);

    for _i in 0..count {
      points.push(Vec3 {
        x: rng.gen_range(0, field_size.x as i32) as f32,
        y: rng.gen_range(0, field_size.y as i32) as f32,
        z: rng.gen_range(1, depth as i32) as f32,
      });
    }
    Stars {
      points,
      field_size: Vec2 {
        x: field_size.x as f32,
        y: field_size.y as f32,
      },
    }
  }
}
