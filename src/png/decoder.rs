use anyhow::{bail, ensure, Result};
use flate2::read::ZlibDecoder;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Read;

use crate::{
    eof,
    png::{
        crc32::compute_crc,
        grammar::{Chunk, ColorType, ImageHeader, Png},
    },
    read,
    util::read_bytes::{U32_BYTES, U8_BYTES},
};

use crate::png::scanline_reader::ScanlineReader;
#[cfg(feature = "time")]
use crate::util::{log_event, Event};
#[cfg(feature = "time")]
use std::time::Instant;

#[derive(Debug)]
pub struct PngDecoder<'a> {
    cursor: usize,
    data: &'a [u8],
}

impl<'a> PngDecoder<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { cursor: 0, data }
    }

    pub fn decode(&mut self) -> Result<Png> {
        ensure!(
            self.read_slice(8)? == b"\x89PNG\r\n\x1A\n",
            "Invalid PNG file: incorrect signature.",
        );

        #[cfg(feature = "time")]
        let a = Instant::now();
        let chunks = self.parse_chunks()?;
        #[cfg(feature = "time")]
        log_event("", Event::ParseChunks, Some(a.elapsed()));

        #[cfg(feature = "time")]
        let b = Instant::now();

        let mut chunks = chunks.into_iter();

        let Some(Chunk::ImageHeader(image_header)) = chunks.next() else {
            bail!("Expected image header chunk.");
        };

        ensure!(
            image_header.compression_method == 0,
            "Compression method should always be 0"
        );

        let mut chunks = chunks.peekable();

        // There may be multiple image data chunks. If so, they shall appear
        // consecutively with no intervening chunks. The compressed stream is then
        // the concatenation of the contents of all image data chunks.
        let mut compressed_stream = Vec::new();

        let mut gamma = 0;

        while let Some(chunk) = chunks.peek() {
            // todo, how would you collect palettes if ColorType::Palette?
            // todo, how do you collect ancillary chunks?
            if let &Chunk::Gamma(g) = chunk {
                gamma = g;
            }

            if let &Chunk::ImageData(sub_data) = chunk {
                compressed_stream.extend_from_slice(sub_data);
            }

            chunks.next();
        }

        #[cfg(feature = "time")]
        log_event("", Event::CollectImageChunks, Some(b.elapsed()));

        #[cfg(feature = "time")]
        let c = Instant::now();

        let mut zlib_decoder = ZlibDecoder::new(&compressed_stream[..]);
        let mut input_buffer = Vec::new();
        zlib_decoder.read_to_end(&mut input_buffer)?;

        #[cfg(feature = "time")]
        log_event("", Event::FlateDecompress, Some(c.elapsed()));

        // filter
        ensure!(
            image_header.filter_method == 0,
            "Only filter method 0 is defined in the standard."
        );

        ensure!(!input_buffer.is_empty(), "Input buffer is empty.");

        #[cfg(feature = "time")]
        let d = Instant::now();

        let scanline_reader = ScanlineReader::new(&input_buffer, &image_header);
        let pixel_buffer = scanline_reader.read_lines()?;

        #[cfg(feature = "time")]
        log_event("", Event::RowFilters, Some(d.elapsed()));

        Ok(Png {
            width: image_header.width,
            height: image_header.height,
            gamma,
            color_type: image_header.color_type,
            pixel_buffer,
        })
    }

    fn validate_crc(&self, chunk_type: &'a [u8], chunk_data: &'a [u8], expected_crc: u32) -> bool {
        expected_crc == compute_crc(chunk_type, chunk_data)
    }

    fn parse_chunks(&mut self) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();

        let mut text_map = BTreeMap::new();

        loop {
            let length = self.read_u32()? as usize;

            {
                ensure!(
                    self.cursor + 4 + length < self.data.len(),
                    "EOF: Failed to read CRC value."
                );

                let expected_crc = u32::from_be_bytes(
                    self.data[self.cursor + 4 + length..self.cursor + 4 + length + 4].try_into()?,
                );

                ensure!(self.validate_crc(
                    &self.data[self.cursor..self.cursor + 4],
                    &self.data[self.cursor + 4..self.cursor + 4 + length],
                    expected_crc
                ));
            }

            let chunk = match self.read_slice(4)? {
                b"IHDR" => {
                    ensure!(chunks.is_empty(), "ImageHeader chunk must appear first.");

                    Chunk::ImageHeader(ImageHeader {
                        width: self.read_u32()?,
                        height: self.read_u32()?,
                        bit_depth: self.read_u8()?,
                        color_type: self.read_u8()?.try_into()?,
                        compression_method: self.read_u8()?,
                        filter_method: self.read_u8()?,
                        interlace_method: self.read_u8()? == 1,
                    })
                }
                b"PLTE" => {
                    ensure!(length % 3 == 0, "Chunk length not divisible by 3.");
                    ensure!(
                        !chunks.is_empty(),
                        "Empty chunks. Expected ImageHeader chunk."
                    );

                    let Chunk::ImageHeader(image_header) = &chunks[0] else {
                        bail!("Expected ImageHeader chunk.");
                    };

                    let color_type = image_header.color_type;

                    ensure!(
                        !matches!(color_type, ColorType::Grayscale)
                            && !matches!(color_type, ColorType::GrayscaleAlpha)
                    );

                    if color_type != ColorType::Palette {
                        self.skip_crc()?;
                        continue;
                    }

                    let entries = self.read_slice(length)?.chunks_exact(3);
                    Chunk::Palette(entries)
                }
                b"IDAT" => Chunk::ImageData(self.read_slice(length)?),
                b"IEND" => break,
                b"gAMA" => Chunk::Gamma(self.read_u32()?),
                // b"sRGB" => todo!("Parse srgb chunks"),
                b"tEXt" => {
                    let cursor_start = self.cursor;
                    let keyword = Cow::from(self.read_slice_until(0)?);

                    let keyword_and_null_bytes = self.cursor - cursor_start;
                    ensure!(
                        length >= keyword_and_null_bytes,
                        "Keyword bytes should not exceed chunk length."
                    );

                    let text_string = Cow::from(self.read_slice(length - keyword_and_null_bytes)?);

                    text_map.insert(keyword, text_string);
                    self.skip_crc()?;
                    continue;
                }
                _foreign => {
                    // todo! how would ancillary chunks be parsed?
                    self.cursor += length;
                    self.skip_crc()?;
                    continue;
                }
            };

            self.skip_crc()?;

            chunks.push(chunk);
        }

        if !text_map.is_empty() {
            chunks.push(Chunk::TextData(text_map));
        }

        Ok(chunks)
    }

    fn skip_crc(&mut self) -> Result<()> {
        self.eof(4)?;
        self.cursor += 4;

        Ok(())
    }

    eof!();
    read!(read_u8, u8, U8_BYTES);
    read!(read_u32, u32, U32_BYTES);

    fn read_slice(&mut self, len: usize) -> Result<&'a [u8]> {
        self.eof(len)?;

        let slice = &self.data[self.cursor..self.cursor + len];
        self.cursor += len;

        Ok(slice)
    }

    fn read_slice_until(&mut self, stop: u8) -> Result<&'a [u8]> {
        let cursor_start = self.cursor;

        while self.read_u8()? != stop {}

        Ok(&self.data[cursor_start..self.cursor - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_file_parser::parse_test_file;
    use anyhow::anyhow;
    use image::ImageReader;
    use pretty_assertions::assert_eq;

    #[allow(dead_code)]
    fn generate_blob(path: &str) -> Result<()> {
        let content = std::fs::read(format!("{}.png", path))?;
        let png = PngDecoder::new(&content).decode()?;

        png.write_to_binary_blob(path)?;

        Ok(())
    }

    fn compare_png(image_title: &str) -> Result<()> {
        let expected_png =
            Png::read_from_binary_blob(&format!("./test_suite/{}", image_title).into())
                .map_err(|err| anyhow!("Try regenerating the blob: {:?}", err))?;

        let path = format!("./test_suite/{}.png", image_title);
        let reference_rgbs = ImageReader::open(&path)?.decode()?.to_rgb8().to_vec();

        let content = std::fs::read(&path)?;
        let generated_png = PngDecoder::new(&content).decode()?;
        let generated_rgbs = generated_png.to_rgb8().to_vec();

        assert_eq!(
            reference_rgbs,
            generated_rgbs,
            "Failed test: {:?}",
            parse_test_file(&path.into())?.test_desc
        );

        if expected_png != generated_png {
            assert_eq!(
                expected_png,
                generated_png,
                "Failed test: {:?}",
                parse_test_file(&path.into())?.test_desc
            );
        }

        Ok(())
    }

    // A note about the following test cases, these images were hand checked. This way, binary blobs can be generated with confidence, not hubris.

    #[test]
    fn test_basic_grayscale_8bit() -> Result<()> {
        // generate_blob("./test_suite/basn0g08")?;
        compare_png("basn0g08")?;
        Ok(())
    }

    #[test]
    fn test_basic_grayscale_alpha_8bit() -> Result<()> {
        // generate_blob("./test_suite/basn4a08")?;
        compare_png("basn4a08")?;
        Ok(())
    }

    #[test]
    fn test_basic_rgb_alpha_8bit() -> Result<()> {
        // generate_blob("./test_suite/basn6a08")?;
        compare_png("basn6a08")?;
        Ok(())
    }

    #[test]
    fn test_filter_0() -> Result<()> {
        // generate_blob("./test_suite/f00n2c08")?;
        compare_png("f00n2c08")?;

        // generate_blob("./test_suite/f00n0g08")?;
        compare_png("f00n0g08")?;

        Ok(())
    }

    #[test]
    fn test_filter_1() -> Result<()> {
        // generate_blob("./test_suite/f01n2c08")?;
        compare_png("f01n2c08")?;

        // generate_blob("./test_suite/f01n0g08")?;
        compare_png("f01n0g08")?;
        Ok(())
    }

    #[test]
    fn test_filter_2() -> Result<()> {
        // generate_blob("./test_suite/f02n2c08")?;
        compare_png("f02n2c08")?;

        // generate_blob("./test_suite/f02n0g08")?;
        compare_png("f02n0g08")?;
        Ok(())
    }

    #[test]
    fn test_filter_3() -> Result<()> {
        // generate_blob("./test_suite/f03n2c08")?;
        compare_png("f03n2c08")?;

        // generate_blob("./test_suite/f03n0g08")?;
        compare_png("f03n0g08")?;
        Ok(())
    }

    #[test]
    fn test_filter_4() -> Result<()> {
        // generate_blob("./test_suite/f04n2c08")?;
        compare_png("f04n2c08")?;

        // generate_blob("./test_suite/f04n0g08")?;
        compare_png("f04n0g08")?;
        Ok(())
    }
}
