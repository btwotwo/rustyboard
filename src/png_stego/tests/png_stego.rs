use std::convert::TryInto;

use image::RgbImage;
use png_stego::{hide_bytes, read_hidden_bytes};

#[test]
fn encoded_data_can_be_decoded() {
    let mock_img = RgbImage::new(10, 10);
    let bytes_to_hide = 0xDEADBEEFu32.to_le_bytes();

    let img_with_data = hide_bytes(mock_img, bytes_to_hide.into()).unwrap();
    let decoded_bytes = read_hidden_bytes(img_with_data).unwrap();

    assert_eq!(
        u32::from_le_bytes(decoded_bytes.try_into().unwrap()),
        0xDEADBEEF
    )
}
