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
      hud: Hud::new(resolution),
      user_controls: UserControls::default(),

      controlled_entity,
    }
  }

  pub fn tick(&mut self, root_grid: &mut Grid) {
    self.grid_use_controls(root_grid);
    self.view.tick();
    self
      .hud
      .update_trackers(self.view.size, &root_grid, self.controlled_entity);
  }

  fn grid_use_controls(&mut self, root_grid: &mut Grid) {
    let id = self.controlled_entity;

    if let Some(entity) = root_grid.find_entity(&id) {
      entity.use_controls(&self.user_controls);
    } else {
      // entity may have changed grid, search all grids
      if let Some(grid) = root_grid.find(&|g| g.get_entity(id.entity_id).is_some()) {
        let grid_id = grid.get_id();
        if let Some(entity) = grid.get_entity_mut(id.entity_id) {
          self.controlled_entity = EntityId {
            grid_id,
            entity_id: entity.get_id(),
          };
          entity.use_controls(&self.user_controls);
          return;
        }
      }
    }
  }

  pub fn get_controlled_entity(&self) -> EntityId {
    self.controlled_entity
  }

  pub fn render<T: RenderTarget>(&mut self, root_grid: &Grid, canvas: &mut Canvas<T>) {
    render(&root_grid, &self.controlled_entity, &mut self.view, canvas);
    self.hud.render(canvas);
  }

  pub fn handle_event(&mut self, event: &Event) -> bool {
    self.hud.handle_event(event) || self.user_controls.handle_event(event, &self.view)
  }
}
