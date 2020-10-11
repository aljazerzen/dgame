use crate::entity::Entity;
use crate::grid::{Grid, Insist};
use crate::client::{EntityId};
use crate::math::{
  bounding_box::BoundingBox,
  polygon::{construct_rect_poly, Polygon},
  vec::*,
};
use crate::stars::Stars;
use gamemath::{Mat3, Vec2, Vec3};
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Canvas, RenderTarget};

/// Represents view used to render the grids.
pub struct View {
  // Relative to focused grid
  pub offset: Vec2<f32>,
  pub size: Vec2<f32>,

  pub stars_position: Insist<Vec2<f32>>,
  pub stars: Stars,

  pub last_grid_position: Mat3,
}

impl View {
  pub fn new(size: Vec2<f32>) -> View {
    View {
      offset: Vec2::default(),
      size,

      stars_position: Insist::default(),
      stars: Stars::new(size),

      last_grid_position: Mat3::default(),
    }
  }

  pub fn tick(&mut self) {
    self.offset = Vec2 {
      x: phase_out(self.offset.x),
      y: phase_out(self.offset.y),
    };
  }
}

pub fn render<T: RenderTarget>(root_grid: &Grid, focus: &EntityId, view: &mut View, canvas: &mut Canvas<T>) {
  canvas.set_draw_color(Color::RGB(0, 0, 0));
  canvas.clear();

  render_stars(view, canvas);

  let center = translation(into_vec(canvas.viewport().center()));
  let position = center * translation(view.offset);

  view.last_grid_position = position;

  render_grid_from_focus(root_grid, focus, position, canvas);
}

fn render_grid_from_focus<T: RenderTarget>(grid: &Grid, focus: &EntityId, position: Mat3, canvas: &mut Canvas<T>) -> Option<Mat3> {
  if grid == &focus.grid_id {
    render_grid_recursive(grid, position, canvas);
    
    Some(position * translation(-grid.relation_to_parent.unwrap_or_default().state))
  } else {
    let mut inverse_position = None;
    let mut to_controlling = None;
    
    for child in &grid.children {
      inverse_position = render_grid_from_focus(child, focus, position, canvas);
      if inverse_position.is_some() {
        to_controlling = Some(child.get_id());
        break;
      }
    }
  
    if let Some(position) = inverse_position {
      grid.render(position, canvas);

      for child in &grid.children {
        if child == &to_controlling.unwrap() {
          continue;
        }

        if let Some(relation) = child.relation_to_parent {
          let child_position = translation(relation.state) * position;
          render_grid_recursive(child, child_position, canvas);
        }
      }

      Some(position * translation(-grid.relation_to_parent.unwrap_or_default().state))
    } else {
      None
    }
  }
}

fn render_grid_recursive<T: RenderTarget>(grid: &Grid, position: Mat3, canvas: &mut Canvas<T>) {
  grid.render(position, canvas);

  for child in &grid.children {
    if let Some(relation) = child.relation_to_parent {
      let child_position = translation(relation.state) * position;
      child.render(child_position, canvas);
    }
  }
}

pub trait Render<T: RenderTarget> {
  fn render(&self, position: Mat3, canvas: &mut Canvas<T>);
}

impl<T: RenderTarget> Render<T> for Grid {
  fn render(&self, position: Mat3, canvas: &mut Canvas<T>) {
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for entity in &self.entities {
      entity.render(position, canvas);
    }
    {
      let mut bounding_box = BoundingBox::default();
      for entity in &self.entities {
        let shape_position =
          translation(entity.position.state) * Mat3::rotation(entity.angle.state);
        for point in &entity.shape.points {
          bounding_box += (shape_position * *point).into_cartesian();
        }
      }
      canvas.set_draw_color(Color::RGB(50, 50, 80));
      construct_rect_poly(
        bounding_box.top_left.x - 1.0,
        bounding_box.bottom_right.x + 1.0,
        bounding_box.top_left.y - 1.0,
        bounding_box.bottom_right.y + 1.0,
      )
      .render(position, canvas);
    }
  }
}

impl<T: RenderTarget> Render<T> for Entity {
  fn render(&self, position: Mat3, canvas: &mut Canvas<T>) {
    let entity_position =
      position * translation(self.position.state) * Mat3::rotation(self.angle.state);

    for block in &self.blocks {
      let block_position =
        entity_position * translation(block.offset) * Mat3::rotation(block.angle);
      block.shape.render(block_position, canvas);
    }

    self.shape.render(entity_position, canvas);
  }
}

impl<T: RenderTarget> Render<T> for Polygon {
  fn render(&self, position: Mat3, canvas: &mut Canvas<T>) {
    let lines = (position * self.clone()).to_segments();
    for line in lines {
      canvas
        .draw_line(into_point(line.a), into_point(line.b))
        .expect("Draw line");
    }
  }
}

fn render_stars<T: RenderTarget>(view: &View, canvas: &mut Canvas<T>) {
  let color = (view.stars_position.velocity.length() * 2.0).min(120.0) as u8 + 80;
  canvas.set_draw_color(Color::RGB(color, color, color));

  let center = into_vec(canvas.viewport().center());
  let stars = &view.stars;

  let view_position = modulo(&view.stars_position.state, &stars.field_size);

  let star_offset = Vec3::from(view_position - center);

  let points: Vec<Point> = stars
    .points
    .iter()
    .map(|point| {
      let position = *point - star_offset;
      let wrapped = Vec3 {
        x: around_zero(position.x, stars.field_size.x),
        y: around_zero(position.y, stars.field_size.y),
        z: position.z,
      };
      wrapped.into_cartesian() + center
    })
    .map(into_point)
    .collect();

  canvas.draw_points(&points[..]).expect("Draw star points");
}

/// Maps value to the interval of width `width` centered around zero.
fn around_zero(value: f32, width: f32) -> f32 {
  if value < -0.5 * width {
    value + width
  } else if value > 0.5 * width {
    value - width
  } else {
    value
  }
}

pub fn into_point(vec: Vec2<f32>) -> Point {
  Point::new(vec.x as i32, vec.y as i32)
}

pub fn into_vec(point: Point) -> Vec2<f32> {
  Vec2::new(point.x as f32, point.y as f32)
}
