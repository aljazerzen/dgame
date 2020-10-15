mod client;
mod engine;
mod math;
mod render;
mod stars;
mod ui;
mod world;

use client::{Client, EntityId};
use engine::engine_tick;
use gamemath::Vec2;
use world::grid::construct_demo_world;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;

fn is_exit_event(event: &Event) -> bool {
    match event {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => true,
        _ => false,
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let attributes = video_subsystem.gl_attr();

    attributes.set_multisample_buffers(1);
    attributes.set_multisample_samples(5);

    let resolution = Vec2::new(1600.0, 900.0);
    let window = video_subsystem
        .window("Example", resolution.x as u32, resolution.y as u32)
        .build()
        .unwrap();

    let mut canvas: Canvas<Window> = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut world = construct_demo_world();
    let grid_id = *world.grids.iter().next().unwrap().0;
    let entity_id = world.grids[&grid_id].entities[0].get_id();
    let mut client = Client::new(resolution, EntityId::new(grid_id, entity_id));

    client.load();

    'running: loop {
        for event in event_pump.poll_iter() {
            if is_exit_event(&event) {
                break 'running;
            }
            client.handle_event(&event);
        }

        engine_tick(&mut world, &mut client.view);

        client.tick(&mut world);

        client.render(&world, &mut canvas);

        canvas.present();

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
