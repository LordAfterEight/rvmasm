mod assemble8;
mod err;
mod load_src;
mod logging;
mod mem;
mod opcodes;

use std::io::Write;

use crate::err::AssembleError;
use crate::logging::LoggingVerbosity;
use crate::mem::Memory;
use crate::opcodes::{OpCodes, OperandKind};

fn main() {
    let mut src_path: &str = "main.rvmasm";
    let mut out_path: &str = "ROM.rvm";
    let mut src_path_provided = false;
    let mut logging = LoggingVerbosity::None;
    let mut live_edit = false;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--src" {
            i += 1;
            if let Some(path) = args.get(i) {
                src_path = path;
                src_path_provided = true;
            } else {
                print!("\x1b[38;2;255;50;0mMissing file path\x1b[0m\n");
                return;
            }
        } else if arg == "--out" {
            i += 1;
            out_path = args.get(i).map(|s| s.as_str()).unwrap_or("ROM.bin");
        } else if arg == "--verbose" {
            logging = logging::LoggingVerbosity::Full;
        } else if arg == "--edit" {
            live_edit = true;
        } else if arg == "--help" {
            println!("Here is a list of available arguments:");
            println!(" --src <path>  | The file to be assembled");
            println!(" --out <name>  | The name of the output file");
            println!(" --verbose     | Show detailed logging information");
            println!(" --edit        | Start live editing shell after assembly");
            return;
        } else {
            eprint!("\x1b[38;2;255;50;0mInvalid argument:\x1b[0m {}\n", arg);
            return;
        }
        i += 1;
    }

    if !src_path_provided {
        eprint!(
            "\x1b[38;2;255;200;50mWARNING: No src path provided. Looking for '\x1b[38;2;255;50;255mmain.rvmasm\x1b[38;2;255;200;0m'\x1b[0m\n"
        );
    }

    // ======== Loading ========

    let buf = match load_src::load_src_file(src_path) {
        Ok(b) => b,
        Err(_) => {
            eprint!("\x1b[38;2;255;50;0mLoading failed\n");
            return;
        }
    };

    let mut line_counter = 0;
    let mut assemble_now = false;
    eprint!("\n");

    // ======== Memory Init ========
    let mut mem = Memory {
        data: Box::new(Vec::new()),
        labels: Vec::new(),
        blocks: Vec::new(),
        mem_ptr: 0,
    };

    // ======== Parsing ========

    let content = std::str::from_utf8(&buf).unwrap_or("");
    let mut bit_width = 0;
    let mut addr_width = 0;

    for line in content.split('\n') {
        line_counter += 1;

        let tokens: Vec<&str> = line.split_whitespace().filter(|t| !t.is_empty()).collect();

        if line.is_empty() || tokens[0] == "//" {
            println!();
            continue;
        }

        eprint!(
            "\x1b[38;2;255;200;50m{:03}  \x1b[38;2;127;255;200m{}\x1b[0m\n",
            line_counter, line
        );

        let first_token_symbol = tokens[0].split_inclusive(">").collect::<Vec<&str>>()[0];

        if !assemble_now {
            match first_token_symbol {
                ">" => match tokens[0] {
                    ">CFG" => match tokens[1] {
                        "bit-width" => {
                            bit_width = bytes_to_usize(&evaluate_value(&tokens, 2).unwrap());
                            logging::log(
                                &format!(
                                    "CPU Bit width: {} bit{}",
                                    bit_width,
                                    if bit_width > 1 { "s" } else { "" }
                                ),
                                &logging,
                            );
                            continue;
                        }
                        "addr-width" => {
                            addr_width = bytes_to_usize(&evaluate_value(&tokens, 2).unwrap());
                            logging::log(
                                &format!("Setting address width to {} byte{}", addr_width, if addr_width != 1 { "s" } else { "" }),
                                &logging,
                            );
                        }
                        _ => {
                            eprintln!(
                                "\x1b[38;2;255;50;0mError: Invalid configurator: \x1b[38;2;127;255;200m{}\x1b[0m",
                                tokens[1]
                            );
                            break;
                        }
                    },
                    ">PRG" => {
                        logging::log(
                            &format!(
                                "Assembling program with bit width: {} bit{}",
                                bit_width,
                                if bit_width > 1 { "s" } else { "" }
                            ),
                            &logging,
                        );
                        assemble_now = true;
                    }
                    _ => {
                        eprintln!(
                            "\x1b[38;2;255;50;0mError: Invalid configurator: \x1b[38;2;127;255;200m{}\x1b[0m",
                            tokens[0]
                        );
                        break;
                    }
                },
                _ => {
                    eprintln!(
                        "\x1b[38;2;255;50;0mError: Invalid Token: \x1b[38;2;127;255;200m{}\x1b[0m | \x1b[38;2;255;200;50mConfigurator '>CFG' expected here\x1b[0m",
                        first_token_symbol
                    );
                    break;
                }
            }
            match bit_width {
                0 => {
                    eprintln!("\x1b[38;2;255;50;0mError: Bit width needs to be defined\x1b");
                    break;
                }
                8 => {}
                _ => {
                    eprintln!(
                        "\x1b[38;2;255;50;0mError: Bit width of {} is not supported\x1b",
                        bit_width
                    );
                    break;
                }
            }
            if addr_width == 0 {
                eprintln!("\x1b[38;2;255;50;0mError: Address width needs to be defined\x1b");
                std::process::exit(1);
            }
        } else {
            match bit_width {
                0 => {
                    eprintln!("\x1b[38;2;255;50;0mError: Bit width needs to be defined\x1b");
                    break;
                }
                8 => match assemble8::assemble(tokens, &mut mem, &logging, bit_width as u8, addr_width as u8) {
                    Ok(_) => continue,
                    Err(err) => {
                        eprintln!("\x1b[38;2;255;50;0mError: {:?}\x1b", err);
                        std::process::exit(0);
                    }
                },
                _ => {
                    eprintln!(
                        "\x1b[38;2;255;50;0mError: Bit width of {} is not supported\x1b",
                        bit_width
                    );
                    break;
                }
            }
        }
    }

    if logging == LoggingVerbosity::Full {
        println!();
        for block in &mem.blocks {
            for variable in &block.variables {
                println!(
                    "Block {}: Variable {}, at address 0x{:04X}",
                    block.name, variable.name, variable.address
                );
            }
        }
    }

    // ======== Bake variables into block data ========
    for block in &mut mem.blocks {
        // Ensure block.data is large enough to hold variables
        for variable in &block.variables {
            let rel_addr = variable.address - block.address;
            let end = rel_addr + variable.value.len();
            if end > block.data.len() {
                block.data.resize(end, 0);
            }
        }
        for variable in &block.variables {
            let rel_addr = variable.address - block.address;
            for idx in 0..variable.value.len() {
                block.data[rel_addr + idx] = variable.value[idx];
            }
        }
        // Update block length to match data
        block.length = block.data.len();
    }

    // ======== Resolve reset vector ========
    let reset_vector = mem
        .blocks
        .iter()
        .find(|b| b.name == "start")
        .map(|b| b.address)
        .unwrap_or_else(|| {
            eprintln!("\x1b[38;2;255;50;0mError: No block named 'start' found for reset vector\x1b[0m");
            std::process::exit(1);
        });

    // ======== Write RVM file ========
    let mut out_file = std::fs::File::create(out_path).unwrap();
    let mut file_data: Vec<u8> = Vec::new();

    // File header
    file_data.extend_from_slice(b"RVM");
    file_data.push(bit_width as u8);
    file_data.push(addr_width as u8);
    file_data.extend_from_slice(&reset_vector.to_le_bytes()[..addr_width]);
    file_data.extend_from_slice(&(mem.blocks.len() as u16).to_le_bytes());

    // Block entries
    for block in &mem.blocks {
        // Block name
        file_data.push(block.name.len() as u8);
        file_data.extend_from_slice(block.name.as_bytes());
        // Load address
        file_data.extend_from_slice(&block.address.to_le_bytes()[..addr_width]);
        // Data length
        file_data.extend_from_slice(&block.data.len().to_le_bytes()[..addr_width]);
        // Data
        file_data.extend_from_slice(&block.data);
    }

    let bytes_written = out_file.write(&file_data).unwrap();
    println!("{} Bytes written", bytes_written);

    if live_edit {
        println!("\nLive Edit\n");
        let file_data = std::fs::read(out_path).unwrap();
        println!("Read {}B", file_data.len());

        loop {
            let mut buf = String::new();
            print!("{} <= ", out_path);
            _ = std::io::stdout().flush();
            let n = std::io::stdin().read_line(&mut buf).unwrap_or(0);
            let input: Vec<&str> = buf.split_whitespace().collect();

            if n == 0 { break; }
            if input.is_empty() { continue; }

            match input[0] {
                "list" => if input.len() > 1 {
                    match input[1] {
                        "variables" => {
                            for block in &mem.blocks {
                                for variable in &block.variables {
                                    println!(
                                        "  \x1b[38;2;100;100;100mat \x1b[38;2;150;100;150m0x{:08X}\x1b[38;2;100;100;100m in Block '\x1b[38;2;150;150;100m{}\x1b[38;2;100;100;100m': Variable '\x1b[38;2;100;150;150m{}\x1b[38;2;100;100;100m'\x1b[m",
                                        variable.address, block.name, variable.name
                                    );
                                }
                            }
                        },
                        "blocks" => {
                            for block in &mem.blocks {
                                println!(
                                    "\nBlock: \x1b[38;2;150;150;100m{}\x1b[38;2;100;100;100m @ \x1b[38;2;150;100;150m0x{:08X}\x1b[38;2;100;100;100m ({} bytes)\x1b[m",
                                    block.name, block.address, block.data.len()
                                );
                                let mut i = 0;
                                while i < block.data.len() {
                                    let abs_addr = block.address + i;

                                    // Check if this offset is a known instruction start
                                    if block.instruction_offsets.contains(&i) {
                                        if let Some(op) = OpCodes::from_byte(block.data[i]) {
                                            let size = op.instruction_size(addr_width);
                                            let end = std::cmp::min(i + size, block.data.len());

                                            // Print hex bytes
                                            let hex: Vec<String> = block.data[i..end].iter().map(|b| format!("{:02X}", b)).collect();
                                            let hex_str = hex.join(" ");

                                            // Format operands
                                            let operand_str = match op.operands() {
                                                OperandKind::None => String::new(),
                                                OperandKind::Reg => {
                                                    if i + 1 < end { format!("reg:{}", block.data[i + 1]) } else { String::new() }
                                                }
                                                OperandKind::Addr => {
                                                    let addr_bytes = &block.data[i + 1..end];
                                                    let addr = bytes_to_usize(addr_bytes);
                                                    format!("${:0width$X}", addr, width = addr_width * 2)
                                                }
                                                OperandKind::RegReg => {
                                                    if i + 2 < end {
                                                        format!("reg:{}, reg:{}", block.data[i + 1], block.data[i + 2])
                                                    } else { String::new() }
                                                }
                                                OperandKind::RegAddr => {
                                                    let reg = if i + 1 < end { block.data[i + 1] } else { 0 };
                                                    let addr_bytes = &block.data[i + 2..end];
                                                    let addr = bytes_to_usize(addr_bytes);
                                                    format!("reg:{}, ${:0width$X}", reg, addr, width = addr_width * 2)
                                                }
                                                OperandKind::RegRegReg => {
                                                    if i + 3 < end {
                                                        format!("reg:{}, reg:{}, reg:{}", block.data[i + 1], block.data[i + 2], block.data[i + 3])
                                                    } else { String::new() }
                                                }
                                            };

                                            println!(
                                                " \x1b[38;2;150;100;150m0x{:08X}:  \x1b[38;2;200;150;50m{:<width$}\x1b[38;2;100;200;255m{}\x1b[38;2;100;100;100m {}\x1b[m",
                                                abs_addr, hex_str, op.name(), operand_str,
                                                width = (1 + addr_width) * 3 + 2,
                                            );
                                            i = end;
                                            continue;
                                        }
                                    }

                                    // Variable or other data byte
                                    let is_var = block.variables.iter().any(|v| abs_addr >= v.address && abs_addr < v.address + v.value.len());
                                    let var_name = block.variables.iter().find(|v| abs_addr == v.address).map(|v| v.name.as_str());
                                    if is_var {
                                        if let Some(name) = var_name {
                                            println!(
                                                " \x1b[38;2;150;100;150m0x{:08X}:  \x1b[38;2;150;200;50m{:02X}\x1b[38;2;100;100;100m           @VAR {}\x1b[m",
                                                abs_addr, block.data[i], name
                                            );
                                        } else {
                                            println!(
                                                " \x1b[38;2;150;100;150m0x{:08X}:  \x1b[38;2;150;200;50m{:02X}\x1b[m",
                                                abs_addr, block.data[i]
                                            );
                                        }
                                    } else if block.data[i] != 0 {
                                        println!(
                                            " \x1b[38;2;150;100;150m0x{:08X}:  \x1b[38;2;200;150;50m{:02X}\x1b[m",
                                            abs_addr, block.data[i]
                                        );
                                    } else {
                                        println!(
                                            " \x1b[38;2;150;100;150m0x{:08X}:  \x1b[38;2;100;100;100m{:02X}\x1b[m",
                                            abs_addr, block.data[i]
                                        );
                                    }
                                    i += 1;
                                }
                                println!();
                            }
                        }
                        _ => println!("Invalid argument: {}", input[1]),
                    }
                } else {
                    println!("Command usage: 'list <option>'")
                }

                "dump" => {
                    // Dump raw file bytes in hex
                    let mut x_idx = 0;
                    for addr in (0..file_data.len()).step_by(16) {
                        let end = std::cmp::min(addr + 16, file_data.len());
                        print!(" \x1b[38;2;150;100;150m0x{:08X}: ", addr);
                        for i in addr..end {
                            if file_data[i] != 0 {
                                print!(" \x1b[38;2;200;150;50m{:02X}\x1b[m ", file_data[i]);
                            } else {
                                print!(" \x1b[38;2;100;100;100m{:02X}\x1b[m ", file_data[i]);
                            }
                            x_idx += 1;
                            if x_idx == 8 {
                                print!(" \x1b[38;2;100;100;100m|\x1b[m ");
                            }
                            if x_idx == 16 {
                                x_idx = 0;
                            }
                        }
                        println!();
                    }
                },
                "header" => {
                    let magic = [file_data[0], file_data[1], file_data[2]];
                    let valid = &magic == b"RVM";
                    println!(
                        "  \x1b[38;2;100;100;100mMagic:        \x1b[38;2;{}m{}\x1b[m",
                        if valid { "100;200;100" } else { "255;50;0" },
                        std::str::from_utf8(&magic).unwrap_or("???")
                    );
                    if valid {
                        let hdr_bit_width = file_data[3];
                        let hdr_addr_width = file_data[4] as usize;
                        let hdr_reset_vector = bytes_to_usize(&file_data[5..5 + hdr_addr_width]);
                        let hdr_block_count = u16::from_le_bytes([file_data[5 + hdr_addr_width], file_data[6 + hdr_addr_width]]);
                        println!(
                            "  \x1b[38;2;100;100;100mBit Width:    \x1b[38;2;150;200;255m{}\x1b[38;2;100;100;100m bit{}\x1b[m",
                            hdr_bit_width,
                            if hdr_bit_width != 1 { "s" } else { "" }
                        );
                        println!(
                            "  \x1b[38;2;100;100;100mAddr Width:   \x1b[38;2;150;200;255m{}\x1b[38;2;100;100;100m byte{}\x1b[m",
                            hdr_addr_width,
                            if hdr_addr_width != 1 { "s" } else { "" }
                        );
                        println!(
                            "  \x1b[38;2;100;100;100mReset Vector: \x1b[38;2;150;255;150m0x{:0width$X}\x1b[m",
                            hdr_reset_vector,
                            width = hdr_addr_width * 2
                        );
                        println!(
                            "  \x1b[38;2;100;100;100mBlocks:       \x1b[38;2;150;200;255m{}\x1b[m",
                            hdr_block_count
                        );

                        // Parse and display block entries
                        let mut cursor = 5 + hdr_addr_width + 2;
                        for _ in 0..hdr_block_count {
                            if cursor >= file_data.len() { break; }
                            let name_len = file_data[cursor] as usize;
                            cursor += 1;
                            let name = std::str::from_utf8(&file_data[cursor..cursor + name_len]).unwrap_or("???");
                            cursor += name_len;
                            let load_addr = bytes_to_usize(&file_data[cursor..cursor + hdr_addr_width]);
                            cursor += hdr_addr_width;
                            let data_len = bytes_to_usize(&file_data[cursor..cursor + hdr_addr_width]);
                            cursor += hdr_addr_width;
                            println!(
                                "    \x1b[38;2;150;150;100m{}\x1b[38;2;100;100;100m @ \x1b[38;2;150;100;150m0x{:0width$X}\x1b[38;2;100;100;100m ({} bytes)\x1b[m",
                                name, load_addr, data_len, width = hdr_addr_width * 2
                            );
                            cursor += data_len;
                        }
                    } else {
                        println!("  \x1b[38;2;255;50;0mNot a valid RVM file\x1b[m");
                    }
                },
                "exit" => break,
                _ => println!("Invalid command"),
            }
        }
    }
}

fn evaluate_value(src: &Vec<&str>, idx: usize) -> Result<Vec<u8>, AssembleError> {
    let mut bytes: Vec<u8>;
    let len;
    match src[idx].split_at(1).0 {
        "$" => {
            bytes = usize::from_str_radix(src[idx].trim_matches('$'), 16)
                .unwrap()
                .to_le_bytes()
                .to_vec();
            len = bytes.iter().rposition(|&b| b != 0).map_or(1, |i| i + 1);
        }
        "#" => {
            bytes = usize::from_str_radix(src[idx].trim_matches('#'), 10)
                .unwrap()
                .to_le_bytes()
                .to_vec();
            len = bytes.iter().rposition(|&b| b != 0).map_or(1, |i| i + 1);
        }
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
            len = bytes.len()
        }
        _ => return Err(AssembleError::InvalidSyntax),
    }
    return Ok(bytes[..len].to_vec());
}

fn bytes_to_usize(slice: &[u8]) -> usize {
    let mut buf = [0u8; 8];
    buf[..slice.len()].copy_from_slice(slice);
    usize::from_le_bytes(buf)
}
