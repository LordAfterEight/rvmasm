use std::path;

mod parser;

fn main() {
    let path = std::env::args().nth(1).expect("No input file provided");
    let source = std::fs::read_to_string(path).expect("Failed to read input file");
    let lines = parser::split_into_lines(&source);

    let mut blocks = Vec::new();
    let mut current_block: Option<Block> = None;

    for line in lines {
        let tokens = parser::tokenize_line(&line);
        match tokens.first().map(|s| s.as_str()) {
            Some("@BLK") => {
                if let Some(block) = current_block {
                    blocks.push(block);
                }
                current_block = Some(Block::new(tokens[1].clone()));
            },
            _ => {},
        }
    }
}

#[derive(Debug)]
pub struct Block {
    name: String,
    content: Vec<String>,
}

impl Block {
    pub fn new(name: String) -> Self {
        Block {
            name,
            content: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.content.push(line);
    }
}