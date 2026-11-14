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

fn validate_number(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("Invalid number".to_string());
    }

    let bytes = value.as_bytes();
    let mut i = 0;

    if bytes[i] == b'-' {
        i += 1;
    }

    if i >= bytes.len() {
        return Err("Missing digits after minus sign".to_string());
    }

    if bytes[i] == b'0' {
        i += 1;
    } else if (b'1'..=b'9').contains(&bytes[i]) {
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    } else {
        return Err("Invalid number".to_string());
    }

    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;

        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return Err("Missing digits after decimal point".to_string());
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
            return Err("Missing exponent digits".to_string());
        }

        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }

    if i != bytes.len() {
        return Err("Invalid characters after number".to_string());
    }

    Ok(())
}

fn validate_primitive(value: &str) -> Result<(), String> {
    match value {
        "true" | "false" | "null" => Ok(()),
        _ => validate_number(value),
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let bytes = input.as_bytes();

    let mut i: usize = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            i += 1;

            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    if i + 1 >= bytes.len() {
                        return Err("Incomplete escape sequence".to_string());
                    }
                    match bytes[i + 1] {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                            i += 2;
                        }
                        b'u' => {
                            if i + 6 > bytes.len() {
                                return Err("Incomplete Unicode escape".to_string());
                            }

                            for byte in &bytes[i + 2..i + 6] {
                                if !byte.is_ascii_hexdigit() {
                                    return Err("Invalid Unicode escape sequence".to_string());
                                }
                            }
                            i += 6;
                        }
                        _ => {
                            return Err("Invalid escape sequence".to_string());
                        }
                    }
                } else if bytes[i] == b'"' {
                    break;
                } else {
                    if bytes[i] < 0x20 {
                        return Err("Unescaped control character in JSON string".to_string());
                    }
                    i += 1;
                }
            }

            if i >= bytes.len() {
                return Err("Unterminated JSON string".to_string());
            }

            let end = i;
            let token = Token {
                kind: TokenType::String,
                start,
                end,
                size: 0,
            };
            tokens.push(token);
        } else if bytes[i].is_ascii_whitespace()
            || matches!(bytes[i], b',' | b':' | b'[' | b']' | b'{' | b'}')
        {
        } else {
            let start = i;
            while i < bytes.len()
                && !bytes[i].is_ascii_whitespace()
                && !matches!(bytes[i], b',' | b':' | b'[' | b']' | b'{' | b'}')
            {
                i += 1;
            }
            let end = i;

            validate_primitive(&input[start..end])?;

            let token = Token {
                kind: TokenType::Primitive,
                start,
                end,
                size: 0,
            };
            tokens.push(token);
        }
        i += 1;
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

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].start, 1);
        assert_eq!(tokens[0].end, 5);
        assert_eq!(tokens[1].start, 6);
        assert_eq!(tokens[1].end, 11);
        assert_eq!(tokens[2].start, 12);
        assert_eq!(tokens[2].end, 14);
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
}
