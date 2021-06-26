//! # png_stego
//! `png_stego` provides methods which can be used to hide arbitrary data (in form of byte arrays) inside RGB images.
//! Implemented on top of old nanoboard source code and http://blog.andersen.im/2014/11/hiding-your-bits-in-the-bytes/

mod consts;
mod converters;

use std::{convert::TryInto, num::TryFromIntError, usize};
use thiserror::Error;

use consts::*;
use converters::{bits_to_bytes, bytes_to_bits, bytes_to_i32, i32_to_bytes, BoardBitVec};
use image::RgbImage;

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

/// Allows you to hide byte data inside a provided image.
/// **Warning!** According to the hiding algorithm, one pixel of the image can store 3 bits of data.

/// If your data can't fit into image, a [`PngStegoError`] will be returned.
/// # Arguments
/// * `img` - An RGB image.
/// * `bytes` - Byte data which you need to hide

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

/// Allows you to read hidden bytes from image.
///
/// **Warning!** Since it's a steganography algorithm, there's no way to know if there's any data hidden beforehand.
/// If there's no IO related errors, the method will return random data.
/// # Arguments
/// * `encoded_img` - An image with data.
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
