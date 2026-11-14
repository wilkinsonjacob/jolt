use jolt::{TokenType, tokenize};

#[test]
fn test_valid_string() {
    let result = tokenize(r#""hello""#);

    assert!(result.is_ok());

    let tokens = result.unwrap();

    assert_eq!(tokens.len(), 1);
}

#[test]
fn test_invalid_string() {
    let result = tokenize(r#""hello"#);

    assert!(result.is_err());
}

#[test]
fn test_multiple_strings() {
    let result = tokenize(r#""hello" "world""#);

    assert!(result.is_ok());

    let tokens = result.unwrap();

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].start, 1);
    assert_eq!(tokens[0].end, 6);
    assert_eq!(tokens[1].start, 9);
    assert_eq!(tokens[1].end, 14);
}

#[test]
fn test_valid_escape() {
    let result = tokenize(r#""hello \"world\"""#);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_escape() {
    let result = tokenize(r#""hello\q""#);
    assert!(result.is_err());
}

#[test]
fn test_incomplete_escape() {
    let result = tokenize(r#""hello\"#);
    assert!(result.is_err());
}

#[test]
fn test_valid_unicode_escape() {
    let tokens = tokenize(r#""hello \u0041""#).unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenType::String);
    assert_eq!(tokens[0].start, 1);
    assert_eq!(tokens[0].end, 13);
}

#[test]
fn test_invalid_unicode_escape() {
    let result = tokenize(r#""hello \u12G4""#);

    assert!(result.is_err());
}

#[test]
fn test_incomplete_unicode_escape() {
    let result = tokenize(r#""hello \u123"#);

    assert!(result.is_err());
}

#[test]
fn test_unescaped_newline() {
    let result = tokenize("\"hello\nworld\"");
    assert!(result.is_err());
}

#[test]
fn test_escaped_newline() {
    let result = tokenize(r#""hello\nworld""#);
    assert!(result.is_ok());
}

#[test]
fn test_unescaped_tab() {
    let result = tokenize("\"hello\tworld\"");
    assert!(result.is_err());
}
