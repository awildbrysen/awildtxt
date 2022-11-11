use std::collections::VecDeque;

mod piece_table;

use piece_table::PieceTable;
use sdl2::{pixels::{Color, PixelFormatEnum}, event::Event, keyboard::Keycode, render::{Canvas, TextureQuery, Texture, TextureCreator, TextureAccess}, video::{Window, WindowContext}, rect::Rect, ttf::{Font}};

#[derive(Clone)]
struct Cursor {
    x: u32,
    y: u32,
    font_size: (u32, u32),
    cursor_line: bool,
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

    fn get_current_line(&self) -> u32 {
        self.y / self.font_size.0
    }
}

type GlyphPosition = (i32, i32);

fn create_glyph_atlas<'canvas>(canvas: &mut Canvas<Window>, creator: &'canvas TextureCreator<WindowContext>, font: &Font, font_size: (u32, u32)) -> (Texture<'canvas>, [GlyphPosition; 128]) {
    let mut texture = creator.create_texture(
        PixelFormatEnum::RGBA32,
        TextureAccess::Target,
        2048,
        2048
    ).unwrap();

    let mut mapping: [GlyphPosition; 128] = [(0,0);128];

    canvas.with_texture_canvas(&mut texture, |canv| {

        for i in 0..128 {
            let c_opt = char::from_u32(i);
            match c_opt {
                Some(c) => {
                    let r = Rect::new((i * font_size.0) as i32, 0, font_size.0, font_size.1);
                    if c == '\0' {
                        continue;
                    }

                    let surface = font.render_char(c)
                        .blended(Color::RGBA(255, 255, 255, 255))
                        .unwrap();
                    let char_texture = creator.create_texture_from_surface(&surface).unwrap();

                    canv.copy(&char_texture, None, Some(r)).unwrap();
                    mapping[i as usize] = (r.x, r.y);
                }
                None => {}
            }
        }
    });

    (texture, mapping)
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

    let font = ttf_context.load_font("font.ttf", 15).expect("Failed to load font.");

    let font_size = font.size_of("W")?;
    let mut cursor = Cursor { x: 0, y: 0, font_size: font_size, cursor_line: true };

    let (mut glyph_atlas, mapping) = create_glyph_atlas(&mut canvas, &texture_creator, &font, font_size);

    let mut event_pump = sdl_context.event_pump().expect("Failed to set up event pump.");

    let mut pt = PieceTable::new();
    pt.append("X");

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
                    // TODO: limit to the text range
                    if cursor.x != 0 {
                        cursor.x -= cursor.font_size.0;
                    }
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    // TODO: limit to the text range
                    cursor.x += cursor.font_size.0;
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    // TODO: limit to the text range
                    if cursor.y != 0 {
                        cursor.y -= cursor.font_size.1;
                    }
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    // TODO: limit to the text range
                    cursor.y += cursor.font_size.1;
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    // Delete 1 char from position
                    if cursor.x != 0 {
                        if !pt.delete((cursor.x / font_size.0) - 1, 1) {
                            println!("Failed to delete character ({})", (cursor.x / font_size.0) - 1);
                        } else {
                            cursor.x -= font_size.0;
                        }
                    }
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    if !pt.insert("\n", cursor.x / font_size.0) {
                        println!("Failed to insert newline at index: {}", cursor.x / font_size.0);
                    }
                },
                Event::TextInput { text, .. } => {
                    if !pt.insert(&text, cursor.x / font_size.0) {
                        println!("Write denied ({} at index: {})", &text, cursor.x / font_size.0);
                    } else {
                        // Move cursor along
                        cursor.x += (text.len() as u32 * font_size.0);
                    }
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGBA(40, 77, 73, 255));
        canvas.clear();

        let content = pt.read();
        &glyph_atlas.set_color_mod(189, 179, 149);
        &glyph_atlas.set_blend_mode(sdl2::render::BlendMode::Blend);
        let mut line = 0;
        let mut carriage = 0;
        for (idx, c) in content.chars().enumerate() {
            if c == '\n' {
                line += 1;
                carriage = 0;
                continue;
            }

            let pos = mapping[c as usize];
            let src = Rect::new(pos.0, pos.1, font_size.0, font_size.1);
            let dst = Rect::new((carriage * font_size.0) as i32, (font_size.1 * line) as i32, font_size.0, font_size.1);
            carriage+=1;
            canvas.copy(&glyph_atlas, Some(src), Some(dst)).unwrap();
        };
        
        cursor.render(&mut canvas);

        canvas.present();
    }

    Ok(())
}
