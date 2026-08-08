use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors produced while validating Archive.org identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentifierError {
    /// The identifier is shorter than the supported minimum length.
    TooShort,
    /// The identifier exceeds the supported maximum length.
    TooLong,
    /// The identifier starts with a forbidden leading character.
    InvalidLeadingCharacter(char),
    /// The identifier contains an unsupported character.
    InvalidCharacter(char),
}

impl Display for IdentifierError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "Identifier should be between 3 and 80 characters in length."),
            Self::TooLong => write!(f, "Identifier should be between 3 and 80 characters in length."),
            Self::InvalidLeadingCharacter(ch) => write!(
                f,
                "Identifier cannot begin with periods \".\", underscores \"_\", or dashes \"-\". Found {ch:?}."
            ),
            Self::InvalidCharacter(ch) => write!(
                f,
                "Identifier can only contain alphanumeric characters, periods \".\", underscores \"_\", or dashes \"-\". Found {ch:?}."
            ),
        }
    }
}

impl Error for IdentifierError {}
