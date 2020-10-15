use crate::world::{Entity, World};
use crate::math::lu::solve_lu;
use crate::math::vec::*;
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

    pub fn load(&mut self) {
        self.hud.load_saved_entities(self.view.size);
    }

    pub fn tick(&mut self, world: &mut World) {
        self.controlled_entity = world.find_entity(&self.controlled_entity);

        self.view.tick();
        self.hud.tick(world, self.controlled_entity);

        let actions = self
            .user_controls
            .poll_actions()
            .chain(self.hud.poll_actions());

        for action in actions {
            let action = Client::map_action(&self.view, action);
            if let Action::LoadEntity { filename } = action {
                Client::spawn_entity(world, filename, self.controlled_entity);
            } else if let Some(entity) = world.get_entity_mut(&self.controlled_entity) {
                entity.apply_action(action);
            }
        }
    }

    fn map_action(view: &View, a: Action) -> Action {
        let invert_transform = view.last_grid_to_screen;
        match a {
            Action::JoinEntity { mut entity } => {
                entity.position.state =
                    solve_lu(&invert_transform, entity.position.state.into_homogeneous())
                        .into_cartesian();
                Action::JoinEntity { entity }
            }
            _ => a,
        }
    }

    fn spawn_entity(world: &mut World, filename: String, controlling: EntityId) {
        if let Ok(entity) = Entity::load_from_file(filename.into()) {
            if let Some(grid) = world.grids.get_mut(&controlling.grid_id) {
                let position = grid
                    .get_entity(controlling.entity_id)
                    .map(|e| e.position.state)
                    .unwrap_or_default();
                grid.spawn_entity(position, entity);
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
