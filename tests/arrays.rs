use jolt::{TokenType, tokenize};

#[test]
fn test_array_token() {
    let tokens = tokenize("[true,false,42]").unwrap();

    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenType::Array);
    assert_eq!(tokens[0].start, 0);
}

#[test]
fn test_array_end() {
    let tokens = tokenize("[true,false,42]").unwrap();

    assert_eq!(tokens[0].kind, TokenType::Array);
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 15);
}

#[test]
fn test_nested_arrays() {
    let tokens = tokenize("[[1,2],[3,4]]").unwrap();

    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 13);

    assert_eq!(tokens[1].start, 1);
    assert_eq!(tokens[1].end, 6);

    assert_eq!(tokens[4].start, 7);
    assert_eq!(tokens[4].end, 12);
}

#[test]
fn test_array_size() {
    let tokens = tokenize("[10,20,30]").unwrap();

    assert_eq!(tokens[0].size, 3);
}

#[test]
fn test_nested_array_sizes() {
    let tokens = tokenize("[[1,2],[3,4]]").unwrap();

    assert_eq!(tokens[0].size, 2);
    assert_eq!(tokens[1].size, 2);
    assert_eq!(tokens[4].size, 2);
}

#[test]
fn test_empty_array_size() {
    let tokens = tokenize("[]").unwrap();

    assert_eq!(tokens[0].size, 0);
}

#[test]
fn test_string_array_size() {
    let tokens = tokenize(r#"["hello","world"]"#).unwrap();

    assert_eq!(tokens[0].size, 2);
}
