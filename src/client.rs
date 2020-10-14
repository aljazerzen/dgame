use crate::entity::Entity;
use crate::grid::World;
use crate::render::{render, View};
use crate::ui::hud::Hud;
use crate::ui::user_controls::{Action, UserControls};
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
            view: View::new(resolution, controlled_entity),
            hud: Hud::new(resolution),
            user_controls: UserControls::default(),

            controlled_entity,
        }
    }

    pub fn tick(&mut self, world: &mut World) {
        self.controlled_entity = world.find_entity(&self.controlled_entity);

        self.view.tick();
        self.hud.tick(&self.view, world, self.controlled_entity);

        let actions = self.user_controls.poll_actions();
        for action in actions {
            if let Action::LoadEntity { filename } = action {
                if let Some(entity) = Entity::load_from_file(filename).ok() {
                    if let Some(grid) = world.grids.get_mut(&self.controlled_entity.grid_id) {
                        let position = grid
                            .get_entity(self.controlled_entity.entity_id)
                            .map(|e| e.position.state)
                            .unwrap_or_default();
                        grid.spawn_entity(position, entity);
                    }
                }
            } else {
                if let Some(entity) = world.get_entity_mut(&self.controlled_entity) {
                    entity.apply_action(&action);
                }
            }
        }
    }

    pub fn render<T: RenderTarget>(&mut self, world: &World, canvas: &mut Canvas<T>) {
        render(&world, &self.controlled_entity, &mut self.view, canvas);
        self.hud.render(canvas);
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        self.hud.handle_event(event) || self.user_controls.handle_event(event, &self.view)
    }
}
