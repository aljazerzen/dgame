use crate::client::EntityId;
use crate::entity::{Entity, Thruster};
use crate::grid::{GridRelation, Insist, World};
use crate::math::bounding_box::BoundingBox;
use crate::math::polygon::{construct_rect_poly, construct_rect_poly_centered, Polygon};
use crate::math::segment::Segment;
use crate::math::vec::*;
use crate::render::{into_vec, Render};
use crate::ui::user_controls::Action;
use gamemath::{Mat3, Vec2};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};

const TRACKER_PADDING: i32 = 30;

pub struct Hud {
    pub grid_trackers: Vec<GridRelation>,

    elements: Vec<HudElement>,

    action_queue: Vec<Action>,
}

impl Hud {
    pub fn new(view_size: Vec2<f32>) -> Hud {
        Hud {
            grid_trackers: Vec::new(),
            elements: vec![
                HudElement::new_toolbar_button(
                    Vec2::new(0, -1),
                    Entity::new_from_block(Box::from(Thruster::new(20.0, Vec2::default(), 0.0))),
                    view_size,
                ),
                HudElement::new_toolbar_button(
                    Vec2::new(1, -1),
                    Entity::new_from_block(Box::from(Thruster::new(30.0, Vec2::default(), 0.0))),
                    view_size,
                ),
                HudElement::new_toolbar_button(
                    Vec2::new(2, -1),
                    Entity::new_from_block(Box::from(Thruster::new(40.0, Vec2::default(), 0.0))),
                    view_size,
                ),
            ],
            action_queue: Vec::new(),
        }
    }

    pub fn load_saved_entities(&mut self, view_size: Vec2<f32>) {
        let mut index = 0;
        for path in Entity::list_saved().unwrap_or_else(|_| vec![]) {
            if let Ok(mut entity) = Entity::load_from_file(path) {
                entity.redistribute_weight();
                entity.position.state = Vec2::default();
                self.elements.push(HudElement::new_toolbar_button(
                    Vec2::new(index, -2),
                    entity,
                    view_size,
                ));

                index += 1;
            }
        }
    }

    pub fn poll_actions(&mut self) -> std::vec::Drain<'_, Action> {
        self.action_queue.drain(..)
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        for element in &mut self.elements {
            if element.handle_event(event) {
                return true;
            }
        }
        false
    }

    /// Pull data from & push actions to grids
    pub fn tick(&mut self, world: &mut World, focus: EntityId) {
        self.update_trackers(world, focus);

        for element in &mut self.elements {
            let mut actions = element.tick();

            self.action_queue.extend(actions.drain(..));
        }
    }

    pub fn update_trackers(&mut self, world: &World, focus: EntityId) {
        self.grid_trackers = world.get_relations(focus.grid_id, Insist::default());
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) {
        canvas.set_draw_color(Color::RGB(128, 128, 172));
        let center = into_vec(canvas.viewport().center());
        let padding = TRACKER_PADDING as f32 * 2.0;

        let poly = construct_rect_poly_centered(2.0 * center.x - padding, 2.0 * center.y - padding);

        for tracker in &self.grid_trackers {
            let ray = Segment::new(tracker.position.state, Vec2::default());

            if let Some((_alpha, intersection)) = poly.intersect_line_segment(ray) {
                let position = translation(center + intersection);

                let size = (20.0 + tracker.position.state.length() / -1000.0)
                    .min(15.0)
                    .max(2.0);
                let rect = construct_rect_poly_centered(size, size);
                rect.render(position, canvas);

                tracker.position.velocity.render(position, canvas);
            }
        }

        for element in &self.elements {
            element.draw(canvas);
        }
    }
}

trait UIElement<T: RenderTarget>: Render<T> {
    // fn click(location: Vec<i32>, controls: UserControls);

    // fn tick(controls: EventHandler) {
    // }

    // fn move(c: Vector, controls: EventHandler) -> Option<bool>;

    // fn end(c: Vector, controls: EventHandler) -> Option<bool>;

    // fn wheel(delta: number);
}

struct HudElement {
    position: Vec2<i32>,
    shape: Polygon,
    variant: HudElementVariant,
    dragging: bool,
}

const HUD_ELEMENT_SIZE: i32 = 40;

enum HudElementVariant {
    ToolbarButton {
        entity: Box<Entity>,
        scale: f32,
        ghost: Option<Ghost>,
    },
}

