mod util;

use std::{convert::TryInto, num::TryFromIntError, usize};
use thiserror::Error;

use image::RgbImage;
use util::{BoardBitVec, bits_to_bytes, bytes_to_bits, bytes_to_i32, i32_to_bytes};

const COLORS_COUNT: u32 = 3;
const BYTES_IN_I32: u32 = 4;
const BITS_IN_BYTES: u32 = 8;
const LENGTH_BITS: u32 = BYTES_IN_I32 * BITS_IN_BYTES;

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
    BufferBiggerThanImage,
}

pub type PngStegoResult<T> = Result<T, PngStegoError>;

pub fn hide_bytes(mut img: RgbImage, bytes: Vec<u8>) -> PngStegoResult<RgbImage> {
    let max_size = img.width() * img.height() * COLORS_COUNT;
    let data_size = (bytes.len() as u32 + BYTES_IN_I32) * 8;

    if max_size < data_size {
        return Err(PngStegoError::BufferBiggerThanImage);
    }

    let combined_bytes = combine_length_and_bytes(bytes)?;
    let bits = bytes_to_bits(&combined_bytes);
    let bits_length = bits.len();

    let pixels = img
        .pixels_mut()
        .map(|p| &mut p.0)
        .flatten()
        .enumerate()
        .take_while(|(i, _)| i < &bits_length);

    for (i, pixel) in pixels {
        let even_pix = *pixel - (*pixel % 2);
        *pixel = even_pix + if bits[i] { 1 } else { 0 };
    }

    Ok(img)
}

pub fn read_hidden_bytes(encoded_img: RgbImage) -> PngStegoResult<Vec<u8>> {
    let pixels: Vec<&u8> = encoded_img.pixels().map(|p| &p.0).flatten().collect();
    let encoded_data_length = get_encoded_data_length(&pixels)?;
    let encoded_data_bits = pixels
        .iter()
        .skip(LENGTH_BITS as usize)
        .take(encoded_data_length as usize * BITS_IN_BYTES as usize)
        .map(pixel_component_to_bit)
        .collect();
    let encoded_data_bytes = bits_to_bytes(encoded_data_bits);

    Ok(encoded_data_bytes)
}

fn pixel_component_to_bit(component: &&u8) -> bool {
    *component % 2 == 1
}

fn get_encoded_data_length(pixels: &[&u8]) -> PngStegoResult<i32> {
    let length_bits: BoardBitVec = pixels
        .iter()
        .take(LENGTH_BITS as usize)
        .map(pixel_component_to_bit)
        .collect();
    let length_bytes = bits_to_bytes(length_bits);

    Ok(bytes_to_i32(length_bytes)?)
}

fn combine_length_and_bytes(bytes: Vec<u8>) -> PngStegoResult<Vec<u8>> {
    let data_length: i32 = bytes.len().try_into()?;
    let length_bytes = i32_to_bytes(data_length)?;

    let mut combined_bytes_buffer = Vec::with_capacity(bytes.len() + length_bytes.len());

    combined_bytes_buffer.extend_from_slice(&length_bytes);
    combined_bytes_buffer.extend_from_slice(&bytes);

    Ok(combined_bytes_buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Pixel;

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

        let result = hide_bytes(mock_img, big_data).expect_err("Expected error!");

        assert!(matches!(result, PngStegoError::BufferBiggerThanImage))
    }

    #[test]
    // This test doesn't check if the image is modified correctly. It only checks if it's modified at all ;)
    fn encode_bytes_in_image_modifies_image() {
        let mut mock_img = RgbImage::new(10, 10);
        let pixel = Pixel::from_slice(&[1, 2, 3]);

        mock_img.put_pixel(0, 0, *pixel);

        let bytes_to_hide = vec![1, 2];

        let updated_image = hide_bytes(mock_img, bytes_to_hide).unwrap();
        let updated_pixel = updated_image.get_pixel(0, 0);

        assert_ne!(pixel, updated_pixel)
    }
}