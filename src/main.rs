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
                let mut var = parser::parse_value(&tokens[2]).unwrap();
                var.name = tokens[1].clone();
                blocks.last_mut().unwrap().add_variable(var);
            },
            _ => {},
        }
    }

    for block in blocks {
        println!("\x1b[38;2;255;200;50mBlock:\x1b[38;2;50;255;200m {}, \x1b[38;2;255;200;50mBase: \x1b[38;2;50;150;255m0x{:X}\x1b[0m", block.name, block.base);
        for variable in block.variables {
            println!("  \x1b[38;2;255;255;255mvariable:\x1b[38;2;150;150;150m {}: {}", variable.name, variable);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Block {
    pub name: String,
    pub base: u32,
    pub variables: Vec<Variable>,
}

impl Block {
    pub fn new(name: String) -> Self {
        Block {
            name,
            base: 0,
            variables: Vec::new(),
        }
    }

    pub fn add_variable(&mut self, variable: Variable) {
        self.variables.push(variable);
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

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub vartype: VariableType,
    pub value: Vec<u8>,
}

impl Variable {
    fn to_u8(&self) -> Option<u8> {
        let mut buf = [0u8; 1];
        let len = self.value.len().min(1);
        buf[..len].copy_from_slice(&self.value[..len]);
        Some(u8::from_le_bytes(buf))
    }
    fn to_u16(&self) -> Option<u16> {
        let mut buf = [0u8; 2];
        let len = self.value.len().min(2);
        buf[..len].copy_from_slice(&self.value[..len]);
        Some(u16::from_le_bytes(buf))
    }
    fn to_u32(&self) -> Option<u32> {
        let mut buf = [0u8; 4];
        let len = self.value.len().min(4);
        buf[..len].copy_from_slice(&self.value[..len]);
        Some(u32::from_le_bytes(buf))
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "raw: {} -> meaning: {}", self.value.iter().map(|b| format!("0x{:02X} ", b)).collect::<Vec<_>>().join(""), match self.vartype {
            VariableType::Int => self.to_u32().map(|n| format!("{}", n)).unwrap_or_else(|| "invalid int".to_string()),
            VariableType::Str => String::from_utf8_lossy(&self.value).to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum VariableType {
    Int,
    Str,
}