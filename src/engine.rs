use crate::entity::Entity;
use crate::grid::Grid;
use crate::render::View;
use gamemath::Vec2;
use std::collections::HashMap;

pub fn engine_tick(mut root_grid: Grid, view: &mut View) -> Grid {
  root_grid = split_grids(root_grid);
  
  grids_tick(&mut root_grid, Some(view));

  root_grid.relink();

  root_grid = join_grids(root_grid);

  root_grid
}

fn split_grids(mut grid: Grid) -> Grid {
  if grid.should_split() {
    grid = grid.split_by_position();
  }

  grid.map_children(&split_grids);

  grid
}

fn join_grids(mut grid: Grid) -> Grid {
  if let Some(join_with) = grid.should_join() {
    grid = grid.join_child(join_with);
  }

  grid.map_children(&join_grids);

  grid
}

fn grids_tick(grid: &mut Grid, associated_view: Option<&mut View>) {
  for child in &mut grid.children {
    if let Some(relation) = &mut child.relation_to_parent {
      relation.state += relation.velocity;
    }

    grids_tick(child, None);
  }

  let common_insist = grid.absorb_common_insist();
  if let Some(view) = associated_view {
    view.offset += common_insist.state;
    view.stars_position += common_insist;
    view.stars_position.state += view.stars_position.velocity;
  }

  entities_tick(grid);
}

fn entities_tick(grid: &mut Grid) {
  // update velocity
  for entity in &mut grid.entities {
    entity.tick();

    let mut dv = Vec2::default();
    let mut dfv = 0.0;

    // Center gravity
    // const distance = c.r.difference(massPoint.r);
    // const force = c.mass * massPoint.mass * G / distance.length / distance.length;
    // const a = force / massPoint.mass;
    // dv.add(distance.product(a / distance.length));

    // Thrust
    let thrust = entity.force();
    dv += thrust.force * (1.0 / entity.mass);
    dfv += thrust.torque / entity.mass_angular;

    entity.position.velocity += dv;
    entity.angle.velocity += dfv;
  }

  // collision detection
  let collisions = get_collisions(&grid.entities);
  // update state
  for (index, entity) in &mut grid.entities.iter_mut().enumerate() {
    if let Some(_collision) = collisions.get(&index) {
      entity.position.velocity = Vec2::default();
      entity.angle.velocity = 0.0;
    }

    entity.position.state += entity.position.velocity;
    entity.angle.state += entity.angle.velocity;
  }
}

fn get_collisions(entities: &[Entity]) -> HashMap<usize, Collision> {
  let mut collisions = HashMap::new();
  // polygon cache
  let mut polys = Vec::with_capacity(entities.len());
  for entity in entities {
    polys.push(entity.projection_to_grid() * entity.shape.clone());
  }

  for (index, entity) in entities.iter().enumerate() {
    for (collided_index, collided_entity) in entities.iter().enumerate() {
      if index <= collided_index {
        continue;
      }

      let res = polys[collided_index].intercept_polygon(
        &polys[index],
        entity.position.velocity - collided_entity.position.velocity,
      );

      if let Some((alpha, intersections)) = res {
        collisions.insert(
          index,
          Collision {
            alpha,
            intersections: intersections.clone(),
          },
        );
        collisions.insert(
          collided_index,
          Collision {
            alpha,
            intersections: intersections.clone(),
          },
        );
      }
    }
  }

  collisions
}

#[allow(dead_code)]
struct Collision {
  alpha: f32,
  intersections: Vec<Vec2<f32>>,
}
