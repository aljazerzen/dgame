use super::{Entity, Insist, Thruster};
use crate::client::EntityId;
use crate::math::{
    bounding_box::{BoundingBox, RectBounds},
    polygon::{construct_rect_poly_centered, Polygon},
    vec::*,
};
use gamemath::Vec2;
use std::collections::HashMap;
use std::ops::Add;

const GRID_SPLIT_DISTANCE: f32 = 500.0;
const GRID_JOIN_DISTANCE: f32 = GRID_SPLIT_DISTANCE * 0.5;

#[derive(Debug)]
pub struct Grid {
    id: u64,
    parent: Option<GridRelation>,
    children: Vec<u64>,

    pub entities: Vec<Entity>,
}

impl Grid {
    pub fn new(parent: Option<GridRelation>, entities: Vec<Entity>) -> Self {
        use rand::RngCore;

        let mut rng = rand::thread_rng();
        Grid {
            id: rng.next_u64(),
            parent,
            children: Vec::new(),
            entities,
        }
    }

    pub fn get_entity_mut(&mut self, entity_id: u64) -> Option<&mut Entity> {
        for entity in &mut self.entities {
            if entity.get_id() == entity_id {
                return Some(entity);
            }
        }
        None
    }

    pub fn get_entity(&self, entity_id: u64) -> Option<&Entity> {
        for entity in &self.entities {
            if entity.get_id() == entity_id {
                return Some(entity);
            }
        }
        None
    }

    pub fn spawn_entity(&mut self, position: Vec2<f32>, mut entity: Entity) {
        let bounds = self.bounding_box();
        let closest_edge = bounds
            .polygon()
            .to_segments()
            .iter()
            .map(|segment| {
                let alpha = segment.project_point(position);
                let perpendicular = ((segment.direction() * alpha) + segment.a) - position;
                let distance = perpendicular.length();
                (perpendicular, distance)
            })
            .min_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|x| x.0);

