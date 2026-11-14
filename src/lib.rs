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

                            for j in i + 2..i + 6 {
                                if !bytes[j].is_ascii_hexdigit() {
                                    return Err("Invalid Unicode escape sequence?".to_string());
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
}
