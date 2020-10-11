use super::polygon::Polygon;
use super::segment::Segment;
use super::vec::*;
use gamemath::{Vec2, Vec3};
use std::iter::Iterator;

impl Polygon {
  pub fn intersection(self, other: Polygon) -> Vec<Polygon> {
    // Greiner–Hormann clipping algorithm
    // http://www.inf.usi.ch/hormann/papers/Greiner.1998.ECO.pdf

    if self.is_empty() {
      return vec![other];
    }
    if other.is_empty() {
      return vec![self];
    }

    let mut a = PolygonLinked::new(&self);
    let mut b = PolygonLinked::new(&other);

    let mut intersection_found = false;

    let mut a_end: usize = 0;
    loop {
      let a_start = a_end;
      a_end = a.find_forward_non_intersection(a.nodes[a_end].next);

      let a_edge = Segment::new(a.nodes[a_start].r, a.nodes[a_end].r);

      let mut b_end: usize = 0;
      loop {
        let b_start = b_end;
        b_end = b.find_forward_non_intersection(b.nodes[b_start].next);

        let b_edge = Segment::new(b.nodes[b_start].r, b.nodes[b_end].r);
        let intersection = a_edge.intersection(&b_edge);

        if let Some((alpha_a, alpha_b)) = intersection {
          let intersection_point = a_edge.a + (a_edge.direction() * alpha_a);
          intersection_found = true;

          let new_a_node = a.insert_intersection(
            intersection_point,
            a_start,
            Intersection {
              alpha: alpha_a,
              entry: false,
              neighbor: None,
            },
          );

          let new_b_node = b.insert_intersection(
            intersection_point,
            b_start,
            Intersection {
              alpha: alpha_b,
              entry: false,
              neighbor: Some(new_a_node),
            },
          );

          if let Some(int) = &mut a.nodes[new_a_node].intersection {
            int.neighbor = Some(new_b_node)
          }
        }
        if b_end == 0 {
          break;
        }
      }
      if a_end == 0 {
        break;
      }
    }

    if !intersection_found {
      if other.contains_point(a.nodes[0].r) {
        return vec![other];
      } else if self.contains_point(b.nodes[0].r) {
        return vec![self];
      } else {
        return vec![self, other];
      }
    }

    {
      // figure out which intersection in poly a are entries into poly b
      let mut inside = other.contains_point(a.nodes[0].r);
      let mut pos_a = 0;
      loop {
        if let Some(intersection) = &mut a.nodes[pos_a].intersection {
          inside = !inside;
          intersection.entry = inside;
        }
        pos_a = a.nodes[pos_a].next;
        if pos_a == 0 {
          break;
        }
      }
    }

    {
      // figure out which intersection in poly b are entries into poly a
      let mut inside = self.contains_point(b.nodes[0].r);
      let mut pos_b = 0;
      loop {
        if let Some(intersection) = &mut b.nodes[pos_b].intersection {
          inside = !inside;
          intersection.entry = inside;
        }
        pos_b = b.nodes[pos_b].next;
        if pos_b == 0 {
          break;
        }
      }
    }
    let mut points: Vec<Vec3<f32>> = Vec::new();
    let first_intersection = BiPolygonNode {
      index: a.find_forward_intersection(0),
      is_in_a: true,
    };
    let mut current: BiPolygonNode = first_intersection;
    loop {
      let direction = current.get(&a, &b).intersection.as_ref().unwrap().entry;
      loop {
        let node = current.get(&a, &b);
        points.push(node.r.into_homogeneous());
        current.step_to(if direction { node.prev } else { node.next });

        if let Some(..) = current.get(&a, &b).intersection {
          break;
        }
      }
      current.step_over(&a, &b);
      if current == first_intersection {
        break;
      }
    }

    return vec![Polygon { points: points }];
  }
}

/// Reference to a node in one of two polygons
#[derive(PartialEq, Clone, Copy)]
struct BiPolygonNode {
  index: usize,
  is_in_a: bool,
}

impl BiPolygonNode {
  fn get<'a>(&self, a: &'a PolygonLinked, b: &'a PolygonLinked) -> &'a PolygonLinkedNode {
    let poly = if self.is_in_a { a } else { b };
    &poly.nodes[self.index]
  }

  fn step_over(&mut self, a: &PolygonLinked, b: &PolygonLinked) {
    self.index = self
      .get(a, b)
      .intersection
      .as_ref()
      .unwrap()
      .neighbor
      .unwrap();
    self.is_in_a = !self.is_in_a;
  }

  fn step_to(&mut self, index: usize) {
    self.index = index;
  }
}

struct Intersection {
  neighbor: Option<usize>,
  alpha: f32,
  entry: bool,
}

struct PolygonLinkedNode {
  r: Vec2<f32>,
  this: usize,
  next: usize,
  prev: usize,

  intersection: Option<Intersection>,
}

#[derive(Default)]
struct PolygonLinked {
  nodes: Vec<PolygonLinkedNode>,
}

impl PolygonLinked {
  fn new(polygon: &Polygon) -> PolygonLinked {
    let mut this = PolygonLinked {
      nodes: Vec::with_capacity(polygon.points.len()),
    };

    let all = polygon.points.len();

    for (index, point) in polygon.points.iter().enumerate() {
      let node = PolygonLinkedNode {
        r: point.into_cartesian(),
        this: index,
        next: (index + 1) % all,
        prev: (index + all - 1) % all,
        intersection: None,
      };

      this.nodes.push(node);
    }

    this
  }

  fn insert_intersection(
    &mut self,
    r: Vec2<f32>,
    position: usize,
    intersection: Intersection,
  ) -> usize {
    let aligned_position = self.align_intersection_alpha(position, intersection.alpha);
    let node_index = self.nodes.len();
    let mut insert_after = &mut self.nodes[aligned_position];

    let node = PolygonLinkedNode {
      r,

      this: node_index,
      next: insert_after.next,
      prev: insert_after.this,

      intersection: Some(intersection),
    };

    insert_after.next = node.this;
    self.nodes[node.next].prev = node.this;

    self.nodes.push(node);

    node_index
  }

  fn align_intersection_alpha(&mut self, index: usize, alpha: f32) -> usize {
    let mut insert_after = &self.nodes[index];
    while let Some(intersection) = &insert_after.intersection {
      if intersection.alpha <= alpha {
        break;
      }
      insert_after = &self.nodes[insert_after.prev];
    }
    while let Some(intersection) = &self.nodes[insert_after.next].intersection {
      if intersection.alpha >= alpha {
        break;
      }

      insert_after = &self.nodes[insert_after.next];
    }
    insert_after.this
  }

  fn find_forward<P>(&self, start: usize, predicate: P) -> usize
  where
    P: Fn(&PolygonLinkedNode) -> bool,
  {
    let mut node = &self.nodes[start];
    while !predicate(node) {
      node = &self.nodes[node.next];
    }

    node.this
  }

  fn find_forward_non_intersection(&self, start: usize) -> usize {
    self.find_forward(start, |node| node.intersection.is_none())
  }

  fn find_forward_intersection(&self, start: usize) -> usize {
    self.find_forward(start, |node| node.intersection.is_some())
  }
}