        if let Some(closest_edge) = closest_edge {
            let entity_bounds = entity.shape.bounding_box();
            let entity_size = entity_bounds.bottom_right - entity_bounds.top_left;

            let direction = closest_edge.normalized();
            let offset = Vec2::new(direction.x * entity_size.x, direction.y * entity_size.y);
            let entity_position = closest_edge + offset + position;

            entity.position.state = entity_position;
            entity.position.velocity = Vec2::default();

            self.entities.push(entity);
        }
    }

    fn get_common_insist(&self) -> Insist<Vec2<f32>> {
        Insist::get_common(self.entities.iter().map(|e| &e.position).collect())
    }

    fn offset_entities(&mut self, insist: Insist<Vec2<f32>>) {
        for entity in &mut self.entities {
            entity.position += insist;
        }
    }

    pub fn tick_parent_relation(&mut self) {
        if let Some(p) = &mut self.parent {
            p.position.state += p.position.velocity;
        }
    }

    fn should_split(&self) -> bool {
        let mut bounding_box = RectBounds::default();
        for entity in &self.entities {
            bounding_box += entity.position.state;
        }
        bounding_box.size() > GRID_SPLIT_DISTANCE
    }

    fn split_by_position(&mut self) -> Option<Grid> {
        let all = self.entities.len();
        if all < 2 {
            return None;
        }

        let (a, b) = self.get_most_distanced_entities();
        let (parent_entities, child_entities) = Grid::segment_to_closest(&mut self.entities, a, b);
        self.entities = parent_entities;

        Some(Grid::new(Some(GridRelation::new(self.id)), child_entities))
    }

    fn segment_to_closest(
        entities: &mut Vec<Entity>,
        a: usize,
        b: usize,
    ) -> (Vec<Entity>, Vec<Entity>) {
        let a_position = entities[a].position.state;
        let b_position = entities[b].position.state;
        let mut a_entities: Vec<Entity> = vec![];
        let mut b_entities: Vec<Entity> = vec![];
        while !entities.is_empty() {
            let entity = entities.pop().unwrap();
            let dist_a = (entity.position.state - a_position).length();
            let dist_b = (entity.position.state - b_position).length();

            if dist_a < dist_b {
                a_entities.push(entity);
            } else {
                b_entities.push(entity);
            }
        }

        if a_entities.len() <= b_entities.len() {
            (a_entities, b_entities)
        } else {
            (b_entities, a_entities)
        }
    }

    fn get_most_distanced_entities(&self) -> (usize, usize) {
        let all = self.entities.len();

        // find the two most distanced entities
        let mut max_dist = -1.0;
        let mut a = 0;
        let mut b = 0;
        for i in 0..all {
            for j in (i + 1)..all {
                let dist =
                    (self.entities[i].position.state - self.entities[j].position.state).length();
                if dist > max_dist {
                    max_dist = dist;
                    a = i;
                    b = j;
                }
            }
        }

        (a, b)
    }

    // /// Reorganizes the graph of grids such that every grid is child or parent to its closest grid.
    // /// O(n^2)
    // pub fn relink(&mut self) {
    //   self.steal_children(&Vec::new());
    // }

    // fn steal_children(&mut self, ancestors: &[GridRelationWeak]) -> Vec<GridTransfer> {
    //   let mut descendant_transfers = Vec::new();

    //   let relation_to_parent = self.relation_to_parent.unwrap_or_default();

    //   let relations: Vec<GridRelationWeak> = ancestors
    //     .iter()
    //     .map(|r| r.clone().offset_relation(relation_to_parent))
    //     .chain(Some(GridRelationWeak::new(self.id)).into_iter())
    //     .collect();

    //   for child in &mut self.children {
    //     descendant_transfers.extend(child.steal_children(&relations));
    //   }

    //   let (to_me, to_ancestors): (Vec<GridTransfer>, Vec<GridTransfer>) = descendant_transfers
    //     .into_iter()
    //     .partition(|t| t.to_id == self.id);
    //   self.children.extend(to_me.into_iter().map(|t| t.grid));

    //   let mut transfers_to_ancestors = to_ancestors;

    //   let mut closer_to_me: Vec<Grid> = Vec::with_capacity(self.children.len());

    //   while !self.children.is_empty() {
    //     let mut child = self.children.pop().unwrap();

    //     let to_me = child.relation_to_parent.unwrap();
    //     let mut min_distance = to_me;
    //     let mut min_ancestor: Option<u64> = None;

    //     for ancestor in ancestors {
    //       let to_ancestor = ancestor.relation
    //         + self.relation_to_parent.unwrap_or_default()
    //         + child.relation_to_parent.unwrap_or_default();
    //       if to_ancestor.length_squared().state < min_distance.length_squared().state {
    //         min_distance = to_ancestor;
    //         min_ancestor = Some(ancestor.grid_id);
    //       }
    //     }

    //     if let Some(ancestor) = min_ancestor {
    //       child.relation_to_parent = Some(min_distance);

    //       transfers_to_ancestors.push(GridTransfer {
    //         grid: child,
    //         to_id: ancestor,
    //       })
    //     } else {
    //       closer_to_me.push(child);
    //     }
    //   }

    //   self.children.extend(closer_to_me);

    //   transfers_to_ancestors
    // }

    // pub fn get_relations<'a>(&'a self, relation: Insist<Vec2<f32>>) -> Vec<GridRelation<'a>> {
    //   let mut res = Vec::new();
    //   res.push(GridRelation {
    //     relation,
    //     grid: self,
    //   });

    //   for child in &self.children {
    //     if let Some(relation_to_parent) = child.relation_to_parent {
    //       let child_relation = relation_to_parent + relation;

    //       res.extend(child.get_descendant_relations(child_relation).into_iter())
    //     }
    //   }

    //   res
    // }
}

pub struct World {
    pub grids: HashMap<u64, Grid>,
}

impl World {
    pub fn grid_ids(&self) -> Vec<u64> {
        self.grids.keys().copied().collect()
    }

    pub fn absorb_common_insist(&mut self, focused_grid: u64) -> Option<Insist<Vec2<f32>>> {
        let mut res = None;
        for grid_id in &self.grid_ids() {
            let insist = self.grids[grid_id].get_common_insist();

            if insist.length_squared().is_zero() {
                continue;
            }

            let children;
            {
                let grid = self.grids.get_mut(grid_id).unwrap();
                grid.offset_entities(-insist);

                if let Some(p) = &mut grid.parent {
                    p.position += -insist;
                }
                children = grid.children.clone();
            }

            for child_id in &children {
                let child = self.grids.get_mut(child_id).unwrap();
                if let Some(p) = &mut child.parent {
                    p.position += insist;
                }
            }

            if grid_id == &focused_grid {
                res = Some(insist);
            }
        }
        res
    }

    pub fn split_grids(&mut self) {
        let mut new_grids = Vec::new();

        for grid in &mut self.grids.values_mut() {
            if grid.should_split() {
                if let Some(new_grid) = grid.split_by_position() {
                    new_grids.push(new_grid);
                }
            }
        }

        for grid in new_grids {
            self.add_grid(grid);
        }
    }

