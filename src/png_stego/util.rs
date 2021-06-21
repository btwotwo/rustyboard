use bitvec::prelude::*;
use byteorder::{LittleEndian, WriteBytesExt};

pub fn convert_length_to_bytes(val: i32) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![];

    buffer.write_i32::<LittleEndian>(val)?;

    Ok(buffer)
}

pub fn convert_bytes_to_bits(bytes: &[u8]) -> &BitSlice<Lsb0, u8> {
    bytes.view_bits()
}

// These tests are mostly used for checking that the behavior is the same as in the C# version
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endianness_is_correct() {
        let value = convert_length_to_bytes(1000).unwrap();
        assert_eq!(value, vec![232, 3, 0, 0])
    }

    #[test]
    fn bit_conversion_same_as_in_prev_version() {
        let expected = bitvec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
        let bytes = vec![1, 2];
        let bits = convert_bytes_to_bits(&bytes);

        assert_eq!(expected, bits)

        // BitArray(16) { true, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false }
    }
}
