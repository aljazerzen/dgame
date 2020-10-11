use crate::math::vec::*;
use crate::render::View;
use gamemath::{Vec2, Vec3};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

#[derive(Default)]
pub struct UserControls {
  pub left: bool,
  pub right: bool,
  pub up: bool,
  pub down: bool,
  pub rotate_right: bool,
  pub rotate_left: bool,
  // onclickListeners: ((c: Vector) => DragElement)[] = [];
  // currentDragElement: DragElement;
  pub clicked: Option<Vec2<f32>>,
}

impl UserControls {
  pub fn handle_event(&mut self, event: &Event, view: &View) -> bool {
    // self.clicked = None;

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
      Event::MouseButtonUp { x, y, .. } => {
        let screen_coordinates = Vec3 {
          x: x as f32,
          y: y as f32,
          z: 1.0,
        };

        let grid_coordinates =
          crate::math::lu::solve_lu(&view.last_grid_position, screen_coordinates).into_cartesian();

        self.clicked = Some(grid_coordinates);
      }
      _ => return false,
    }
    true
  }

  fn handle_key_event(&mut self, keycode: Keycode, pressed: bool) {
    match keycode {
      Keycode::Left => {
        self.left = pressed;
      }
      Keycode::A => {
        self.left = pressed;
      }
      Keycode::Right => {
        self.right = pressed;
      }
      Keycode::D => {
        self.right = pressed;
      }
      Keycode::Up => {
        self.up = pressed;
      }
      Keycode::W => {
        self.up = pressed;
      }
      Keycode::Down => {
        self.down = pressed;
      }
      Keycode::S => {
        self.down = pressed;
      }
      Keycode::E => {
        self.rotate_left = pressed;
      }
      Keycode::Q => {
        self.rotate_right = pressed;
      }
      _ => {}
    }
  }
}
