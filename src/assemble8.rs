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
    bit_width: u8,
    addr_width: u8,
) -> Result<(), err::AssembleError> {
    let first_token_symbol = tokens[0].split_at(1).0;

    match first_token_symbol {
        "@" => match tokens[0] {
            "@MAP" => {
                mem.mem_ptr = usize::from_str_radix(tokens[1].trim_matches('$'), 16).unwrap();
            }
            "@LBL" => {
                mem.blocks.last_mut().unwrap().labels.push(Label::new(tokens[1], mem.mem_ptr));
            }
            "@BLK" => {
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
                let (value, vartype) = match evaluate_value(&tokens, 2, 0) {
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
                    vartype: vartype,
                };
                mem.mem_ptr += variable.length;
                let block = mem.blocks.last_mut().unwrap();
                block.length += variable.length;
                block.variables.push(variable);
            },
            "@END" => {},
            _ => return Err(AssembleError::InvalidToken),
        },
        "#" => match tokens[0] {
            "#PSH" => {
                let register = match get_register_index(tokens[1]) {
                    Ok(val) => val,
                    Err(err) => return Err(err)
                };

                let block = mem.blocks.last_mut().unwrap();
                block.data.push(OpCodes::PUSH_RGST as u8);
                block.data.push(register as u8);
                log(&format!("Push register {} to stack", register), logging);
            },
            "#POP" => {
                let register = match get_register_index(tokens[1]) {
                    Ok(val) => val,
                    Err(err) => return Err(err)
                };

                let block = mem.blocks.last_mut().unwrap();
                block.data.push(OpCodes::POP_RGST as u8);
                block.data.push(register as u8);
                log(&format!("Pop value from stack to register {}", register), logging);
            },
            "#JMP" => {
                let address = match get_address(&tokens, 1, &mem, addr_width) {
                    Ok(val) => val,
                    Err(err) => return Err(err),
                };
                let block = mem.blocks.last_mut().unwrap();
                block.data.push(OpCodes::JUMP_IMM as u8);
                for byte in address {
                    block.data.push(byte);
                }
            },
            "#RTR" => {
                let block = mem.blocks.last_mut().unwrap();
                block.data.push(OpCodes::RTRN as u8);
                block.length += 1;
            },
            "#BRA" => {
                let address = match get_address(&tokens, 1, &mem, addr_width) {
                    Ok(val) => val,
                    Err(err) => return Err(err),
                };
                let block = mem.blocks.last_mut().unwrap();
                block.data.push(OpCodes::BRAN_IMM as u8);
                for byte in address {
                    block.data.push(byte);
                }
            },
            "#HLT" => {
                let block = mem.blocks.last_mut().unwrap();
                block.data.push(OpCodes::HALT as u8);
                block.length += 1;
            },
            _ => return Err(AssembleError::InvalidToken),
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
                return Err(AssembleError::InvalidSyntax);
            }
        },
    }
    Ok(())
}

fn get_register_index(src: &str) -> Result<usize, crate::err::AssembleError> {
    if src.contains("reg:") {
        let register = usize::from_str_radix(src.split(':').last().unwrap(), 10).unwrap();
        Ok(register)
    } else {
        return Err(AssembleError::InvalidSyntax);
    }
}

fn evaluate_value(src: &Vec<&str>, idx: usize, bit_width: u8) -> Result<(Vec<u8>, VariableType), AssembleError> {
    let mut bytes: Vec<u8>;
    let vartype;
    let len;
    match src[idx].split_at(1).0 {
        "$"  => {
            let raw = usize::from_str_radix(src[idx].trim_matches('$'), 16).unwrap();
            if bit_width > 0 && bit_width < 64 && raw >= (1usize << bit_width) {
                return Err(AssembleError::InvalidValueWidth);
            }
            bytes = raw.to_le_bytes().to_vec();
            len = bytes.iter().rposition(|&b| b != 0).map_or(1, |i| i + 1);
            vartype = VariableType::Integer;
        },
        "#"  => {
            let raw = usize::from_str_radix(src[idx].trim_matches('#'), 10).unwrap();
            if bit_width > 0 && bit_width < 64 && raw >= (1usize << bit_width) {
                return Err(AssembleError::InvalidValueWidth);
            }
            bytes = raw.to_le_bytes().to_vec();
            len = bytes.iter().rposition(|&b| b != 0).map_or(1, |i| i + 1);
            vartype = VariableType::Integer;
        },
        "\"" => {
            bytes = Vec::new();
            for token in &src[idx..] {
                for c in token.trim_matches('"').chars() {
                    bytes.push(c as u8);
                }
                if !token.ends_with("\"") {
                    bytes.push(b' ');
                }
            }
            bytes.push(b'\0');
            vartype = VariableType::String;
            len = bytes.len();
        }
        _ => return Err(AssembleError::InvalidSyntax),
    }
    return Ok((bytes[..len].to_vec(), vartype));
}

fn handle_identifier(src: &str, mem: &Memory, addr_width: u8) -> Result<Vec<u8>, AssembleError> {
    let addr_to_bytes = |address: usize| -> Result<Vec<u8>, AssembleError> {
        let raw = address.to_le_bytes();
        for i in (addr_width as usize)..8 {
            if raw[i] != 0 {
                return Err(AssembleError::InvalidValueWidth);
            }
        }
        Ok(raw[..addr_width as usize].to_vec())
    };

    if src.contains(".") {
        let split: Vec<&str> = src.split('.').collect();
        for block in &mem.blocks {
            if block.name == split[0] {
                for label in &block.labels {
                    if label.name == split[1] {
                        return addr_to_bytes(label.address);
                    }
                }
            }
        }
        return Err(AssembleError::NoSuchLabel(src.to_string()))
    } else {
        for label in &mem.blocks.last().unwrap().labels {
            if label.name == src {
                return addr_to_bytes(label.address);
            }
        }
        return Err(AssembleError::NoSuchLabelInScope(src.to_string()))
    }
}

fn get_address(src: &Vec<&str>, idx: usize, mem: &Memory, addr_width: u8) -> Result<Vec<u8>, AssembleError> {
    match evaluate_value(src, idx, (addr_width as u16 * 8) as u8) {
        Ok((bytes, _)) => pad_to_width(bytes, addr_width as usize),
        Err(AssembleError::InvalidSyntax) => handle_identifier(src[idx], mem, addr_width),
        Err(err) => Err(err),
    }
}

fn pad_to_width(mut bytes: Vec<u8>, width: usize) -> Result<Vec<u8>, AssembleError> {
    if bytes.len() > width {
        return Err(AssembleError::InvalidValueWidth);
    }
    bytes.resize(width, 0);
    Ok(bytes)
}

fn check_condition_presence(src: Vec<&str>) -> bool {
    for token in src {
        if token == "?" {
            return true
        }
    }
    false
}
