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
    let mut file_size = 0;
    let mut reset_vector;
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
                        "file-size" => {
                            file_size = bytes_to_usize(&evaluate_value(&tokens, 2).unwrap());
                            logging::log(
                                &format!("Setting output file size to {}B", file_size),
                                &logging,
                            );
                        }
                        "reset-vector" => {
                            reset_vector = bytes_to_usize(&evaluate_value(&tokens, 2).unwrap());
                            logging::log(
                                &format!("Setting reset vector to {:08X}", reset_vector),
                                &logging,
                            );
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

    // ======== ROM writing ========
    let mut out_file = std::fs::File::create(out_path).unwrap();

    mem.data.resize(file_size, 0);

    for block in &mem.blocks {
        for variable in &block.variables {
            for idx in 0..variable.value.len() {
                mem.data[variable.address + idx] = variable.value[idx];
            }
        }
        for idx in 0..block.data.len() {
            mem.data[block.address + idx] = block.data[idx];
        }
    }

    // ======== RVM Header ========
    mem.data[0] = b'R';
    mem.data[1] = b'V';
    mem.data[2] = b'M';
    mem.data[3] = bit_width as u8;
    mem.data[4] = addr_width as u8;

    let bytes_written = out_file.write(&mem.data).unwrap();
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
                                println!("\nBlock: \x1b[38;2;150;150;100m{}\x1b[m", block.name);
                                let mut byte_printed = false;
                                for addr in block.address..block.address+block.length {
                                    print!(" \x1b[38;2;150;100;150m0x{:08X}: ", addr);
                                    for variable in block.variables.iter() {
                                        if addr == variable.address {
                                            println!(" \x1b[38;2;150;200;50m{:02X}\x1b[m ", file_data[addr]);
                                            byte_printed = true;
                                        }
                                    }
                                    if byte_printed == false {
                                        if file_data[addr] != 0 {
                                            println!(" \x1b[38;2;200;150;50m{:02X}\x1b[m ", file_data[addr]);
                                        } else {
                                            println!(" \x1b[38;2;100;100;100m{:02X}\x1b[m ", file_data[addr]);
                                        }
                                    }
                                    byte_printed = false;
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
                    if input.len() == 1 {
                        let mut x_idx = 0;
                        let mut byte_printed = false;
                        let mut found_variables = Vec::new();
                        for addr in (0..file_data.len()).step_by(16) {
                            for sub_addr in 0..16 {
                                if x_idx == 0 {
                                    print!(" \x1b[38;2;150;100;150m0x{:08X} - 0x{:08X}: ", addr, addr + 15);
                                }
                                for block in &mem.blocks {
                                    for variable in block.variables.iter() {
                                        if addr + sub_addr == variable.address {
                                            print!(" \x1b[38;2;150;200;50m{:02X}\x1b[m ", file_data[addr+ sub_addr]);
                                            found_variables.push(variable);
                                            byte_printed = true;
                                        }
                                    }
                                }
                                if byte_printed == false {
                                    if file_data[addr + sub_addr] != 0 {
                                        print!(" \x1b[38;2;200;150;50m{:02X}\x1b[m ", file_data[addr + sub_addr]);
                                    } else {
                                        print!(" \x1b[38;2;100;100;100m{:02X}\x1b[m ", file_data[addr + sub_addr]);
                                    }
                                }
                                byte_printed = false;
                                x_idx += 1;
                                if x_idx == 8 {
                                    print!(" \x1b[38;2;100;100;100m|\x1b[m ");
                                }

                                if x_idx == 16 {
                                    if found_variables.len() != 0 {
                                        for variable in &found_variables {
                                            print!(" {}:0x{:08X} ", variable.name, variable.address);
                                        }
                                        found_variables.clear();
                                    }

                                    x_idx = 0;
                                    println!();
                                }
                            }
                        }
                    } else {
                        match input[1] {
                            "non-null" => {
                            }
                            _ => println!("Invalid option: {}", input[1])
                        }
                    }
                },
                "header" => {
                    let magic = [file_data[0], file_data[1], file_data[2]];
                    let valid = &magic == b"RVM";
                    println!(
                        "  \x1b[38;2;100;100;100mMagic:      \x1b[38;2;{}m{}\x1b[m",
                        if valid { "100;200;100" } else { "255;50;0" },
                        std::str::from_utf8(&magic).unwrap_or("???")
                    );
                    if valid {
                        println!(
                            "  \x1b[38;2;100;100;100mBit Width:  \x1b[38;2;150;200;255m{}\x1b[38;2;100;100;100m bit{}\x1b[m",
                            file_data[3],
                            if file_data[3] != 1 { "s" } else { "" }
                        );
                        println!(
                            "  \x1b[38;2;100;100;100mAddr Width: \x1b[38;2;150;200;255m{}\x1b[38;2;100;100;100m byte{}\x1b[m",
                            file_data[4],
                            if file_data[4] != 1 { "s" } else { "" }
                        );
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
