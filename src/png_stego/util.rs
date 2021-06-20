use byteorder::{LittleEndian, WriteBytesExt};

pub fn convert_to_bytes(val: i32) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![];
    buffer.write_i32::<LittleEndian>(val)?;

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endianness_is_correct() {
        let value = convert_to_bytes(1000).unwrap();
        assert_eq!(value, vec![232, 3, 0, 0])
    }
}
