use shlex::split;

mod opcodes;

use opcodes::*;

fn main() -> Result<(), ParseError> {
    let bytes = Vec::<u8>::new();
    let instructions = Vec::<u32>::new();
    let source_file_text = std::fs::read_to_string("test.txt").unwrap();
    let source_file_lines = source_file_text.split('\n');
    let mut token_lines = Vec::new();
    for token in source_file_lines {
        token_lines.push(split(token).unwrap());
    }

    for (row, line) in token_lines.iter().enumerate() {
        // println!("{:?}", line);
        let instruction;
        if line.is_empty() || (line[0] == "//") { continue }
        match &line[1] as &str {
            "<-" => {
                match line.len() {
                    3 => {
                        instruction = from_7_5_25(OpCode::LOAD_IMM as u32, &line[0], &line[2]);
                        println!("Line {}: {:032b}", row, instruction);
                    }
                    _ => println!("Error in line {}", row)
                }
            },
            "->" => {
            },
            _ => return Err(ParseError("Invalid operator".to_string()))
        }
    }
    Ok(())
}

#[derive(Debug)]
struct ParseError(String);

fn from_7_5_25(opcode: u32, rde: &String, mut imm:& String) -> u32 {
    let mut imm = u32::from_str_radix(imm.trim_start_matches('$'), 16).unwrap();
    let rde = u32::from_str_radix(&rde.split(':').last().unwrap(), 16).unwrap();
    if !(imm <= 0x1FFFFFF) {
        println!("Immediate value bigger than 25 bits!");
        imm = 0x1FFFFFF;
    }
    return opcode << 25 | rde << 20 | imm
}
