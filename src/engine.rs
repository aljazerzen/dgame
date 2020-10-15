use crate::world::{Grid, World, Entity};
use crate::render::View;
use gamemath::Vec2;
use std::collections::HashMap;

pub fn engine_tick(world: &mut World, view: &mut View) {
    world.split_grids();
    
    absorb_common_insists(world, view);

    for grid in world.grids.values_mut() {
        grid.tick_parent_relation();

        entities_tick(grid);
    }

    // world.relink();

    world.join_grids();
}

fn absorb_common_insists(world: &mut World, view: &mut View) {
    view.focus = world.find_entity(&view.focus);

    let common_insist = world.absorb_common_insist(view.focus.grid_id);
    if let Some(common_insist) = common_insist {
      view.offset += common_insist.state;
      view.stars_position += common_insist;
    }
    view.stars_position.velocity *= 0.999;
    view.stars_position.state += view.stars_position.velocity;
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
