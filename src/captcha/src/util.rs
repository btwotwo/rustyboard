use std::fmt::{Error, Write};
pub fn byte_array_to_string(input: &[u8]) -> Result<String, Error> {
    let mut result = String::with_capacity(input.len() * 2);
    for byte in input {
        write!(result, "{:02x}", byte)?;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_correctly() {
        let expected_result = "06fdfeff";
        let input = vec![6, 253, 254, 255];

        assert_eq!(byte_array_to_string(&input).unwrap(), expected_result)
    }
}
