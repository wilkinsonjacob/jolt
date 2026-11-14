use jolt::{TokenizeError, tokenize};

#[test]
fn test_unexpected_closing_bracket() {
    assert!(tokenize("]").is_err());
}

#[test]
fn test_mismatched_brackets() {
    assert!(tokenize("{]").is_err());
    assert!(tokenize("[}").is_err());
    assert!(tokenize("{[}]").is_err());
}

#[test]
fn test_invalid_colon() {
    assert!(tokenize(":").is_err());
}

#[test]
fn test_unclosed_object() {
    assert!(tokenize(r#"{"name":"Alice""#).is_err());
}

#[test]
fn test_unexpected_closing_brace() {
    assert!(tokenize("}").is_err());
}

#[test]
fn test_mismatched_nested_containers() {
    assert!(tokenize("{[}]").is_err());
    assert!(tokenize("[{]}").is_err());
}

#[test]
fn test_incomplete_nested_object() {
    assert!(tokenize(r#"{"user":{"name":"Alice"}"#).is_err());
}

#[test]
fn test_incomplete_json() {
    assert_eq!(tokenize("[1,2"), Err(TokenizeError::Incomplete));
    assert_eq!(tokenize(r#""hello"#), Err(TokenizeError::Incomplete));
}

#[test]
fn test_invalid_json() {
    assert_eq!(tokenize("[}"), Err(TokenizeError::Invalid));
    assert_eq!(tokenize(r#""hello\q""#), Err(TokenizeError::Invalid));
}
