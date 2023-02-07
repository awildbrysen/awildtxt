use piece_table::PieceTable;

use crate::piece_table;

pub struct Buffer {
    pub path: Option<String>,
    pub pt: PieceTable,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            path: None,
            pt: PieceTable::new()
        }
    }

    pub fn from(path: &str, content: String) -> Self {
        Buffer {
            path: Some(path.to_owned()),
            pt: PieceTable::init(content)
        }
    }
}