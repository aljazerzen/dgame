use crate::entity::Thruster;
use crate::grid::{Grid, Insist};
use crate::client::{EntityId};
use crate::math::polygon::{construct_rect_poly, construct_rect_poly_centered, Polygon};
use crate::math::segment::Segment;
use crate::math::vec::*;
use crate::render::{into_vec, Render};
use gamemath::Vec2;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::event::{Event};

const TRACKER_PADDING: i32 = 30;

pub struct Hud {
  pub tracker_indicators: Vec<Vec2<f32>>,

  elements: Vec<HudElement>,
}

impl Hud {
  pub fn new() -> Hud {
    Hud {
      tracker_indicators: Vec::new(),
      elements: vec![HudElement::new_button(
        Vec2::new(0, -1),
        Thruster::shape(HUD_ELEMENT_SIZE as f32 * 0.4),
      ),
      HudElement::new_button(
        Vec2::new(1, -1),
        Thruster::shape(HUD_ELEMENT_SIZE as f32 * 0.4),
      ),
      HudElement::new_button(
        Vec2::new(0, -2),
        Thruster::shape(HUD_ELEMENT_SIZE as f32 * 0.4),
      )],
    }
  }

  pub fn handle_event(&mut self, event: &Event) -> bool {
    false
  }

  pub fn update_trackers(&mut self, view_size: Vec2<f32>, grid: &Grid, focus: EntityId) {
    let relations = grid.get_descendant_relations(Insist::default());

    let offset = relations.iter()
      .find(|r| r.grid == &focus.grid_id)
      .map(|r| -r.relation.state)
      .unwrap_or_default();

    let padding = TRACKER_PADDING as f32 * 2.0;
    let poly = construct_rect_poly_centered(view_size.x - padding, view_size.y - padding);

    let mut trackers: Vec<Vec2<f32>> = Vec::with_capacity(relations.len());

    for relation in &relations {
      let ray = Segment::new(relation.relation.state + offset, Vec2::default());

      if let Some((_alpha, intersection)) = poly.intersect_line_segment(ray) {
        trackers.push(intersection);
      }
    }

    self.tracker_indicators = trackers;
  }

  pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) {
    canvas.set_draw_color(Color::RGB(128, 128, 172));
    let rect = construct_rect_poly_centered(10.0, 10.0);
    let center = translation(into_vec(canvas.viewport().center()));
    for tracker in &self.tracker_indicators {
      let position = center * translation(*tracker);
      rect.render(position, canvas);
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
  variant: HudElementVariant,
}

const HUD_ELEMENT_SIZE: i32 = 40;

enum HudElementVariant {
  Button { diagonal: Vec2<i32>, icon: Polygon },
}

impl HudElement {
  fn new_button(slot: Vec2<i32>, icon: Polygon) -> HudElement {
    HudElement {
      position: Vec2::new(5, 5) + slot * (HUD_ELEMENT_SIZE + 10),
      variant: HudElementVariant::Button {
        diagonal: Vec2::from(HUD_ELEMENT_SIZE),
        icon,
      },
    }
  }

  fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>) {
    match &self.variant {
      HudElementVariant::Button { diagonal, icon } => {
        let position = translation(self.get_position(&canvas));
        let rect = construct_rect_poly(0.0, diagonal.x as f32, 0.0, diagonal.y as f32);

        canvas.set_draw_color(Color::RED);
        rect.render(position, canvas);

        icon.render(position * translation(from_int(*diagonal) * 0.5), canvas);
      }
    }
  }

  fn get_position<T: RenderTarget>(&self, canvas: &Canvas<T>) -> Vec2<f32> {
    let viewport = canvas.viewport();
    let view_size = Vec2::new(viewport.width() as f32, viewport.height() as f32);
    modulo(&from_int(self.position), &view_size)
  }
}

// impl <T: RenderTarget> HudElement {
//   fn as_ui_element(&self) -> Option<& impl UIElement<T>> {
//     match self.variant {
//       Button ->
//     }
//   }
// }
