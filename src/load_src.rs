use std::fs;
use std::io::Read;

/// Loads a file using a relative path.
/// Returns a Vec<u8> containing the raw bytes of the file.
///
/// Can return various errors depending on whether the file cannot
/// be opened or read, or the file buffer or memory instance could
/// not be initialized.
pub fn load_src_file(src_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(src_path).map_err(|err| {
        eprint!("\x1b[38;2;255;50;0mCould not open file at path: {}\nError:\x1b[0m {}\n", src_path, err);
        err
    })?;

    let mut buf = vec![0u8; 65536];

    let size = file.read(&mut buf).map_err(|err| {
        eprint!("\x1b[38;2;255;50;0mCould not read file\nError:\x1b[0m {}\n", err);
        err
    })?;

    buf.truncate(size);

    if size < 1024 {
        eprintln!("\x1b[38;2;50;255;50mRead {} Bytes\x1b[0m", size);
    } else if size < 1024 * 1024 {
        eprintln!("\x1b[38;2;50;255;50mRead {}KiB and {}B\x1b[0m", size / 1024, size - 1024);
    } else if size < 1024 * 1024 * 1024 {
        eprintln!("\x1b[38;2;50;255;50mRead {}MiB\x1b[0m", size / 1024 / 1024);
    }

    Ok(buf)
}
