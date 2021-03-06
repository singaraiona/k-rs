#[derive(Debug)]
pub enum Error {
    RuntimeError,
    Rank,
    Type,
    Length,
    Condition,
    Call,
    Undefined,
    Stack,
    InvalidString,
    NotImplemented,
    InvalidType,
    InvalidNativeCall,
}