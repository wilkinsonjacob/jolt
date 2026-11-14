use jolt::{TokenType, tokenize};

#[test]
fn test_object_token() {
    let tokens = tokenize("{}").unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenType::Object);
    assert_eq!(tokens[0].start, 0);
}

#[test]
fn test_empty_object() {
    let tokens = tokenize("{}").unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenType::Object);
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 2);
    assert_eq!(tokens[0].size, 0);
}

#[test]
fn test_nested_object_and_array() {
    let tokens = tokenize(r#"{"items":[1,2]}"#).unwrap();

    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].kind, TokenType::Object);
    assert_eq!(tokens[0].size, 1);
    assert_eq!(tokens[1].size, 1);
    assert_eq!(tokens[2].kind, TokenType::Array);
    assert_eq!(tokens[2].size, 2);
}

#[test]
fn test_object_properties() {
    let tokens = tokenize(r#"{"name":"Alice","age":21}"#).unwrap();

    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].size, 2);
    assert_eq!(tokens[1].size, 1);
    assert_eq!(tokens[2].size, 0);
    assert_eq!(tokens[3].size, 1);
    assert_eq!(tokens[4].size, 0);
}

#[test]
fn test_nested_objects() {
    let tokens = tokenize(r#"{"user":{"name":"Alice"}}"#).unwrap();

    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].size, 1);
    assert_eq!(tokens[1].size, 1);
    assert_eq!(tokens[2].size, 1);
    assert_eq!(tokens[3].size, 1);
    assert_eq!(tokens[4].size, 0);
}

#[test]
fn test_array_of_objects() {
    let tokens = tokenize(r#"[{"id":1},{"id":2}]"#).unwrap();

    assert_eq!(tokens.len(), 7);
    assert_eq!(tokens[0].size, 2);
    assert_eq!(tokens[1].size, 1);
    assert_eq!(tokens[2].size, 1);
    assert_eq!(tokens[4].size, 1);
    assert_eq!(tokens[5].size, 1);
}

#[test]
fn test_object_with_multiple_nested_values() {
    let tokens = tokenize(r#"{"a":[1,2],"b":{"c":3}}"#).unwrap();

    assert_eq!(tokens[0].size, 2);
    assert_eq!(tokens[1].size, 1);
    assert_eq!(tokens[2].size, 2);
    assert_eq!(tokens[5].size, 1);
    assert_eq!(tokens[6].size, 1);
}
