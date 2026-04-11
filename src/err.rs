#[derive(Debug)]
pub enum AssembleError {
    InvalidToken,
    InvalidSyntax,
    InvalidIdentifier,
    NoSuchIdentifier,
    NoSuchLabel(String),
    NoSuchLabelInScope(String),
    InvalidValue,
    InvalidValueWidth,
    InternalLogicError,
    NotImplemented,
}
