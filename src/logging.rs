pub fn log(msg: &str, verbosity: &LoggingVerbosity) {
    match verbosity {
        LoggingVerbosity::Full => {
            let mut line = String::from("━━━━━");
            for _ in 0..msg.len() + 2 {
                line.push_str("━");
            }
            eprintln!("{}┓", line);
            eprintln!(">>>  {}  ┃", msg);
            eprintln!("{}┛\n", line);
        },
        LoggingVerbosity::None => {}
    }
}

#[derive(PartialEq, Eq)]
pub enum LoggingVerbosity {
    Full,
    None
}
