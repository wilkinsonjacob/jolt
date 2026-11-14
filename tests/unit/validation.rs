use super::*;

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
