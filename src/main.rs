mod parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().len() < 2 {
        eprintln!("Usage: rvmasm <input_file>");
        std::process::exit(1);
    }

    let path = std::env::args().nth(1).expect("No input file provided");
    let source = std::fs::read_to_string(path).expect("Failed to read input file");

    _ = parser::blockify(source);

    Ok(())
}