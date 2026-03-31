mod assemble8;
mod err;
mod load_src;
mod logging;
mod mem;
mod opcodes;

use std::io::Write;

use crate::logging::{log, LoggingVerbosity};
use crate::mem::Memory;

fn main() {
    let mut src_path: &str = "main.rvmasm";
    let mut out_path: &str = "ROM.bin";
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
        eprint!("\x1b[38;2;255;200;50mWARNING: No src path provided. Looking for '\x1b[38;2;255;50;255mmain.rvmasm\x1b[38;2;255;200;0m'\x1b[0m\n");
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
        data: Vec::with_capacity(0x1_0000_0000),
        labels: Vec::new(),
        blocks: Vec::new(),
        block_tracker: 0,
        mem_ptr: 0,
    };

    // ======== Parsing ========

    let content = std::str::from_utf8(&buf).unwrap_or("");
    let mut bit_width = 0;

    for line in content.split('\n') {
        line_counter += 1;

        let tokens: Vec<&str> = line.split_whitespace().filter(|t| !t.is_empty()).collect();

        if line.is_empty() || tokens[0] == "//" {
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
                            bit_width = u8::from_str_radix(tokens[2], 10).unwrap();
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
                            logging::log(
                                &format!("Setting output file size to {} B", tokens[2]),
                                &logging,
                            );
                            mem.data = Vec::with_capacity(
                                u32::from_str_radix(tokens[2], 10).unwrap() as usize,
                            )
                        }
                        _ => {
                            eprintln!("\x1b[38;2;255;50;0mError: Invalid configurator: \x1b[38;2;127;255;200m{}\x1b[0m", tokens[1]);
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
                        eprintln!("\x1b[38;2;255;50;0mError: Invalid configurator: \x1b[38;2;127;255;200m{}\x1b[0m", tokens[0]);
                        break;
                    }
                },
                _ => {
                    eprintln!("\x1b[38;2;255;50;0mError: Invalid Token: \x1b[38;2;127;255;200m{}\x1b[0m | \x1b[38;2;255;200;50mConfigurator '>CFG' expected here\x1b[0m", first_token_symbol);
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
        } else {
            match bit_width {
                0 => {
                    eprintln!("\x1b[38;2;255;50;0mError: Bit width needs to be defined\x1b");
                    break;
                }
                8 => match assemble8::assemble(tokens, &mut mem, &logging) {
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
                println!("Block {}: Variable {}, at address 0x{:04X}", block.name, variable.name, variable.address);
            }
        }
    }

    if live_edit {
        println!("\nLive Edit\n");

        loop {
            let mut buf = String::new();
            print!("{} <= ", out_path);
            _ = std::io::stdout().flush();
            _ = std::io::stdin().read_line(&mut buf);
            let input: Vec<&str> = buf.split_whitespace().collect();

            match input[0] {
                "list" => match input[1] {
                    "variables" => {
                        for block in &mem.blocks {
                            for variable in &block.variables {
                                println!("  \x1b[38;2;100;100;100mBlock {}: Variable {}, at address 0x{:04X}\x1b[m", block.name, variable.name, variable.address);
                            }
                        }
                    },
                    _ => println!("Invalid argument: {}", input[1])
                },
                "exit" => break,
                _ => println!("Invalid command"),
            }
        }
    }
}
