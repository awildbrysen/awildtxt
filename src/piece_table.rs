#[derive(Debug, Clone, Copy)]
pub struct Piece {
    is_add: bool,
    offset: u32,
    length: u32
}

#[derive(Debug, Clone)]
pub struct PieceTable {
    ro_buffer: String,
    add_buffer: String,
    pub pieces: Vec<Piece>
}

#[derive(Debug, Clone, Copy)]
pub struct PieceSearch {
    piece: Piece,
    index: u32,
    piece_start: u32,
}

impl PieceTable {
    pub fn new() -> Self {
        PieceTable {
            ro_buffer: String::new(),
            add_buffer: String::new(),
            pieces: Vec::new(),
        }
    }

    pub fn init(base_content: String) -> Self {
        let mut pt = PieceTable {
            ro_buffer: base_content.clone(),
            add_buffer: String::new(),
            pieces: Vec::new()
        };

        pt.pieces.push(Piece {
            is_add: false,
            length: base_content.len() as u32,
            offset: 0,
        });

        pt 
    }

    pub fn append(&mut self, content: &str) {
        let p = Piece {
            is_add: true,
            offset: self.add_buffer.len() as u32,
            length: content.len() as u32,
        };
        self.pieces.push(p);
        self.add_buffer.push_str(content);
    }

    fn find_piece_at_offset(pieces: &mut Vec<Piece>, offset: u32) -> Option<PieceSearch> {
        let mut cursor = 0;
        for (i, p) in pieces.iter().enumerate() {
            if cursor + p.length >= offset {
                return Some(PieceSearch {
                    piece: *p, 
                    index: i as u32, 
                    piece_start: cursor,
                });
            }

            cursor += p.length;
        }

        None
    }

    fn find_pieces_in_range(pieces: &mut Vec<Piece>, offset: u32, length: u32) -> Vec<PieceSearch> {
        let mut cursor = 0;
        let mut searches: Vec<PieceSearch> = Vec::new();

        for (i, p) in pieces.iter().enumerate() {
            if cursor + p.length >= offset {
                searches.push(PieceSearch {
                    piece: *p, 
                    index: i as u32, 
                    piece_start: cursor,
                });
                
                if cursor + p.length >= offset + length {
                    return searches;
                }
            }

            cursor += p.length;
        }

        searches
    }

    fn is_offset_valid(pieces: &Vec<Piece>, offset: u32) -> bool {
        let mut max_offset = 0;
        for p in pieces.iter() {
            max_offset += p.length;
        }

        return !offset > max_offset
    }

    pub fn insert(&mut self, content: &str, offset: u32) -> bool {
        let pieces = &mut self.pieces;

        // Validate if offset is possible
        if !Self::is_offset_valid(pieces, offset) {
            return false;
        }

        // Find the current Piece that matches the offset
        let search_opt = Self::find_piece_at_offset(pieces, offset);

        match search_opt {
            Some(piece_search) => {
                let piece_end = piece_search.piece_start + piece_search.piece.length;

                // Split Piece into 2
                let piece_one_len = offset - piece_search.piece_start;
                let piece_two_len = piece_end - offset;

                let mut idx = piece_search.index;
                // Remove the piece from the pieces
                pieces.remove(piece_search.index as usize);

                // Create Piece for the first split
                if piece_one_len > 0 {
                    let p_one = Piece {
                        is_add: piece_search.piece.is_add,
                        offset: piece_search.piece.offset,
                        length: piece_one_len,
                    };

                    pieces.insert(idx as usize, p_one);
                    idx += 1;
                }

                // Insert new content
                let new_piece = Piece {
                    is_add: true,
                    offset: self.add_buffer.len() as u32,
                    length: content.len() as u32,
                };
                pieces.insert(idx as usize, new_piece);
                self.add_buffer.push_str(content);
                idx += 1;

                // Create Piece for the second split
                if piece_two_len > 0 {
                    let p_two = Piece {
                        is_add: piece_search.piece.is_add,
                        offset: piece_search.piece.offset + piece_one_len,
                        length: piece_two_len,
                    };
                    pieces.insert(idx as usize, p_two);
                }
            },
            None => {
                if offset > 0 {
                    return false;
                }

                self.pieces.insert(0, Piece {
                    is_add: true,
                    offset: self.add_buffer.len() as u32,
                    length: content.len() as u32,
                });
                self.add_buffer.push_str(content);
            }
        }
         

        true
    }
    
