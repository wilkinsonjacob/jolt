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

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let bytes = input.as_bytes();

    let mut i: usize = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            i += 1;

            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                } else if bytes[i] == b'"' {
                    break;
                } else {
                    i += 1;
                }
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
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_token() {
        let input = r#""hello""#;
        let tokens = tokenize(input);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenType::String);
        assert_eq!(tokens[0].start, 1);
        assert_eq!(tokens[0].end, 6);
    }

    #[test]
    fn test_multiple_strings() {
        let tokens = tokenize(r#""hello" "world""#);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].start, 1);
        assert_eq!(tokens[0].end, 6);
        assert_eq!(tokens[1].start, 9);
        assert_eq!(tokens[1].end, 14);
    }
}
