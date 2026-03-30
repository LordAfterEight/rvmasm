mod opcodes;
mod load_src;

fn main() {
    let mut src_path: &str = "main.rvmasm";
    let mut out_path: &str = "ROM.bin";
    let mut src_path_provided = false;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-s" {
            i += 1;
            if let Some(path) = args.get(i) {
                src_path = path;
                src_path_provided = true;
            } else {
                eprint!("\x1b[38;2;255;50;0mMissing file path\x1b[0m\n");
                return;
            }
        } else if arg == "-o" {
            i += 1;
            out_path = args.get(i).map(|s| s.as_str()).unwrap_or("ROM.bin");
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

    let mut line_counter: usize = 1;
    eprint!("\n");

    // ======== Parsing ========

    let content = std::str::from_utf8(&buf).unwrap_or("");
    for line in content.split('\n').filter(|l| !l.is_empty()) {
        eprint!("\x1b[38;2;255;200;50m{:03}  \x1b[38;2;127;255;200m{}\x1b[0m\n", line_counter, line);
        line_counter += 1;
        let _tokens: Vec<&str> = line.split(' ').filter(|t| !t.is_empty()).collect();
    }

    let _ = out_path;
}
