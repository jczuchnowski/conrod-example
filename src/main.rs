#[macro_use]
extern crate conrod;
extern crate futures;
extern crate futures_timer;
extern crate rand;

use conrod::{widget, Colorable, Positionable, Borderable, Widget, color};
use conrod::backend::glium::glium::{self, Surface};
use futures::{Future, Stream};
use futures_timer::Interval;
use rand::random;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn main() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let blocks_num = Arc::new(Mutex::new(0));

    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Conrod Example")
        .with_dimensions(WIDTH, HEIGHT);
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    {
        let blocks_num = Arc::clone(&blocks_num);

        thread::spawn(move || {
            Interval::new(Duration::from_secs(3))
                .for_each(|()| {
                    let mut bc_num = blocks_num.lock().unwrap();
                
                    let x = random::<i32>();

                    *bc_num = x;
                    Ok(())
                })
                .wait()
                .unwrap();    
        });
    }

    // Generate the widget identifiers.
    widget_ids!(struct Ids { 
        master, body, text_blocks });
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    const FONT_PATH: &'static str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/UbuntuMono-R.ttf");

    ui.fonts.insert_from_file(FONT_PATH).unwrap();

    // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    let mut events = Vec::new();

    let background_color = color::Color::Rgba(89.0, 74.0, 81.0, 0.1);

    'render: loop {
        events.clear();

        // Get all the new events since the last frame.
        events_loop.poll_events(|event| { events.push(event); });

        // If there are no new events, wait for one.
        if events.is_empty() {
            events_loop.run_forever(|event| {
                events.push(event);
                glium::glutin::ControlFlow::Break
            });
        }

        // Process the events.
        for event in events.drain(..) {

            // Break from the loop upon `Escape` or closed window.
            match event.clone() {
                glium::glutin::Event::WindowEvent { event, .. } => {
                    match event {
                        glium::glutin::WindowEvent::Closed |
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => break 'render,
                        _ => (),
                    }
                }
                _ => (),
            };

            // Use the `winit` backend feature to convert the winit event to a conrod input.
            let input = match conrod::backend::winit::convert_event(event, &display) {
                None => continue,
                Some(input) => input,
            };

            // Handle the input with the `Ui`.
            ui.handle_event(input);

            // Set the widgets.
            let ui = &mut ui.set_widgets();

            widget::Canvas::new().flow_down(&[
                (ids.body, widget::Canvas::new().border(0.0).color(background_color))
            ]).set(ids.master, ui);            
            
            let bc_num = *blocks_num.lock().unwrap();

            widget::Text::new(&bc_num.to_string()[..])
                .middle_of(ids.body)
                .color(conrod::color::WHITE)
                .font_size(72)
                .set(ids.text_blocks, ui);            

            //set_ui(ui.set_widgets(), &ids, &mut demo_text);

        }
        // Draw the `Ui` if it has changed.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
