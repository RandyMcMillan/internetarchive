use crate::errors::IdentifierError;

/// Validate that a string is a legal Archive.org identifier.
///
/// The rules mirror the Python implementation:
/// - length must be between 3 and 100 characters inclusive
/// - the identifier may not begin with `.`, `_`, or `-`
/// - `@` is allowed as a leading character for user items
/// - remaining characters must be alphanumeric, `.`, `_`, or `-`
pub fn validate_s3_identifier(identifier: &str) -> Result<(), IdentifierError> {
    let length = identifier.chars().count();
    if length < 3 {
        return Err(IdentifierError::TooShort);
    }
    if length > 100 {
        return Err(IdentifierError::TooLong);
    }

    let trimmed = identifier.strip_prefix('@').unwrap_or(identifier);
    let mut chars = trimmed.chars();

    if let Some(first) = chars.next() {
        if matches!(first, '.' | '_' | '-') {
            return Err(IdentifierError::InvalidLeadingCharacter(first));
        }
        if !is_legal_identifier_char(first) {
            return Err(IdentifierError::InvalidCharacter(first));
        }
    }

    for ch in chars {
        if !is_legal_identifier_char(ch) {
            return Err(IdentifierError::InvalidCharacter(ch));
        }
    }

    Ok(())
}

fn is_legal_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_identifiers() {
        for identifier in ["nasa", "archive.org", "@user_item"] {
            assert!(validate_s3_identifier(identifier).is_ok());
        }
    }

    #[test]
    fn rejects_short_identifiers() {
        assert_eq!(
            validate_s3_identifier("ab"),
            Err(IdentifierError::TooShort)
        );
    }

    #[test]
    fn rejects_invalid_leading_characters() {
        assert!(matches!(
            validate_s3_identifier(".abc"),
            Err(IdentifierError::InvalidLeadingCharacter('.'))
        ));
    }

    #[test]
    fn rejects_invalid_characters() {
        assert!(matches!(
            validate_s3_identifier("føø"),
            Err(IdentifierError::InvalidCharacter('ø'))
        ));
    }
}