struct Ghost {
    screen_coordinates: Vec2<i32>,
    done: bool,
}

impl Ghost {
    fn new(screen_coordinates: Vec2<i32>) -> Self {
        Ghost {
            screen_coordinates,
            done: false,
        }
    }
}

impl HudElement {
    fn new_toolbar_button(slot: Vec2<i32>, entity: Entity, view_size: Vec2<f32>) -> HudElement {
        let position = from_float(modulo(
            &from_int(Vec2::new(5, 5) + slot * (HUD_ELEMENT_SIZE + 10)),
            &view_size,
        ));
        let shape = construct_rect_poly(0.0, HUD_ELEMENT_SIZE as f32, 0.0, HUD_ELEMENT_SIZE as f32);

        let bb = entity.shape.bounding_box();
        let diagonal = bb.bottom_right - bb.top_left;
        let max_dimen = diagonal.x.max(diagonal.y);
        let scale_factor = (0.01 * max_dimen + 0.2).min(0.9).max(0.6);
        let scale = scale_factor * (HUD_ELEMENT_SIZE as f32) / max_dimen;

        HudElement {
            position,
            shape,
            variant: HudElementVariant::ToolbarButton {
                entity: Box::from(entity),
                scale,
                ghost: None,
            },
            dragging: false,
        }
    }

    fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>) {
        canvas.set_draw_color(Color::RED);
        let position = translation(from_int(self.position));
        self.shape.render(position, canvas);

        match &self.variant {
            HudElementVariant::ToolbarButton {
                entity,
                ghost,
                scale,
            } => {
                let center = Vec2::from(HUD_ELEMENT_SIZE as f32) * 0.5;

                entity.render(
                    position * translation(center) * Mat3::identity().scaled((*scale).into()),
                    canvas,
                );

                if let Some(ghost) = ghost {
                    entity.render(translation(from_int(ghost.screen_coordinates)), canvas);
                }
            }
        }
    }

    fn tick(&mut self) -> Vec<Action> {
        let mut actions = Vec::new();

        match &mut self.variant {
            HudElementVariant::ToolbarButton {
                entity: button_entity,
                ghost,
                ..
            } => {
                if let Some(Ghost {
                    done: true,
                    screen_coordinates,
                }) = ghost
                {
                    let mut entity = button_entity.clone();
                    entity.position.state = from_int(*screen_coordinates);
                    entity.position.velocity = Vec2::default();

                    actions.push(Action::JoinEntity { entity });

                    *ghost = None;
                }
            }
        }
        actions
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        if self.dragging {
            match event {
                Event::MouseMotion { x, y, .. } => {
                    self.drag_move(Vec2::new(*x, *y));
                    return true;
                }
                Event::MouseButtonUp { x, y, .. } => {
                    if self.drag_end(Vec2::new(*x, *y)) {
                        self.dragging = false;
                        return true;
                    }
                }
                _ => {}
            }
        }
        match event {
            Event::MouseButtonUp { x, y, .. } => {
                let coordinates = Vec2::new(*x, *y);
                let shape_relative = from_int(coordinates - self.position);
                if self.shape.contains_point(shape_relative) && self.click(coordinates) {
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn click(&mut self, coordinates: Vec2<i32>) -> bool {
        match &mut self.variant {
            HudElementVariant::ToolbarButton { ghost, .. } => {
                *ghost = Some(Ghost::new(coordinates));
                self.dragging = true;
                true
            }
        }
    }
    fn drag_end(&mut self, coordinates: Vec2<i32>) -> bool {
        match &mut self.variant {
            HudElementVariant::ToolbarButton { ghost, .. } => {
                if let Some(g) = ghost {
                    g.screen_coordinates = coordinates;
                    g.done = true;
                    return true;
                }
            }
        }
        false
    }

    fn drag_move(&mut self, coordinates: Vec2<i32>) {
        match &mut self.variant {
            HudElementVariant::ToolbarButton { ghost, .. } => {
                if ghost.is_some() {
                    *ghost = Some(Ghost::new(coordinates));
                }
            }
        }
    }
}

// impl <T: RenderTarget> HudElement {
//   fn as_ui_element(&self) -> Option<& impl UIElement<T>> {
//     match self.variant {
//       Button ->
//     }
//   }
// }
