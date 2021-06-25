use std::io::{Cursor, Result};

use bitvec::prelude::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub type BoardBitVec = BitVec<Lsb0, u8>;
pub type BoardBitSlice = BitSlice<Lsb0, u8>;
pub type BoardBitBox = BitBox<Lsb0, u8>;

pub fn i32_to_bytes(val: i32) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![];

    buffer.write_i32::<LittleEndian>(val)?;

    Ok(buffer)
}

pub fn bytes_to_i32(val: Vec<u8>) -> Result<i32> {
    let mut reader = Cursor::new(val);

    reader.read_i32::<LittleEndian>()
}

pub fn bytes_to_bits(bytes: &[u8]) -> &BoardBitSlice {
    bytes.view_bits()
}

pub fn bits_to_bytes(bits: BoardBitVec) -> Vec<u8> {
    bits.into_vec()
}

// These tests are mostly used for checking that the behavior is the same as in the C# version
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn i32_to_bytes_endianness_is_correct() {
        let bytes = i32_to_bytes(1000).unwrap();
        assert_eq!(bytes, vec![232, 3, 0, 0])
    }

    #[test]
    fn bytes_to_i32_endianness_is_correct() {
        let i32 = bytes_to_i32(vec![111, 23, 0, 0]).unwrap();
        assert_eq!(i32, 5999);
    }

    #[test]
    fn bit_conversion_same_as_in_prev_version() {
        let expected = bitvec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
        let bytes = vec![1, 2];
        let bits = bytes_to_bits(&bytes);

        assert_eq!(expected, bits)

        // BitArray(16) { true, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false }
    }
}