    pub fn delete(&mut self, offset: u32, length: u32) -> bool {
        let pieces = &mut self.pieces;

        if !Self::is_offset_valid(pieces, offset) {
            // Should it delete the last character (-length) if this happens? 
            return false;
        }

        let mut searches = Self::find_pieces_in_range(pieces, offset, length);

        match searches.len() {
            n if n >= 2 => {
                // Remove all the inner pieces and grab the total length 
                let last = searches.pop().unwrap();
                let first = searches.swap_remove(0);

                let mut total_length = 0;
                for s in searches {
                    total_length += s.piece.length;
                }

                for idx in (first.index..(last.index + 1)).rev() {
                    pieces.remove(idx as usize);
                }

                let rest = length - (first.piece_start + first.piece.length - offset) - total_length;
                let piece_one_len = offset - first.piece_start;
                let piece_two_offset = last.piece.offset + rest;
                let piece_two_len = last.piece.length - rest;

                let mut idx = first.index;
                if piece_one_len > 0 {
                    let p_one = Piece {
                        is_add: first.piece.is_add,
                        offset: first.piece.offset,
                        length: piece_one_len,
                    };
                    pieces.insert(idx as usize, p_one);
                    idx += 1;
                }

                if piece_two_len > 0 {
                    let p_two = Piece {
                        is_add: last.piece.is_add,
                        offset: piece_two_offset,
                        length: piece_two_len,
                    };
                    pieces.insert(idx as usize, p_two);
                }
                
            },
            1 => {
                let p = searches[0];

                pieces.remove(p.index as usize);

                let piece_one_len = offset - p.piece_start;
                let piece_two_len = (p.piece_start + p.piece.length) - (offset + length);

                let mut idx = p.index;
                if piece_one_len > 0 {
                    let p_one = Piece {
                        is_add: p.piece.is_add,
                        offset: p.piece.offset,
                        length: piece_one_len,
                    };
                    pieces.insert(idx as usize, p_one);
                    idx += 1;
                }

                if piece_two_len > 0 {
                    let p_two = Piece {
                        is_add: p.piece.is_add,
                        offset: p.piece.offset + piece_one_len + length,
                        length: piece_two_len,
                    };
                    pieces.insert(idx as usize, p_two);
                }
            },
            _ => {
                return false;
            },
        };

        true
    }

    pub fn read(&mut self) -> String {
        let mut s: String = String::new();
        for p in self.pieces.iter() {
            let buffer = if p.is_add {
                &self.add_buffer
            } else {
                &self.ro_buffer
            };

            s.push_str(&buffer[(p.offset as usize)..(p.offset as usize + p.length as usize)]);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use crate::piece_table::{PieceTable};

    #[test]
    fn append_string_to_empty_table() {
        let mut pt = PieceTable::new();
        pt.append("Hello World");

        let expected = "Hello World";
        assert_eq!(pt.read(), expected);
    }

    #[test]
    fn append_multiple_string_after_each_other() {
        let mut pt = PieceTable::new();
        pt.append("Hello");

        let mut expected = "Hello";
        assert_eq!(pt.read(), expected);

        pt.append("World");

        expected = "HelloWorld";
        assert_eq!(pt.read(), expected);
    }

    #[test]
    fn insert_string_on_empty_table() {
        let mut pt = PieceTable::new();

        pt.insert("Hello", 0);
        assert_eq!(pt.read(), "Hello");
        
    }

    #[test]
    fn insert_string_with_invalid_offset() {
        let mut pt = PieceTable::new();
        assert_eq!(pt.insert("Hello", 5), false);
        assert_eq!(pt.read(), "");
    }

    #[test]
    fn insert_string_in_between_existing_text() {
        let mut pt = PieceTable::new();
        pt.append("Hello World");

        pt.insert("insert", 5);

        assert_eq!(pt.read(), "Helloinsert World");
    }

    #[test]
    fn append_string_to_table_with_base_content() {
        let mut pt = PieceTable::init(String::from("Hello World"));

        pt.append(" 2: Electric Boogaloo");

        assert_eq!(pt.read(), "Hello World 2: Electric Boogaloo");
    }

    #[test]
    fn insert_string_inside_table_with_base_content() {
        let mut pt = PieceTable::init(String::from("Hello World"));

        pt.insert("insert", 5);

        assert_eq!(pt.read(), "Helloinsert World");
    }

    #[test]
    fn delete_1_character_inside_base_content_string() {
        let mut pt = PieceTable::init(String::from("Hello"));

        pt.delete(2, 1);

        assert_eq!(pt.read(), "Helo");
    }

    #[test]
    fn delete_1_character_inside_added_string() {
        let mut pt = PieceTable::new();
        pt.append("Hello");

        pt.delete(2, 1);

        assert_eq!(pt.read(), "Helo");
    }

    #[test]
    fn delete_multiple_characters_inside_base_content() {
        let mut pt = PieceTable::init(String::from("Hello"));

        pt.delete(2, 3);

        assert_eq!(pt.read(), "He");
    }

    #[test]
    fn delete_multiple_characters_inside_added_string() {
        let mut pt = PieceTable::new();
        pt.append("Hello");

        pt.delete(2, 3);

        assert_eq!(pt.read(), "He");
    }

    #[test]
    fn delete_characters_over_multiple_pieces() {
        let mut pt = PieceTable::init(String::from("Hello"));
        pt.append("World");

        pt.delete(4, 3);

        assert_eq!(pt.read(), "Hellrld");
    }

    #[test]
    fn delete_one_character_in_pieces_of_1_char() {
        let mut pt = PieceTable::init(String::from("A"));
        pt.append("B");
        pt.append("C");
        pt.append("D");

        pt.delete(1, 1);

        assert_eq!(pt.read(), "ACD");
    }

    #[test]
    fn delete_on_empty_buffer() {
        let mut pt = PieceTable::new();

        pt.delete(0, 1);

        assert_eq!(pt.read(), "");
    }
}
