//! A minimal, jsmn-inspired JSON tokenizer.
//!
//! `jolt` splits JSON input into tokens describing strings, primitives, arrays,
//! and objects. Tokens reference byte ranges in the original input rather than
//! allocating separate strings for their values.
//!
//! # Example
//!
//! ```
//! use jolt::{tokenize, TokenType};
//!
//! let json = r#"{"name":"Alice"}"#;
//! let tokens = tokenize(json).unwrap();
//!
//! assert_eq!(tokens[0].kind, TokenType::Object);
//! assert_eq!(tokens[0].size, 1);
//! assert_eq!(&json[tokens[1].start..tokens[1].end], "name");
//! ```
//!
//! This tokenizer validates string escapes and JSON primitives, but does not
//! currently enforce every rule of JSON document structure.

/// An error encountered while tokenizing JSON input.
#[derive(Debug, PartialEq)]
pub enum TokenizeError {
    /// The input contains an invalid token, escape, or closing delimiter.
    Invalid,
    /// The input ends before a string or container is complete.
    Incomplete,
}

/// The kind of JSON value represented by a token.
#[derive(Debug, PartialEq)]
pub enum TokenType {
    /// A JSON object (`{...}`).
    Object,
    /// A JSON array (`[...]`).
    Array,
    /// A quoted JSON string, excluding the surrounding quotation marks.
    String,
    /// A number, boolean, or `null`.
    Primitive,
}

/// A token describing a range of the original JSON input.
///
/// `start` is inclusive and `end` is exclusive. For strings, the
/// surrounding quotation marks are excluded; for arrays and objects,
/// the delimiters are included.
#[derive(Debug, PartialEq)]
pub struct Token {
    /// The token's JSON value type.
    pub kind: TokenType,
    /// Starting byte offset in the input (inclusive).
    pub start: usize,
    /// Ending byte offset in the input (exclusive).
    pub end: usize,
    /// Number of direct children, following jsmn semantics: arrays count
    /// elements, objects count keys, and object keys count their value.
    pub size: usize,
}

// Checks whether a complete primitive value follows JSON number syntax.
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

// Accepts JSON literals or a valid JSON number.
fn validate_primitive(value: &str) -> Result<(), TokenizeError> {
    match value {
        "true" | "false" | "null" => Ok(()),
        _ => validate_number(value),
    }
}

/// Tokenizes JSON text into a flat sequence of tokens.
///
/// Container tokens appear before their children. Each token's byte offsets
/// refer to `input`, allowing callers to retrieve its original text.
///
/// Returns [`TokenizeError::Invalid`] for invalid tokens or mismatched
/// delimiters and [`TokenizeError::Incomplete`] for unfinished strings or
/// containers.
///
/// # Example
///
/// ```
/// use jolt::{tokenize, TokenType};
///
/// let input = r#"[true,42]"#;
/// let tokens = tokenize(input).unwrap();
///
/// assert_eq!(tokens.len(), 3);
/// assert_eq!(tokens[0].kind, TokenType::Array);
/// assert_eq!(tokens[0].size, 2);
/// assert_eq!(&input[tokens[2].start..tokens[2].end], "42");
/// ```
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
