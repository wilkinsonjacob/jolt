#[derive(Debug, PartialEq)]
pub enum TokenizeError {
    Invalid,
    Incomplete,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Object,
    Array,
    String,
    Primitive,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenType,
    pub start: usize,
    pub end: usize,
    pub size: usize,
}

fn validate_number(value: &str) -> Result<(), TokenizeError> {
    if value.is_empty() {
        return Err(TokenizeError::Invalid);
    }

    let bytes = value.as_bytes();
    let mut i = 0;

    if bytes[i] == b'-' {
        i += 1;
    }

    if i >= bytes.len() {
        return Err(TokenizeError::Invalid);
    }

    if bytes[i] == b'0' {
        i += 1;
    } else if (b'1'..=b'9').contains(&bytes[i]) {
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    } else {
        return Err(TokenizeError::Invalid);
    }

    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;

        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return Err(TokenizeError::Invalid);
        }

        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }

    if i < bytes.len() && matches!(bytes[i], b'e' | b'E') {
        i += 1;

        if i < bytes.len() && matches!(bytes[i], b'+' | b'-') {
            i += 1;
        }

        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return Err(TokenizeError::Invalid);
        }

        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }

    if i != bytes.len() {
        return Err(TokenizeError::Invalid);
    }

    Ok(())
}

fn validate_primitive(value: &str) -> Result<(), TokenizeError> {
    match value {
        "true" | "false" | "null" => Ok(()),
        _ => validate_number(value),
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError> {
    let bytes = input.as_bytes();
    let mut tokens: Vec<Token> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                let start = i + 1;
                i += 1;

                while i < bytes.len() {
                    match bytes[i] {
                        b'"' => break,
                        b'\\' => {
                            if i + 1 >= bytes.len() {
                                return Err(TokenizeError::Incomplete);
                            }

                            match bytes[i + 1] {
                                b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                                    i += 2;
                                }
                                b'u' => {
                                    if i + 6 > bytes.len() {
                                        return Err(TokenizeError::Incomplete);
                                    }

                                    for byte in &bytes[i + 2..i + 6] {
                                        if !byte.is_ascii_hexdigit() {
                                            return Err(TokenizeError::Invalid);
                                        }
                                    }
                                    i += 6;
                                }
                                _ => return Err(TokenizeError::Invalid),
                            }
                        }
                        byte if byte < 0x20 => {
                            return Err(TokenizeError::Invalid);
                        }
                        _ => i += 1,
                    }
                }

                if i >= bytes.len() {
                    return Err(TokenizeError::Incomplete);
                }

                if let Some(&index) = stack.last() {
                    tokens[index].size += 1;
                }

                tokens.push(Token {
                    kind: TokenType::String,
                    start,
                    end: i,
                    size: 0,
                });

                i += 1;
            }
            b'[' => {
                if let Some(&index) = stack.last() {
                    tokens[index].size += 1;
                }

                stack.push(tokens.len());
                tokens.push(Token {
                    kind: TokenType::Array,
                    start: i,
                    end: 0,
                    size: 0,
                });

                i += 1;
            }
            b']' => {
                let index = stack.pop().ok_or(TokenizeError::Invalid)?;

                if tokens[index].kind != TokenType::Array {
                    return Err(TokenizeError::Invalid);
                }

                tokens[index].end = i + 1;
                i += 1;
            }
            b'{' => {
                let token = Token {
                    kind: TokenType::Object,
                    start: i,
                    end: 0,
                    size: 0,
                };

                if let Some(&index) = stack.last() {
                    tokens[index].size += 1;
                }

                stack.push(tokens.len());
                tokens.push(token);
                i += 1;
            }
            b'}' => {
                if let Some(&index) = stack.last()
                    && tokens[index].kind == TokenType::String
                {
                    stack.pop();
                }

                let index = stack.pop().ok_or(TokenizeError::Invalid)?;

                if tokens[index].kind != TokenType::Object {
                    return Err(TokenizeError::Invalid);
                }

                tokens[index].end = i + 1;
                i += 1;
            }
            b':' => {
                let index = tokens.len().checked_sub(1).ok_or(TokenizeError::Invalid)?;

                stack.push(index);
                i += 1;
            }
            b',' => {
                if let Some(&index) = stack.last()
                    && tokens[index].kind != TokenType::Array
                    && tokens[index].kind != TokenType::Object
                {
                    stack.pop();
                }

                i += 1;
            }
            byte if byte.is_ascii_whitespace() => i += 1,
            _ => {
                let start = i;

                while i < bytes.len()
                    && !bytes[i].is_ascii_whitespace()
                    && !matches!(bytes[i], b',' | b':' | b'[' | b']' | b'{' | b'}')
                {
                    i += 1;
                }

                validate_primitive(&input[start..i])?;

                if let Some(&index) = stack.last() {
                    tokens[index].size += 1;
                }

                tokens.push(Token {
                    kind: TokenType::Primitive,
                    start,
                    end: i,
                    size: 0,
                });
            }
        }
    }

    if !stack.is_empty() {
        return Err(TokenizeError::Incomplete);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_valid_primitives() {
        assert!(validate_primitive("true").is_ok());
        assert!(validate_primitive("false").is_ok());
        assert!(validate_primitive("null").is_ok());
    }

    #[test]
    fn test_invalid_primitives() {
        assert!(validate_primitive("hello").is_err());
        assert!(validate_primitive("True").is_err());
        assert!(validate_primitive("undefined").is_err());
    }

    #[test]
    fn test_valid_integers() {
        for value in ["0", "42", "-42", "123456"] {
            assert!(validate_number(value).is_ok(), "{value}");
        }
    }

    #[test]
    fn test_invalid_integers() {
        for value in ["", "-", "01", "-01", "+3", "12abc"] {
            assert!(validate_number(value).is_err(), "{value}");
        }
    }

    #[test]
    fn test_valid_decimals() {
        for value in ["3.14", "0.5", "-12.75"] {
            assert!(validate_number(value).is_ok(), "{value}");
        }
    }

    #[test]
    fn test_invalid_decimals() {
        for value in ["1.", ".5", "1.2.3", "01.5"] {
            assert!(validate_number(value).is_err(), "{value}");
        }
    }

    #[test]
    fn test_valid_exponents() {
        for value in ["1e10", "1E10", "2.5e-3", "-3E+8", "0e0"] {
            assert!(validate_number(value).is_ok(), "{value}");
        }
    }

    #[test]
    fn test_invalid_exponents() {
        for value in ["1e", "1e+", "1e-", "1e2.3", "1e++2"] {
            assert!(validate_number(value).is_err(), "{value}");
        }
    }

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
    fn test_unexpected_closing_bracket() {
        assert!(tokenize("]").is_err());
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
    fn test_mismatched_brackets() {
        assert!(tokenize("{]").is_err());
        assert!(tokenize("[}").is_err());
        assert!(tokenize("{[}]").is_err());
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
}
