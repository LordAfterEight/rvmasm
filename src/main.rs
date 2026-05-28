use std::path;

mod parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().len() < 2 {
        eprintln!("Usage: rvmasm <input_file>");
        std::process::exit(1);
    }

    let path = std::env::args().nth(1).expect("No input file provided");
    let source = std::fs::read_to_string(path).expect("Failed to read input file");
    let lines = parser::split_into_lines(&source);

    let mut blocks = Vec::new();
    let mut block_counter = 0;

    let mut config = Config::new();

    for line in lines {
        let tokens = parser::tokenize_line(&line);
        match tokens.first().map(|s| s.as_str()) {
            Some(">CFG") => {
                match tokens[1].as_str() {
                    "bit-width" => {
                        config.bit_width = parser::parse_value(&tokens[2]).unwrap().to_u32().unwrap();
                    },
                    "addr-width" => {
                        config.address_width = parser::parse_value(&tokens[2]).unwrap().to_u32().unwrap();
                    },
                    _ => println!("Unknown config: {}", tokens[1]),
                }
            },
            Some("@BLK") => {
                let mut block = Block::new(tokens[1].clone());
                block.base = parser::parse_value(&tokens[2]).unwrap().to_u32().unwrap();
                blocks.push(block);
            },
            Some("@VAR") => {
                // For simplicity, we just print the variable definition here
                println!("Variable: {}, Value: {}", tokens[1], parser::parse_value(&tokens[2]).unwrap());
            },
            _ => {},
        }
    }

    for block in blocks {
        println!("Block: {}, Base: {:X}", block.name, block.base);
        for line in block.content {
            println!("  {}", line);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Block {
    pub name: String,
    pub base: u32,
    pub content: Vec<String>,
}

impl Block {
    pub fn new(name: String) -> Self {
        Block {
            name,
            base: 0,
            content: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.content.push(line);
    }
}


pub struct Config {
    pub bit_width: u32,
    pub address_width: u32,
}

impl Config {
    pub fn new() -> Self {
        Config {
            bit_width: 32,
            address_width: 32,
        }
    }
}

pub struct Variable {
    pub name: String,
    pub value: Vec<u8>,
}

impl Variable {
    pub fn to_u8(&self) -> Option<u8> {
        if self.value.len() != 1 {
            return None;
        }
        Some(self.value[0])
    }
    pub fn to_u16(&self) -> Option<u16> {
        if self.value.len() > 2 {
            return None;
        }
        Some(self.value.iter().fold(0, |acc, &b| (acc << 8) | b as u16))
    }
    pub fn to_u32(&self) -> Option<u32> {
        if self.value.len() > 4 {
            return None;
        }
        Some(self.value.iter().fold(0, |acc, &b| (acc << 8) | b as u32))
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.iter().map(|b| format!("0x{:02X} ", b)).collect::<Vec<_>>().join(""))
    }
}