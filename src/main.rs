use std::collections::VecDeque;

mod piece_table;
mod cursor;

use piece_table::PieceTable;
use cursor::Cursor;
use sdl2::{pixels::{Color, PixelFormatEnum}, event::Event, keyboard::Keycode, render::{Canvas, TextureQuery, Texture, TextureCreator, TextureAccess}, video::{Window, WindowContext}, rect::Rect, ttf::{Font}};

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
    let mut cursor = Cursor::new(font_size);

    let (mut glyph_atlas, mapping) = create_glyph_atlas(&mut canvas, &texture_creator, &font, font_size);

    let mut event_pump = sdl_context.event_pump().expect("Failed to set up event pump.");

    let mut pt = PieceTable::new();

    'running: loop { 
        // TODO: Only read this again when there are changes
        let content = pt.read();

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
                    if cursor.index != 0 {
                        cursor.index -= 1;
                    }
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    cursor.index += 1;
                    if cursor.index > content.len() as u32 {
                        cursor.index = content.len() as u32;
                    }
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    if let Some(index_diff) = Cursor::calc_new_index(&cursor, &content, -1) {
                        cursor.index -= index_diff;
                        // TODO: DO NOT UNWRAP
                        if content.chars().nth(cursor.index as usize).unwrap() == '\n' {
                            cursor.index -= 1;
                        }
                    }
                },
                Event::KeyDown {
                   keycode: Some(Keycode::Down),
                    ..
                } => {
                    if let Some(index_diff) = Cursor::calc_new_index(&cursor, &content, 1) {
                        cursor.index += index_diff;
                        // TODO: DO NOT UNWRAP
                        if content.chars().nth(cursor.index as usize).unwrap() == '\n' {
                            cursor.index -= 1;
                        }
                    }
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    println!("Attempt to remove character at index: {}", cursor.index);

                    if cursor.index != 0 {
                        if !pt.delete(cursor.index - 1, 1) {
                            println!("Failed to delete character ({})", cursor.index - 1);
                        } else {
                            if cursor.index != 0 {
                                cursor.index -= 1;
                            }
                        }
                    }
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    if !pt.insert("\n", cursor.index) {
                        println!("Failed to insert newline at index: {}", cursor.index);
                    } else {
                        cursor.index += 1;
                    }
                },
                Event::TextInput { text, .. } => {
                    // TODO: This should use the last piece as long as possible
                    // Just expand the length and keep adding onto the add buffer until another
                    // piece has been added

                    if !pt.insert(&text, cursor.index) {
                        println!("Write denied ({} at index: {})", &text, cursor.index);
                    } else {
                        cursor.index += text.len() as u32;
                    }
                },
                _ => {}
            }
        
            // Only print on events;
            println!("{:?}", cursor);
        }


        //canvas.set_draw_color(Color::RGBA(40, 77, 73, 255));
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        // &glyph_atlas.set_color_mod(189, 179, 149);
        &glyph_atlas.set_color_mod(255, 255, 255);
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

        if cursor.y > font_size.1 * line {
            cursor.y = line * font_size.1;
        }

        if cursor.x < 0  {
            cursor.x = 0;
        }

        if cursor.y < 0 {
            cursor.y = 0
        }
        
        //let mut chars_on_current_line = 0;
        //let mut newlines_count = 0;
        //for c in content.chars() {
        //    if c == '\n' {
        //        if newlines_count == cursor.get_current_line(&content) {
        //            break;
        //        }

        //        newlines_count += 1;
        //        chars_on_current_line = 0;
        //        continue;
        //    }

        //    chars_on_current_line += 1;
        //}

        //if cursor.x > chars_on_current_line * font_size.0 {
        //    cursor.x = chars_on_current_line * font_size.0;
        //}

        cursor.render(&mut canvas, &content);

        canvas.present();
    }

    Ok(())
}
