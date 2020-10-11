use crate::grid::Grid;
use crate::render::{render, View};
use crate::ui::hud::Hud;
use crate::ui::user_controls::UserControls;
use gamemath::Vec2;
use sdl2::event::Event;
use sdl2::render::{Canvas, RenderTarget};

pub struct Client {
  pub view: View,
  hud: Hud,

  user_controls: UserControls,

  controlled_entity: EntityId,
}

#[derive(Clone, Copy)]
pub struct EntityId {
  pub entity_id: u64,
  pub grid_id: u64,
}

impl EntityId {
  pub fn new(grid_id: u64, entity_id: u64) -> Self {
    EntityId { grid_id, entity_id }
  }
}

impl Client {
  pub fn new(resolution: Vec2<f32>, controlled_entity: EntityId) -> Self {
    Client {
      view: View::new(resolution),
      hud: Hud::new(),
      user_controls: UserControls::default(),

      controlled_entity,
    }
  }

  pub fn tick(&mut self, root_grid: &mut Grid) {
    {
      let found = root_grid.visit_mut(&|g| {
        if g == &self.controlled_entity.grid_id {
          for entity in &mut g.entities {
            if entity == &self.controlled_entity.entity_id {
              entity.use_controls(&self.user_controls);
              return true;
            }
          }
        }
        false
      });

      if !found {
        self.controlled_entity = root_grid
          .visit(&|g| {
            for entity in &g.entities {
              if entity == &self.controlled_entity.entity_id {
                return Some(EntityId {
                  grid_id: g.get_id(),
                  entity_id: entity.get_id(),
                });
              }
            }
            None
          })
          .unwrap();
      }
    }
    self.view.tick();
    self
      .hud
      .update_trackers(self.view.size, &root_grid, self.controlled_entity);
  }

  pub fn get_controlled_entity(&self) -> EntityId {
    self.controlled_entity
  }

  pub fn render<T: RenderTarget>(&mut self, root_grid: &Grid, canvas: &mut Canvas<T>) {
    render(
      &root_grid,
      &self.controlled_entity,
      &mut self.view,
      canvas,
    );
    self.hud.render(canvas);
  }

  pub fn handle_event(&mut self, event: &Event) -> bool {
    self.hud.handle_event(event) || self.user_controls.handle_event(event, &self.view)
  }
}
