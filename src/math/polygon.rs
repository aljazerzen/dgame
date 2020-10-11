use super::line::Line;
use super::segment::Segment;
use super::vec::*;
use gamemath::{Mat3, Vec2, Vec3};
use std::iter::Iterator;

#[derive(Clone, Debug)]
pub struct Polygon {
  /// Vertices of the polygon in homogeneous coordinates.
  pub points: Vec<Vec3<f32>>,
}

impl Polygon {
  pub fn is_empty(self: &Self) -> bool {
    self.points.is_empty()
  }

  pub fn to_segments(&self) -> Vec<Segment> {
    (0..self.points.len())
      .map(|i1| {
        let i2 = (i1 + 1) % self.points.len();

        Segment {
          a: self.points[i1].into_cartesian(),
          b: self.points[i2].into_cartesian(),
        }
      })
      .collect()
  }

  pub fn intrude_point(self: &mut Self, point: Vec2<f32>) {
    let point_hom = point.into_homogeneous();
    let distances: Vec<f32> = self
      .points
      .iter()
      .map(|p| (*p - point_hom).length())
      .collect();

    let min_distance = distances
      .iter()
      .fold(std::f32::MAX, |acc, d| if acc < *d { acc } else { *d });

    if let Some(closest) = distances.iter().position(|d| (*d - min_distance).abs() < std::f32::EPSILON) {
      let prev = (closest + self.points.len() - 1) % self.points.len();
      let next = (closest + 1) % self.points.len();

      let between = self.points[next] - self.points[prev];
      let to_closest = self.points[closest] - self.points[prev];
      let to_point = point_hom - self.points[prev];

      let closest_position = to_closest.dot(between) / between.length();
      let point_position = to_point.dot(between) / between.length();

      let insert_to_index = if closest_position < point_position {
        next
      } else {
        closest
      };

      self.points.insert(insert_to_index, point_hom);
    }
  }

  pub fn contains_point(&self, point: Vec2<f32>) -> bool {
    let mut is_in = false;

    let horizontal_line = Line::horizontal(point.y);

    for segment in self.to_segments() {
      if let Some(intersection) = segment.intersection_line(&horizontal_line) {
        if point == intersection {
          // on polygon edge
          return true;
        }
        if point.x < intersection.x {
          is_in = !is_in;
        }
      }
    }
    is_in
  }

  pub fn area_and_centroid(&self) -> (f32, Vec2<f32>) {
    let all = self.points.len();

    let mut sum_area = 0.0;
    let mut sum = Vec2::default();
    for i in 0..all {
      let this = self.points[i].into_cartesian();
      let next = self.points[(i + 1) % all].into_cartesian();

      let cross = this.x * next.y - this.y * next.x;

      sum_area += cross;
      sum += (this + next) * cross;
    }

    let area = sum_area / 2.0;
    let centroid = sum * (1.0 / area / 6.0);
    (area, centroid)
  }

  pub fn radius_of_gyration(&self, offset: Vec2<f32>) -> f32 {
    let all = self.points.len();
    let mut sum = 0.0;
    for point in &self.points {
      let from_origin = point.into_cartesian() + offset;
      sum += from_origin.length_squared();
    }

    sum / all as f32
  }

  pub fn mul_left(&mut self, left: Mat3) {
    for p in &mut self.points {
      *p = left * *p;
    }
  }

  pub fn intersect_line_segment(&self, segment: Segment) -> Option<(f32, Vec2<f32>)> {
    let segment_direction = segment.direction();

    let mut first_intersection = None;
    let mut min_alpha = -1.0;

    for edge in &self.to_segments() {
      if let Some((alpha_p, _alpha_q)) = segment.intersection(edge) {
        if first_intersection == None || alpha_p < min_alpha {
          min_alpha = alpha_p;
          first_intersection = Some(segment.a + segment_direction * alpha_p);
        }
      }
    }
    first_intersection.map(|int| (min_alpha, int))
  }

  pub fn intercept_polygon(
    &self,
    poly: &Polygon,
    path: Vec2<f32>,
  ) -> Option<(f32, Vec<Vec2<f32>>)> {
    let mut intersections: Vec<Vec2<f32>> = Vec::new();
    let mut min_alpha: f32 = -1.0;

    let mut on_new_intersection = |(alpha, intersection)| {
      if min_alpha < 0.0 || alpha < min_alpha {
        min_alpha = alpha;
        intersections = Vec::new();
      } else if (alpha - min_alpha).abs() < std::f32::EPSILON {
        intersections.push(intersection);
      }
    };

    for point in &poly.points {
      let point_cart = point.into_cartesian();
      let segment = Segment::new(point_cart, point_cart + path);

      if let Some(int) = self.intersect_line_segment(segment) {
        on_new_intersection(int);
      }
    }

    let reverse_path = path * (-1.0);
    for point in &self.points {
      let point_cart = point.into_cartesian();
      let segment = Segment::new(point_cart, point_cart + reverse_path);

      if let Some(int) = poly.intersect_line_segment(segment) {
        on_new_intersection(int);
      }
    }

    if min_alpha >= 0.0 {
      Some((min_alpha, intersections))
    } else {
      None
    }
  }
}

impl From<Vec<Vec2<f32>>> for Polygon {
  fn from(points: Vec<Vec2<f32>>) -> Polygon {
    Polygon {
      points: points.iter().map(|p| p.into_homogeneous()).collect(),
    }
  }
}

impl From<&[[f32; 2]]> for Polygon {
  fn from(points: &[[f32; 2]]) -> Polygon {
    Polygon {
      points: points.iter().map(|p| Vec3::new(p[0], p[1], 1.0)).collect(),
    }
  }
}

impl std::ops::Mul<Polygon> for Mat3 {
  type Output = Polygon;

  fn mul(self, right: Polygon) -> Polygon {
    Polygon {
      points: right.points.iter().map(|p| self * *p).collect(),
    }
  }
}

pub fn construct_rect_poly(left: f32, right: f32, top: f32, bottom: f32) -> Polygon {
  Polygon::from(vec![
    Vec2 { x: left, y: top },
    Vec2 { x: left, y: bottom },
    Vec2 {
      x: right,
      y: bottom,
    },
    Vec2 { x: right, y: top },
  ])
}

pub fn construct_rect_poly_centered(width: f32, height: f32) -> Polygon {
  construct_rect_poly(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0)
}
