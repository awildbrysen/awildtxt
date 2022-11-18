mod piece_table;
mod cursor;
mod file;

use piece_table::PieceTable;
use cursor::Cursor;
use file::read_file;
use sdl2::{pixels::{Color, PixelFormatEnum}, event::Event, keyboard::Keycode, render::{Canvas, Texture, TextureCreator, TextureAccess}, video::{Window, WindowContext}, rect::Rect, ttf::{Font}};
use std::{env, thread, time, ptr::{self, addr_of}, mem};
use sdl2::sys::SDL_PushEvent;

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
    }).expect("Failed to create glyph atlas");

    (texture, mapping)
}

fn render_text(canvas: &mut Canvas<Window>, glyph_atlas: &mut Texture, mapping: [GlyphPosition; 128], font_size: (u32, u32), text: &String, x: i32, y: i32) {
    let mut line = 0;
    let mut carriage = 0;
    for c in text.chars() {
        if c == '\n' {
            line += 1;
            carriage = 0;
            continue;
        }

        let pos = mapping[c as usize];
        let src = Rect::new(pos.0, pos.1, font_size.0, font_size.1);
        let dst = Rect::new(x + (carriage * font_size.0) as i32, y + (font_size.1 * line) as i32, font_size.0, font_size.1);
        carriage+=1;
        canvas.copy(&glyph_atlas, Some(src), Some(dst)).unwrap();
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_mode: bool = env::var("AWILDTXT_TEST").is_ok();
    if test_mode {
        println!("Launching in test mode");
    }

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

    // TODO: This should be extracted, no clue how yet
    let mut render_file_path_input = false;
    let mut file_path_input_pt = PieceTable::new();

    let mut pt = PieceTable::new();

    let event_subsystem = sdl_context.event().unwrap();
    let event_sender = event_subsystem.event_sender();

    // TODO: Can I spawn a thread here and then feed keyboard commands in?
    thread::spawn(move ||{
        thread::sleep(time::Duration::from_secs(5));

        let event = sdl2::sys::SDL_TextInputEvent {
            type_: sdl2::sys::SDL_EventType::SDL_TEXTINPUT as u32,
            windowID: 0, 
            timestamp: 0,
            text: [72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        };
        unsafe {
            let addr = addr_of!(event);
            SDL_PushEvent(addr as *mut sdl2::sys::SDL_Event);
         }
    });

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
                        if let Some(c) = content.chars().nth(cursor.index as usize) {
                            if c == '\n' {
                                cursor.index -= 1;
                            }
                        }
                    }
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    cursor.index += 1;
                    if cursor.index >= content.len() as u32 {
                        cursor.index = content.len() as u32;
                    } else {
                        if let Some(c) = content.chars().nth(cursor.index as usize) {
                            if c == '\n' {
                                cursor.index += 1;
                            }
                        }
                    }


                },
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    if let Some(index_diff) = Cursor::calc_new_index(&cursor, &content, -1) {
                        cursor.index -= index_diff;
                        if let Some(c) = content.chars().nth(cursor.index as usize) {
                            if c == '\n' {
                                cursor.index -= 1;
                            }
                        }
                    }
                },
                Event::KeyDown {
                   keycode: Some(Keycode::Down),
                    ..
                } => {
                    if let Some(index_diff) = Cursor::calc_new_index(&cursor, &content, 1) {
                        cursor.index += index_diff;
                        if let Some(c) = content.chars().nth(cursor.index as usize) {
                            if c == '\n' {
                                cursor.index -= 1;
                            }
                        }
                    }
                },
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    if render_file_path_input {
                        let data = &file_path_input_pt.read();
                        if data.len() > 0 {
                            file_path_input_pt.delete(data.len() as u32 - 1, 1);
                        }
                        continue;
                    }

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
                    if render_file_path_input {
                        // TODO: don't just overwrite current content
                        // Either create some buffer system or (prompt for) save
                        if let Ok(content) = read_file(&file_path_input_pt.read()) {
                            pt = PieceTable::init(content);
                            cursor.index = 0;
                        }
                        render_file_path_input = false;
                        continue;
                    }

                    if !pt.insert("\n", cursor.index) {
                        println!("Failed to insert newline at index: {}", cursor.index);
                    } else {
                        cursor.index += 1;
                    }
                },
                Event::KeyDown {
                    keymod: sdl2::keyboard::Mod::LCTRLMOD,
                    keycode: Some(Keycode::O),
                    ..
                } => {
                    // file picker -> type file path at the bottom of the window
                    render_file_path_input = true;
                    file_path_input_pt = PieceTable::new();
                    
                },
                Event::TextInput { text, .. } => {
                    if render_file_path_input {
                        file_path_input_pt.append(&text);
                        continue;
                    }

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
        }


        //canvas.set_draw_color(Color::RGBA(40, 77, 73, 255));
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        // glyph_atlas.set_color_mod(189, 179, 149);
        glyph_atlas.set_color_mod(255, 255, 255);
        glyph_atlas.set_blend_mode(sdl2::render::BlendMode::Blend);
        let mut line = 0;
        let mut carriage = 0;
        for c in content.chars() {
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

        //let window_size = &window.size();
        if render_file_path_input {
            render_text(&mut canvas, &mut glyph_atlas, mapping, font_size, &String::from("Open file:"), 3, 878 - font_size.1 as i32);

            canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
            let file_picker_bg = Rect::new(0, 880, 1920, font_size.1 + 10);
            canvas.fill_rect(file_picker_bg).unwrap();

            glyph_atlas.set_color_mod(0, 0, 0);
            glyph_atlas.set_blend_mode(sdl2::render::BlendMode::Blend);
            render_text(&mut canvas, &mut glyph_atlas, mapping, font_size, &file_path_input_pt.read(), 3, 885);
        }

        cursor.render(&mut canvas, &content);

        canvas.present();
    }

    Ok(())
}