    pub fn add_grid(&mut self, grid: Grid) {
        let own_id = grid.id;
        let parent_id = grid.parent.as_ref().map(|p| p.id).unwrap();
        self.grids.insert(own_id, grid);

        let parent = self.grids.get_mut(&parent_id).unwrap();
        parent.children.push(own_id);
    }

    pub fn join_grids(&mut self) {
        let first_grid = self.grids.iter().next().map(|g| *g.0).unwrap();
        let relations = self.get_relations(first_grid, Insist::default());

        for relation in &relations {
            let join_with = relations
                .iter()
                .filter(|r| r.id != relation.id)
                .filter(|r| self.grids.contains_key(&r.id))
                .map(|r| {
                    let relative = r.position + -relation.position;
                    (r.id, relative, relative.state.length())
                })
                .filter(|r| r.2 < GRID_JOIN_DISTANCE)
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            if let Some(join_with) = join_with {
                if let Some(mut grid) = self.remove_grid(relation.id) {
                    for c in &mut grid.entities {
                        c.position += -join_with.1;
                    }
                    let parent = &mut self.grids.get_mut(&join_with.0).unwrap();
                    parent.entities.extend(grid.entities);
                }
            }
        }
    }

    pub fn remove_grid(&mut self, grid: u64) -> Option<Grid> {
        if let Some(mut grid) = self.grids.remove(&grid) {
            let new_parent = grid.parent.clone().or_else(|| {
                if grid.children.is_empty() {
                    return None;
                }

                let first_child = self.grids.get_mut(&grid.children.pop().unwrap()).unwrap();
                let position = first_child.parent.as_ref().unwrap().position;
                first_child.parent = None;

                Some(GridRelation {
                    id: first_child.id,
                    position,
                })
            });

            if let Some(new_parent) = new_parent {
                if let Some(p) = &mut self.grids.get_mut(&new_parent.id) {
                    if let Some(child_index) = p.children.iter().position(|c| c == &grid.id) {
                        p.children.remove(child_index);
                    }
                    p.children.extend(grid.children.clone());
                }

                for c in &grid.children {
                    if let Some(c) = &mut self.grids.get_mut(&c) {
                        c.parent = Some(new_parent.clone() + c.parent.as_ref().unwrap().position);
                    }
                }

                Some(grid)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_relations(&self, grid_id: u64, position: Insist<Vec2<f32>>) -> Vec<GridRelation> {
        if let Some(p) = &self.grids[&grid_id].parent {
            self.get_relations(p.id, position + p.position)
        } else {
            self.get_descendant_relations(grid_id, position)
        }
    }

    fn get_descendant_relations(&self, id: u64, position: Insist<Vec2<f32>>) -> Vec<GridRelation> {
        let mut res = Vec::new();
        res.push(GridRelation { position, id });

        for child_id in self.grids[&id].children.clone() {
            let child_position = position
                + self.grids[&child_id]
                    .parent
                    .as_ref()
                    .map(|p| -p.position)
                    .unwrap_or_default();

            res.extend(self.get_descendant_relations(child_id, child_position));
        }

        res
    }

    #[allow(dead_code)]
    pub fn get_relation_between(
        &self,
        a: u64,
        b: u64,
        relation: Insist<Vec2<f32>>,
    ) -> Insist<Vec2<f32>> {
        if let Some(p) = &self.grids[&a].parent {
            self.get_relation_between(p.id, b, relation + p.position)
        } else if let Some(p) = &self.grids[&b].parent {
            self.get_relation_between(p.id, b, relation + p.position)
        } else {
            relation
        }
    }

    pub fn find_entity(&self, id: &EntityId) -> EntityId {
        if self
            .grids
            .get(&id.grid_id)
            .map(|g| g.get_entity(id.entity_id))
            .flatten()
            .is_some()
        {
            return *id;
        }

        // entity may have changed grid, search all grids
        for grid in self.grids.values() {
            if let Some(entity) = grid.get_entity(id.entity_id) {
                return EntityId {
                    grid_id: grid.id,
                    entity_id: entity.get_id(),
                };
            }
        }
        panic!("cannot find controlled entity");
    }

    pub fn get_entity_mut<'a>(&'a mut self, id: &EntityId) -> Option<&'a mut Entity> {
        self.grids
            .get_mut(&id.grid_id)
            .unwrap()
            .get_entity_mut(id.entity_id)
    }
}

#[derive(Clone, Debug)]
pub struct GridRelation {
    pub position: Insist<Vec2<f32>>,
    pub id: u64,
}

impl GridRelation {
    pub fn new(id: u64) -> Self {
        GridRelation {
            id,
            position: Insist::default(),
        }
    }
}

impl Add<Insist<Vec2<f32>>> for GridRelation {
    type Output = GridRelation;

