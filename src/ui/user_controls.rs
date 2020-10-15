use crate::world::{Entity};
use crate::math::polygon::Polygon;
use crate::render::View;
use gamemath::{Vec2};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

#[derive(Default)]
pub struct UserControls {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    rotate_right: bool,
    rotate_left: bool,

    action_queue: Vec<Action>,
}

impl UserControls {
    pub fn poll_actions(&mut self) -> std::vec::Drain<'_, Action> {
        self.action_queue.drain(..)
    }

    pub fn handle_event(&mut self, event: &Event, _view: &View) -> bool {
        match *event {
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                self.handle_key_event(keycode, true);
            }
            Event::KeyUp {
                keycode: Some(keycode),
                ..
            } => {
                self.handle_key_event(keycode, false);
            }
            Event::MouseButtonUp { .. } => {
                // let screen_coordinates = Vec3 {
                //     x: x as f32,
                //     y: y as f32,
                //     z: 1.0,
                // };

                // let grid_coordinates =
                //     crate::math::lu::solve_lu(&view.last_grid_to_screen, screen_coordinates)
                //         .into_cartesian();

                // self.clicked = Some(grid_coordinates);
            }
            _ => return false,
        }
        true
    }

    fn handle_key_event(&mut self, keycode: Keycode, pressed: bool) {
        match keycode {
            Keycode::Left => {
                self.left = pressed;
                self.emit_acceleration_action();
            }
            Keycode::A => {
                self.left = pressed;
                self.emit_acceleration_action();
            }
            Keycode::Right => {
                self.right = pressed;
                self.emit_acceleration_action();
            }
            Keycode::D => {
                self.right = pressed;
                self.emit_acceleration_action();
            }
            Keycode::Up => {
                self.up = pressed;
                self.emit_acceleration_action();
            }
            Keycode::W => {
                self.up = pressed;
                self.emit_acceleration_action();
            }
            Keycode::Down => {
                self.down = pressed;
                self.emit_acceleration_action();
            }
            Keycode::S => {
                self.down = pressed;
                self.emit_acceleration_action();
            }
            Keycode::E => {
                self.rotate_left = pressed;
                self.emit_rotate_action();
            }
            Keycode::Q => {
                self.rotate_right = pressed;
                self.emit_rotate_action();
            }
            Keycode::F5 => {
                if pressed {
                    self.action_queue.push(Action::SaveEntity)
                }
            }
            Keycode::F6 => {
                if pressed {
                    self.action_queue.push(Action::LoadEntity { filename: "./data/entities/12094447930535717060".to_owned() })
                }
            }
            _ => {}
        }
    }

    fn emit_acceleration_action(&mut self) {
        let mut direction = Vec2::default();
        if self.left {
            direction += Vec2 { x: -1.0, y: 0.0 };
        }
        if self.right {
            direction += Vec2 { x: 1.0, y: 0.0 };
        }
        if self.up {
            direction += Vec2 { x: 0.0, y: -1.0 };
        }
        if self.down {
            direction += Vec2 { x: 0.0, y: 1.0 };
        }
        self.action_queue.push(Action::Accelerate {
            direction,
            throttle: if direction.length() > 0.0 { 1.0 } else { 0.0 },
        });
    }

    fn emit_rotate_action(&mut self) {
        let direction = if self.rotate_left {
            1.0
        } else if self.rotate_right {
            -1.0
        } else {
            0.0
        };
        self.action_queue.push(Action::Rotate {
            direction,
            throttle: direction.abs(),
        });
    }
}

#[allow(dead_code)]
pub enum Action {
    Accelerate { direction: Vec2<f32>, throttle: f32 },
    Rotate { direction: f32, throttle: f32 },

    UpdateShape { new_shape: Box<Polygon> },
    JoinEntity { entity: Box<Entity> },

    SaveEntity,
    LoadEntity { filename: String },
}
