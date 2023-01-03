use sdl2::{pixels::{Color}, render::{Canvas}, video::{Window}, rect::Rect};

#[derive(Clone, Debug)]
pub struct Cursor {
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
        let current_line_number = cursor.get_current_line_number(content);
        let lines: Vec<&str> = content.split_inclusive('\n').collect();

        if current_line_number == 0 {
            return None;
        }

        let previous_line = lines[(current_line_number - 1) as usize];
        let line_char = cursor.get_line_char_count_until_cursor(content, current_line_number);
        let move_size = if previous_line == "\n" {
            line_char + (previous_line.len() as u32)
        } else {
            if previous_line.len() <= line_char as usize {
                (line_char - previous_line.len() as u32) + previous_line.len() as u32 + 1
            } else {
                previous_line.len() as u32
            }
        };

        println!("To move up decrease index by: {move_size}");
        Some(move_size)
    }

    fn move_down(cursor: &Cursor, content: &String) -> Option<u32> {
        let current_line_number = cursor.get_current_line_number(content);
        let lines: Vec<&str> = content.split_inclusive('\n').collect();

        if lines.len() <= (current_line_number + 1) as usize {
            return None;
        }
        
        let l = lines[current_line_number as usize];
        let next_line = lines[(current_line_number + 1) as usize];
        let move_size = if next_line == "\n" {
            let line_chars_until_cursor = cursor.get_line_char_count_until_cursor(content, current_line_number);
            l.len() as u32 - line_chars_until_cursor
        } else {
            let line_char = cursor.get_line_char_count_until_cursor(content, current_line_number);
            if line_char >= next_line.len() as u32 {
                let remaining_chars_len = l.len() as u32 - line_char;
                remaining_chars_len + (next_line.len() - 1) as u32
            } else {
                l.len() as u32
            }
        };

        println!("To move down increase index by: {move_size}");
        Some(move_size as u32)
    }

    pub fn new(font_size: (u32, u32)) -> Self {
        Cursor {
            index: 0,
            font_size,
            cursor_line: false,
        }
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>, content: &String) {
        let mut lines = 0; 
        let mut chars_on_line = 0;

        let current_line_number = self.get_current_line_number(content);
        
        for (idx, line) in content.split_inclusive('\n').enumerate() {
            if idx == current_line_number as usize {
                break;
            }
            lines += 1;
            chars_on_line += line.len() as u32;
        }

        chars_on_line = self.index - (chars_on_line as u32);

        let x = (chars_on_line * self.font_size.0) as i32;
        let y = (lines * self.font_size.1) as i32;

        let cursor_width = self.font_size.0;
        let r = Rect::new(x, y, /*w*/cursor_width, /*h*/self.font_size.1);
        let original_blend = canvas.blend_mode();

        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 100));
        canvas.fill_rect(r).unwrap();

        println!("Rendering cursor\tIndex: {}\tPosition:({};{})", self.index, r.x, r.y);

        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(255, 0, 0, 50));
        let canvas_size = canvas.output_size().expect("");

        if self.cursor_line {
            let cursor_line = Rect::new(0, y as i32, canvas_size.0, self.font_size.1);
            canvas.fill_rect(cursor_line).unwrap();
        }

        canvas.set_blend_mode(original_blend);
    }

    pub fn get_current_line_number(&self, content: &String) -> u32 {
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

    fn get_line_char_count_until_cursor(&self, content: &String, current_line_number: u32) -> u32 {
        let mut chars = 0;
        for (idx, line) in content.split_inclusive('\n').enumerate() {
            if idx == current_line_number as usize {
                break;
            }
            chars += line.len();
        }

        self.index - (chars as u32)
    }
}