    fn add(self, right: Insist<Vec2<f32>>) -> Self {
        GridRelation {
            id: self.id,
            position: self.position + right,
        }
    }
}

impl PartialEq<Grid> for Grid {
    fn eq(&self, right: &Grid) -> bool {
        self.id == right.id
    }
}

impl PartialEq<u64> for Grid {
    fn eq(&self, right: &u64) -> bool {
        self.id == *right
    }
}

#[allow(dead_code)]
pub fn construct_demo_world() -> World {
    let mut grids = HashMap::new();

    {
        let mut grid = Grid::new(None, Vec::new());

        let a = construct_rect_poly_centered(50.0, 70.0);

        let _b = Polygon::from(vec![
            Vec2 { x: 11.0, y: -68.0 },
            Vec2 { x: -3.0, y: -49.0 },
            Vec2 { x: -20.0, y: -54.0 },
            Vec2 { x: -25.0, y: -35.0 },
            Vec2 { x: -33.0, y: 1.0 },
            Vec2 { x: -25.0, y: 35.0 },
            Vec2 { x: -8.0, y: 53.0 },
            Vec2 { x: 25.0, y: 35.0 },
            Vec2 { x: 17.0, y: 21.0 },
            Vec2 { x: 41.0, y: 6.0 },
            Vec2 { x: 42.0, y: -20.0 },
            Vec2 { x: 25.0, y: -35.0 },
        ]);

        let _c = Polygon::from(vec![
            Vec2 { x: 146.0, y: 129.0 },
            Vec2 { x: 144.0, y: 122.0 },
            Vec2 { x: 148.0, y: 102.0 },
            Vec2 { x: 143.0, y: 105.0 },
            Vec2 { x: 136.0, y: 111.0 },
            Vec2 { x: 132.0, y: 110.0 },
            Vec2 { x: 125.0, y: 112.0 },
            Vec2 { x: 95.0, y: 94.0 },
            Vec2 { x: 108.0, y: 106.0 },
            Vec2 { x: 125.0, y: 115.0 },
            Vec2 { x: 129.0, y: 119.0 },
            Vec2 { x: 128.0, y: 119.0 },
            Vec2 { x: 124.0, y: 129.0 },
            Vec2 { x: 125.0, y: 135.0 },
            Vec2 { x: 122.0, y: 145.0 },
            Vec2 { x: 109.0, y: 161.0 },
            Vec2 { x: 111.0, y: 134.0 },
            Vec2 { x: 112.0, y: 133.0 },
            Vec2 { x: 107.0, y: 137.0 },
            Vec2 { x: 102.0, y: 129.0 },
            Vec2 { x: 75.0, y: 135.0 },
            Vec2 { x: 73.0, y: 135.0 },
            Vec2 { x: 73.0, y: 111.0 },
            Vec2 { x: 66.0, y: 98.0 },
            Vec2 { x: 72.0, y: 84.0 },
            Vec2 { x: 66.0, y: 57.0 },
            Vec2 { x: 75.0, y: 65.0 },
            Vec2 { x: 89.0, y: 69.0 },
            Vec2 { x: 89.0, y: 66.0 },
            Vec2 { x: 93.0, y: 52.0 },
            Vec2 { x: 120.0, y: 65.0 },
            Vec2 { x: 125.0, y: 72.0 },
            Vec2 { x: 123.0, y: 70.0 },
            Vec2 { x: 121.0, y: 55.0 },
            Vec2 { x: 123.0, y: 54.0 },
            Vec2 { x: 110.0, y: 53.0 },
            Vec2 { x: 118.0, y: 53.0 },
            Vec2 { x: 125.0, y: 45.0 },
            Vec2 { x: 97.0, y: 32.0 },
            Vec2 { x: 81.0, y: 19.0 },
            Vec2 { x: 112.0, y: 22.0 },
            Vec2 { x: 130.0, y: 26.0 },
            Vec2 { x: 147.0, y: 31.0 },
            Vec2 { x: 135.0, y: 3.0 },
            Vec2 { x: 156.0, y: 4.0 },
            Vec2 { x: 161.0, y: 12.0 },
            Vec2 { x: 170.0, y: 19.0 },
            Vec2 { x: 179.0, y: 19.0 },
            Vec2 { x: 199.0, y: 23.0 },
            Vec2 { x: 181.0, y: -18.0 },
            Vec2 { x: 206.0, y: -18.0 },
            Vec2 { x: 226.0, y: -2.0 },
            Vec2 { x: 230.0, y: 7.0 },
            Vec2 { x: 216.0, y: 32.0 },
            Vec2 { x: 208.0, y: 38.0 },
            Vec2 { x: 245.0, y: 71.0 },
            Vec2 { x: 231.0, y: 63.0 },
            Vec2 { x: 213.0, y: 74.0 },
            Vec2 { x: 212.0, y: 60.0 },
            Vec2 { x: 238.0, y: 48.0 },
            Vec2 { x: 188.0, y: 34.0 },
            Vec2 { x: 176.0, y: 44.0 },
            Vec2 { x: 175.0, y: 45.0 },
            Vec2 { x: 178.0, y: 56.0 },
            Vec2 { x: 173.0, y: 62.0 },
            Vec2 { x: 176.0, y: 68.0 },
            Vec2 { x: 175.0, y: 90.0 },
            Vec2 { x: 171.0, y: 81.0 },
            Vec2 { x: 174.0, y: 80.0 },
            Vec2 { x: 165.0, y: 50.0 },
            Vec2 { x: 151.0, y: 49.0 },
            Vec2 { x: 149.0, y: 60.0 },
            Vec2 { x: 138.0, y: 54.0 },
            Vec2 { x: 133.0, y: 51.0 },
            Vec2 { x: 130.0, y: 52.0 },
            Vec2 { x: 130.0, y: 53.0 },
            Vec2 { x: 139.0, y: 57.0 },
            Vec2 { x: 155.0, y: 63.0 },
            Vec2 { x: 192.0, y: 60.0 },
            Vec2 { x: 190.0, y: 102.0 },
            Vec2 { x: 200.0, y: 97.0 },
            Vec2 { x: 202.0, y: 97.0 },
            Vec2 { x: 211.0, y: 90.0 },
            Vec2 { x: 222.0, y: 95.0 },
            Vec2 { x: 239.0, y: 95.0 },
            Vec2 { x: 232.0, y: 112.0 },
            Vec2 { x: 215.0, y: 122.0 },
            Vec2 { x: 206.0, y: 111.0 },
            Vec2 { x: 191.0, y: 86.0 },
            Vec2 { x: 167.0, y: 101.0 },
            Vec2 { x: 157.0, y: 122.0 },
            Vec2 { x: 159.0, y: 112.0 },
            Vec2 { x: 147.0, y: 89.0 },
            Vec2 { x: 134.0, y: 83.0 },
            Vec2 { x: 142.0, y: 80.0 },
            Vec2 { x: 159.0, y: 92.0 },
            Vec2 { x: 158.0, y: 98.0 },
            Vec2 { x: 160.0, y: 101.0 },
            Vec2 { x: 175.0, y: 115.0 },
            Vec2 { x: 182.0, y: 119.0 },
            Vec2 { x: 180.0, y: 122.0 },
            Vec2 { x: 182.0, y: 128.0 },
            Vec2 { x: 183.0, y: 135.0 },
            Vec2 { x: 177.0, y: 143.0 },
            Vec2 { x: 166.0, y: 152.0 },
            Vec2 { x: 147.0, y: 168.0 },
            Vec2 { x: 130.0, y: 169.0 },
            Vec2 { x: 142.0, y: 149.0 },
            Vec2 { x: 142.0, y: 133.0 },
        ]);

        {
            use std::f32::consts::{FRAC_PI_2, PI};
            let entity = Entity::new(
                a.clone(),
                vec![
                    Box::from(Thruster::new(20.0, Vec2::new(0.0, 10.0), 0.0)),
                    Box::from(Thruster::new(20.0, Vec2::new(-10.0, 0.0), FRAC_PI_2)),
                    Box::from(Thruster::new(20.0, Vec2::new(10.0, 0.0), -FRAC_PI_2)),
                    Box::from(Thruster::new(20.0, Vec2::new(0.0, -10.0), PI)),
                ],
            );

            grid.entities.push(entity);
        }

        {
            // let mut child = Grid::default();
            // child.relation_to_parent = Some(Insist {
            // state: Vec2 { x: 0.0, y: 0.0 },
            // velocity: Vec2 { x: 1.0, y: 0.0 },
            // });

            let mut entity = Entity::new(a, vec![]);

            entity.position.state = Vec2 { x: 100.0, y: 60.0 };

            grid.entities.push(entity);

            // grid.children.push(child);
        }

        {
            let mut entity = Entity::new(
                translation(Vec2 {
                    x: 100.0,
                    y: -100.0,
                }) * construct_rect_poly_centered(50.0, 70.0),
                vec![],
            );
            entity.angle.state = 1.0;

            grid.entities.push(entity);
        }

        grids.insert(grid.id, grid);
    }

    World { grids }
}
