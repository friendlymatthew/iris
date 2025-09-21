use crate::image::grammar::{ColorType, ImageExt};
use anyhow::{bail, Result};
#[cfg(test)]
use std::io::Write;
use std::{
    borrow::Cow, collections::BTreeMap, fs::File, io::Read, path::PathBuf, slice::ChunksExact,
};

#[derive(Debug)]
pub enum Chunk<'a> {
    ImageHeader(ImageHeader),
    Palette(ChunksExact<'a, u8>),
    ImageData(&'a [u8]),
    TextData(BTreeMap<Cow<'a, [u8]>, Cow<'a, [u8]>>),
    Gamma(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImageHeader {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) bit_depth: u8,
    pub(crate) color_type: ColorType,

    // Compression method should always be 0.
    pub(crate) compression_method: u8,
    pub(crate) filter_method: u8,
    pub(crate) interlace_method: bool,
}

impl ImageHeader {
    pub(crate) const fn num_bytes_per_pixel(&self) -> usize {
        let bits_per_pixel = self.color_type.num_channels() * self.bit_depth;

        bits_per_pixel.div_ceil(8) as usize
    }
}

#[derive(Debug)]
pub enum Filter {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

impl TryFrom<u8> for Filter {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let f = match value {
            0 => Self::None,
            1 => Self::Sub,
            2 => Self::Up,
            3 => Self::Average,
            4 => Self::Paeth,
            foreign => bail!("Unrecognized filter method: {}", foreign),
        };

        Ok(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Png {
    pub(crate) image_header: ImageHeader,
    pub(crate) gamma: u32,
    pub(crate) pixel_buffer: Vec<u8>,
}

impl ImageExt for Png {
    fn width(&self) -> u32 {
        self.image_header.width
    }

    fn height(&self) -> u32 {
        self.image_header.height
    }

    fn gamma(&self) -> u32 {
        self.gamma
    }

    fn color_type(&self) -> ColorType {
        self.image_header.color_type
    }

    fn rgb8(&self) -> Cow<'_, [u8]> {
        match self.color_type() {
            ColorType::RGB => Cow::from(&self.pixel_buffer),
            ColorType::RGBA => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(4)
                    .flat_map(|b| [b[0], b[1], b[2]])
                    .collect::<Vec<_>>();

                Cow::from(b)
            }
            ColorType::GrayscaleAlpha => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(2)
                    .flat_map(|b| [b[0], b[0], b[0]])
                    .collect::<Vec<u8>>();

                Cow::from(b)
            }
            ColorType::Grayscale => {
                let b = self
                    .pixel_buffer
                    .iter()
                    .flat_map(|&y| [y, y, y])
                    .collect::<Vec<u8>>();

                Cow::from(b)
            }
            foreign => unimplemented!("{:?}", foreign),
        }
    }

    fn rgba8(&self) -> Cow<'_, [u8]> {
        match self.color_type() {
            ColorType::RGBA => Cow::from(&self.pixel_buffer),
            ColorType::RGB => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(3)
                    .flat_map(|b| [b[0], b[1], b[2], 0])
                    .collect::<Vec<_>>();

                Cow::from(b)
            }
            ColorType::Grayscale => {
                let b = self
                    .pixel_buffer
                    .iter()
                    .flat_map(|&y| [y, y, y, 0])
                    .collect::<Vec<_>>();

                Cow::from(b)
            }
            ColorType::GrayscaleAlpha => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(2)
                    .flat_map(|b| [b[0], b[0], b[0], b[1]])
                    .collect::<Vec<_>>();

                Cow::from(b)
            }
            foreign => unimplemented!("{:?}", foreign),
        }
    }

    fn bitmap(&self) -> Cow<'_, [u32]> {
        match self.color_type() {
            ColorType::RGB => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(3)
                    .map(|b| u32::from_be_bytes([0, b[0], b[1], b[2]]))
                    .collect::<Vec<u32>>();

                Cow::from(b)
            }
            ColorType::RGBA => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(4)
                    .map(|b| u32::from_be_bytes([b[3], b[0], b[1], b[2]]))
                    .collect::<Vec<u32>>();

                Cow::from(b)
            }
            ColorType::Grayscale => {
                let l = self
                    .pixel_buffer
                    .iter()
                    .map(|&b| u32::from_be_bytes([0, b, b, b]))
                    .collect::<Vec<u32>>();

                Cow::from(l)
            }
            ColorType::GrayscaleAlpha => {
                let l = self
                    .pixel_buffer
                    .chunks_exact(2)
                    .map(|b| u32::from_be_bytes([b[1], b[0], b[0], b[0]]))
                    .collect::<Vec<u32>>();

                Cow::from(l)
            }
            _ => todo!("What do other color type pixels look like?"),
        }
    }
}

