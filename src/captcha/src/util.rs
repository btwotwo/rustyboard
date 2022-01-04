use std::{
    fmt::{Error, Write},
    num::ParseIntError,
};
pub fn byte_array_to_hex_string(input: &[u8]) -> Result<String, Error> {
    let mut result = String::with_capacity(input.len() * 2);
    for byte in input {
        write!(result, "{:02x}", byte)?;
    }

    Ok(result)
}

pub fn hex_string_to_byte_array(input: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..input.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&input[i..i + 2], 16))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_array_to_hex_string_converts_correctly() {
        let expected_result = "06fdfeff";
        let input = vec![6, 253, 254, 255];

        assert_eq!(byte_array_to_hex_string(&input).unwrap(), expected_result)
    }

    #[test]
    fn hex_string_to_byte_array_converts_correctly() {
        let expected_result: [u8; 4] = [1, 254, 78, 93];
        let input = "01fe4e5d";

        assert_eq!(hex_string_to_byte_array(&input).unwrap(), expected_result)
    }
}
