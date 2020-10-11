use crate::ui::user_controls::UserControls;
use crate::entity::ForcePoint;
use gamemath::Vec2;

#[derive(Clone, Debug, Default)]
pub struct EntityController {
  pub force: ForcePoint,
}

impl EntityController {
  pub fn update(&mut self, controls: &UserControls) {
    let mut force = Vec2::default();
    let mut torque = 0.0;
    if controls.left {
      force += Vec2 { x: -20.0, y: 0.0 };
    }
    if controls.right {
      force += Vec2 { x: 20.0, y: 0.0 };
    }
    if controls.up {
      force += Vec2 { x: 0.0, y: -20.0 };
    }
    if controls.down {
      force += Vec2 { x: 0.0, y: 20.0 };
    }
    if controls.rotate_left {
      torque += 50.0;
    }
    if controls.rotate_right {
      torque -= 50.0;
    }

    self.force = ForcePoint { force, torque };
  }
}
