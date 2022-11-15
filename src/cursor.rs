use sdl2::{pixels::{Color, PixelFormatEnum}, event::Event, keyboard::Keycode, render::{Canvas, TextureQuery, Texture, TextureCreator, TextureAccess}, video::{Window, WindowContext}, rect::Rect, ttf::{Font}};

#[derive(Clone, Debug)]
pub struct Cursor {
    pub x: u32,
    pub y: u32,
    pub index: u32,
    pub font_size: (u32, u32),
    pub cursor_line: bool,
}

impl Cursor {
    /// TODO: lines_diff should indicate how many lines are being moved
    /// for now it will just indicate up or down +1 or -1
    pub fn calc_new_index(cursor: &Cursor, content: &String, lines_diff: i32) -> Option<u32> {
        if lines_diff != 1 && lines_diff != -1 {
            panic!("Don't use a lines_diff that isn't 1 or -1");
        }

        // TODO: cleanup and refactor (ideally would be combined)
        match lines_diff {
            n if n > 0 => Self::move_down(cursor, content),
            n if n < 0 => Self::move_up(cursor, content),
            _ => None,
        }
    }

    fn move_up(cursor: &Cursor, content: &String) -> Option<u32> {
        let lines: Vec<&str> = content.split('\n').collect();
        let current_line = cursor.get_current_line(content);

        if current_line == 0 {
            return None;
        }

        let mut char_count = 0;
        for i in 0..current_line {
            char_count += lines.get(i as usize).unwrap().len();
        }

        let gap = cursor.index - char_count as u32;

        let mut char_count_new_line = 0;
        for i in 0..current_line-1 {
            char_count_new_line += lines.get(i as usize).unwrap().len();
        }

        let index_diff = cursor.index - (char_count_new_line as u32 + gap) + 1/*newline*/;

        Some(if cursor.index - index_diff > char_count as u32 {
            cursor.index - char_count as u32 + 1
        } else {
            index_diff
        })
    }

    fn move_down(cursor: &Cursor, content: &String) -> Option<u32> {
        let lines: Vec<&str> = content.split('\n').collect();
        let current_line = cursor.get_current_line(content);

        if lines.len() <= (current_line + 1) as usize {
            return None;
        }

        let mut char_count = 0;
        for i in 0..current_line {
            char_count += lines.get(i as usize).unwrap().len();
        }

        let current_cursor_line = lines.get(current_line as usize).unwrap();
        let gap = cursor.index - char_count as u32;
        let rest_chars = current_cursor_line.len() as u32 - gap + 1/*new line*/;

        let new_index = rest_chars + gap;
        
        Some(if cursor.index + new_index > content.len() as u32 {
            content.len() as u32 - cursor.index
        } else {
            new_index
        })
    }

    pub fn new(font_size: (u32, u32)) -> Self {
        Cursor {
            x: 0, 
            y: 0,
            index: 0,
            font_size,
            cursor_line: false,
        }
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>, content: &String) {
        let mut lines = 0;
        let mut chars_on_line = 0;
        for (i, c) in content.chars().enumerate() {
            if c == '\n' {
                chars_on_line = 0;
                lines += 1;
                continue;
            }

            if i == self.index as usize {
                break;
            }
            chars_on_line += 1;
        }

        let x = (chars_on_line * self.font_size.0) as i32;
        let y = (lines * self.font_size.1) as i32;

        let r = Rect::new(x, y, /*w*/self.font_size.0, /*h*/self.font_size.1);
        let original_blend = canvas.blend_mode();

        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 100));
        canvas.fill_rect(r).unwrap();

        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(255, 0, 0, 50));
        let canvas_size = canvas.output_size().expect("");

        if self.cursor_line {
            let cursor_line = Rect::new(0, self.y as i32, canvas_size.0, self.font_size.1);
            canvas.fill_rect(cursor_line).unwrap();
        }

        canvas.set_blend_mode(original_blend);
    }

    pub fn get_current_line(&self, content: &String) -> u32 {
        let mut lines = 0;
        for (i, c) in content.chars().enumerate() {
            if i == self.index as usize{
                return lines;
            }

            if c == '\n' {
                lines += 1;
            }
        }

        lines
    }

    pub fn get_current_index(&self, content: &String) -> Option<u32> {
        let current_line = self.get_current_line(content);
        let mut chars_on_line = 0;
        let mut chars_total = 0;
        let mut lines = 0;

        let chars_till_cursor_pos = self.x / self.font_size.0;

        for (i, c) in content.chars().enumerate() {
            if c == '\n' {
                lines+=1;
                chars_on_line = 0;
                continue;
            }
            
            if lines == current_line && chars_on_line == chars_till_cursor_pos {
                return Some(i as u32);
                break;
            }
            chars_on_line+=1;
        }

        None
    }
}

