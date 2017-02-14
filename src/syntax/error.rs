
#[derive(Debug)]
pub enum Error {
    ParseError(String),
    UnparsedText,
    NotImplemented,
    Assign,
    UnexpectedToken,
    Type,
}