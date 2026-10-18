//! TGA images, as the game stores its sky boxes
//!
//! Uncompressed and run-length truecolour and greyscale images of 8, 24 or
//! 32 bits a pixel are read, rows from either corner. Colour-mapped images,
//! which the game's skies do not use, are refused. Sizes are checked
//! before anything is allocated

use crate::{AssetError, Limits};

/// An image in 8-bit RGBA, rows from the top
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

const HEADER: usize = 18;
/// Bit 5 of the descriptor puts the first row at the top
const TOP_FIRST: u8 = 0x20;

fn invalid(reason: impl Into<String>) -> AssetError {
    AssetError::Image(reason.into())
}

pub fn decode(bytes: &[u8], limits: Limits) -> Result<Image, AssetError> {
    let header = bytes
        .get(..HEADER)
        .ok_or_else(|| invalid("shorter than a TGA header"))?;
    let id_length = usize::from(header[0]);
    let map_type = header[1];
    let image_type = header[2];
    let map_length = usize::from(u16::from_le_bytes([header[5], header[6]]));
    let map_bits = usize::from(header[7]);
    let width = u32::from(u16::from_le_bytes([header[12], header[13]]));
    let height = u32::from(u16::from_le_bytes([header[14], header[15]]));
    let bits = header[16];
    let descriptor = header[17];

    let (run_length, grey) = match image_type {
        2 => (false, false),
        3 => (false, true),
        10 => (true, false),
        11 => (true, true),
        other => {
            return Err(invalid(format!(
                "image type {other} is not truecolour or greyscale"
            )));
        }
    };
    let pixel = match (grey, bits) {
        (true, 8) => 1,
        (false, 24) => 3,
        (false, 32) => 4,
        _ => return Err(invalid(format!("{bits} bits a pixel are not read"))),
    };
    let pixels = u64::from(width) * u64::from(height);
    if pixels == 0 {
        return Err(invalid("the image is empty"));
    }
    if pixels * 4 > limits.max_file {
        return Err(AssetError::TooLarge {
            path: "TGA image".into(),
            size: pixels * 4,
            limit: limits.max_file,
        });
    }

    // The colour map, when present, comes before the pixels
    let map_bytes = if map_type == 1 {
        map_length * map_bits.div_ceil(8)
    } else {
        0
    };
    let mut data = bytes
        .get(HEADER + id_length + map_bytes..)
        .ok_or_else(|| invalid("the pixels are missing"))?;

    let count = pixels as usize;
    let mut stored = Vec::with_capacity(count * pixel);
    if run_length {
        while stored.len() < count * pixel {
            let (&packet, rest) = data
                .split_first()
                .ok_or_else(|| invalid("the pixels end early"))?;
            let repeat = usize::from(packet & 0x7f) + 1;
            if packet & 0x80 != 0 {
                let value = rest
                    .get(..pixel)
                    .ok_or_else(|| invalid("the pixels end early"))?;
                for _ in 0..repeat {
                    stored.extend_from_slice(value);
                }
                data = &rest[pixel..];
            } else {
                let values = rest
                    .get(..repeat * pixel)
                    .ok_or_else(|| invalid("the pixels end early"))?;
                stored.extend_from_slice(values);
                data = &rest[repeat * pixel..];
            }
        }
        stored.truncate(count * pixel);
    } else {
        let values = data
            .get(..count * pixel)
            .ok_or_else(|| invalid("the pixels end early"))?;
        stored.extend_from_slice(values);
    }

    let mut rgba = vec![0u8; count * 4];
    let row = width as usize;
    for (i, value) in stored.chunks_exact(pixel).enumerate() {
        let (x, y) = (i % row, i / row);
        let y = if descriptor & TOP_FIRST != 0 {
            y
        } else {
            height as usize - 1 - y
        };
        let at = (y * row + x) * 4;
        let colour = match value {
            [g] => [*g, *g, *g, 255],
            [b, g, r] => [*r, *g, *b, 255],
            [b, g, r, a] => [*r, *g, *b, *a],
            _ => unreachable!("pixels are 1, 3 or 4 bytes"),
        };
        rgba[at..at + 4].copy_from_slice(&colour);
    }
    Ok(Image {
        width,
        height,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(image_type: u8, width: u16, height: u16, bits: u8, descriptor: u8) -> Vec<u8> {
        let mut h = vec![0u8; HEADER];
        h[2] = image_type;
        h[12..14].copy_from_slice(&width.to_le_bytes());
        h[14..16].copy_from_slice(&height.to_le_bytes());
        h[16] = bits;
        h[17] = descriptor;
        h
    }

    #[test]
    fn rows_from_the_bottom_come_out_from_the_top() {
        // Two by two, bottom row blue then green, top row red then white
        let mut bytes = header(2, 2, 2, 24, 0);
        bytes.extend([255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]);
        let image = decode(&bytes, Limits::default()).unwrap();
        assert_eq!(
            image.rgba,
            [
                255, 0, 0, 255, 255, 255, 255, 255, 0, 0, 255, 255, 0, 255, 0, 255
            ]
        );
    }

    #[test]
    fn run_length_packets_expand() {
        // A run of three grey 7s and one raw 9, first row at the top
        let mut bytes = header(11, 4, 1, 8, TOP_FIRST);
        bytes.extend([0x82, 7, 0x00, 9]);
        let image = decode(&bytes, Limits::default()).unwrap();
        assert_eq!(
            image.rgba.chunks(4).map(|p| p[0]).collect::<Vec<_>>(),
            [7, 7, 7, 9]
        );

        let mut short = header(10, 4, 1, 24, 0);
        short.extend([0x83, 1, 2]);
        assert!(decode(&short, Limits::default()).is_err());
        assert!(decode(&header(1, 4, 1, 8, 0), Limits::default()).is_err());
    }
}
