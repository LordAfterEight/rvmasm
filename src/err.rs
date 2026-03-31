#[derive(Debug)]
pub enum AssembleError {
    InvalidToken,
    InvalidSyntax,
    InvalidIdentifier,
    InvalidValue,
}
