
use super::line::Line;
use super::segment::Segment;
use super::vec::*;
use gamemath::{Mat3, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::iter::Iterator;
use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Polygon {
    /// Vertices of the polygon in homogeneous coordinates.
    #[serde_as(as = "Vec<Vec3Serde<f32>>")]
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

    pub fn intersection(self, right: Self) -> Vec<Self> {
        clipping::intersection(self, right)
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

        if let Some(closest) = distances
            .iter()
            .position(|d| (*d - min_distance).abs() < std::f32::EPSILON)
        {
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

    pub fn contains_polygon(&self, right: &Polygon) -> bool {
        right.points.iter().all(|p| self.contains_point(p.into_cartesian()))
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
            sum += from_origin.length();
        }

        sum / (all as f32) * sum / (all as f32)
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

mod clipping {
    use crate::math::{polygon::Polygon, segment::Segment, vec::*};
    use gamemath::{Vec2, Vec3};
    use std::iter::Iterator;

    pub fn intersection(a_poly: Polygon, b_poly: Polygon) -> Vec<Polygon> {
        // Greiner–Hormann clipping algorithm
        // http://www.inf.usi.ch/hormann/papers/Greiner.1998.ECO.pdf

        if a_poly.is_empty() {
            return vec![b_poly];
        }
        if b_poly.is_empty() {
            return vec![a_poly];
        }

        let mut a = PolygonLinked::new(&a_poly);
        let mut b = PolygonLinked::new(&b_poly);

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
            if b_poly.contains_point(a.nodes[0].r) {
                return vec![b_poly];
            } else if a_poly.contains_point(b.nodes[0].r) {
                return vec![a_poly];
            } else {
                return vec![a_poly, b_poly];
            }
        }

        {
            // figure out which intersection in poly a are entries into poly b
            let mut inside = b_poly.contains_point(a.nodes[0].r);
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
            let mut inside = a_poly.contains_point(b.nodes[0].r);
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
}
