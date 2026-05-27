pub fn split_into_lines(source: &str) -> Vec<String> {
    source.lines().map(|line| line.trim().to_string()).collect()
}

pub fn tokenize_line(line: &str) -> Vec<String> {
    line.split_whitespace().map(|token| token.to_string()).collect()
}