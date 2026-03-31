use crate::err;
use crate::err::AssembleError;
use crate::logging;
use crate::logging::log;
use crate::logging::LoggingVerbosity;
use crate::mem::*;
use crate::opcodes::OpCodes;

pub fn assemble(
    tokens: Vec<&str>,
    mem: &mut Memory,
    logging: &LoggingVerbosity,
) -> Result<(), err::AssembleError> {
    let first_token_symbol = tokens[0].split_at(1).0;

    match first_token_symbol {
        "@" => match tokens[0] {
            "@MAP" => {
                mem.mem_ptr = usize::from_str_radix(tokens[1].trim_matches('$'), 16).unwrap();
            }
            "@LBL" => {
                mem.blocks[mem.block_tracker].labels.push(Label::new(tokens[1], mem.mem_ptr));
            }
            "@BLK" => {
                if !mem.blocks.is_empty() {
                    mem.block_tracker += 1;
                }
                let address = usize::from_str_radix(tokens[2].trim_matches('$'), 16).unwrap();
                mem.blocks.push(Block {
                    name: tokens[1].to_string(),
                    labels: Vec::new(),
                    variables: Vec::new(),
                    address: address,
                    length: 0,
                    data: Vec::new(),
                });
                mem.mem_ptr = address;
                log(
                    &format!("Created new block at address 0x{:08X}", mem.mem_ptr),
                    logging,
                );
            }
            "@VAR" => {
                let value = match evaluate_value(&tokens, 2) {
                    Ok(val) => val,
                    Err(err) => return Err(err)
                };
                log(
                    &format!(
                        "Created variable {} with size {}B and data {:?}",
                        tokens[1],
                        value.len(),
                        value
                    ),
                    logging,
                );
                let variable = Variable {
                    name: tokens[1].to_string(),
                    address: mem.mem_ptr,
                    length: value.len(),
                    value: value,
                };
                mem.mem_ptr += variable.length;
                mem.blocks[mem.block_tracker].variables.push(variable);
            },
            "@END" => {},
            _ => return Err(err::AssembleError::InvalidToken),
        },
        "#" => match tokens[0] {
            "#PSH" => {
                let register = match get_register_index(tokens[1]) {
                    Ok(val) => val,
                    Err(err) => return Err(err)
                };

                mem.blocks[mem.block_tracker].data.push(OpCodes::PUSH_RGST as u8);
                mem.blocks[mem.block_tracker].data.push(register as u8);
                log(&format!("Push register {} to stack", register), logging);
            },
            "#POP" => {
                let register = match get_register_index(tokens[1]) {
                    Ok(val) => val,
                    Err(err) => return Err(err)
                };

                mem.blocks[mem.block_tracker].data.push(OpCodes::POP_RGST as u8);
                mem.blocks[mem.block_tracker].data.push(register as u8);
                log(&format!("Pop value from stack to register {}", register), logging);
            },
            "#JMP" => {
                let address = match evaluate_value(&tokens, 1) {
                    Ok(val) => val,
                    Err(_) => match handle_identifier(tokens[1], &mem) {
                        Ok(val) => val,
                        Err(err) => return Err(err),
                    }
                };
            },
            "#RTR" => {},
            "#BRA" => {},
            "#HLT" => {},
            _ => return Err(err::AssembleError::InvalidToken),
        },
        _ => match tokens[1] {
            "->" => {
                logging::log("Storing is not yet supported", logging);
            }
            "<-" => {
                let register_index = match get_register_index(tokens[0]) {
                    Ok(val) => val,
                    Err(err) => return Err(err),
                };
            }
            _ => {
                eprintln!(
                    "\x1b[38;2;255;50;0mError: Invalid Token: \x1b[38;2;127;255;200m{}\x1b[0m",
                    first_token_symbol
                );
                return Err(err::AssembleError::InvalidSyntax);
            }
        },
    }
    Ok(())
}

fn get_register_index(src: &str) -> Result<usize, crate::err::AssembleError> {
    if src.contains("reg:") {
        #[allow(unused)]
        let register = usize::from_str_radix(src.split(':').last().unwrap(), 10).unwrap();
        Ok(register)
    } else {
        return Err(crate::err::AssembleError::InvalidSyntax);
    }
}

fn evaluate_value(src: &Vec<&str>, idx: usize) -> Result<Vec<u8>, AssembleError> {
    let mut bytes: Vec<u8>;
    let len;
    match src[idx].split_at(1).0 {
        "$"  => {
            bytes = usize::from_str_radix(src[idx].trim_matches('$'), 16).unwrap().to_le_bytes().to_vec();
            len = bytes.iter().rposition(|&b| b != 0).map_or(1, |i| i + 1);
        },
        "#"  => {
            bytes = usize::from_str_radix(src[idx].trim_matches('#'), 10).unwrap().to_le_bytes().to_vec();
            len = bytes.iter().rposition(|&b| b != 0).map_or(1, |i| i + 1);
        },
        "\"" => {
            bytes = Vec::new();
            for token in &src[idx..] {
                let token = token.trim_matches('"');
                for c in token.chars() {
                    bytes.push(c as u8);
                }
            }
            bytes.push(b'\0');
            len = bytes.len()
        }
        _ => return Err(err::AssembleError::InvalidSyntax),
    }
    return Ok(bytes[..len].to_vec());
}

fn handle_identifier(src: &str, mem: &Memory) -> Result<Vec<u8>, AssembleError> {
    let split: Vec<&str> = src.split('.').collect();
    for block in &mem.blocks {
        if block.name == split[0] {
            for label in &block.labels {
                if label.name == split[1] {
                    return Ok(label.address.to_le_bytes().to_vec());
                }
            }
        }
    }
    Err(AssembleError::InvalidIdentifier)
}
