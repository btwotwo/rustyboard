mod util;

use std::convert::TryInto;
use std::num::TryFromIntError;
use thiserror::Error;

use image::RgbImage;
use util::convert_to_bytes;
const COLORS_COUNT: u32 = 3;
const BYTES_IN_I32: u32 = 4;
#[derive(Debug, Error)]
pub enum PngStegoError {
    #[error("Data you are trying to encode is too large")]
    BufferTooBig {
        #[from]
        source: TryFromIntError,
    },

    #[error("IO Error")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Data you are trying to encode would not fit into image")]
    BufferBiggerThanImage
}

pub type PngStegoResult<T> = Result<T, PngStegoError>;

pub fn hide_bytes(empty_img: &RgbImage, bytes: Vec<u8>) -> PngStegoResult<RgbImage> {
    let max_size = empty_img.width() * empty_img.height() * COLORS_COUNT as u32;
    let data_size = (bytes.len() as u32 + BYTES_IN_I32) * 8;
    if max_size < data_size {
        return Err(PngStegoError::BufferBiggerThanImage)
    } 
    
    let combined_bytes = combine_length_and_bytes(bytes)?;

    unimplemented!()
}

pub fn read_hidden_bytes(encoded_img: RgbImage) -> Vec<u8> {
    unimplemented!()
}

fn combine_length_and_bytes(bytes: Vec<u8>) -> PngStegoResult<Vec<u8>> {
    let data_length: i32 = bytes.len().try_into()?;
    let length_bytes = convert_to_bytes(data_length)?;

    let mut combined_bytes_buffer = Vec::with_capacity(bytes.len() + length_bytes.len());

    combined_bytes_buffer.extend_from_slice(&length_bytes);
    combined_bytes_buffer.extend_from_slice(&bytes);

    Ok(combined_bytes_buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_length_combines_correctly() {
        let bytes = vec![1, 2, 3, 4];
        let combined = combine_length_and_bytes(bytes).unwrap();

        assert_eq!(combined, vec![4, 0, 0, 0, 1, 2, 3, 4])
    }

    #[test]
    /// Each pixel of image can fit 3 bits of data.
    fn hide_bytes_returns_error_when_cannot_fit_into_image() {
        let width = 10;
        let height = 10;
        let max_bits: usize = width * height * 3; // 100 pixels in image, 3 bits per pix

        let mock_img = RgbImage::new(width.try_into().unwrap(), height.try_into().unwrap());
        let big_data = vec![0; max_bits + 8];

        assert_eq!(big_data.len(), max_bits + 8);

        let result = hide_bytes(&mock_img, big_data).expect_err("Expected error!");

        assert!(matches!(result, PngStegoError::BufferBiggerThanImage))
    }
}
