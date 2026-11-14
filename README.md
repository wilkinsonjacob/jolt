# jolt

**jolt** is a lightweight JSON tokenizer written in Rust, inspired by [jsmn](https://github.com/zserge/jsmn).

Instead of deserializing JSON into structs or building an intermediate tree of values, jolt breaks JSON input into a flat sequence of tokens. Each token describes a portion of the original input using its type, byte offsets, and number of direct children.

The result is a small, straightforward API for inspecting JSON without copying its values into separate strings.

## Philosophy

JSON parsing doesn't always need to involve complex data structures, deserialization frameworks, or a large API.

Sometimes, all you need is to know where values begin, where they end, and how they relate to one another.

jolt follows three principles:

- **Simplicity** — Keep the implementation understandable and the public API small.
- **Minimalism** — Represent JSON using token boundaries rather than allocating separate objects for every value.
- **Predictability** — Use explicit token types, byte offsets, and structured errors.

jolt is inspired by jsmn's token-based design, adapted to Rust's type system and error handling.

## Features

- Written entirely in Rust
- No external dependencies
- Token-based JSON representation
- Support for nested objects and arrays
- JSON string and escape-sequence handling
- Validation of numbers, booleans, and `null`
- Byte offsets into the original input
- Structured error handling
- Unit and integration tests

## Design

The fundamental building block of jolt is a **token**.

Consider the following JSON:

```json
{
  "name": "Jack",
  "age": 27
}
```

jolt produces five tokens:

| Type | Value | Size |
|---|---|---|
| Object | Entire JSON object | 2 |
| String | `name` | 1 |
| String | `Jack` | 0 |
| String | `age` | 1 |
| Primitive | `27` | 0 |

Rather than storing the actual values, tokens contain offsets pointing into the original JSON string.

Each token has four fields:

```rust
pub struct Token {
    pub kind: TokenType,
    pub start: usize,
    pub end: usize,
    pub size: usize,
}
```

- `kind` identifies the token type.
- `start` is the starting byte offset.
- `end` is the exclusive ending byte offset.
- `size` represents the number of direct child tokens, or key-value pairs for objects.

For strings, the offsets exclude the surrounding quotation marks. For objects and arrays, the offsets include their opening and closing delimiters.

This representation allows values to be retrieved directly from the original input.

## Usage

Add jolt as a dependency using a local path or Git repository:

```toml
[dependencies]
jolt = { git = "https://github.com/wilkinsonjacob/jolt" }
```

Then tokenize a JSON string:

```rust
use jolt::{tokenize, TokenType};

fn main() {
    let json = r#"{"name":"Jack","age":27}"#;

    let tokens = tokenize(json).unwrap();

    for token in &tokens {
        println!("{:?}", token);
    }

    assert_eq!(tokens[0].kind, TokenType::Object);
    assert_eq!(tokens[0].size, 2);
}
```

### Accessing values

Tokens store byte offsets rather than owning the JSON values.

```rust
use jolt::tokenize;

fn main() {
    let json = r#"{"name":"Jack"}"#;
    let tokens = tokenize(json).unwrap();

    let name = &json[tokens[2].start..tokens[2].end];

    assert_eq!(name, "Jack");
}
```

This retrieves the value directly from the original input without allocating a new string.

Note that string tokens retain JSON escape sequences rather than decoding them.

## API

jolt exposes a small set of public types and a single tokenization function.

### Token types

```rust
pub enum TokenType {
    Object,
    Array,
    String,
    Primitive,
}
```

| Type | Description |
|---|---|
| `Object` | A collection of key-value pairs |
| `Array` | An ordered sequence of values |
| `String` | A quoted JSON string |
| `Primitive` | A number, boolean, or `null` |

Numbers, booleans, and `null` share the `Primitive` type. Their individual values can be determined by examining the corresponding input slice.

### Tokenization

```rust
pub fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError>
```

The `tokenize()` function scans the provided JSON input and returns a vector of tokens in the order they are encountered.

On success, it returns `Ok(Vec<Token>)`. If tokenization fails, it returns a `TokenizeError`.

### Error handling

jolt defines two error variants:

```rust
pub enum TokenizeError {
    Invalid,
    Incomplete,
}
```

| Error | Description |
|---|---|
| `Invalid` | Invalid syntax, primitive, escape sequence, or mismatched delimiter |
| `Incomplete` | Input ends before a string or container is complete |

Example:

```rust
use jolt::{tokenize, TokenizeError};

fn main() {
    assert_eq!(
        tokenize("[1,2"),
        Err(TokenizeError::Incomplete)
    );

    assert_eq!(
        tokenize("[}"),
        Err(TokenizeError::Invalid)
    );
}
```

## Limitations

jolt is currently an early-stage tokenizer rather than a full JSON deserialization library.

- It does not deserialize JSON into Rust structs or values.
- It does not provide automatic JSON string unescaping.
- It does not currently support incremental or streaming tokenization.
- It allocates a vector to store the resulting tokens.
- It does not yet enforce every rule of the JSON grammar, including some structural constraints.

The API and behavior may change as the library develops.

## Development

Clone the repository and run the test suite:

```bash
git clone https://github.com/wilkinsonjacob/jolt.git
cd jolt
cargo test
```

Check formatting and run Clippy:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Generate the API documentation:

```bash
cargo doc --no-deps --open
```

## License

See the repository's `LICENSE` file for licensing information.
