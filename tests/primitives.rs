use jolt::{TokenType, tokenize};

#[test]
fn test_primitive_boolean() {
    let tokens = tokenize("true").unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenType::Primitive);
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 4);
}

#[test]
fn test_multiple_primitives() {
    let tokens = tokenize("[true,false,42]").unwrap();

    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenType::Array);

    assert_eq!(tokens[1].start, 1);
    assert_eq!(tokens[1].end, 5);
    assert_eq!(tokens[2].start, 6);
    assert_eq!(tokens[2].end, 11);
    assert_eq!(tokens[3].start, 12);
    assert_eq!(tokens[3].end, 14);
}