impl Png {
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn write_to_binary_blob(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        file.write_all(&self.width().to_be_bytes())?;
        file.write_all(&self.height().to_be_bytes())?;
        file.write_all(&self.image_header.bit_depth.to_be_bytes())?;
        file.write_all(&(self.image_header.color_type as u8).to_be_bytes())?;
        file.write_all(&self.image_header.compression_method.to_be_bytes())?;
        file.write_all(&self.image_header.filter_method.to_be_bytes())?;
        file.write_all(&(self.image_header.interlace_method as u8).to_be_bytes())?;

        file.write(&self.gamma.to_be_bytes())?;
        file.write_all(&self.pixel_buffer)?;

        Ok(())
    }

    pub fn read_from_binary_blob(path: &PathBuf) -> Result<Self> {
        let mut file = File::open(path)?;

        let mut width = [0; 4];
        file.read_exact(&mut width)?;

        let mut height = [0; 4];
        file.read_exact(&mut height)?;

        let mut bit_depth = [0; 1];
        file.read_exact(&mut bit_depth)?;

        let mut color_type = [0; 1];
        file.read_exact(&mut color_type)?;

        let mut compression_method = [0; 1];
        file.read_exact(&mut compression_method)?;

        let mut filter_method = [0; 1];
        file.read_exact(&mut filter_method)?;

        let mut interlace_method = [0; 1];
        file.read_exact(&mut interlace_method)?;

        let mut gamma = [0; 4];
        file.read_exact(&mut gamma)?;

        let mut pixel_buffer = Vec::new();
        file.read_to_end(&mut pixel_buffer)?;

        Ok(Self {
            image_header: ImageHeader {
                width: u32::from_be_bytes(width),
                height: u32::from_be_bytes(height),
                bit_depth: bit_depth[0],
                color_type: color_type[0].try_into()?,
                compression_method: compression_method[0],
                filter_method: filter_method[0],
                interlace_method: interlace_method[0] != 0,
            },
            gamma: u32::from_be_bytes(gamma),
            pixel_buffer,
        })
    }
}

/* todo!("What would custom ZLib decompression look like?)
#[derive(Debug)]
pub struct ZLib {
    pub(crate) compression_method_flags: u8,
    pub(crate) additional_flags: u8,
    pub(crate) check_value: u32,
}

impl ZLib {
    pub fn compression_method(&self) -> u8 {
        self.compression_method_flags & 0b1111
    }

    pub fn compression_info(&self) -> u8 {
        (self.compression_method_flags & 0b1111_0000) >> 4
    }

    pub fn flag_check(&self) -> u8 {
        self.additional_flags & 0b1_1111
    }

    pub fn preset_dictionary(&self) -> bool {
        self.additional_flags & 0b10_0000 != 0
    }

    pub fn compression_level(&self) -> u8 {
        (self.additional_flags & 0b1100_0000) >> 6
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Block {
    NoCompression = 0b00,
    FixedHuffmanCodes = 0b01,
    DynamicHuffmanCodes = 0b10,
    Reserved = 0b11,
}

impl TryFrom<usize> for Block {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let bt = match value {
            0b00 => Self::NoCompression,
            0b01 => Self::FixedHuffmanCodes,
            0b10 => Self::DynamicHuffmanCodes,
            0b11 => Self::Reserved,
            foreign => bail!("Unrecognized block type: {}", foreign),
        };

        Ok(bt)
    }
}
*/
