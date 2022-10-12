use std::collections::VecDeque;
use sdl2::{pixels::Color, event::Event, keyboard::Keycode, render::{Canvas, TextureQuery}, video::Window, rect::Rect, sys::Font};

///
/// H e l [              ] l o
///
struct GapBuffer {
    buffer: Vec<char>
}

impl GapBuffer {
    fn new() -> GapBuffer {
        GapBuffer {
            buffer: Vec::new()
        }
    }

    fn grow(index: i32, gap_size: i32) {
t
    }
}

#[derive(Clone)]
struct Cursor {
    x: u32,
    y: u32,
    font_size: (u32, u32),
    cursor_line: bool
}

impl Cursor {
    fn render(&mut self, canvas: &mut Canvas<Window>) {
        let r = Rect::new(self.x as i32, self.y as i32, self.font_size.0, self.font_size.1);
        let original_blend = canvas.blend_mode();

        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 100));
        canvas.fill_rect(r).unwrap();

        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(255, 0, 0, 50));
        let canvas_size = canvas.output_size().expect("");
        let cursor_line = Rect::new(0, self.y as i32, canvas_size.0, self.font_size.1);
        canvas.fill_rect(cursor_line).unwrap();

        canvas.set_blend_mode(original_blend);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init().expect("Failed to initialize SDL");
    let video_subsystem = sdl_context.video().expect("Failed to initialize video subsystem");
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).expect("Failed to initialize TTF");

    let window = video_subsystem
        .window("awildtxt", 1920, 1080)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string()).expect("Failed creating window");

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).expect("Failed conversion into video subsystem canvas.");
    let texture_creator = canvas.texture_creator();

    let font = ttf_context.load_font("font.ttf", 13).expect("Failed to load font.");

    let font_size = font.size_of("W")?;
    let mut cursor = Cursor { x: 0, y: 0, font_size: font_size, cursor_line: true };

    let surface = font
        .render("Hello world")
        .blended(Color::RGBA(255, 255, 255, 255))
        .map_err(|e| e.to_string())
        .expect("Failed to create surface.");
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string()).expect("Failed to create texture from surface.");

    let TextureQuery { width, height, .. } = texture.query();
    let target = Rect::new(0 as i32, 0 as i32, width as u32, height as u32);


    let mut event_pump = sdl_context.event_pump().expect("Failed to set up event pump.");

    'running: loop { 
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown { 
                    keycode: Some(Keycode::Left),
                    ..
                } =>  {
                    cursor.x -= cursor.font_size.0;
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    cursor.x += cursor.font_size.0;
                },
                Event::TextInput { .. } => {

                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGBA(40, 77, 73, 255));
        canvas.clear();

        canvas.copy(&texture, None, Some(target)).unwrap();
        cursor.render(&mut canvas);

        canvas.present();
    }

    Ok(())
}
